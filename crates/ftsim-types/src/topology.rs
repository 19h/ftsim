//! # ftsim-types::topology
//!
//! Defines the declarative specifications for network topologies.
//! The engine uses these specifications to construct the initial network graph.

use crate::id::NodeId;
use serde::{Deserialize, Serialize};

/// An enum representing different ways to specify the network graph.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub enum TopologySpec {
    /// Every node is connected to every other node.
    FullMesh,
    /// Nodes are connected in a ring: 0-1, 1-2, ..., (N-1)-0.
    Ring,
    /// All nodes connect to a central hub node.
    Star { hub: NodeId },
    /// A k-ary tree structure.
    KaryTree { k: usize },
    /// A graph defined by an explicit list of directed edges.
    FromEdges { edges: Vec<(NodeId, NodeId)> },
    /// A random graph where each possible edge is created with probability `p`.
    ErdosRenyi { p: f64 },
}
