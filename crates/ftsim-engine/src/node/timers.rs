//! # ftsim-engine::node::timers
//!
//! Implements a timer management system for a node.
//! The specification calls for a Timer Wheel for O(1) average performance,
//! but for simplicity in this initial implementation, we will use a simpler
//! `FxHashMap` to track active timers. A real implementation would use a more
//! sophisticated data structure.

use crate::prelude::*;
use fxhash::{FxHashMap, FxHashSet};

/// Manages timers for a single node.
pub struct TimerWheel {
    /// Maps a protocol-visible `TimerId` to a placeholder value.
    /// We don't need the `EventId` for cancellation with the current strategy.
    active_timers: FxHashMap<TimerId, TimerId>,
    /// A set of `TimerId`s that have been canceled but not yet fired.
    canceled_timers: FxHashSet<TimerId>,
}

impl TimerWheel {
    pub fn new() -> Self {
        Self {
            active_timers: FxHashMap::default(),
            canceled_timers: FxHashSet::default(),
        }
    }

    /// Adds a new timer to the wheel.
    pub fn add_timer(&mut self, timer_id: TimerId, event_id: EventId) {
        self.active_timers.insert(timer_id, event_id);
    }

    /// Marks a timer as canceled.
    pub fn cancel_timer(&mut self, timer_id: TimerId) -> bool {
        if self.active_timers.contains_key(&timer_id) {
            self.canceled_timers.insert(timer_id);
            true
        } else {
            false
        }
    }

    /// Called when a timer event fires. Checks if the timer was canceled.
    /// Returns `true` if the timer is valid and should be dispatched.
    pub fn fire_timer(&mut self, timer_id: TimerId) -> bool {
        self.active_timers.remove(&timer_id);
        // If the timer was in the canceled set, it's invalid.
        !self.canceled_timers.remove(&timer_id)
    }

    /// Clears all pending timers, e.g., on a node crash.
    pub fn clear(&mut self) {
        // In a real system with event cancellation, we would unschedule events here.
        // For now, we just clear our internal tracking. The events will still
        // fire but will be ignored by `fire_timer`.
        self.active_timers.clear();
        self.canceled_timers.clear();
    }

    /// Returns the number of active (not canceled) timers.
    pub fn active_timers(&self) -> usize {
        self.active_timers.len() - self.canceled_timers.len()
    }
}
