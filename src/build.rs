use std::{path::PathBuf, process};

use crate::{
    claims::{sign_file, ActorMetadata, SignCommand},
    util::{CommandOutput, OutputKind},
};
use anyhow::{anyhow, bail, Result};
use clap::Parser;
use wash_lib::parser::{
    ActorConfig, CommonConfig, InterfaceConfig, LanguageConfig, ProviderConfig, RustConfig,
    TinyGoConfig, TypeConfig,
};

/// Build (and sign) a wasmCloud actor, provider, or interface
#[derive(Debug, Parser, Clone)]
#[clap(name = "build")]
pub(crate) struct BuildCommand {
    // If true, pushes the signed actor to the registry.
    #[clap(short = 'p', long = "push")]
    pub(crate) push: bool,
}

pub(crate) fn handle_command(
    command: BuildCommand,
    output_kind: OutputKind,
) -> Result<CommandOutput> {
    let config = wash_lib::parser::get_config(None, Some(true))?;

    match config.project_type {
        TypeConfig::Actor(actor_config) => build_actor(
            command,
            output_kind,
            actor_config,
            config.language,
            config.common,
        ),
        TypeConfig::Provider(provider_config) => build_provider(
            command,
            output_kind,
            provider_config,
            config.language,
            config.common,
        ),
        TypeConfig::Interface(interface_config) => build_interface(
            command,
            output_kind,
            interface_config,
            config.language,
            config.common,
        ),
    }
}

fn build_actor(
    command: BuildCommand,
    output_kind: OutputKind,
    actor_config: ActorConfig,
    language_config: LanguageConfig,
    common_config: CommonConfig,
) -> Result<CommandOutput> {
    // build it
    println!("Building actor...");
    let file_path = match language_config {
        LanguageConfig::Rust(rust_config) => {
            build_rust_actor(common_config.clone(), rust_config, actor_config.clone())
        }
        LanguageConfig::TinyGo(tinygo_config) => {
            build_tinygo_actor(common_config.clone(), tinygo_config)
        }
    }?;
    println!("Done building actor");

    // sign it
    println!("Signing actor...");
    let file_path_string = file_path
        .to_str()
        .ok_or_else(|| anyhow!("Could not convert file path to string"))?
        .to_string();

    let sign_options = SignCommand {
        source: file_path_string,
        destination: Some(format!("build/{}_s.wasm", common_config.name)),
        metadata: ActorMetadata {
            name: common_config.name,
            ver: Some(common_config.version.to_string()),
            custom_caps: actor_config.claims,
            call_alias: actor_config.call_alias,
            ..Default::default()
        },
    };
    let sign_output = sign_file(sign_options, output_kind)?;

    if !command.push {
        return Ok(sign_output);
    }

    println!("Signed actor: {}", sign_output.text);

    // push it
    Ok(CommandOutput::from_key_and_text(
        "result",
        "Pushing has not be implemented yet, please use wash reg push.".to_string(),
    ))
}

/// Builds a rust actor and returns the path to the file.
pub fn build_rust_actor(
    common_config: CommonConfig,
    rust_config: RustConfig,
    actor_config: ActorConfig,
) -> Result<PathBuf> {
    let mut command = match rust_config.cargo_path {
        Some(path) => process::Command::new(path),
        None => process::Command::new("cargo"),
    };

    let result = command.args(["build", "--release"]).status()?;

    if !result.success() {
        bail!("Compiling actor failed: {}", result.to_string())
    }

    let wasm_file = PathBuf::from(format!(
        "{}/{}/release/{}.wasm",
        rust_config
            .target_path
            .unwrap_or_else(|| PathBuf::from("target"))
            .to_string_lossy(),
        actor_config.wasm_target,
        common_config.name,
    ));

    if !wasm_file.exists() {
        bail!(
            "Could not find compiled wasm file to sign: {}",
            wasm_file.display()
        );
    }

    // move the file out into the build/ folder for parity with tinygo and convienience for users.
    let copied_wasm_file = PathBuf::from(format!("build/{}.wasm", common_config.name));
    std::fs::create_dir_all(copied_wasm_file.parent().unwrap())?;
    std::fs::copy(&wasm_file, &copied_wasm_file)?;
    std::fs::remove_file(&wasm_file)?;

    Ok(copied_wasm_file)
}

/// Builds a tinygo actor and returns the path to the file.
pub fn build_tinygo_actor(
    common_config: CommonConfig,
    tinygo_config: TinyGoConfig,
) -> Result<PathBuf> {
    let filename = format!("build/{}.wasm", common_config.name);

    let mut command = match tinygo_config.tinygo_path {
        Some(path) => process::Command::new(path),
        None => process::Command::new("tinygo"),
    };

    let result = command
        .args([
            "build",
            "-o",
            filename.as_str(),
            "-target",
            "wasm",
            "-scheduler",
            "none",
            "-no-debug",
            ".",
        ])
        .status()?;

    if !result.success() {
        bail!("Compiling actor failed: {}", result.to_string())
    }

    let wasm_file = PathBuf::from(filename);

    if !wasm_file.exists() {
        bail!(
            "Could not find compiled wasm file to sign: {}",
            wasm_file.display()
        );
    }

    Ok(wasm_file)
}

fn build_provider(
    _command: BuildCommand,
    _output_kind: OutputKind,
    _provider_config: ProviderConfig,
    _language_config: LanguageConfig,
    _common_config: CommonConfig,
) -> Result<CommandOutput> {
    Ok(CommandOutput::from_key_and_text(
        "result",
        "wash build has not be implemented for providers yet. Please use the Makefiles for now!"
            .to_string(),
    ))
}

fn build_interface(
    _command: BuildCommand,
    _output_kind: OutputKind,
    _interface_config: InterfaceConfig,
    _language_config: LanguageConfig,
    _common_config: CommonConfig,
) -> Result<CommandOutput> {
    Ok(CommandOutput::from_key_and_text(
        "result",
        "wash build has not be implemented for interface yet. Please use the Makefiles for now!"
            .to_string(),
    ))
}

#[cfg(test)]
mod test {

    use super::*;
    use clap::Parser;

    #[test]
    fn test_build_comprehensive() {
        let cmd: BuildCommand = Parser::try_parse_from(&["build", "--push"]).unwrap();
        assert!(cmd.push);

        let cmd: BuildCommand = Parser::try_parse_from(&["build"]).unwrap();
        assert!(!cmd.push);
    }
}
