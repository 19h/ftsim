//! # ftsim-engine::node::runtime
//!
//! Contains the `Node` struct and its core logic for handling events.
//! The `Node` acts as a host for a `ProtocolDyn` instance, providing it with
//! the necessary context to interact with the simulation engine.

use super::timers::TimerWheel;
use crate::{
    events::FaultEventInternal,
    prelude::*,
    sim::EngineCtx,
    store::{Store, StoreFaultModel, StoreView},
};
use ftsim_proto::{FaultEvent, ProtocolDyn};

/// The operational status of a node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeStatus {
    /// The node is running normally.
    Up,
    /// The node is crashed and cannot process events.
    Down,
    /// The node is in a recovery state (e.g., warming up).
    Recovering,
}

/// Represents a single node in the simulated system.
pub struct Node {
    pub id: NodeId,
    pub status: NodeStatus,
    /// A logical clock skew applied to this node's perception of time.
    pub clock_skew_ns: i128,
    /// The protocol logic running on this node.
    proto: Box<dyn ProtocolDyn>,
    /// The persistent storage backend for this node.
    store: Box<dyn Store>,
    /// The fault model for this node's storage.
    store_faults: StoreFaultModel,
    /// The timer management system for this node.
    timers: TimerWheel,
    /// A list of peers this node can communicate with.
    peers: Vec<NodeId>,
    /// Flag indicating if Byzantine behaviors are enabled for this node.
    byzantine: bool,
}

impl Node {
    /// Creates a new node.
    pub fn new(id: NodeId, proto: Box<dyn ProtocolDyn>, store: Box<dyn Store>) -> Self {
        Self {
            id,
            status: NodeStatus::Up,
            clock_skew_ns: 0,
            proto,
            store,
            store_faults: StoreFaultModel::default(),
            timers: TimerWheel::new(),
            peers: Vec::new(),
            byzantine: false,
        }
    }

    /// Forwards the `init` call to the hosted protocol.
    pub fn init(&mut self, ctx: &mut dyn ProtoCtx) {
        self.proto.init(ctx);
    }

    /// Returns the protocol tag of the hosted protocol.
    pub fn proto_tag(&self) -> ProtoTag {
        self.proto.proto_tag()
    }

    /// Sets the list of peers for this node.
    pub fn set_peers(&mut self, peers: Vec<NodeId>) {
        self.peers = peers;
    }

    /// Returns a mutable view into the node's storage.
    pub fn store_view(&mut self) -> &mut dyn StoreView {
        self.store.as_view()
    }

    /// Returns a mutable reference to the node's storage fault model.
    pub fn store_faults(&mut self) -> &mut StoreFaultModel {
        &mut self.store_faults
    }

    /// Returns the number of active timers.
    pub fn timers_len(&self) -> usize {
        self.timers.active_timers()
    }

    /// Returns whether the node is in byzantine mode.
    pub fn byzantine(&self) -> bool {
        self.byzantine
    }

    /// Handles an incoming message delivery event.
    pub fn handle_message(&mut self, ctx: &mut EngineCtx, env: Envelope) {
        if self.status != NodeStatus::Up {
            tracing::debug!(node_id = self.id, msg_id = env.msg_id, "Message dropped, node is down");
            // TODO: Increment omission metric
            return;
        }

        // Dispatch to the protocol.
        if let Err(e) = self.proto.on_message(ctx, env.src, &env.payload) {
            tracing::error!(error = %e, "Protocol failed to handle message");
        }
    }

    /// Handles a timer firing event.
    pub fn handle_timer(&mut self, ctx: &mut EngineCtx, timer_id: TimerId) {
        if self.status != NodeStatus::Up {
            tracing::debug!(node_id = self.id, %timer_id, "Timer ignored, node is down");
            return;
        }

        // Check if the timer is still valid before dispatching.
        if self.timers.fire_timer(timer_id) {
            ::metrics::counter!(
                ftsim_types::metrics::MET_TIMER_FIRED,
                ftsim_types::metrics::LBL_NODE => self.id.to_string()
            ).increment(1);
            self.proto.on_timer(ctx, timer_id);
        }
    }

    /// Applies a fault to the node, changing its state.
    pub fn apply_fault(&mut self, ctx: &mut EngineCtx, f: FaultEventInternal) {
        match f {
            FaultEventInternal::Crash { .. } => {
                self.status = NodeStatus::Down;
                self.timers.clear(); // Drop all pending timers on crash
                self.proto.on_fault(ctx, FaultEvent::NodeCrashed);
            }
            FaultEventInternal::Restart { .. } => {
                self.status = NodeStatus::Up;
                // Re-initialize the protocol state
                self.proto.init(ctx);
                self.proto.on_fault(ctx, FaultEvent::NodeRecovered);
            }
            FaultEventInternal::ClockSkew { skew_ns, .. } => {
                self.clock_skew_ns = skew_ns;
                self.proto
                    .on_fault(ctx, FaultEvent::ClockSkewed { skew_ns });
            }
            FaultEventInternal::StoreFault { kind, .. } => {
                // The store fault model is already updated in sim.rs handle_fault
                // Now notify the protocol
                self.proto.on_fault(ctx, FaultEvent::StoreFaulted { kind });
            }
            FaultEventInternal::ByzantineFlip { enabled, .. } => {
                self.byzantine = enabled;
                self.proto.on_fault(ctx, FaultEvent::ByzantineEnabled(enabled));
            }
            // Other faults would be handled here.
            _ => {}
        }
    }

    /// Sets a new timer for this node.
    pub fn set_timer(&mut self, ctx: &mut EngineCtx, after: SimTime) -> TimerId {
        let fire_at = ctx.sim.now().saturating_add(after);
        let timer_id = ctx.sim.id_gen.next_timer_id();
        let event = Event::TimerFired {
            node_id: self.id,
            timer_id,
        };
        ctx.sim
            .schedule_at(fire_at, event, EventDiscriminant::timer(self.id));
        self.timers.add_timer(timer_id, timer_id); // EventId not needed for cancellation
        timer_id
    }

    /// Cancels a pending timer.
    pub fn cancel_timer(&mut self, timer_id: TimerId) -> bool {
        self.timers.cancel_timer(timer_id)
    }

    /// Returns the list of peers.
    pub fn peers(&self) -> &[NodeId] {
        &self.peers
    }
}
