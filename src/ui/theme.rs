use ratatui::style::{Color, Style, Modifier};
use ratatui::text::{Span, Line};
use ratatui::layout::{Layout, Direction, Constraint, Rect, Alignment};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

// btop-inspired color palette
pub const GREEN: Color = Color::Rgb(78, 197, 108);
pub const YELLOW: Color = Color::Rgb(232, 197, 71);
pub const RED: Color = Color::Rgb(224, 108, 117);
pub const BLUE: Color = Color::Rgb(126, 200, 227);
pub const DIM: Color = Color::Rgb(80, 80, 80);
pub const TEXT: Color = Color::Rgb(224, 224, 224);
pub const SUBTEXT: Color = Color::Rgb(136, 136, 136);
pub const BAR_BG: Color = Color::Rgb(51, 51, 51);

/// Pick a color based on percentage used (0.0–100.0).
pub fn gauge_color(pct: f64) -> Color {
    if pct >= 90.0 {
        RED
    } else if pct >= 70.0 {
        YELLOW
    } else {
        GREEN
    }
}

/// Render a btop-style gauge bar as a Line of Spans.
pub fn gauge_bar(pct: f64, width: usize) -> Line<'static> {
    let filled = ((pct / 100.0) * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);
    let color = gauge_color(pct);

    Line::from(vec![
        Span::styled("█".repeat(filled), Style::default().fg(color)),
        Span::styled("░".repeat(empty), Style::default().fg(BAR_BG)),
    ])
}

/// Colored percentage span, right-aligned friendly.
pub fn pct_span(pct: f64) -> Span<'static> {
    Span::styled(
        format!("{:>3.0}%", pct),
        Style::default()
            .fg(gauge_color(pct))
            .add_modifier(Modifier::BOLD),
    )
}

/// Render a full gauge row: [label | bar | pct%]
/// `label_width` is the fixed width for the label column. Pass 0 for no label.
pub fn render_gauge_row(f: &mut Frame, area: Rect, label: &str, pct: f64, label_width: u16) {
    if label_width > 0 {
        let row = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(label_width),
                Constraint::Min(1),
                Constraint::Length(5),
            ])
            .split(area);

        let label_widget = Paragraph::new(Span::styled(
            label.to_string(),
            Style::default().fg(SUBTEXT),
        ));
        f.render_widget(label_widget, row[0]);

        let bar = gauge_bar(pct, row[1].width as usize);
        f.render_widget(Paragraph::new(bar), row[1]);

        let pct_widget = Paragraph::new(pct_span(pct))
            .alignment(Alignment::Right);
        f.render_widget(pct_widget, row[2]);
    } else {
        let row = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(1),
                Constraint::Length(5),
            ])
            .split(area);

        let bar = gauge_bar(pct, row[0].width as usize);
        f.render_widget(Paragraph::new(bar), row[0]);

        let pct_widget = Paragraph::new(pct_span(pct))
            .alignment(Alignment::Right);
        f.render_widget(pct_widget, row[1]);
    }
}
