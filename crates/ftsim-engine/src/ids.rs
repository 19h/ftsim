//! # ftsim-engine::ids
//!
//! Provides a generator for unique, monotonic IDs for events, messages, and timers.

use crate::prelude::*;

/// A generator for various kinds of simulation IDs.
pub struct IdGen {
    event_id: EventId,
    msg_id: u64,
    timer_id: TimerId,
    /// Used for deterministic tie-breaking in the event queue.
    insertion_seq: u64,
}

impl IdGen {
    pub fn new() -> Self {
        Self {
            event_id: 0,
            msg_id: 0,
            timer_id: 0,
            insertion_seq: 0,
        }
    }

    pub fn next_event_id(&mut self) -> EventId {
        let id = self.event_id;
        self.event_id = self.event_id.checked_add(1).expect("EventId overflow");
        id
    }

    pub fn next_msg_id(&mut self) -> u64 {
        let id = self.msg_id;
        self.msg_id = self.msg_id.checked_add(1).expect("MsgId overflow");
        id
    }

    pub fn next_timer_id(&mut self) -> TimerId {
        let id = self.timer_id;
        self.timer_id = self.timer_id.checked_add(1).expect("TimerId overflow");
        id
    }

    pub fn next_insertion_seq(&mut self) -> u64 {
        let id = self.insertion_seq;
        self.insertion_seq = self
            .insertion_seq
            .checked_add(1)
            .expect("InsertionSeq overflow");
        id
    }
}
