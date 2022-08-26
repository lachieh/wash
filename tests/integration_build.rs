use anyhow::Result;

mod common;
use common::wash;
use std::{
    env::temp_dir,
    fs::{create_dir_all, remove_dir_all},
};

#[test]
fn build_rust_actor() -> Result<()> {
    const SUBFOLDER: &str = "build_rust_actor";

    let test_dir = temp_dir().join(SUBFOLDER);
    let _ = remove_dir_all(&test_dir);
    create_dir_all(&test_dir)?;

    std::env::set_current_dir(&test_dir)?;

    wash()
        .args(&[
            "new",
            "actor",
            "hello",
            "--git",
            "wasmcloud/project-templates",
            "--subfolder",
            "actor/hello-build",
            "--silent",
            "--no-git-init",
        ])
        .output()
        .expect("Failed to generate project");

    std::env::set_current_dir(&test_dir.join("hello"))?;

    wash()
        .args(&["build"])
        .output()
        .expect("Failed to build project");

    let unsigned_file = test_dir.join("hello/build/hello.wasm");
    assert!(unsigned_file.exists(), "unsigned file not found!");

    let signed_file = test_dir.join("hello/build/hello_s.wasm");
    assert!(signed_file.exists(), "signed file not found!");

    remove_dir_all(test_dir).unwrap();
    Ok(())
}

#[test]
fn build_tinygo_actor() -> Result<()> {
    const SUBFOLDER: &str = "build_tinygo_actor";
    let test_dir = temp_dir().join(SUBFOLDER);
    let _ = remove_dir_all(&test_dir);
    create_dir_all(&test_dir)?;

    std::env::set_current_dir(&test_dir)?;

    wash()
        .args(&[
            "new",
            "actor",
            "echo",
            "--git",
            "wasmcloud/project-templates",
            "--subfolder",
            "actor/echo-tinygo-build",
            "--silent",
            "--no-git-init",
        ])
        .status()
        .expect("Failed to generate project");

    std::env::set_current_dir(&test_dir.join("echo"))?;

    wash()
        .args(&["build"])
        .status()
        .expect("Failed to build project");

    let unsigned_file = test_dir.join("echo/build/echo.wasm");
    assert!(unsigned_file.exists(), "unsigned file not found!");

    let signed_file = test_dir.join("echo/build/echo_s.wasm");
    assert!(signed_file.exists(), "signed file not found!");

    remove_dir_all(test_dir).unwrap();
    Ok(())
}