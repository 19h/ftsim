//! # ftsim-tui::ui::widgets::status
//!
//! Renders the status bar and the node status grid.

use crate::{app::App, theme};
use ftsim_engine::node::NodeStatus;
use ratatui::{prelude::*, widgets::*};

pub fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let time_str = app
        .snapshot
        .as_ref()
        .map(|s| format!("{:.3} ms", s.time as f64 / 1_000_000.0))
        .unwrap_or_else(|| "N/A".to_string());

    let text = Line::from(vec![
        Span::styled(" FTSim ", Style::new().bg(Color::Cyan).fg(Color::Black)),
        Span::raw(" | "),
        Span::styled(time_str, Style::new().fg(Color::Green)),
        Span::raw(" | Press '?' for help, 'q' to quit"),
    ]);
    f.render_widget(Paragraph::new(text), area);
}

pub fn draw_node_status_grid(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Node Status ")
        .borders(Borders::ALL)
        .border_style(theme::BORDER_STYLE);

    let Some(snapshot) = &app.snapshot else {
        f.render_widget(block, area);
        return;
    };

    let rows = snapshot.nodes.iter().map(|node| {
        let status_style = match node.status {
            NodeStatus::Up => Style::new().fg(Color::Green),
            NodeStatus::Down => Style::new().fg(Color::Red),
            NodeStatus::Recovering => Style::new().fg(Color::Yellow),
        };
        let role = node
            .custom
            .get("role")
            .and_then(|v| v.as_str())
            .unwrap_or("?");
        let term = node.custom.get("term").map(|v| {
            if let Some(n) = v.as_u64() {
                n.to_string()
            } else if let Some(s) = v.as_str() {
                s.to_string()
            } else {
                "-".into()
            }
        }).unwrap_or_else(|| "-".into());

        Row::new(vec![
            Cell::from(node.id.to_string()),
            Cell::from(format!("{:?}", node.status)).style(status_style),
            Cell::from(role.to_string()),
            Cell::from(term),
        ])
    });

    let table = Table::new(
        rows,
        [
            Constraint::Length(4),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Min(8),
        ],
    )
    .header(
        Row::new(vec!["ID", "Status", "Role", "Term"]).style(theme::TITLE_STYLE),
    )
    .block(block);

    f.render_widget(table, area);
}
