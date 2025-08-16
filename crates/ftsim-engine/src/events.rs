//! # ftsim-engine::events
//!
//! Defines the core `Event` enum and the `Queued` wrapper struct.
//! The `Event` enum represents all possible state transitions in the simulation.
//! The `Queued` struct wraps an `Event` with its scheduled time and an
//! insertion sequence number for deterministic tie-breaking, making it suitable
//! for the `BinaryHeap` used as a priority queue.

use crate::prelude::*;
use std::cmp::Ordering;

/// A discriminant to ensure stable tie-breaking in the event queue.
/// The tuple is (event_type_priority, source_node_id).
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EventDiscriminant(u8, NodeId);

impl EventDiscriminant {
    pub fn new(kind: u8, node: NodeId) -> Self {
        Self(kind, node)
    }
    pub fn fault() -> Self {
        Self(0, u32::MAX)
    } // Faults have highest priority
    pub fn timer(src: NodeId) -> Self {
        Self(1, src)
    }
    pub fn delivery(src: NodeId) -> Self {
        Self(2, src)
    }
    pub fn ui() -> Self {
        Self(255, u32::MAX)
    } // UI ticks have lowest priority
}

/// Represents all possible events that can be scheduled in the simulation.
#[derive(Debug)]
pub enum Event {
    /// Deliver a network message to a destination node.
    Deliver { env: Envelope, link_id: LinkId },
    /// A timer set by a protocol has fired.
    TimerFired { node_id: NodeId, timer_id: TimerId },
    /// A fault injection event scheduled by the scenario runner.
    Fault(FaultEventInternal),
    /// A periodic tick to generate a snapshot for the TUI.
    UiSnapshotTick,
}

/// A wrapper for an `Event` that includes scheduling information.
/// This is the type stored in the simulation's priority queue.
#[derive(Debug)]
pub struct Queued<T> {
    pub id: EventId,
    pub time: SimTime,
    /// A monotonic sequence number to ensure stable ordering for events
    /// scheduled at the exact same time.
    pub insert_seq: u64,
    pub discriminant: EventDiscriminant,
    pub payload: T,
}

impl<T> Queued<T> {
    pub fn new(
        id: EventId,
        time: SimTime,
        insert_seq: u64,
        discriminant: EventDiscriminant,
        payload: T,
    ) -> Self {
        Self {
            id,
            time,
            insert_seq,
            discriminant,
            payload,
        }
    }
}

// The following implementations are crucial for the `BinaryHeap` to function
// as a min-heap and to maintain deterministic ordering.

impl<T> PartialEq for Queued<T> {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
            && self.insert_seq == other.insert_seq
            && self.discriminant == other.discriminant
    }
}

impl<T> Eq for Queued<T> {}

impl<T> PartialOrd for Queued<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Queued<T> {
    /// Compares events for the priority queue.
    /// `BinaryHeap` is a max-heap, so we reverse the ordering to make it a min-heap.
    /// The primary sort key is `time` (earlier is greater).
    /// The secondary sort key is `insert_seq` (earlier is greater).
    /// The tertiary sort key is `discriminant` for stable tie-breaking.
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .time
            .cmp(&self.time)
            .then_with(|| other.insert_seq.cmp(&self.insert_seq))
            .then_with(|| other.discriminant.cmp(&self.discriminant))
    }
}

/// Represents specific changes to a link's fault model.
#[derive(Debug, Clone)]
pub enum LinkModelChange {
    SetDelay(ftsim_types::scenario::DelaySpec),
    SetDrop(f64),
    SetDuplicate(f64),
    SetCorrupt(f64),
}

/// Internal representation of fault events, distinct from the `FaultEvent`
/// exposed to protocols. These map directly to actions on the engine's state.
#[derive(Debug, Clone)]
pub enum FaultEventInternal {
    Crash {
        node_id: NodeId,
        duration: SimTime,
    },
    Restart {
        node_id: NodeId,
    },
    Partition {
        sets: Vec<Vec<NodeId>>,
    },
    HealPartition,
    LinkModelUpdate {
        link_id: LinkId,
        change: LinkModelChange,
    },
    ClockSkew {
        node_id: NodeId,
        skew_ns: i128,
    },
    StoreFault {
        node_id: NodeId,
        kind: StoreFaultKind,
        rate: f64,
    },
    ByzantineFlip {
        node_id: NodeId,
        enabled: bool,
    },
    BroadcastBytes {
        payload_hex: String,
        proto_tag: Option<ProtoTag>,
    },
    Custom {
        name: String,
        args: toml::Value,
    },
}
