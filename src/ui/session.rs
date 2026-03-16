use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Sparkline},
    Frame,
};

use crate::app::App;
use super::theme::{self, BLUE, DIM, GREEN, SUBTEXT, TEXT, YELLOW};

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(DIM))
        .title(Line::from(vec![
            Span::raw(" "),
            Span::styled("¹", Style::default().fg(BLUE)),
            Span::styled("session ", Style::default().fg(TEXT).add_modifier(Modifier::BOLD)),
        ]));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let pct = app.data.session_percent_used.unwrap_or(0.0);
    let reset_str = App::format_reset_time(app.data.session_reset_at.as_deref());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // "5-hour rolling window"
            Constraint::Length(1), // gauge bar
            Constraint::Min(1),   // sparkline — fills ALL remaining space
            Constraint::Length(1), // reset text (anchored to bottom)
        ])
        .split(inner);

    // Subtitle
    let subtitle = Paragraph::new(Span::styled(
        "5-hour rolling window",
        Style::default().fg(SUBTEXT),
    ));
    f.render_widget(subtitle, chunks[0]);

    // Gauge bar (btop-style)
    theme::render_gauge_row(f, chunks[1], "", pct, 0);

    // Sparkline (braille graph filling remaining vertical space)
    if !app.sparkline_data.is_empty() {
        let data: Vec<u64> = app.sparkline_data
            .iter()
            .map(|v| (*v as u64).clamp(0, 100))
            .collect();
        let sparkline = Sparkline::default()
            .data(&data)
            .max(100)
            .style(Style::default().fg(GREEN));
        f.render_widget(sparkline, chunks[2]);

        // Overlay reset markers (vertical dashed lines)
        let area_w = chunks[2].width as usize;
        let data_len = app.sparkline_resets.len();
        let visible_start = data_len.saturating_sub(area_w);
        for (i, is_reset) in app.sparkline_resets.iter().enumerate() {
            if *is_reset && i >= visible_start {
                let x = chunks[2].x + (i - visible_start) as u16;
                for y in 0..chunks[2].height {
                    let cell = &mut f.buffer_mut()[(x, chunks[2].y + y)];
                    cell.set_symbol("┆").set_style(Style::default().fg(YELLOW));
                }
            }
        }
    }

    // Reset timer (anchored to bottom)
    let reset_line = Paragraph::new(Span::styled(reset_str, Style::default().fg(SUBTEXT)));
    f.render_widget(reset_line, chunks[3]);
}
