use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::{App, ConnectionStatus};
use super::theme::{DIM, GREEN, RED, SUBTEXT, TEXT, YELLOW};

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let (dot, dot_color) = match app.connection {
        ConnectionStatus::Online => ("●", GREEN),
        ConnectionStatus::Offline => ("●", RED),
        ConnectionStatus::Disconnected => ("●", YELLOW),
    };

    let connection_label = match app.connection {
        ConnectionStatus::Online => "online",
        ConnectionStatus::Offline => "offline",
        ConnectionStatus::Disconnected => "connecting…",
    };

    let line = Line::from(vec![
        Span::styled(" ", Style::default()),
        Span::styled(&app.plan_name, Style::default().fg(TEXT)),
        Span::styled("  ", Style::default()),
        Span::styled(dot, Style::default().fg(dot_color)),
        Span::styled(format!(" {}  ", connection_label), Style::default().fg(SUBTEXT)),
        Span::styled(
            format!("↻ {}s  ", app.refresh_interval),
            Style::default().fg(DIM),
        ),
        Span::styled("q", Style::default().fg(TEXT)),
        Span::styled(" quit  ", Style::default().fg(DIM)),
        Span::styled("+/-", Style::default().fg(TEXT)),
        Span::styled(" interval  ", Style::default().fg(DIM)),
        Span::styled("r", Style::default().fg(TEXT)),
        Span::styled(" refresh", Style::default().fg(DIM)),
    ]);

    let para = Paragraph::new(line);
    f.render_widget(para, area);
}
