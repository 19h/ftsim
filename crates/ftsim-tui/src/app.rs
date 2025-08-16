//! # ftsim-tui::app
//!
//! Defines the `App` struct, which holds the state for the TUI.

use ftsim_engine::{control::ControlMsg, telemetry::snapshot::Snapshot, prelude::NodeId};

/// Represents the state of the TUI application.
pub struct App {
    /// The most recently received snapshot of the simulation state.
    pub snapshot: Option<Snapshot>,
    /// Whether the help screen is visible.
    pub show_help: bool,
    /// Whether the simulation is paused.
    pub is_paused: bool,
    /// Whether log filtering is enabled.
    pub filter_logs: bool,
    /// Current focused panel index.
    pub focused_panel: usize,
    /// Channel to send control messages to the simulation engine.
    control_tx: crossbeam_channel::Sender<ControlMsg>,
    /// Selected node for operations (kill, restart, etc.).
    pub selected_node: Option<NodeId>,
    // Add other UI state here, e.g., scroll positions, etc.
}

impl App {
    pub fn new(control_tx: crossbeam_channel::Sender<ControlMsg>) -> Self {
        Self {
            snapshot: None,
            show_help: false,
            is_paused: false,
            filter_logs: false,
            focused_panel: 0,
            control_tx,
            selected_node: None,
        }
    }

    /// Called on every UI tick.
    pub fn on_tick(&mut self) {}

    /// Updates the app's state with a new snapshot from the engine.
    pub fn update_snapshot(&mut self, snapshot: Snapshot) {
        self.snapshot = Some(snapshot);
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn toggle_pause(&mut self) {
        self.is_paused = !self.is_paused;
        let msg = if self.is_paused {
            ControlMsg::Pause
        } else {
            ControlMsg::Resume
        };
        if let Err(e) = self.control_tx.send(msg) {
            eprintln!("Failed to send control message: {}", e);
        }
    }

    pub fn single_step(&mut self) {
        if let Err(e) = self.control_tx.send(ControlMsg::Step) {
            eprintln!("Failed to send step message: {}", e);
        }
    }

    pub fn inject_partition(&mut self) {
        // For demo: partition nodes 0,1 from nodes 2,3 (if they exist)
        // In a real implementation, this would use UI to select partition sets
        if let Some(snapshot) = &self.snapshot {
            let num_nodes = snapshot.nodes.len();
            if num_nodes >= 2 {
                let mid = num_nodes / 2;
                let set1: Vec<NodeId> = (0..mid as u32).collect();
                let set2: Vec<NodeId> = (mid as u32..num_nodes as u32).collect();
                
                if let Err(e) = self.control_tx.send(ControlMsg::InjectPartition {
                    sets: vec![set1, set2],
                }) {
                    eprintln!("Failed to send partition message: {}", e);
                }
            }
        }
    }

    pub fn kill_node(&mut self) {
        // Kill the selected node, or node 0 if none selected
        let node_id = self.selected_node.unwrap_or(0);
        if let Err(e) = self.control_tx.send(ControlMsg::KillNode(node_id)) {
            eprintln!("Failed to send kill node message: {}", e);
        }
    }

    pub fn restart_node(&mut self) {
        // Restart the selected node, or node 0 if none selected
        let node_id = self.selected_node.unwrap_or(0);
        if let Err(e) = self.control_tx.send(ControlMsg::RestartNode(node_id)) {
            eprintln!("Failed to send restart node message: {}", e);
        }
    }

    pub fn toggle_filter_logs(&mut self) {
        self.filter_logs = !self.filter_logs;
        // TODO: Implement log filtering UI
        eprintln!("Log filtering {}", if self.filter_logs { "enabled" } else { "disabled" });
    }

    pub fn cycle_focus(&mut self) {
        // Cycle through available panels (adjust max value based on number of panels)
        self.focused_panel = (self.focused_panel + 1) % 4;
        eprintln!("Focus moved to panel {}", self.focused_panel);
    }
}
