//! # ftsim-tui::ui::widgets::logs
//!
//! Renders the Logs and Timeline widget. This is currently a placeholder.

use crate::{app::App, theme};
use ratatui::{prelude::*, widgets::*};

pub fn draw_logs_panel(f: &mut Frame, _app: &App, area: Rect) {
    let block = Block::default()
        .title(" Logs / Timeline ")
        .borders(Borders::ALL)
        .border_style(theme::BORDER_STYLE);
    f.render_widget(block, area);
}
