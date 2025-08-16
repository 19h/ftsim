//! # ftsim-engine::world
//!
//! Defines the `World` struct, which is the top-level container for the
//! simulation's state, including all nodes and the network that connects them.

use crate::{net::Net, node::Node, prelude::*};

/// Represents the entire state of the simulated distributed system.
pub struct World {
    pub nodes: Vec<Node>,
    pub net: Net,
}

impl World {
    /// Creates an empty world (primarily for testing).
    #[cfg(test)]
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            net: Net::from_topology(0, &TopologySpec::FullMesh),
        }
    }

    /// Returns a reference to a node by its ID. Panics if the ID is invalid.
    pub fn node(&self, id: NodeId) -> &Node {
        &self.nodes[id as usize]
    }

    /// Returns a mutable reference to a node by its ID. Panics if the ID is invalid.
    pub fn node_mut(&mut self, id: NodeId) -> &mut Node {
        &mut self.nodes[id as usize]
    }
}
