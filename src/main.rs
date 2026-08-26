#![warn(
    clippy::pedantic,
    clippy::nursery,
    clippy::expect_used,
    clippy::unwrap_used
)]
#![allow(clippy::significant_drop_tightening)]

use deno_core::anyhow::{Error, Result};
use deno_core::v8::{self, Local};
use deno_core::{FastString, JsRuntime, RuntimeOptions, exception_to_err, extension, op2, scope};
use deno_error::JsErrorBox;
use std::cell::{Cell, RefCell};
use std::env::args_os;
use std::io::{
    BufRead, BufReader, BufWriter, Error as IoError, ErrorKind, IsTerminal, StdinLock, StdoutLock,
    Write, stdin, stdout,
};
use std::process::exit;
use tokio::runtime::Builder;

static RUNTIME_SNAPSHOT: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/KAW_SNAPSHOT.bin"));

thread_local! {
    static STDIN: RefCell<BufReader<StdinLock<'static>>> =
        RefCell::new(BufReader::new(stdin().lock()));
    static STDOUT: RefCell<BufWriter<StdoutLock<'static>>> =
        RefCell::new(BufWriter::new(stdout().lock()));
    static LINE_BUFFERED: Cell<bool> = const { Cell::new(false) };
    static BROKEN_PIPE: Cell<bool> = const { Cell::new(false) };
}

#[op2]
#[string]
fn op_stdin_line() -> Result<Option<String>, JsErrorBox> {
    STDIN.with_borrow_mut(|reader| {
        // No newline in the buffer means that `read_line` may block, so we flush output before
        // potentially blocking on input
        if !reader.buffer().contains(&b'\n') {
            STDOUT
                .with_borrow_mut(Write::flush)
                .map_err(|err| stdout_error(&err))?;
        }
        let mut line = String::new();
        if let Err(err) = reader.read_line(&mut line) {
            return Err(JsErrorBox::generic(format!("Failed to read stdin: {err}")));
        }
        if line.is_empty() {
            return Ok(None);
        }
        line.truncate(line.trim_end_matches(['\r', '\n']).len());
        Ok(Some(line))
    })
}

#[op2]
#[serde]
fn op_args() -> Vec<String> {
    // Skip the first (command) and second (expression) arguments and return the rest
    args_os()
        .skip(2)
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect()
}

fn write_line(line: &str) -> Result<(), IoError> {
    STDOUT.with_borrow_mut(|writer| {
        writeln!(writer, "{line}")?;
        if LINE_BUFFERED.get() {
            writer.flush()?;
        }
        Ok(())
    })
}

fn stdout_error(err: &IoError) -> JsErrorBox {
    // Remember the kind before JsErrorBox erases it
    BROKEN_PIPE.set(err.kind() == ErrorKind::BrokenPipe);
    JsErrorBox::generic(format!("Failed to write stdout: {err}"))
}

#[op2(fast)]
fn op_write_line(#[string] line: &str) -> Result<(), JsErrorBox> {
    write_line(line).map_err(|err| stdout_error(&err))
}

/// Stringifies without invoking user code, matching how V8 formats values in its own messages
#[op2]
#[string]
fn op_detail_string<'s>(scope: &v8::PinScope<'s, '_>, value: Local<'s, v8::Value>) -> String {
    value
        .to_detail_string(scope)
        .map_or_else(String::new, |detail| detail.to_rust_string_lossy(scope))
}

extension!(
    kaw,
    ops = [op_stdin_line, op_args, op_write_line, op_detail_string]
);

fn execute_expression(expression: String) -> Result<()> {
    let mut js_runtime = JsRuntime::new(RuntimeOptions {
        extensions: vec![kaw::init()],
        startup_snapshot: Some(RUNTIME_SNAPSHOT),
        ..Default::default()
    });
    let result = js_runtime.execute_script("kaw:expression.js", expression)?;

    scope!(scope, js_runtime);
    let result = Local::new(scope, result);
    let key = FastString::from_static("__kawWrite").v8_string(scope)?;
    let global = scope.get_current_context().global(scope);
    let write = global
        .get(scope, key.into())
        .and_then(|write| write.try_cast::<v8::Function>().ok())
        .ok_or_else(|| Error::msg("__kawWrite is missing from the runtime"))?;

    let written = {
        v8::tc_scope!(tc, scope);
        let receiver = v8::undefined(tc).into();
        if write.call(tc, receiver, &[result]).is_some() {
            Ok(())
        } else {
            Err(tc.exception().map_or_else(
                || Error::msg("failed to write the result"),
                |exception| exception_to_err(tc, exception, false, true).into(),
            ))
        }
    };

    let flushed = STDOUT.with_borrow_mut(Write::flush);
    written?;
    flushed?;

    Ok(())
}

fn is_broken_pipe(err: &Error) -> bool {
    BROKEN_PIPE.get()
        || err
            .downcast_ref::<IoError>()
            .is_some_and(|err| err.kind() == ErrorKind::BrokenPipe)
}

fn main() -> Result<()> {
    // Skip the first argument (program) and consume the next argument to get the expression
    let Some(expression) = args_os().nth(1) else {
        eprintln!("Usage: kaw [expression] [args...]");
        exit(2);
    };

    let runtime = Builder::new_current_thread().enable_all().build()?;
    let _guard = runtime.enter();
    let expression = expression.to_string_lossy().into_owned();
    // Optimize for latency instead of throughput when there is a terminal
    LINE_BUFFERED.set(stdout().is_terminal());
    match execute_expression(expression) {
        Err(err) if is_broken_pipe(&err) => Ok(()),
        result => result,
    }
}
