//! # ftsim-tui::ui::layout
//!
//! Defines the `ratatui` layout structures for the TUI.

use ratatui::prelude::*;
use std::rc::Rc;

/// Creates the main layout with four vertical chunks.
pub fn create_main_layout(area: Rect) -> Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),      // Status bar
            Constraint::Percentage(40), // Cluster Graph
            Constraint::Percentage(30), // Middle row (Status + Metrics)
            Constraint::Min(10),        // Logs / Timeline
        ])
        .split(area)
}

/// Creates the middle layout with two horizontal chunks.
pub fn create_middle_layout(area: Rect) -> Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Node Status Grid
            Constraint::Percentage(50), // Metrics Panel
        ])
        .split(area)
}
