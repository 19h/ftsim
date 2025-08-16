//! # ftsim-cli::commands::validate
//!
//! Implements the `validate` subcommand.

use anyhow::Result;
use ftsim_types::scenario::Scenario;
use std::{fs, path::PathBuf};

pub fn exec(path: PathBuf) -> Result<()> {
    println!("Validating scenario: {:?}", path);
    let content = fs::read_to_string(&path)?;
    let scenario: Scenario = match path.extension().and_then(|s| s.to_str()) {
        Some("yaml") | Some("yml") => serde_yaml::from_str(&content)?,
        Some("toml") => toml::from_str(&content)?,
        _ => return Err(anyhow::anyhow!("Unsupported scenario file extension")),
    };

    scenario.validate().map_err(|e| anyhow::anyhow!(e))?;

    println!("Scenario '{}' is valid.", scenario.name);
    Ok(())
}
