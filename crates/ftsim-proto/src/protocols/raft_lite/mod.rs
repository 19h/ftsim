//! # ftsim-proto::protocols::raft_lite
//!
//! A simplified implementation of the Raft consensus algorithm.
//! It focuses on leader election and log replication to demonstrate a more
//! complex protocol using the FTSim SDK.

use super::super::{Ctx, FaultEvent, Protocol};
use ftsim_types::{
    envelope::ProtoTag,
    id::{NodeId, TimerId},
    time::sim_from_ms,
};
use serde::{Deserialize, Serialize};

mod logic;
mod rpc;
mod state;

use rpc::{AppendEntries, AppendEntriesReply, RequestVote, RequestVoteReply};
use state::{Role, State};

const TAG: ProtoTag = ProtoTag(1);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Message {
    RequestVote(RequestVote),
    RequestVoteReply(RequestVoteReply),
    AppendEntries(AppendEntries),
    AppendEntriesReply(AppendEntriesReply),
}

pub struct RaftLite {
    state: State,
    election_timer: Option<TimerId>,
}

impl Default for RaftLite {
    fn default() -> Self {
        Self {
            state: State::new(),
            election_timer: None,
        }
    }
}

impl Protocol<Message> for RaftLite {
    fn name(&self) -> &'static str {
        "raft_lite"
    }

    fn proto_tag(&self) -> ProtoTag {
        TAG
    }

    fn init(&mut self, ctx: &mut Ctx<Message>) {
        self.state.id = ctx.node_id();
        // Assume 5 nodes for now. A real implementation would get this from config.
        self.state.peers = (0..5).filter(|&i| i != self.state.id).collect();
        self.state.role = Role::Follower;
        self.reset_election_timer(ctx);
        ctx.log_kv("role", "follower");
        ctx.log_kv("term", &self.state.current_term.to_string());
    }

    fn on_message(&mut self, ctx: &mut Ctx<Message>, src: NodeId, msg: Message) {
        match msg {
            Message::RequestVote(args) => logic::handle_request_vote(self, ctx, src, args),
            Message::RequestVoteReply(reply) => {
                logic::handle_request_vote_reply(self, ctx, src, reply)
            }
            Message::AppendEntries(args) => logic::handle_append_entries(self, ctx, src, args),
            Message::AppendEntriesReply(reply) => {
                logic::handle_append_entries_reply(self, ctx, src, reply)
            }
        }
        // Update TUI-visible state
        ctx.log_kv("term", &self.state.current_term.to_string());
        ctx.log_kv("role", &self.state.role.to_string());
    }

    fn on_timer(&mut self, ctx: &mut Ctx<Message>, timer: TimerId) {
        if self.election_timer == Some(timer) {
            logic::handle_election_timeout(self, ctx);
        }
    }

    fn on_fault(&mut self, _ctx: &mut Ctx<Message>, _fault: FaultEvent) {
        // Raft is designed to be resilient to these, so often no special
        // handling is needed, but we could log the event.
        tracing::info!("Raft node received a fault notification.");
    }
}

impl RaftLite {
    /// Resets the election timer to a new random duration.
    fn reset_election_timer(&mut self, ctx: &mut Ctx<Message>) {
        if let Some(timer) = self.election_timer.take() {
            ctx.cancel_timer(timer);
        }
        // Use the deterministic RNG for election timeouts.
        let timeout_ms = 150 + (ctx.rng_u64() % 151); // Raft's recommended 150-300ms
        let timer = ctx.set_timer(sim_from_ms(timeout_ms));
        self.election_timer = Some(timer);
    }

    /// Converts the node to a follower state.
    fn become_follower(&mut self, ctx: &mut Ctx<Message>, term: u64) {
        self.state.current_term = term;
        self.state.role = Role::Follower;
        self.state.voted_for = None;
        self.reset_election_timer(ctx);
    }
}
