use deno_core::extension;
use deno_core::snapshot::{CreateSnapshotOptions, create_snapshot};
use std::env::var_os;
use std::path::PathBuf;
use std::{env, fs};

fn main() {
    println!("cargo:rerun-if-changed=src/runtime.js");

    extension!(
        kaw,
        esm_entry_point = "ext:kaw/src/runtime.js",
        esm = ["src/runtime.js"]
    );

    let out_dir = PathBuf::from(var_os("OUT_DIR").unwrap());
    let snapshot_path = out_dir.join("KAW_SNAPSHOT.bin");

    let snapshot = create_snapshot(
        CreateSnapshotOptions {
            cargo_manifest_dir: env!("CARGO_MANIFEST_DIR"),
            startup_snapshot: None,
            skip_op_registration: false,
            extensions: vec![kaw::init_ops_and_esm()],
            with_runtime_cb: None,
            extension_transpiler: None,
        },
        None,
    )
    .unwrap();

    fs::write(snapshot_path, snapshot.output).unwrap();
}
