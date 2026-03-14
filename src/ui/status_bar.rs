use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
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

    let sep = Span::styled(" │ ", Style::default().fg(DIM));

    // Left side: auth + connection + interval controls + key hints
    let left = Line::from(vec![
        Span::styled(" ", Style::default()),
        Span::styled(&app.plan_name, Style::default().fg(SUBTEXT)), // dimmed (was TEXT)
        sep.clone(),
        Span::styled(dot, Style::default().fg(dot_color)),
        Span::styled(format!(" {}", connection_label), Style::default().fg(SUBTEXT)),
        sep.clone(),
        Span::styled("- ", Style::default().fg(RED)),
        Span::styled(format!("{}s", app.refresh_interval), Style::default().fg(TEXT)),
        Span::styled(" +", Style::default().fg(GREEN)),
        sep.clone(),
        Span::styled("q", Style::default().fg(TEXT)),
        Span::styled(" quit", Style::default().fg(DIM)),
        Span::styled("  ", Style::default()),
        Span::styled("r", Style::default().fg(TEXT)),
        Span::styled(" refresh", Style::default().fg(DIM)),
    ]);

    // Right side: next refresh countdown
    let right = Line::from(vec![
        Span::styled(format!("Next: {}s ", app.refresh_interval), Style::default().fg(DIM)),
    ]);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(right.width() as u16),
        ])
        .split(area);

    f.render_widget(Paragraph::new(left), cols[0]);
    f.render_widget(Paragraph::new(right).alignment(Alignment::Right), cols[1]);
}

/// Check if a mouse click at (col, row) hit the "-" or "+" in the status bar.
/// Returns Some(true) for "+", Some(false) for "-", None if miss.
pub fn check_interval_click(area: Rect, col: u16, row: u16, app: &App) -> Option<bool> {
    if row != area.y {
        return None;
    }

    // Calculate approximate positions of - and + in the status bar
    // " {plan_name} │ ● {status} │ - {N}s + │ ..."
    let plan_len = app.plan_name.len() as u16;
    let status_label = match app.connection {
        ConnectionStatus::Online => "online",
        ConnectionStatus::Offline => "offline",
        ConnectionStatus::Disconnected => "connecting…",
    };
    let status_len = status_label.len() as u16;

    // Position: " " + plan + " │ " + "● " + status + " │ " + "- "
    let minus_start = area.x + 1 + plan_len + 3 + 2 + status_len + 3;
    let interval_str_len = format!("{}s", app.refresh_interval).len() as u16;
    let plus_start = minus_start + 2 + interval_str_len + 1;

    if col >= minus_start && col < minus_start + 2 {
        Some(false) // clicked "-"
    } else if col >= plus_start && col <= plus_start + 1 {
        Some(true) // clicked "+"
    } else {
        None
    }
}
