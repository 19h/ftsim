//! # ftsim-proto
//!
//! This crate provides the Software Development Kit (SDK) for implementing
//! distributed protocols to be run within FTSim. It defines the core traits
//! (`Protocol`, `ProtocolDyn`) and the context object (`Ctx`) that protocols
//! use to interact with the simulation engine.

#![forbid(unsafe_code)]

pub mod api;
pub mod ctx_ext;
pub mod protocols;

pub use api::{FaultEvent, Protocol, ProtocolDyn};
pub use ctx_ext::Ctx;
