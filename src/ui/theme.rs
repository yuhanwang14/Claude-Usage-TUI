use ratatui::style::{Color, Style};

// btop-inspired color palette
pub const GREEN: Color = Color::Rgb(0x4e, 0xc5, 0x6c);
pub const YELLOW: Color = Color::Rgb(0xe8, 0xc5, 0x47);
pub const RED: Color = Color::Rgb(0xe0, 0x6c, 0x75);
pub const BLUE: Color = Color::Rgb(0x7e, 0xc8, 0xe3);
pub const DIM: Color = Color::Rgb(0x66, 0x66, 0x66);
pub const TEXT: Color = Color::Rgb(0xe0, 0xe0, 0xe0);
pub const SUBTEXT: Color = Color::Rgb(0x88, 0x88, 0x88);

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

/// Return a filled `Style` using the appropriate gauge color.
pub fn gauge_style(pct: f64) -> Style {
    Style::default().fg(gauge_color(pct))
}
