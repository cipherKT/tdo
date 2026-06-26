use crate::app::AppState;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
};

mod header;
mod hint;
mod list;
mod stats;

pub fn render(frame: &mut Frame, state: &AppState) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(frame.area());

    let middle = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(26)])
        .split(outer[1]);

    header::render_header(frame, state, outer[0]);
    list::render_list(frame, state, middle[0]);
    stats::render_stats(frame, state, middle[1]);
    hint::render_hint_bar(frame, state, outer[2]);
}
