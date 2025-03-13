#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::significant_drop_tightening)]

use deno_core::anyhow::Result;
use deno_core::error::CoreError;
use deno_core::v8::{self, Local};
use deno_core::{FastString, JsRuntime, ModuleSpecifier, RuntimeOptions, extension, op2};
use std::env::args;
use std::io::{BufWriter, Write, stdin, stdout};
use std::process::exit;
use tokio::runtime::Builder;

static RUNTIME_SNAPSHOT: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/KAW_SNAPSHOT.bin"));

#[op2]
#[string]
fn op_stdin_line() -> Result<Option<String>, CoreError> {
    let mut line = String::new();
    stdin().read_line(&mut line)?;
    if line.is_empty() {
        return Ok(None);
    }
    line.truncate(line.trim_matches(['\r', '\n']).len());
    Ok(Some(line))
}

#[op2]
#[serde]
fn op_args() -> Vec<String> {
    // Skip the first (command) and second (expression) arguments and return the rest
    args().skip(2).collect()
}

extension!(kaw, ops = [op_stdin_line, op_args]);

#[allow(clippy::future_not_send)]
async fn execute_expression(expression: String) -> Result<()> {
    let mut js_runtime = JsRuntime::new(RuntimeOptions {
        extensions: vec![kaw::init_ops()],
        startup_snapshot: Some(RUNTIME_SNAPSHOT),
        ..Default::default()
    });

    let internal_mod_id = js_runtime
        .load_side_es_module_from_code(
            &ModuleSpecifier::parse("kaw:runtime.js")?,
            include_str!("./runtime.js"),
        )
        .await?;
    js_runtime.mod_evaluate(internal_mod_id).await?;

    let global_result = js_runtime.execute_script("kaw:expression.js", expression)?;
    let scope = &mut js_runtime.handle_scope();
    let local_result = Local::new(scope, global_result);

    // Check whether the result is an array or is an iterable that can be converted into an array by
    // calling toArray()
    let to_array_key = FastString::from_static("toArray").v8_string(scope)?;
    let Some(lines_array) = local_result.try_cast::<v8::Array>().ok().or_else(|| {
        local_result
            .try_cast::<v8::Object>()
            .ok()
            .and_then(|iterable| iterable.get(scope, to_array_key.into()))
            .and_then(|iterator| iterator.try_cast::<v8::Function>().ok())
            .and_then(|iterator_fn| iterator_fn.call(scope, local_result, &[]))
            .and_then(|iterator| iterator.try_cast::<v8::Array>().ok())
    }) else {
        // If the result isn't an array or an iterable, just print the result
        if !local_result.is_null_or_undefined() {
            println!("{}", local_result.to_rust_string_lossy(scope));
        }
        return Ok(());
    };

    // If the result is an array, write all lines at once and only flush once
    let mut writer = BufWriter::new(stdout().lock());
    for index in 0..lines_array.length() {
        let line = lines_array.get_index(scope, index).unwrap();
        if !line.is_null_or_undefined() {
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
    runtime.block_on(execute_expression(expression))
}
