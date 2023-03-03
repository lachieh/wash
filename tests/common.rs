use anyhow::{Context, Result};
use std::{
    env,
    fs::{create_dir_all, remove_dir_all},
    path::PathBuf,
};
use tempfile::{Builder, NamedTempFile, TempDir};

#[allow(unused)]
pub(crate) const LOCAL_REGISTRY: &str = "localhost:5001";

/// Helper function to create the `wash` binary process
#[allow(unused)]
pub(crate) fn wash() -> std::process::Command {
    test_bin::get_test_bin("wash")
}

#[allow(unused)]
pub(crate) fn output_to_string(output: std::process::Output) -> Result<String> {
    String::from_utf8(output.stdout).with_context(|| "Failed to convert output bytes to String")
}

#[allow(unused)]
pub(crate) fn get_json_output(output: std::process::Output) -> Result<serde_json::Value> {
    let output_str = output_to_string(output)?;

    let json: serde_json::Value = serde_json::from_str(&output_str)
        .with_context(|| "Failed to parse json from output string")?;

    Ok(json)
}

#[allow(unused)]
/// Creates a subfolder in the test directory for use with a specific test
/// that will be dropped when the `TempDir` struct goes out of scope
pub(crate) fn tmp_test_dir_with_subfolder(subfolder: &str) -> TempDir {
    let root_dir = &env::var("CARGO_MANIFEST_DIR").expect("$CARGO_MANIFEST_DIR");
    // Ensure test dir exists
    let test_dir = format!("{root_dir}/tests/fixtures");
    create_dir_all(test_dir.clone());
    Builder::new()
        .prefix(subfolder)
        .rand_bytes(5)
        .tempdir_in(test_dir)
        .unwrap()
}

#[allow(unused)]
/// Returns a PathBuf by appending the subfolder and file arguments
/// to the test fixtures directory. This does _not_ create the file,
/// so the test is responsible for initialization and modification of this file
pub(crate) fn tmp_test_file(subfolder: &TempDir, file: &str) -> NamedTempFile {
    Builder::new().tempfile_in(subfolder.path()).unwrap()
}
