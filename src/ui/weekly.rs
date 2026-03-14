use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use super::theme::{self, BLUE, DIM, SUBTEXT, TEXT};

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(DIM))
        .title(Line::from(vec![
            Span::raw(" "),
            Span::styled("²", Style::default().fg(BLUE)),
            Span::styled("weekly ", Style::default().fg(TEXT).add_modifier(Modifier::BOLD)),
        ]));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let reset_str = App::format_reset_time(app.data.weekly_reset_at.as_deref());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // All gauge row
            Constraint::Length(1), // Sonnet gauge row
            Constraint::Length(1), // Opus gauge row
            Constraint::Min(0),   // spacer
            Constraint::Length(1), // reset text (anchored bottom)
        ])
        .split(inner);

    theme::render_gauge_row(f, chunks[0], "All    ", app.data.weekly_percent_used.unwrap_or(0.0), 8);
    theme::render_gauge_row(f, chunks[1], "Sonnet ", app.data.weekly_sonnet_percent.unwrap_or(0.0), 8);
    theme::render_gauge_row(f, chunks[2], "Opus   ", app.data.weekly_opus_percent.unwrap_or(0.0), 8);

    let reset_line = Paragraph::new(Span::styled(reset_str, Style::default().fg(SUBTEXT)));
    f.render_widget(reset_line, chunks[4]);
}
