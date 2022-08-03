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
pub(crate) struct BuildCli {}

pub(crate) fn handle_command(command: BuildCli, output_kind: OutputKind) -> Result<CommandOutput> {
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

fn build_rust_actor(
    rust_config: RustConfig,
    actor_config: ActorConfig,
    common_config: CommonConfig,
) -> Result<PathBuf> {
    let result = process::Command::new("cargo")
        .args(["build", "--release"])
        .output()?;

    let wasm_file = PathBuf::from(format!(
        "{}/{}/release/{}.wasm",
        rust_config
            .target_path
            .unwrap_or(PathBuf::from("target"))
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

    Ok(wasm_file)
}

fn build_tinygo(tinygo_config: TinyGoConfig) -> Result<PathBuf> {
    todo!()
}

fn build_actor(
    command: BuildCli,
    output_kind: OutputKind,
    actor_config: ActorConfig,
    language_config: LanguageConfig,
    common_config: CommonConfig,
) -> Result<CommandOutput> {
    // build it
    let file_path = match language_config {
        LanguageConfig::Rust(rust_config) => {
            build_rust_actor(rust_config, actor_config.clone(), common_config.clone())
        }
        LanguageConfig::TinyGo(tinygo_config) => build_tinygo(tinygo_config),
    }?;

    // sign it

    let file_path_string = file_path
        .to_str()
        .ok_or_else(|| anyhow!("Could not convert file path to string"))?
        .to_string();

    let sign_options = SignCommand {
        source: file_path_string,
        destination: Some("todo".to_string()),
        metadata: ActorMetadata {
            name: common_config.name,
            ver: Some(common_config.version.to_string()),
            custom_caps: actor_config.claims,
            call_alias: actor_config.call_alias,
            ..Default::default()
        },
    };

    let sign_output = sign_file(sign_options, output_kind)?;

    // push it

    // bop it

    todo!()
}

fn build_provider(
    command: BuildCli,
    output_kind: OutputKind,
    provider_config: ProviderConfig,
    language_config: LanguageConfig,
    common_config: CommonConfig,
) -> Result<CommandOutput> {
    todo!()
}

fn build_interface(
    command: BuildCli,
    output_kind: OutputKind,
    interface_config: InterfaceConfig,
    language_config: LanguageConfig,
    common_config: CommonConfig,
) -> Result<CommandOutput> {
    todo!()
}
