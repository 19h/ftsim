//! # ftsim-cli
//!
//! The main entry point for the FTSim binary. It parses command-line arguments
//! and dispatches to the appropriate subcommand handler.

#![forbid(unsafe_code)]

use crate::args::{Cli, Command};
use anyhow::Result;
use clap::Parser;

mod args;
mod commands;
mod logging;
mod wiring;

fn main() -> Result<()> {
    let args = Cli::parse();

    // Note: Tracing initialization is now handled inside the `run` command
    // to ensure it has access to the simulation-specific telemetry bus.
    // A simple logger is used for other commands.
    if !matches!(args.command, Command::Run(_)) {
        tracing_subscriber::fmt().with_env_filter("info").init();
    }

    match args.command {
        Command::Run(opts) => commands::run::exec(opts),
        Command::ListProtocols => commands::list_protocols::exec(),
        Command::Validate { scenario } => commands::validate::exec(scenario),
    }
}
