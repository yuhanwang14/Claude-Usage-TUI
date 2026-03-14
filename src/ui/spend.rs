use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use super::theme::{BLUE, DIM, SUBTEXT, TEXT, YELLOW};

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(DIM))
        .title(Line::from(vec![
            Span::styled("³", Style::default().fg(BLUE)),
            Span::styled("spend", Style::default().fg(TEXT).add_modifier(Modifier::BOLD)),
        ]));

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Spend data isn't available via rate-limit headers
    // Show overage status from headers if available
    let enabled = app.data.spend_limit_enabled.unwrap_or(false);

    let msg = if enabled {
        Span::styled("Extra usage: enabled", Style::default().fg(TEXT))
    } else {
        Span::styled(
            "Extra usage: not available (use cookie auth for spend data)",
            Style::default().fg(YELLOW),
        )
    };

    let para = Paragraph::new(Line::from(msg));
    f.render_widget(para, inner);
}
