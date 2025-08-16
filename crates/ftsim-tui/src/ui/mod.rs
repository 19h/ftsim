//! # ftsim-tui::ui
//!
//! The main rendering module. It orchestrates the drawing of all UI components.

use crate::app::App;
use ratatui::{prelude::*, widgets::*};

mod help;
mod layout;
mod widgets;

/// The main draw function that renders the entire UI.
pub fn draw(f: &mut Frame, app: &App) {
    let main_layout = layout::create_main_layout(f.size());
    f.render_widget(
        Block::new().style(Style::new().bg(Color::Black)),
        f.size(),
    );

    if app.snapshot.is_some() {
        // Render the main widgets
        widgets::status::draw_status_bar(f, app, main_layout[0]);
        widgets::graph::draw_graph(f, app, main_layout[1]);

        let mid_layout = layout::create_middle_layout(main_layout[2]);
        widgets::status::draw_node_status_grid(f, app, mid_layout[0]);
        widgets::metrics::draw_metrics_panel(f, app, mid_layout[1]);

        widgets::logs::draw_logs_panel(f, app, main_layout[3]);
    } else {
        // Show a loading/waiting message
        let area = f.size();
        let block = Block::default().title(" FTSim ").borders(Borders::ALL);
        let text = Paragraph::new("Waiting for simulation to start...")
            .alignment(Alignment::Center)
            .block(block);
        f.render_widget(text, area);
    }

    // Render the help popup if active
    if app.show_help {
        help::draw_help_popup(f);
    }
}
