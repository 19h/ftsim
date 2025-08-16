//! # ftsim-engine::telemetry::snapshot
//!
//! Defines the stable `Snapshot` struct used to communicate the state of the
//! simulation world to external consumers like the TUI.

use crate::prelude::*;
use indexmap::IndexMap;
use serde_json::Value;

/// A point-in-time snapshot of the entire simulation state.
#[derive(Clone, Debug)]
pub struct Snapshot {
    pub time: SimTime,
    pub nodes: Vec<NodeSnap>,
    pub links: Vec<LinkSnap>,
    pub recent_events: Vec<LogSnap>,
    pub metrics: MetricsSnapshot,
}

/// A snapshot of a single node's state.
#[derive(Clone, Debug)]
pub struct NodeSnap {
    pub id: NodeId,
    pub status: NodeStatus,
    pub timers: usize,
    pub byzantine: bool,
    /// Protocol-specific state exposed for visualization.
    pub custom: IndexMap<String, Value>,
}

/// A snapshot of a single network link's state.
#[derive(Clone, Debug)]
pub struct LinkSnap {
    pub id: LinkId,
    pub src: NodeId,
    pub dst: NodeId,
    pub is_partitioned: bool,
}

/// A snapshot of a recent simulation event.
#[derive(Clone, Debug)]
pub struct LogSnap {
    pub event_id: EventId,
    pub time: SimTime,
    pub event_type: String,
    pub details: String,
    pub node_id: Option<NodeId>,
}

/// A snapshot of current metric values.
#[derive(Clone, Debug, Default)]
pub struct MetricsSnapshot {
    pub messages_sent: u64,
    pub messages_delivered: u64,
    pub timers_fired: u64,
    pub faults_injected: u64,
}
