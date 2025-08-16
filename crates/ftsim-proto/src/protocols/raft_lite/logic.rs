//! # ftsim-proto::protocols::raft_lite::logic
//!
//! Contains the business logic for handling Raft RPCs and timeouts.

use super::{rpc::*, state::Role, Message, RaftLite};
use crate::Ctx;
use ftsim_types::id::NodeId;

pub fn handle_election_timeout(raft: &mut RaftLite, ctx: &mut Ctx<Message>) {
    if raft.state.role == Role::Leader {
        return;
    }

    tracing::info!("Election timeout, starting new election");

    // Transition to Candidate
    raft.state.role = Role::Candidate;
    raft.state.current_term += 1;
    raft.state.voted_for = Some(raft.state.id);
    raft.state.votes_received.clear();
    raft.state.votes_received.insert(raft.state.id);

    // Reset timer for the new election
    raft.reset_election_timer(ctx);

    // Send RequestVote RPCs to all peers
    let args = RequestVote {
        term: raft.state.current_term,
        candidate_id: raft.state.id,
        last_log_index: raft.state.last_log_index(),
        last_log_term: raft.state.last_log_term(),
    };

    ctx.broadcast(&Message::RequestVote(args), None).ok();
}

pub fn handle_request_vote(
    raft: &mut RaftLite,
    ctx: &mut Ctx<Message>,
    src: NodeId,
    args: RequestVote,
) {
    if args.term > raft.state.current_term {
        raft.become_follower(ctx, args.term);
    }

    let mut vote_granted = false;
    if args.term == raft.state.current_term {
        if raft.state.voted_for.is_none() || raft.state.voted_for == Some(args.candidate_id) {
            // Simplified log check: a real implementation would be more rigorous.
            if args.last_log_term >= raft.state.last_log_term() {
                vote_granted = true;
                raft.state.voted_for = Some(args.candidate_id);
            }
        }
    }

    let reply = RequestVoteReply {
        term: raft.state.current_term,
        vote_granted,
    };
    ctx.send(src, &Message::RequestVoteReply(reply)).ok();
}

pub fn handle_request_vote_reply(
    raft: &mut RaftLite,
    ctx: &mut Ctx<Message>,
    src: NodeId,
    reply: RequestVoteReply,
) {
    if reply.term > raft.state.current_term {
        raft.become_follower(ctx, reply.term);
        return;
    }

    if raft.state.role == Role::Candidate
        && reply.term == raft.state.current_term
        && reply.vote_granted
    {
        raft.state.votes_received.insert(src);
        if raft.state.votes_received.len() >= raft.state.quorum() {
            become_leader(raft, ctx);
        }
    }
}

pub fn handle_append_entries(
    raft: &mut RaftLite,
    ctx: &mut Ctx<Message>,
    src: NodeId,
    args: AppendEntries,
) {
    if args.term > raft.state.current_term {
        raft.become_follower(ctx, args.term);
    }

    let mut success = false;
    if args.term == raft.state.current_term {
        success = true;
        // This is where a follower would append entries to its log.
        // Since this is a heartbeat, we just reset the timer.
        raft.reset_election_timer(ctx);
    }

    let reply = AppendEntriesReply {
        term: raft.state.current_term,
        success,
    };
    ctx.send(src, &Message::AppendEntriesReply(reply)).ok();
}

pub fn handle_append_entries_reply(
    _raft: &mut RaftLite,
    _ctx: &mut Ctx<Message>,
    _src: NodeId,
    _reply: AppendEntriesReply,
) {
    // Logic to update next_index and match_index for the follower would go here.
}

fn become_leader(raft: &mut RaftLite, ctx: &mut Ctx<Message>) {
    tracing::info!(term = raft.state.current_term, "Elected as leader");
    raft.state.role = Role::Leader;

    // Stop the election timer, leaders don't need it.
    if let Some(timer) = raft.election_timer.take() {
        ctx.cancel_timer(timer);
    }

    // Initialize leader state
    let last_log_index = raft.state.last_log_index();
    raft.state.next_index = raft
        .state
        .peers
        .iter()
        .map(|&id| (id, last_log_index + 1))
        .collect();
    raft.state.match_index = raft.state.peers.iter().map(|&id| (id, 0)).collect();

    // Send initial empty AppendEntries (heartbeat) to all peers
    let args = AppendEntries {
        term: raft.state.current_term,
        leader_id: raft.state.id,
    };
    ctx.broadcast(&Message::AppendEntries(args), None).ok();
    // A real leader would have a heartbeat timer.
}
