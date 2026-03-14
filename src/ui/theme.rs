use ratatui::style::Color;

// btop-inspired color palette
pub const GREEN: Color = Color::Rgb(78, 197, 108);
pub const YELLOW: Color = Color::Rgb(232, 197, 71);
pub const RED: Color = Color::Rgb(224, 108, 117);
pub const BLUE: Color = Color::Rgb(126, 200, 227);
pub const DIM: Color = Color::Rgb(102, 102, 102);
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

