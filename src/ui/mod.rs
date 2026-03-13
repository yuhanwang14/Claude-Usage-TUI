pub mod session;
pub mod spend;
pub mod status_bar;
pub mod theme;
pub mod weekly;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use crate::app::App;

pub fn draw(f: &mut Frame, app: &App) {
    let size = f.area();

    // Vertical split: top 60%, middle 30%, bottom min 1
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Ratio(6, 10),
            Constraint::Ratio(3, 10),
            Constraint::Min(1),
        ])
        .split(size);

    // Top row: 40% session | 60% weekly
    let top_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(rows[0]);

    session::render(f, top_cols[0], app);
    weekly::render(f, top_cols[1], app);
    spend::render(f, rows[1], app);
    status_bar::render(f, rows[2], app);
}
