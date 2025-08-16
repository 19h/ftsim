//! # ftsim-cli::wiring
//!
//! Contains the logic for instantiating and connecting all the components
//! of the simulator (engine, world, protocols, telemetry).

use ftsim_engine::{node::Node, prelude::*, store::MemStore, world::World};
use ftsim_proto::{
    api::boxed_dyn,
    protocols::{primary_backup::PrimaryBackup, raft_lite::RaftLite},
};
use rand::Rng;

type ProtoFactory = fn() -> Box<dyn ProtocolDyn>;

/// The central registry of all available protocols.
static REGISTRY: &[(&'static str, ProtoTag, ProtoFactory)] = &[
    (
        "raft_lite",
        ProtoTag(1),
        || boxed_dyn(RaftLite::default()),
    ),
    (
        "primary_backup",
        ProtoTag(2),
        || boxed_dyn(PrimaryBackup::new()),
    ),
];

/// Finds a protocol factory in the registry by its tag.
pub fn get_proto_factory(tag: ProtoTag) -> Option<ProtoFactory> {
    REGISTRY
        .iter()
        .find(|(_, t, _)| *t == tag)
        .map(|(_, _, factory)| *factory)
}

/// Returns the entire protocol registry.
pub fn get_registry() -> &'static [(&'static str, ProtoTag, ProtoFactory)] {
    REGISTRY
}

/// Constructs the initial `World` state from a scenario.
pub fn build_world(scenario: &Scenario) -> anyhow::Result<World> {
    let factory = get_proto_factory(scenario.initial.proto)
        .ok_or_else(|| anyhow::anyhow!("Protocol with tag {:?} not found", scenario.initial.proto))?;

    let nodes = (0..scenario.initial.nodes)
        .map(|i| {
            let proto = factory();
            // For now, all nodes get a simple in-memory store.
            // A more advanced setup could configure this from the scenario.
            let store = Box::new(MemStore::new());
            Node::new(i as NodeId, proto, store)
        })
        .collect();

    let net = Net::from_topology(scenario.initial.nodes, &scenario.topology);

    Ok(World { nodes, net })
}

/// Performs final setup on the world after construction, like populating
/// peer lists.
pub fn finalize_world_setup(world: &mut World) {
    let all_node_ids: Vec<NodeId> = (0..world.nodes.len() as NodeId).collect();
    for node_id in all_node_ids {
        let peers: Vec<NodeId> = world.net.peers_of(node_id).collect();
        world.nodes[node_id as usize].set_peers(peers);
    }
}

/// Generates a seed if one is not provided.
pub fn get_seed(opts_seed: Option<u64>, scenario_seed: Option<u64>) -> u64 {
    opts_seed
        .or(scenario_seed)
        .unwrap_or_else(|| rand::thread_rng().gen())
}
