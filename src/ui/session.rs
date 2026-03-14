use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Gauge, Paragraph, Sparkline},
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
            Span::styled("¹", Style::default().fg(BLUE)),
            Span::styled("session", Style::default().fg(TEXT).add_modifier(Modifier::BOLD)),
        ]));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let pct = app.data.session_percent_used.unwrap_or(0.0);
    let reset_str = App::format_reset_time(app.data.session_reset_at.as_deref());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // gauge
            Constraint::Length(1), // spacer
            Constraint::Length(1), // reset text
            Constraint::Min(1),    // sparkline
        ])
        .split(inner);

    // Gauge with just percentage
    let gauge = Gauge::default()
        .gauge_style(theme::gauge_style(pct))
        .ratio((pct / 100.0).clamp(0.0, 1.0))
        .label(format!("{:.0}%", pct));
    f.render_widget(gauge, chunks[0]);

    // Reset timer
    let reset_line = Paragraph::new(Span::styled(reset_str, Style::default().fg(SUBTEXT)));
    f.render_widget(reset_line, chunks[2]);

    // Sparkline
    if !app.sparkline_data.is_empty() {
        let spark_data: Vec<u64> = app
            .sparkline_data
            .iter()
            .map(|v| (*v * 10.0).max(0.0) as u64)
            .collect();
        let sparkline = Sparkline::default()
            .style(Style::default().fg(BLUE))
            .data(&spark_data);
        f.render_widget(sparkline, chunks[3]);
    }
}
