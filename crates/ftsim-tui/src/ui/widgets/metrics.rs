//! # ftsim-tui::ui::widgets::metrics
//!
//! Renders the Metrics Panel widget. This is currently a placeholder.

use crate::{app::App, theme};
use ratatui::{prelude::*, widgets::*};

pub fn draw_metrics_panel(f: &mut Frame, _app: &App, area: Rect) {
    let block = Block::default()
        .title(" Metrics ")
        .borders(Borders::ALL)
        .border_style(theme::BORDER_STYLE);
    f.render_widget(block, area);
}
