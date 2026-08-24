#![warn(
    clippy::pedantic,
    clippy::nursery,
    clippy::expect_used,
    clippy::unwrap_used
)]
#![allow(clippy::significant_drop_tightening)]

use deno_core::anyhow::Result;
use deno_core::error::CoreError;
use deno_core::v8::{self, Local};
use deno_core::{FastString, JsRuntime, RuntimeOptions, exception_to_err, extension, op2, scope};
use std::env::args;
use std::io::{BufWriter, Write, stdin, stdout};
use std::process::exit;
use tokio::runtime::Builder;

static RUNTIME_SNAPSHOT: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/KAW_SNAPSHOT.bin"));

#[op2]
#[string]
fn op_stdin_line() -> Result<Option<String>, Box<CoreError>> {
    let mut line = String::new();
    if let Err(err) = stdin().read_line(&mut line) {
        return Err(Box::new(err.into()));
    }
    if line.is_empty() {
        return Ok(None);
    }
    line.truncate(line.trim_end_matches(['\r', '\n']).len());
    Ok(Some(line))
}

#[op2]
#[serde]
fn op_args() -> Vec<String> {
    // Skip the first (command) and second (expression) arguments and return the rest
    args().skip(2).collect()
}

extension!(kaw, ops = [op_stdin_line, op_args]);

fn execute_expression(expression: String) -> Result<()> {
    let mut js_runtime = JsRuntime::new(RuntimeOptions {
        extensions: vec![kaw::init()],
        startup_snapshot: Some(RUNTIME_SNAPSHOT),
        ..Default::default()
    });

    let global_result = js_runtime.execute_script("kaw:expression.js", expression)?;
    scope!(scope, js_runtime);
    v8::tc_scope!(scope, scope);
    let local_result = Local::new(scope, global_result);

    // Check whether the result is an array or is an iterable that can be converted into an array by
    // calling toArray()
    let to_array_key = FastString::from_static("toArray").v8_string(scope)?;
    let lines_array = local_result.try_cast::<v8::Array>().ok().or_else(|| {
        local_result
            .try_cast::<v8::Object>()
            .ok()
            .and_then(|iterable| iterable.get(scope, to_array_key.into()))
            .and_then(|iterator| iterator.try_cast::<v8::Function>().ok())
            .and_then(|iterator_fn| iterator_fn.call(scope, local_result, &[]))
            .and_then(|iterator| iterator.try_cast::<v8::Array>().ok())
    });

    if let Some(exception) = scope.exception() {
        return Err(exception_to_err(scope, exception, false, true).into());
    }

    let Some(lines_array) = lines_array else {
        // If the result isn't an array or an iterable, just print the result
        if !local_result.is_null_or_undefined() {
            println!("{}", local_result.to_rust_string_lossy(scope));
        }
        return Ok(());
    };

    // If the result is an array, write all lines at once and only flush once
    let mut writer = BufWriter::new(stdout().lock());
    for index in 0..lines_array.length() {
        if let Some(line) = lines_array.get_index(scope, index)
            && !line.is_null_or_undefined()
        {
            writeln!(writer, "{}", line.to_rust_string_lossy(scope))?;
        }
    }
    writer.flush()?;

    Ok(())
}

fn main() -> Result<()> {
    // Skip the first argument (program) and consume the next argument to get the expression
    let Some(expression) = args().nth(1) else {
        eprintln!("Usage: kaw [expression] [args...]");
        exit(2);
    };

    let runtime = Builder::new_current_thread().enable_all().build()?;
    let _guard = runtime.enter();
    execute_expression(expression)
}
