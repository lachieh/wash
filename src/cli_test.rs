use crate::util::CommandOutput;

use anyhow::{bail, Result};
use clap::Parser;
use wash_lib::parser::{self, LanguageConfig};

/// Test the current wasmCloud project.
#[derive(Parser, Debug, Clone)]
pub(crate) struct TestCommand {}

pub(crate) fn handle_command(_command: TestCommand) -> Result<CommandOutput> {
    let config = parser::get_config(None, Some(true))?;

    match config.language {
        LanguageConfig::Rust(rust_config) => {
            let mut cargo = rust_config.cargo();

            let result = cargo
                .args(["clippy", "--all-features", "--all-targets"])
                .status()?;

            if !result.success() {
                bail!("`cargo clippy` failed: {}", result.to_string());
            }
        }
        LanguageConfig::TinyGo(tinygo_config) => {
            let mut tinygo = tinygo_config.tinygo();

            let result = tinygo.args(["test"]).status()?;

            if !result.success() {
                bail!("`tinygo test` failed: {}", result.to_string());
            }
        }
    };
    Ok(CommandOutput::from_key_and_text(
        "result",
        "tests passed!".to_string(),
    ))
}
