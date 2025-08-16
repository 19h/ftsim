//! # ftsim-engine::net::link
//!
//! Defines the data structures for network links, including their fault models.

use crate::prelude::*;

/// Represents a directed link in the network graph.
#[derive(Clone, Debug)]
pub struct NetLink {
    pub id: LinkId,
    pub src: NodeId,
    pub dst: NodeId,
    pub faults: LinkFaultModel,
}

/// A collection of fault models that can be applied to a network link.
#[derive(Clone, Debug)]
pub struct LinkFaultModel {
    pub drop: Bernoulli,
    pub duplicate: Bernoulli,
    pub corrupt: Bernoulli,
    pub base_delay: ftsim_types::scenario::DelaySpec,
    pub jitter: ftsim_types::scenario::DelaySpec,
    pub reorder_window: usize,
    pub partitioned: bool,
    pub bandwidth_bytes_per_ms: Option<u64>,
    pub mtu_bytes: Option<usize>,
}

impl Default for LinkFaultModel {
    fn default() -> Self {
        Self {
            drop: Bernoulli(0.0),
            duplicate: Bernoulli(0.0),
            corrupt: Bernoulli(0.0),
            // Default to a 10ms base delay with some jitter
            base_delay: ftsim_types::scenario::DelaySpec::Const(10),
            jitter: ftsim_types::scenario::DelaySpec::Uniform {
                lo: 0,
                hi: 2,
            },
            reorder_window: 0,
            partitioned: false,
            bandwidth_bytes_per_ms: None,
            mtu_bytes: None,
        }
    }
}

