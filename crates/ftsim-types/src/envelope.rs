//! # ftsim-types::envelope
//!
//! Defines the `Envelope`, the fundamental wrapper for all messages exchanged
//! between nodes in the simulation. It contains not only the protocol payload
//! but also essential metadata for routing, tracing, and fault injection.

use crate::{
    id::{NodeId},
    time::SimTime,
};
use bytes::Bytes;

/// A unique tag identifying the protocol namespace for a message.
/// This allows multiple protocols to run on the same node without interference.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ProtoTag(pub u16);

/// A wrapper for all messages sent over the simulated network.
///
/// Invariants:
/// - `src != dst` unless loopback is explicitly allowed by the network model.
/// - `payload.len() <= MAX_MSG_BYTES` (enforced by the network layer).
#[derive(Clone, Debug)]
pub struct Envelope {
    /// The ID of the sending node.
    pub src: NodeId,
    /// The ID of the destination node.
    pub dst: NodeId,
    /// The tag identifying the protocol this message belongs to.
    pub proto_tag: ProtoTag,
    /// The protocol-specific payload, serialized into raw bytes.
    pub payload: Bytes,
    /// A unique, deterministically-assigned ID for this message instance.
    pub msg_id: u64,
    /// The simulation time when this message was created.
    pub create_time: SimTime,
    /// An ID used to correlate related events (e.g., a request and its response)
    /// for observability and debugging.
    pub trace_id: u64,
}
