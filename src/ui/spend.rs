use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Gauge, Paragraph},
    Frame,
};

use crate::app::App;
use super::theme::{self, BLUE, RED, SUBTEXT, TEXT};

fn currency_symbol(amount: f64) -> &'static str {
    // Default to £ for now; could be made configurable
    let _ = amount;
    "£"
}

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(Line::from(vec![
            Span::styled("³", Style::default().fg(BLUE)),
            Span::styled("spend", Style::default().fg(TEXT).add_modifier(Modifier::BOLD)),
        ]));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let enabled = app.data.spend_limit_enabled.unwrap_or(false);

    if !enabled {
        let msg = Paragraph::new(Span::styled(
            "Spend limits not enabled",
            Style::default().fg(SUBTEXT),
        ));
        f.render_widget(msg, inner);
        return;
    }

    let current = app.data.current_spend_dollars.unwrap_or(0.0);
    let limit = app.data.spend_limit_dollars.unwrap_or(0.0);
    let pct = if limit > 0.0 { (current / limit * 100.0).clamp(0.0, 100.0) } else { 0.0 };

    let sym = currency_symbol(current);
    let spend_text = format!("{}{:.2} / {}{:.2}", sym, current, sym, limit);

    // Credit balance
    let remaining = app.data.credit_remaining_dollars.unwrap_or(0.0);
    let balance_sym = currency_symbol(remaining);
    let balance_text = format!("Balance: {}{:.2}", balance_sym, remaining);
    let balance_style = if remaining < 0.0 {
        Style::default().fg(RED)
    } else {
        Style::default().fg(TEXT)
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // gauge
            Constraint::Length(1), // spend text
            Constraint::Length(1), // balance
            Constraint::Min(0),
        ])
        .split(inner);

    let gauge = Gauge::default()
        .gauge_style(theme::gauge_style(pct))
        .ratio((pct / 100.0).clamp(0.0, 1.0))
        .label(spend_text.clone());
    f.render_widget(gauge, chunks[0]);

    let spend_line = Paragraph::new(Span::styled(spend_text, Style::default().fg(SUBTEXT)));
    f.render_widget(spend_line, chunks[1]);

    let balance_line = Paragraph::new(Span::styled(balance_text, balance_style));
    f.render_widget(balance_line, chunks[2]);
}
