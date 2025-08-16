//! # ftsim-types::errors
//!
//! Defines the common error types used throughout the FTSim workspace.
//! Using `thiserror` provides clean, descriptive error handling. All error
//! variants must have a deterministic `Debug` implementation for reproducibility.

use crate::time::SimTime;
use thiserror::Error;

/// A general-purpose error for the simulation engine.
#[derive(Error, Debug, Clone)]
pub enum SimError {
    #[error("Simulation time overflow: {base} + {offset}")]
    TimeOverflow { base: SimTime, offset: SimTime },
    #[error("Simulation time underflow: {base} - {offset}")]
    TimeUnderflow { base: SimTime, offset: SimTime },
    #[error("Monotonic ID counter overflowed")]
    IdOverflow,
    #[error("Node with ID {0} not found")]
    NodeNotFound(u32),
    #[error("Link with ID {0} not found")]
    LinkNotFound(u64),
    #[error("Protocol with tag {0:?} not registered")]
    ProtocolNotRegistered(super::envelope::ProtoTag),
}

/// An error related to parsing or validating configuration files.
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("I/O error reading config file: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML parsing error: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("Validation error in scenario '{name}': {message}")]
    Validation { name: String, message: String },
}

/// An error during message serialization or deserialization.
#[derive(Error, Debug)]
#[error("Codec error: {0}")]
pub struct CodecError(pub String);

/// An error originating from the storage subsystem.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum StoreError {
    #[error("I/O error (simulated): {0}")]
    Io(String),
    #[error("No space left on device (simulated)")]
    NoSpace,
    #[error("Record at index {0} not found")]
    NotFound(u64),
    #[error("Operation failed due to injected fault")]
    FaultInjected,
}

/// An error originating from the network subsystem.
#[derive(Error, Debug, Clone)]
pub enum NetError {
    #[error("Message exceeds MTU of {mtu} bytes")]
    ExceedsMtu { mtu: usize },
}
