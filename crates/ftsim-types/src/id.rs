//! # ftsim-types::id
//!
//! Defines the core identifier types used throughout the simulation.
//! Using distinct types for different kinds of IDs helps prevent bugs where,
//! for example, a `NodeId` might be accidentally used as a `LinkId`.

/// A unique identifier for a node in the simulation.
/// Invariant: Initially spawned nodes MUST have contiguous IDs from 0 to N-1.
pub type NodeId = u32;

/// A unique identifier for a directed link between two nodes.
pub type LinkId = u64;

/// A unique identifier for a timer set by a protocol.
pub type TimerId = u64;

/// A unique identifier for a scheduled event in the simulation's master queue.
pub type EventId = u64;
