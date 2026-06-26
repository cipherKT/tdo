use crate::app::{AppContext, AppMode, AppState, recompute_filter};
use engine::Engine;

pub(super) fn handle_browsing(
    state: &mut AppState,
    key: crossterm::event::KeyEvent,
    engine: &Engine,
) -> anyhow::Result<()> {
    use crossterm::event::KeyCode;

    let list_len = match &state.context {
        AppContext::Home => state.projects.len(),
        AppContext::Project { .. } => state.tasks.len(),
    };

    match key.code {
        KeyCode::Char('j') => {
            if list_len > 0 {
                state.selected = (state.selected + 1) % list_len;
            }
        }
        KeyCode::Char('k') => {
            if list_len > 0 {
                state.selected = state.selected.saturating_sub(1);
            }
        }
        KeyCode::Char('/') => {
            state.mode = AppMode::Search {
                buffer: String::new(),
            };
            recompute_filter(state);
        }
        KeyCode::Char(' ') => {
            if let AppContext::Project { name, .. } = &state.context {
                if let Some(task) = state.tasks.get(state.selected) {
                    let project_name = name.clone();
                    let task_name = task.name.clone();
                    engine.toggle_done(&project_name, &task_name)?;
                    state.tasks = engine.list_tasks(&project_name)?;
                    state.filtered_tasks = (0..state.tasks.len()).collect();
                }
            }
        }
        KeyCode::Char('d') => match &state.context {
            AppContext::Home => {
                if let Some(project) = state.projects.get(state.selected) {
                    state.mode = AppMode::ConfirmPrompt {
                        message: format!(
                            "delete project '{}' and all its tasks? (y/n)",
                            project.name
                        ),
                        target_name: project.name.clone(),
                    };
                }
            }
            AppContext::Project { .. } => {
                if let Some(task) = state.tasks.get(state.selected) {
                    state.mode = AppMode::ConfirmPrompt {
                        message: format!("delete task '{}'? (y/n)", task.name),
                        target_name: task.name.clone(),
                    };
                }
            }
        },
        KeyCode::Enter => {
            if let AppContext::Home = &state.context {
                if let Some(project) = state.projects.get(state.selected) {
                    let tasks = engine.list_tasks(&project.name)?;
                    state.context = AppContext::Project {
                        name: project.name.clone(),
                        id: project.id,
                    };
                    state.filtered_tasks = (0..tasks.len()).collect();
                    state.tasks = tasks;
                    state.selected = 0;
                }
            }
        }
        KeyCode::Esc => {
            if let AppContext::Project { .. } = &state.context {
                state.context = AppContext::Home;
                state.tasks = Vec::new();
                state.filtered_tasks = Vec::new();
                state.selected = 0;
            }
        }
        _ => {}
    }
    Ok(())
}
