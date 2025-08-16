//! # ftsim-cli::commands::list_protocols
//!
//! Implements the `list-protocols` subcommand.

use crate::wiring::get_registry;
use anyhow::Result;

pub fn exec() -> Result<()> {
    println!("Available Protocols:");
    println!("{:<20} | {:<10}", "Name", "ProtoTag");
    println!("{:-<20}-|-{:-<10}", "", "");

    for (name, tag, _) in get_registry() {
        println!("{:<20} | {:<10}", name, tag.0);
    }

    Ok(())
}
