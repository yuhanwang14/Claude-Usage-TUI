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

    // Two rows: main content (fills all available space) + status bar (1 line)
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(6),    // three-column panel row
            Constraint::Length(1), // status bar
        ])
        .split(size);

    // Three columns: session 33% | weekly 34% | spend 33%
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
        ])
        .split(rows[0]);

    session::render(f, cols[0], app);
    weekly::render(f, cols[1], app);
    spend::render(f, cols[2], app);
    status_bar::render(f, rows[1], app);
}
