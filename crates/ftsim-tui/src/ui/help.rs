//! # ftsim-tui::ui::help
//!
//! Renders the help popup widget.

use crate::theme;
use ratatui::{prelude::*, widgets::*};

pub fn draw_help_popup(f: &mut Frame) {
    let block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .border_style(theme::FOCUSED_BORDER_STYLE);

    let text = "
    q - Quit
    ? - Toggle Help
    Space - Pause/Resume
    . - Single Step
    p - Inject Partition
    k - Kill Node
    r - Restart Node
    / - Filter Logs
    Tab - Cycle Focus
    ";

    let paragraph = Paragraph::new(text)
        .style(theme::TEXT_STYLE)
        .block(block)
        .alignment(Alignment::Left);

    // Create a centered area for the popup
    let area = centered_rect(60, 50, f.size());
    f.render_widget(Clear, area); // this clears the background
    f.render_widget(paragraph, area);
}

/// Helper to create a centered rectangle.
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
