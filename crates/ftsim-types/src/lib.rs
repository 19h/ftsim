//! # ftsim-types
//!
//! This crate provides the foundational, shared data types used across the
//! entire FTSim workspace. Its purpose is to break dependency cycles by
//! providing a stable, central location for types that `ftsim-engine`,
//! `ftsim-proto`, `ftsim-cli`, and `ftsim-tui` all need to agree upon.

#![forbid(unsafe_code)]

pub mod config;
pub mod envelope;
pub mod errors;
pub mod id;
pub mod metrics;
pub mod scenario;
pub mod time;
pub mod topology;
