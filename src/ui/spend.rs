use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use super::theme::{self, BLUE, DIM, SUBTEXT, TEXT, YELLOW};

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(DIM))
        .title(Line::from(vec![
            Span::raw(" "),
            Span::styled("³", Style::default().fg(BLUE)),
            Span::styled("spend ", Style::default().fg(TEXT).add_modifier(Modifier::BOLD)),
        ]));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let spend_enabled = app.data.spend_limit_enabled == Some(true);
    let has_spend = spend_enabled && app.data.current_spend_dollars.is_some();

    if has_spend {
        let current = app.data.current_spend_dollars.unwrap_or(0.0);
        let limit = app.data.spend_limit_dollars.unwrap_or(0.0);
        let pct = if limit > 0.0 { (current / limit * 100.0).clamp(0.0, 100.0) } else { 0.0 };

        let sym = match app.data.spend_currency.as_deref() {
            Some("GBP") => "£",
            Some("EUR") => "€",
            Some("JPY") | Some("CNY") => "¥",
            _ => "$",
        };

        let balance = limit - current;

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // gauge bar
                Constraint::Length(1), // spend text
                Constraint::Length(1), // balance text
                Constraint::Min(0),   // absorb remaining space
            ])
            .split(inner);

        // Gauge bar (btop-style)
        theme::render_gauge_row(f, chunks[0], "", pct, 0);

        let spend_line = Paragraph::new(Span::styled(
            format!("{sym}{:.2} / {sym}{:.2}", current, limit),
            Style::default().fg(TEXT),
        ));
        f.render_widget(spend_line, chunks[1]);

        let balance_line = Paragraph::new(Span::styled(
            format!("Balance: {sym}{:.2}", balance),
            Style::default().fg(SUBTEXT),
        ));
        f.render_widget(balance_line, chunks[2]);
    } else {
        let msg = Paragraph::new(Span::styled(
            "Spend data requires --cookie auth",
            Style::default().fg(YELLOW),
        ));
        f.render_widget(msg, inner);
    }
}
