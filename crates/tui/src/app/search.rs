use crate::app::{AppContext, AppMode, AppState, FormKind, recompute_filter};
use engine::Engine;

pub(super) fn handle_search(
    state: &mut AppState,
    key: crossterm::event::KeyEvent,
    engine: &Engine,
) -> anyhow::Result<()> {
    use crossterm::event::KeyCode;

    match key.code {
        KeyCode::Esc => {
            state.mode = AppMode::Browsing;
            state.filtered_projects = (0..state.projects.len()).collect();
            state.filtered_tasks = (0..state.tasks.len()).collect();
            state.selected = 0;
        }
        KeyCode::Backspace => {
            if let AppMode::Search { buffer } = &mut state.mode {
                buffer.pop();
            }
            recompute_filter(state);
        }
        KeyCode::Down => {
            let len = match &state.context {
                AppContext::Home => state.filtered_projects.len(),
                AppContext::Project { .. } => state.filtered_tasks.len(),
            };
            if len > 0 {
                state.selected = (state.selected + 1) % len;
            }
        }
        KeyCode::Up => {
            let len = match &state.context {
                AppContext::Home => state.filtered_projects.len(),
                AppContext::Project { .. } => state.filtered_tasks.len(),
            };
            if len > 0 {
                state.selected = state.selected.saturating_sub(1);
            }
        }
        KeyCode::Enter => {
            let buffer = if let AppMode::Search { buffer } = &state.mode {
                buffer.clone()
            } else {
                String::new()
            };

            match &state.context {
                AppContext::Home => {
                    if !state.filtered_projects.is_empty() {
                        let idx = state.filtered_projects[state.selected];
                        let project = &state.projects[idx];
                        state.context = AppContext::Project {
                            name: project.name.clone(),
                            id: project.id,
                        };
                        crate::app::update_stats(state, engine)?;
                        state.mode = AppMode::Browsing;
                        state.selected = 0;
                    } else if !buffer.is_empty() {
                        state.mode = AppMode::MultiStepForm {
                            kind: FormKind::CreateProject,
                            step: 0,
                            name: buffer.clone(),
                            answers: vec![buffer, String::new(), String::new()],
                            current_input: String::new(),
                            warning: None,
                            in_insert_mode: false,
                            show_save_confirm: false,
                            save_confirm_selected: 0,
                        };
                    }
                }
                AppContext::Project { name, .. } => {
                    if !state.filtered_tasks.is_empty() {
                        state.selected = state.filtered_tasks[state.selected];
                        state.filtered_tasks = (0..state.tasks.len()).collect();
                        state.mode = AppMode::Browsing;
                    } else if !buffer.is_empty() {
                        state.mode = AppMode::MultiStepForm {
                            kind: FormKind::CreateTask,
                            step: 0,
                            name: name.clone(),
                            answers: vec![
                                buffer,
                                String::new(),
                                String::new(),
                                "3".to_string(),
                                String::new(),
                                String::new(),
                            ],
                            current_input: String::new(),
                            warning: None,
                            in_insert_mode: false,
                            show_save_confirm: false,
                            save_confirm_selected: 0,
                        };
                    }
                }
            }
        }
        KeyCode::Char(c) => {
            if let AppMode::Search { buffer } = &mut state.mode {
                buffer.push(c);
            }
            recompute_filter(state);
        }
        _ => {}
    }
    Ok(())
}
