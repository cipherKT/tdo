use crate::app::{AppContext, AppState};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::Paragraph,
};

pub(super) fn render_header(frame: &mut Frame, state: &AppState, area: Rect) {
    let breadcrumb = match &state.context {
        AppContext::Home => " tdo  ~  home".to_string(),
        AppContext::Project { name, .. } => format!(" tdo  ~  home  /  {}", name),
    };
    let p = Paragraph::new(breadcrumb).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(p, area);
}
