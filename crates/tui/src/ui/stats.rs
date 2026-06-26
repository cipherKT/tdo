use crate::app::AppState;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders},
};

pub(super) fn render_stats(frame: &mut Frame, _state: &AppState, area: Rect) {
    let block = Block::default()
        .title(" stats ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    frame.render_widget(block, area);
}
