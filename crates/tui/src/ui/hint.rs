use crate::app::{AppContext, AppMode, AppState};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::Paragraph,
};

pub(super) fn render_hint_bar(frame: &mut Frame, state: &AppState, area: Rect) {
    match &state.mode {
        AppMode::Browsing => {
            let hint = "  j/k move  ·  enter open  ·  / search  ·  d delete  ·  q quit";
            frame.render_widget(
                Paragraph::new(hint).style(Style::default().fg(Color::DarkGray)),
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
                Paragraph::new(content).style(Style::default().fg(Color::Yellow)),
                area,
            );
        }
        AppMode::ConfirmPrompt { message, .. } => {
            frame.render_widget(
                Paragraph::new(message.as_str()).style(Style::default().fg(Color::Red)),
                area,
            );
        }
        AppMode::MultiStepForm {
            kind,
            step,
            current_input,
            ..
        } => {
            let prompt = crate::app::form_prompt(kind, *step);
            let total = crate::app::form_total_steps(kind);
            let content = format!(" {} ({}/{})  {}▌", prompt, step, total, current_input);
            frame.render_widget(
                Paragraph::new(content).style(Style::default().fg(Color::Yellow)),
                area,
            );
        }
    }
}
