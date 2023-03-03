mod common;

use crate::common::LOCAL_REGISTRY;
use anyhow::Result;
use assert_json_diff::assert_json_include;
use common::{get_json_output, output_to_string, wash};
use scopeguard::defer;
use serde_json::json;
use std::{
    env::temp_dir,
    fs::{remove_dir_all, File},
    io::prelude::*,
};
use tempfile::TempDir;

#[test]
/// Running create and insert tests together
fn integration_create_and_insert() -> Result<()> {
    const ISSUER: &str = "SAACTTUPKR55VUWUDK7GJ5SU5KGED455FR7BDO46RUVOTHUWKBLECLH2UU";
    const SUBJECT: &str = "SVAOZUSBWWFL65P255DOHIETPTXUQMM5ETLSYPITI5G4K4HI6M2CDAPWAU";
    let test_dir = TempDir::new()?;
    let pargz = test_dir.path().join("test.par.gz");

    integration_par_create(ISSUER, SUBJECT, pargz.to_str().unwrap())?;
    integration_par_insert(ISSUER, SUBJECT, pargz.to_str().unwrap())?;
    Ok(())
}

/// Tests creation of a provider archive file with an initial binary
fn integration_par_create(issuer: &str, subject: &str, archive: &str) -> Result<()> {
    const ARCH: &str = "x86_64-linux";
    let bin_folder = TempDir::new()?;
    let binary = bin_folder.path().join("linux.so");
    let mut bin_file = File::create(binary.clone()).unwrap();
    bin_file.write_all(b"01100010 01110100 01110111").unwrap();

    let create = wash()
        .args([
            "par",
            "create",
            "-a",
            ARCH,
            "-b",
            binary.to_str().unwrap(),
            "-c",
            "wasmcloud:testing",
            "-n",
            "Test parJEEzy",
            "-v",
            "TestRunner",
            "--compress",
            "--issuer",
            issuer,
            "--subject",
            subject,
            "--disable-keygen",
            "--version",
            "3.2.1",
            "--revision",
            "42",
            "--destination",
            archive,
        ])
        .output()
        .expect("failed to create provider archive file");

    assert!(create.status.success());
    assert_eq!(
        output_to_string(create).unwrap(),
        format!("\nSuccessfully created archive {archive}\n")
    );

    let inspect_created = wash()
        .args(["par", "inspect", archive, "-o", "json"])
        .output()
        .expect("failed to inspect created provider archive file");
    assert!(inspect_created.status.success());
    let output = get_json_output(inspect_created).unwrap();
    let expected = json!({
        "capability_contract_id": "wasmcloud:testing",
        "name": "Test parJEEzy",
        "service": "VBM5JMFOVUJDHGTOJSPUJ33ZGHCRCJ3LYHUJ3HND5ZMRVORYCMAVPZQF",
        "issuer": "AA7R5L74E45BJ4XVUYTELQ56P5VCOSPOAA474L7QWH4ZAILLKTZFWYYW",
        "revision": "42",
        "targets": ["x86_64-linux"],
        "vendor": "TestRunner",
        "version": "3.2.1"
    });
    assert_json_include!(actual: output, expected: expected);
    Ok(())
}

