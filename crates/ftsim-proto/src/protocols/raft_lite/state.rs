//! # ftsim-proto::protocols::raft_lite::state
//!
//! Defines the core state machine for the RaftLite protocol.

use ftsim_types::id::NodeId;
use std::collections::{BTreeMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Follower,
    Candidate,
    Leader,
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Represents a single entry in the Raft log.
#[derive(Clone, Debug)]
pub struct LogEntry {
    pub term: u64,
    pub command: Vec<u8>,
}

/// The persistent and volatile state for a Raft node.
pub struct State {
    // --- Persistent state on all servers ---
    pub id: NodeId,
    pub peers: Vec<NodeId>,
    pub current_term: u64,
    pub voted_for: Option<NodeId>,
    pub log: Vec<LogEntry>,

    // --- Volatile state on all servers ---
    pub role: Role,
    pub commit_index: u64,
    pub last_applied: u64,

    // --- Volatile state on leaders ---
    pub next_index: BTreeMap<NodeId, u64>,
    pub match_index: BTreeMap<NodeId, u64>,

    // --- Volatile state on candidates ---
    pub votes_received: HashSet<NodeId>,
}

impl State {
    pub fn new() -> Self {
        Self {
            id: 0,
            peers: Vec::new(),
            current_term: 0,
            voted_for: None,
            log: vec![],
            role: Role::Follower,
            commit_index: 0,
            last_applied: 0,
            next_index: BTreeMap::new(),
            match_index: BTreeMap::new(),
            votes_received: HashSet::new(),
        }
    }

    pub fn quorum(&self) -> usize {
        (self.peers.len() + 1) / 2 + 1
    }

    pub fn last_log_index(&self) -> u64 {
        self.log.len() as u64
    }

    pub fn last_log_term(&self) -> u64 {
        self.log.last().map_or(0, |entry| entry.term)
    }
}
