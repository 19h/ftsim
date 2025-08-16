//! # ftsim-types::config
//!
//! Defines strongly-typed structs for configuration, mirroring the structure
//! of the scenario files. These types are used by `serde` to parse TOML/YAML
//! into safe, usable Rust objects.

use serde::{Deserialize, Serialize};

/// A wrapper for the RNG seed to make its purpose clear.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RngSeed(pub u64);

/// A specification for a deterministic probability distribution for delays.
/// The simulation engine uses these specifications to sample from the master RNG.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DelayDist {
    Const(u64),
    Uniform { lo: u64, hi: u64 },
    Normal { mu: f64, sigma: f64 },
    Pareto { scale: f64, shape: f64 },
}

/// A specification for a Bernoulli trial (a coin flip).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bernoulli(pub f64);
