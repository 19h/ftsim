//! # ftsim-types::scenario
//!
//! Defines the Rust structs that map directly to the Scenario DSL (YAML/TOML).
//! This is the authoritative schema for defining simulation experiments.

use crate::{
    envelope::ProtoTag,
    id::{LinkId, NodeId},
    time::{deserialize_sim_time, SimTime},
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// The top-level structure for a scenario definition file.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Scenario {
    pub name: String,
    pub seed: Option<u64>,
    pub initial: InitialSpec,
    pub topology: super::topology::TopologySpec,
    pub directives: Vec<Directive>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_at: Option<SimTime>,
}

impl Scenario {
    /// Validates the scenario for logical consistency.
    pub fn validate(&self) -> Result<(), String> {
        let num_nodes = self.initial.nodes;
        for (i, directive) in self.directives.iter().enumerate() {
            let action = directive.action();
            // Validate NodeIds are in range
            if let Some(node_id) = action.node_id() {
                if (node_id as usize) >= num_nodes {
                    return Err(format!(
                        "Directive {} contains invalid NodeId {}; max is {}",
                        i,
                        node_id,
                        num_nodes - 1
                    ));
                }
            }
            // Validate partition sets
            if let Action::Partition { sets } = action {
                let mut seen_nodes = HashSet::new();
                let mut total_nodes_in_sets = 0;
                for set in sets {
                    if set.is_empty() {
                        return Err(format!("Directive {} contains an empty partition set", i));
                    }
                    for &node_id in set {
                        if !seen_nodes.insert(node_id) {
                            return Err(format!(
                                "Directive {} has duplicate node {} in partition sets",
                                i, node_id
                            ));
                        }
                    }
                    total_nodes_in_sets += set.len();
                }
                if total_nodes_in_sets >= num_nodes {
                    return Err(format!(
                        "Directive {} partition must cover a strict subset of nodes",
                        i
                    ));
                }
            }
        }
        Ok(())
    }
}

/// Specifies the initial state of the simulation world.
#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub struct InitialSpec {
    pub nodes: usize,
    pub proto: ProtoTag,
}

/// A directive that schedules an action to occur at a specific time.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub enum Directive {
    At(#[serde(deserialize_with = "deserialize_sim_time")] SimTime, Action),
    Every {
        #[serde(deserialize_with = "deserialize_sim_time")]
        period: SimTime,
        repeats: u64,
        action: Action,
    },
    After {
        #[serde(deserialize_with = "deserialize_sim_time")]
        offset: SimTime,
        action: Action,
    },
}

impl Directive {
    pub fn action(&self) -> &Action {
        match self {
            Directive::At(_, action) => action,
            Directive::Every { action, .. } => action,
            Directive::After { action, .. } => action,
        }
    }
}

/// An action that modifies the state of the simulation world, typically to inject a fault.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub enum Action {
    Partition { sets: Vec<Vec<NodeId>> },
    HealPartition,
    Crash {
        node: NodeId,
        #[serde(deserialize_with = "deserialize_sim_time")]
        duration: SimTime
    },
    Restart { node: NodeId },
    LinkDelay { link: LinkId, dist: DelaySpec },
    LinkDrop { link: LinkId, p: f64 },
    BroadcastBytes { payload_hex: String, #[serde(default)] proto_tag: Option<ProtoTag> },
    ClockSkew { node: NodeId, skew: i128 },
    StoreFault { node: NodeId, kind: StoreFaultKind, rate: f64 },
    ByzantineFlip { node: NodeId, enabled: bool },
    Custom { name: String, args: toml::Value },
}

impl Action {
    /// Returns the node ID associated with the action, if any.
    pub fn node_id(&self) -> Option<NodeId> {
        match self {
            Action::Crash { node, .. }
            | Action::Restart { node }
            | Action::ClockSkew { node, .. }
            | Action::StoreFault { node, .. }
            | Action::ByzantineFlip { node, .. } => Some(*node),
            _ => None,
        }
    }
}

/// A serializable version of `DelayDist` for scenarios.
#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
#[serde(rename_all = "PascalCase")]
pub enum DelaySpec {
    Const(u64),
    Uniform { lo: u64, hi: u64 },
    Normal { mu: f64, sigma: f64 },
    Pareto { scale: f64, shape: f64 },
}

/// Kinds of storage faults that can be injected.
#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub enum StoreFaultKind {
    WriteError,
    TornWrite,
    StaleRead,
    ReadError,
    FsyncFail,
    FsyncDelay,
}
