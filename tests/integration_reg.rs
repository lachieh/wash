mod common;
use common::{get_json_output, output_to_string, tmp_test_dir_with_subfolder, tmp_test_file, wash};
use serde_json::json;
use std::{fs::remove_dir_all, io::prelude::*};

const ECHO_WASM: &str = "wasmcloud.azurecr.io/echo:0.2.0";
const LOGGING_PAR: &str = "wasmcloud.azurecr.io/logging:0.9.1";
const LOCAL_REGISTRY: &str = "localhost:5001";

#[test]
fn integration_pull_basic() {
    const SUBFOLDER: &str = "pull_basic";
    let pull_dir = tmp_test_dir_with_subfolder(SUBFOLDER);

    let basic_echo = tmp_test_file(&pull_dir, "basic_echo.wasm");

    let pull_basic = wash()
        .args([
            "reg",
            "pull",
            ECHO_WASM,
            "--destination",
            basic_echo.path().to_str().unwrap(),
            "--allow-latest",
        ])
        .output()
        .unwrap_or_else(|_| panic!("failed to pull {ECHO_WASM}"));
    assert!(pull_basic.status.success());
    // Very important
    assert!(output_to_string(pull_basic).unwrap().contains('\u{1F6BF}'));

    remove_dir_all(pull_dir).unwrap();
}

#[test]
fn integration_pull_comprehensive() {
    const SUBFOLDER: &str = "pull_comprehensive";
    let pull_dir = tmp_test_dir_with_subfolder(SUBFOLDER);

    let comprehensive_echo = tmp_test_file(&pull_dir, "comprehensive_echo.wasm");
    let comprehensive_logging = tmp_test_file(&pull_dir, "comprehensive_logging.par.gz");

    let pull_echo_comprehensive = wash()
        .args([
            "reg",
            "pull",
            ECHO_WASM,
            "--destination",
            comprehensive_echo.path().to_str().unwrap(),
            "--digest",
            "sha256:a17a163afa8447622055deb049587641a9e23243a6cc4411eb33bd4267214cf3",
            "--output",
            "json",
        ])
        .output()
        .unwrap_or_else(|_| panic!("failed to pull {ECHO_WASM}"));

    assert!(pull_echo_comprehensive.status.success());
    let output = get_json_output(pull_echo_comprehensive).unwrap();

    let expected_json =
        json!({"file": comprehensive_echo.path().to_str().unwrap(), "success": true});

    assert_eq!(output, expected_json);

    let pull_logging_comprehensive = wash()
        .args([
            "reg",
            "pull",
            LOGGING_PAR,
            "--destination",
            comprehensive_logging.path().to_str().unwrap(),
            "--digest",
            "sha256:169f2764e529c2b57ad20abb87e0854d67bf6f0912896865e2911dee1bf6af98",
            "--output",
            "json",
        ])
        .output()
        .unwrap_or_else(|_| panic!("failed to pull {ECHO_WASM}"));

    assert!(pull_logging_comprehensive.status.success());
    let output = get_json_output(pull_logging_comprehensive).unwrap();

    let expected_json =
        json!({"file": comprehensive_logging.path().to_str().unwrap(), "success": true});

    assert_eq!(output, expected_json);

    remove_dir_all(pull_dir).unwrap();
}

#[test]
fn integration_push_basic() {
    const SUBFOLDER: &str = "push_basic";
    let push_dir = tmp_test_dir_with_subfolder(SUBFOLDER);

    let pull_echo_wasm = tmp_test_file(&push_dir, "echo.wasm");

    // Pull echo.wasm for push tests
    wash()
        .args([
            "reg",
            "pull",
            ECHO_WASM,
            "--destination",
            pull_echo_wasm.path().to_str().unwrap(),
        ])
        .output()
        .unwrap_or_else(|_| panic!("failed to pull {ECHO_WASM} for push basic"));

    // Push echo.wasm and pull from local registry
    let echo_push_basic = &format!("{LOCAL_REGISTRY}/echo:pushbasic");
    let localregistry_echo_wasm = tmp_test_file(&push_dir, "echo_local.wasm");
    let push_echo = wash()
        .args([
            "reg",
            "push",
            echo_push_basic,
            pull_echo_wasm.path().to_str().unwrap(),
            "--insecure",
        ])
        .output()
        .expect("failed to push echo.wasm to local registry");
    assert!(push_echo.status.success());

    let pull_local_registry_echo = wash()
        .args([
            "reg",
            "pull",
            echo_push_basic,
            "--insecure",
            "--destination",
            localregistry_echo_wasm.path().to_str().unwrap(),
        ])
        .output()
        .expect("failed to pull echo.wasm from local registry");

    assert!(pull_local_registry_echo.status.success());

    remove_dir_all(push_dir).unwrap();
}

#[test]
fn integration_push_comprehensive() {
    const SUBFOLDER: &str = "push_comprehensive";
    let push_dir = tmp_test_dir_with_subfolder(SUBFOLDER);

    let pull_echo_wasm = tmp_test_file(&push_dir, "echo.wasm");
    let pull_logging_par = tmp_test_file(&push_dir, "logging.par.gz");

    // Pull echo.wasm and logging.par.gz for push tests
    wash()
        .args([
            "reg",
            "pull",
            ECHO_WASM,
            "--destination",
            pull_echo_wasm.path().to_str().unwrap(),
        ])
        .output()
        .unwrap_or_else(|_| panic!("failed to pull {ECHO_WASM} for push basic"));
    wash()
        .args([
            "reg",
            "pull",
            LOGGING_PAR,
            "--destination",
            pull_logging_par.path().to_str().unwrap(),
        ])
        .output()
        .unwrap_or_else(|_| panic!("failed to pull {LOGGING_PAR} for push basic"));

    let mut config_json = tmp_test_file(&push_dir, "config.json");
    config_json.write_all(b"{}").unwrap();

    let logging_push_all_options = &format!("{LOCAL_REGISTRY}/logging:alloptions");
    let push_all_options = wash()
        .args([
            "reg",
            "push",
            logging_push_all_options,
            pull_logging_par.path().to_str().unwrap(),
            "--allow-latest",
            "--insecure",
            "--config",
            config_json.path().to_str().unwrap(),
            "--output",
            "json",
            "--password",
            "supers3cr3t",
            "--user",
            "localuser",
        ])
        .output()
        .unwrap_or_else(|_| panic!("failed to push {LOGGING_PAR} for push comprehensive"));
    assert!(push_all_options.status.success());

    let output = get_json_output(push_all_options).unwrap();

    let expected_json = json!({"url": logging_push_all_options, "success": true});

    assert_eq!(output, expected_json);

    remove_dir_all(push_dir).unwrap();
}
