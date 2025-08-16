//! # ftsim-tui::theme
//!
//! Defines the color palette and styling constants for the TUI to ensure a
//! consistent and aesthetically pleasing look.

use ratatui::style::{Color, Style};

pub const BORDER_STYLE: Style = Style::new().fg(Color::DarkGray);
pub const FOCUSED_BORDER_STYLE: Style = Style::new().fg(Color::Cyan);
pub const TEXT_STYLE: Style = Style::new().fg(Color::White);
pub const TITLE_STYLE: Style = Style::new().fg(Color::LightCyan);
