//! # ftsim-engine::prelude
//!
//! A convenience module that re-exports the most commonly used types from the
//! engine and its dependencies. This simplifies imports for other crates in the
//! workspace that depend on the engine.

pub use crate::{
    events::{Event, EventDiscriminant, Queued},
    net::{Net, NetLink},
    node::{Node, NodeStatus},
    sim::Simulation,
    store::{FaultyStoreView, MemStore, Store, StoreFaultModel, StoreView},
    telemetry::{snapshot::Snapshot, TelemetryBus},
    world::World,
};

pub use ftsim_types::{
    self, config::*, envelope::*, errors::*, id::*, metrics::*, scenario::*, time::*, topology::*,
};

pub use ftsim_proto::{self, api::*, ctx_ext::*, FaultEvent, Protocol, ProtocolDyn};
