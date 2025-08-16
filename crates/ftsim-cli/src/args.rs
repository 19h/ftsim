//! # ftsim-cli::args
//!
//! Defines the command-line argument structure using `clap`.

use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    #[arg(long, global = true, default_value = "human")]
    pub log: LogFormat,

    #[arg(long, global = true)]
    pub log_file: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Run a simulation from a scenario file.
    Run(RunOpts),
    /// List all compiled and available protocols.
    ListProtocols,
    /// Validate a scenario file for correctness.
    Validate {
        #[arg(value_name = "SCENARIO_PATH")]
        scenario: PathBuf,
    },
}

#[derive(Args, Debug)]
pub struct RunOpts {
    /// Path to the scenario file (YAML or TOML).
    #[arg(short, long)]
    pub scenario: PathBuf,

    /// Override the RNG seed from the scenario file.
    #[arg(long)]
    pub seed: Option<u64>,

    /// Override the stop time from the scenario file (in milliseconds).
    #[arg(long)]
    pub stop_at: Option<u64>,

    /// Run in headless mode without the TUI.
    #[arg(long)]
    pub headless: bool,

    // Other options from the spec would go here.
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogFormat {
    Human,
    Json,
}