/// Tests inserting multiple binaries into an existing provider archive file
fn integration_par_insert(issuer: &str, subject: &str, archive: &str) -> Result<()> {
    const ARCH1: &str = "mips64-android";
    const ARCH2: &str = "aarch64-ios";

    let insert_dir = TempDir::new()?;

    let bin1 = insert_dir.path().join("android.so");
    let mut bin1_file = File::create(bin1.clone()).unwrap();
    bin1_file.write_all(b"01101100 01100111").unwrap();

    let bin2 = insert_dir.path().join("ios.dylib");
    let mut bin2_file = File::create(bin2.clone()).unwrap();
    bin2_file.write_all(b"01101001 01101111 01110011").unwrap();

    let insert_bin1 = wash()
        .args([
            "par",
            "insert",
            archive,
            "-a",
            ARCH1,
            "-b",
            bin1.to_str().unwrap(),
            "-i",
            issuer,
            "-s",
            subject,
            "--disable-keygen",
        ])
        .output()
        .expect("failed to insert binary into provider archive");
    assert!(insert_bin1.status.success());
    assert_eq!(
        output_to_string(insert_bin1).unwrap(),
        format!(
            "\nSuccessfully inserted {} into archive {}\n",
            bin1.to_str().unwrap(),
            archive
        )
    );
    let inspect_after_bin1 = wash()
        .args(["par", "inspect", archive, "-o", "json"])
        .output()
        .expect("failed to inspect created provider archive file");
    assert!(inspect_after_bin1.status.success());
    let output = get_json_output(inspect_after_bin1).unwrap();
    let expected = json!({
        "capability_contract_id": "wasmcloud:testing",
        "name": "Test parJEEzy",
        "service": "VBM5JMFOVUJDHGTOJSPUJ33ZGHCRCJ3LYHUJ3HND5ZMRVORYCMAVPZQF",
        "issuer": "AA7R5L74E45BJ4XVUYTELQ56P5VCOSPOAA474L7QWH4ZAILLKTZFWYYW",
        "revision": "42",
        "vendor": "TestRunner",
        "version": "3.2.1"
    });
    assert_json_include!(actual: output, expected: expected);
    let targets: Vec<String> = output
        .get("targets")
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t.as_str().unwrap().to_string())
        .collect();
    assert!(targets.contains(&ARCH1.to_string()));
    assert!(targets.contains(&"x86_64-linux".to_string()));

    let insert_bin2 = wash()
        .args([
            "par",
            "insert",
            archive,
            "-a",
            ARCH2,
            "-b",
            bin2.to_str().unwrap(),
            "-i",
            issuer,
            "-s",
            subject,
            "--disable-keygen",
        ])
        .output()
        .expect("failed to insert binary into provider archive");
    assert!(insert_bin2.status.success());
    assert_eq!(
        output_to_string(insert_bin2).unwrap(),
        format!(
            "\nSuccessfully inserted {} into archive {}\n",
            bin2.to_str().unwrap(),
            archive
        )
    );

    let inspect_after_bin2 = wash()
        .args(["par", "inspect", archive, "-o", "json"])
        .output()
        .expect("failed to inspect created provider archive file");
    assert!(inspect_after_bin2.status.success());
    let output = get_json_output(inspect_after_bin2).unwrap();
    let expected = json!({
        "capability_contract_id": "wasmcloud:testing",
        "name": "Test parJEEzy",
        "service": "VBM5JMFOVUJDHGTOJSPUJ33ZGHCRCJ3LYHUJ3HND5ZMRVORYCMAVPZQF",
        "issuer": "AA7R5L74E45BJ4XVUYTELQ56P5VCOSPOAA474L7QWH4ZAILLKTZFWYYW",
        "revision": "42",
        "vendor": "TestRunner",
        "version": "3.2.1"
    });
    assert_json_include!(actual: output, expected: expected);
    let targets: Vec<String> = output
        .get("targets")
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t.as_str().unwrap().to_string())
        .collect();
    assert!(targets.contains(&ARCH1.to_string()));
    assert!(targets.contains(&ARCH2.to_string()));
    assert!(targets.contains(&"x86_64-linux".to_string()));
    Ok(())
}

