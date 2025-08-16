//! # ftsim-proto::protocols
//!
//! This module contains example protocol implementations that demonstrate
//! how to use the FTSim SDK.

#[cfg(feature = "primary_backup")]
pub mod primary_backup;

#[cfg(feature = "raft_lite")]
pub mod raft_lite;
