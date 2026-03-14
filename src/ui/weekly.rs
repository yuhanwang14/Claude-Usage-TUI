use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Gauge, Paragraph},
    Frame,
};

use crate::app::App;
use super::theme::{self, BAR_BG, BLUE, DIM, SUBTEXT, TEXT};

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(DIM))
        .title(Line::from(vec![
            Span::styled("²", Style::default().fg(BLUE)),
            Span::styled("weekly", Style::default().fg(TEXT).add_modifier(Modifier::BOLD)),
        ]));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let reset_str = App::format_reset_time(app.data.weekly_reset_at.as_deref());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // All gauge
            Constraint::Length(1), // Sonnet gauge
            Constraint::Length(1), // Opus gauge
            Constraint::Length(1), // spacer
            Constraint::Length(1), // reset text
            Constraint::Min(0),
        ])
        .split(inner);

    render_gauge_row(f, chunks[0], "All   ", app.data.weekly_percent_used.unwrap_or(0.0));
    render_gauge_row(f, chunks[1], "Sonnet", app.data.weekly_sonnet_percent.unwrap_or(0.0));
    render_gauge_row(f, chunks[2], "Opus  ", app.data.weekly_opus_percent.unwrap_or(0.0));

    let reset_line = Paragraph::new(Span::styled(reset_str, Style::default().fg(SUBTEXT)));
    f.render_widget(reset_line, chunks[4]);
}

fn render_gauge_row(f: &mut Frame, area: Rect, label: &str, pct: f64) {
    let row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(8), Constraint::Min(1)])
        .split(area);

    let label_widget = Paragraph::new(Span::styled(label, Style::default().fg(SUBTEXT)));
    f.render_widget(label_widget, row[0]);

    let gauge = Gauge::default()
        .gauge_style(
            Style::default()
                .fg(theme::gauge_color(pct))
                .bg(BAR_BG),
        )
        .ratio((pct / 100.0).clamp(0.0, 1.0))
        .label(format!("{:.0}%", pct));
    f.render_widget(gauge, row[1]);
}