#[test]
fn integration_par_inspect() -> Result<()> {
    const HTTP_OCI: &str = "wasmcloud.azurecr.io/httpclient:0.3.5";
    const HTTP_ISSUER: &str = "ACOJJN6WUP4ODD75XEBKKTCCUJJCY5ZKQ56XVKYK4BEJWGVAOOQHZMCW";
    const HTTP_SERVICE: &str = "VCCVLH4XWGI3SGARFNYKYT2A32SUYA2KVAIV2U2Q34DQA7WWJPFRKIKM";
    let inspect_dir = TempDir::new()?;
    let httpclient_parinspect = &format!("{LOCAL_REGISTRY}/httpclient:parinspect");

    // Pull the echo module and push to local registry to test local inspect
    let local_http_client_path = inspect_dir.path().join("httpclient.wasm");
    let get_http_client = wash()
        .args([
            "reg",
            "pull",
            HTTP_OCI,
            "--destination",
            local_http_client_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to pull httpclient for par inspect test");
    assert!(
        get_http_client.status.success(),
        "Reg Pull failed: {get_http_client:?}"
    );
    assert!(
        local_http_client_path.is_file(),
        "Artifact file {local_http_client_path:?} does not exist. Reg Pull output: {get_http_client:?}"
    );
    let push_echo = wash()
        .args([
            "reg",
            "push",
            httpclient_parinspect,
            local_http_client_path.to_str().unwrap(),
            "--insecure",
        ])
        .output()
        .expect("failed to push echo.wasm to local registry");
    assert!(push_echo.status.success());

    // Inspect local, local registry, and remote registry actor wasm
    // `String.contains` is used here to ensure we aren't relying on relative json field position.
    // This also allows tests to pass if information is _added_ but not if information is _omitted_
    // from the command output
    let local_inspect = wash()
        .args([
            "par",
            "inspect",
            local_http_client_path.to_str().unwrap(),
            "--output",
            "json",
        ])
        .output()
        .expect("failed to inspect local http server");
    assert!(local_inspect.status.success());
    let local_inspect_output = get_json_output(local_inspect).unwrap();
    let inspect_expected = json!({
        "issuer": HTTP_ISSUER,
        "service": HTTP_SERVICE,
        "capability_contract_id": "wasmcloud:httpclient",
    });
    assert_json_include!(actual: local_inspect_output, expected: inspect_expected);

    let local_reg_inspect = wash()
        .args([
            "par",
            "inspect",
            httpclient_parinspect,
            "--insecure",
            "-o",
            "json",
        ])
        .output()
        .expect("failed to inspect local registry wasm");
    assert!(local_reg_inspect.status.success());
    let local_reg_inspect_output = get_json_output(local_reg_inspect).unwrap();
    assert_json_include!(actual: local_reg_inspect_output, expected: inspect_expected);

    let remote_inspect = wash()
        .args(["par", "inspect", HTTP_OCI, "-o", "json"])
        .output()
        .expect("failed to inspect local registry wasm");
    assert!(remote_inspect.status.success());
    let remote_inspect_output = get_json_output(remote_inspect).unwrap();
    assert_json_include!(actual: remote_inspect_output, expected: inspect_expected);
    Ok(())
}

#[test]
fn integration_par_inspect_cached() {
    const HTTP_OCI: &str = "wasmcloud.azurecr.io/httpclient:0.3.5";
    const HTTP_FAKE_OCI: &str = "foo.bar.io/httpclient:0.3.5";
    const HTTP_FAKE_CACHED: &str = "foo_bar_io_httpclient_0_3_5";
    const HTTP_ISSUER: &str = "ACOJJN6WUP4ODD75XEBKKTCCUJJCY5ZKQ56XVKYK4BEJWGVAOOQHZMCW";
    const HTTP_SERVICE: &str = "VCCVLH4XWGI3SGARFNYKYT2A32SUYA2KVAIV2U2Q34DQA7WWJPFRKIKM";

    let cache_dir = temp_dir().join("wasmcloud_ocicache");
    let mut http_client_cache_path = cache_dir.join(HTTP_FAKE_CACHED);
    let _ = ::std::fs::create_dir_all(&http_client_cache_path);
    defer! {
        remove_dir_all(cache_dir).unwrap();
    }
    http_client_cache_path.set_extension("bin");

    let get_http_client = wash()
        .args([
            "reg",
            "pull",
            HTTP_OCI,
            "--destination",
            http_client_cache_path.to_str().unwrap(),
        ])
        .output()
        .expect("failed to pull echo for claims sign test");
    assert!(
        get_http_client.status.success(),
        "Reg Pull failed: {get_http_client:?}"
    );
    assert!(
        http_client_cache_path.is_file(),
        "Artifact file {http_client_cache_path:?} does not exist. Reg Pull output: {get_http_client:?}"
    );

    let remote_inspect = wash()
        .args(["par", "inspect", HTTP_FAKE_OCI, "-o", "json"])
        .output()
        .expect("failed to inspect remote cached registry");
    assert!(
        remote_inspect.status.success(),
        "Remote inspect output: {remote_inspect:?}",
    );
    let remote_inspect_output = get_json_output(remote_inspect).unwrap();
    let expected_output = json!({
        "issuer": HTTP_ISSUER,
        "service": HTTP_SERVICE,
        "capability_contract_id": "wasmcloud:httpclient",
    });
    assert_json_include!(actual: remote_inspect_output, expected: expected_output);

    let remote_inspect_no_cache = wash()
        .args(["par", "inspect", HTTP_FAKE_OCI, "-o", "json", "--no-cache"])
        .output()
        .expect("failed to inspect remote cached registry");

    assert!(
        !remote_inspect_no_cache.status.success(),
        "Remote inspect output: {remote_inspect_no_cache:?}"
    );
}
