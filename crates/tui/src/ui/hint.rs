use crate::app::{AppContext, AppMode, AppState, RightPane};
use ratatui::{Frame, layout::Rect, style::Style, widgets::Paragraph};

pub(super) fn render_hint_bar(frame: &mut Frame, state: &AppState, area: Rect) {
    match &state.mode {
        AppMode::Browsing => {
            let hint = if state.right_pane == RightPane::Calendar {
                "  h/←/l/→: prev/next day  ·  j/↓/k/↑: prev/next week  ·  Tab: tasks pane  ·  Esc: focus list  ·  q quit"
            } else {
                "  j/↓/k/↑ move  ·  enter open  ·  / search  ·  d delete  ·  Tab: calendar  ·  q quit"
            };
            frame.render_widget(
                Paragraph::new(hint).style(Style::default().fg(state.theme.label)),
                area,
            );
        }
        AppMode::Search { buffer } => {
            let no_match = match &state.context {
                AppContext::Home => state.filtered_projects.is_empty() && !buffer.is_empty(),
                AppContext::Project { .. } => state.filtered_tasks.is_empty() && !buffer.is_empty(),
            };
            let content = if no_match {
                format!(" / {}  —  hit enter to create", buffer)
            } else {
                format!(" / {}", buffer)
            };
            frame.render_widget(
                Paragraph::new(content).style(Style::default().fg(state.theme.secondary_accent)),
                area,
            );
        }
        AppMode::ConfirmPrompt { message, .. } => {
            frame.render_widget(
                Paragraph::new(message.as_str())
                    .style(Style::default().fg(state.theme.status_overdue)),
                area,
            );
        }
        AppMode::MultiStepForm { in_insert_mode, .. } => {
            let content = if *in_insert_mode {
                "  [INSERT]  Press ESC to finish editing this field"
            } else {
                "  [NORMAL]  j/↓/k/↑: navigate  ·  i: edit field  ·  enter: save  ·  esc: cancel"
            };
            frame.render_widget(
                Paragraph::new(content).style(Style::default().fg(state.theme.secondary_accent)),
                area,
            );
        }
    }
}
