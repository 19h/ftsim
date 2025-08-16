//! # ftsim-engine::control
//!
//! Defines control messages that can be sent from the TUI to the simulation engine.

use crate::prelude::*;

/// Control messages sent from the TUI to the simulation engine.
#[derive(Debug, Clone)]
pub enum ControlMsg {
    /// Pause the simulation execution.
    Pause,
    /// Resume simulation execution.
    Resume,
    /// Execute a single step (process one event).
    Step,
    /// Kill a specific node.
    KillNode(NodeId),
    /// Restart a specific node.
    RestartNode(NodeId),
    /// Inject a network partition.
    InjectPartition {
        /// Sets of nodes that can communicate within each set but not across sets.
        sets: Vec<Vec<NodeId>>,
    },
    /// Heal all network partitions.
    HealPartition,
    /// Adjust simulation speed (1.0 = normal, 0.5 = half speed, 2.0 = double speed).
    SetSpeed(f32),
}

/// The state of simulation execution control.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimulationState {
    /// Simulation is running normally.
    Running,
    /// Simulation is paused.
    Paused,
    /// Simulation is stepping (will pause after next event).
    Stepping,
    /// Simulation has completed.
    Completed,
}
