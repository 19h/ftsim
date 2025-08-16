//! # ftsim-engine::net
//!
//! The network subsystem. This module is responsible for modeling the network
//! topology, applying fault models to messages, and scheduling their delivery
//! as `Event::Deliver` events in the simulation's main queue.

use crate::{
    events::{Event, EventDiscriminant},
    prelude::*,
    sim::EngineCtx,
};
use fxhash::FxHashMap;
use petgraph::{
    graph::{EdgeIndex, NodeIndex},
    Directed, Graph,
};

mod faults;
mod link;

pub use faults::sample_delay;
pub use link::{LinkFaultModel, NetLink};

/// Represents a node in the network graph.
#[derive(Default, Debug)]
pub struct NetNode {
    pub id: NodeId,
}

/// The main network state container.
pub struct Net {
    /// The graph structure of the network. Edge weights are empty as link
    /// properties are stored in the `links` map for stable access.
    pub graph: Graph<NetNode, (), Directed>,
    /// A map from our stable `LinkId` to the full link properties.
    pub links: FxHashMap<LinkId, NetLink>,
    /// A map from our `NodeId` to petgraph's volatile `NodeIndex`.
    node_indices: Vec<NodeIndex>,
    /// A map from our stable `LinkId` to petgraph's volatile `EdgeIndex`.
    link_index: FxHashMap<LinkId, EdgeIndex>,
    link_id_counter: LinkId,
}

impl Net {
    /// Creates a new network from a topology specification.
    pub fn from_topology(num_nodes: usize, spec: &TopologySpec) -> Self {
        let mut graph = Graph::new();
        let node_indices: Vec<NodeIndex> = (0..num_nodes)
            .map(|i| graph.add_node(NetNode { id: i as NodeId }))
            .collect();

        let mut net = Self {
            graph,
            links: FxHashMap::default(),
            node_indices,
            link_index: FxHashMap::default(),
            link_id_counter: 0,
        };

        let edges = match spec {
            TopologySpec::FullMesh => {
                let mut edges = Vec::new();
                for i in 0..num_nodes {
                    for j in 0..num_nodes {
                        if i != j {
                            edges.push((i as NodeId, j as NodeId));
                        }
                    }
                }
                edges
            }
            // Other topologies would be implemented here.
            _ => unimplemented!("This topology is not yet supported"),
        };

        for (src, dst) in edges {
            net.add_link(src, dst, LinkFaultModel::default());
        }

        net
    }

    fn add_link(&mut self, src: NodeId, dst: NodeId, faults: LinkFaultModel) {
        let id = self.link_id_counter;
        self.link_id_counter += 1;
        let link = NetLink { id, src, dst, faults };
        let edge_index = self.graph.add_edge(
            self.node_indices[src as usize],
            self.node_indices[dst as usize],
            (),
        );
        self.links.insert(id, link);
        self.link_index.insert(id, edge_index);
    }

    /// Returns an iterator over the peer IDs of a given node.
    pub fn peers_of(&self, nid: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        let idx = self.node_indices[nid as usize];
        self.graph.neighbors(idx).map(move |i| self.graph[i].id)
    }

    /// Processes an outgoing message from a node, applies the relevant link
    /// fault model, and schedules 0 or more `Deliver` events.
    pub fn send(&mut self, ctx: &mut EngineCtx, env: Envelope) {
        // Find the link ID based on src/dst
        let link_id = self
            .links
            .values()
            .find(|l| l.src == env.src && l.dst == env.dst)
            .map(|l| l.id);

        if let Some(link_id) = link_id {
            let link = self.links.get(&link_id).unwrap();

            // --- Apply Fault Model ---
            if link.faults.partitioned {
                tracing::debug!(msg_id = env.msg_id, "Message dropped due to partition");
                ::metrics::counter!(
                    ftsim_types::metrics::MET_NET_MSG_DROPPED,
                    ftsim_types::metrics::LBL_REASON => "partition",
                    ftsim_types::metrics::LBL_SRC => env.src.to_string(),
                    ftsim_types::metrics::LBL_DST => env.dst.to_string()
                ).increment(1);
                return;
            }

            if faults::trial(ctx.rng("net.drop"), &link.faults.drop) {
                tracing::debug!(msg_id = env.msg_id, "Message dropped by fault model");
                ::metrics::counter!(
                    ftsim_types::metrics::MET_NET_MSG_DROPPED,
                    ftsim_types::metrics::LBL_REASON => "drop_probability",
                    ftsim_types::metrics::LBL_SRC => env.src.to_string(),
                    ftsim_types::metrics::LBL_DST => env.dst.to_string()
                ).increment(1);
                return;
            }

            let base_delay = sample_delay(ctx.rng("net.delay.base"), &link.faults.base_delay);
            let jitter = sample_delay(ctx.rng("net.delay.jitter"), &link.faults.jitter);
            let total_delay = base_delay + jitter;
            let delivery_time = ctx.sim.now() + total_delay;

            let deliver_event = Event::Deliver {
                env: env.clone(),
                link_id,
            };
            // Use SOURCE node for tie-breaking
            let discriminant = EventDiscriminant::delivery(env.src);
            ctx.sim
                .schedule_at(delivery_time, deliver_event, discriminant);

            // Handle duplication
            if faults::trial(ctx.rng("net.duplicate"), &link.faults.duplicate) {
                tracing::debug!(msg_id = env.msg_id, "Message duplicated by fault model");
                let dup_delay = sample_delay(ctx.rng("net.delay.dup"), &link.faults.base_delay);
                let dup_delivery_time = ctx.sim.now() + dup_delay;
                let dup_event = Event::Deliver { env, link_id };
                ctx.sim
                    .schedule_at(dup_delivery_time, dup_event, discriminant);
            }
        }
    }

    pub fn set_partition(&mut self, sets: Vec<Vec<NodeId>>) {
        // A simple implementation: for any two nodes in different sets,
        // mark the link between them as partitioned.
        for link in self.links.values_mut() {
            let src_set = sets.iter().find(|s| s.contains(&link.src));
            let dst_set = sets.iter().find(|s| s.contains(&link.dst));

            if let (Some(s1), Some(s2)) = (src_set, dst_set) {
                // If the pointers are not equal, they are in different sets.
                if !std::ptr::eq(s1, s2) {
                    link.faults.partitioned = true;
                }
            }
        }
    }

    pub fn heal_partition(&mut self) {
        for link in self.links.values_mut() {
            link.faults.partitioned = false;
        }
    }
}
