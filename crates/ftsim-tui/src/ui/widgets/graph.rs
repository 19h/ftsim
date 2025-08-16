//! # ftsim-tui::ui::widgets::graph
//!
//! Renders the Cluster Graph widget. This is currently a placeholder.

use crate::{app::App, theme};
use ratatui::{prelude::*, widgets::*};

pub fn draw_graph(f: &mut Frame, _app: &App, area: Rect) {
    let block = Block::default()
        .title(" Cluster Graph ")
        .borders(Borders::ALL)
        .border_style(theme::BORDER_STYLE);
    let text = Paragraph::new("Graph rendering not yet implemented.")
        .alignment(Alignment::Center)
        .block(block);
    f.render_widget(text, area);
}
