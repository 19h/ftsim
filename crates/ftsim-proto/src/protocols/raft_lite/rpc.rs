//! # ftsim-proto::protocols::raft_lite::rpc
//!
//! Defines the structs for Raft's Remote Procedure Calls (RPCs), which are
//! serialized as messages.

use ftsim_types::id::NodeId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RequestVote {
    pub term: u64,
    pub candidate_id: NodeId,
    pub last_log_index: u64,
    pub last_log_term: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RequestVoteReply {
    pub term: u64,
    pub vote_granted: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppendEntries {
    pub term: u64,
    pub leader_id: NodeId,
    // In a real implementation, this would contain log entries.
    // Simplified for this example.
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppendEntriesReply {
    pub term: u64,
    pub success: bool,
}
