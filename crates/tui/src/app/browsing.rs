use crate::app::{AppContext, AppMode, AppState, FormKind, recompute_filter};
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
            if let AppContext::Project { name, .. } = &state.context {
                let target_name = name.clone();
                state.context = AppContext::Home;
                if let Some(pos) = state
                    .filtered_projects
                    .iter()
                    .position(|&idx| state.projects[idx].name == target_name)
                {
                    state.selected = pos;
                } else {
                    state.selected = 0;
                }
            }
        }
        KeyCode::Char('i') => match &state.context {
            AppContext::Home => {
                if let Some(project) = state.projects.get(state.selected) {
                    let tags = engine.get_tags_for_project(&project.name)?;
                    let tags_str = tags
                        .iter()
                        .map(|t| format!("#{}", t.name))
                        .collect::<Vec<_>>()
                        .join(" ");
                    let prefill = vec![project.name.clone(), project.description.clone(), tags_str];
                    state.mode = AppMode::MultiStepForm {
                        kind: FormKind::ModifyProject {
                            original_name: project.name.clone(),
                        },
                        step: 0,
                        name: project.name.clone(),
                        answers: prefill.clone(),
                        current_input: String::new(),
                        warning: None,
                        in_insert_mode: false,
                    };
                }
            }
            AppContext::Project { name, .. } => {
                if let Some(task) = state.tasks.get(state.selected) {
                    let tags = engine.get_tags_for_task(name, &task.name)?;
                    let tags_str = tags
                        .iter()
                        .map(|t| format!("#{}", t.name))
                        .collect::<Vec<_>>()
                        .join(" ");
                    let due_str = task
                        .due_date
                        .map(|d| d.format("%Y-%m-%d").to_string())
                        .unwrap_or_default();
                    let prefill = vec![
                        task.name.clone(),
                        task.description.clone(),
                        tags_str,
                        task.priority.to_string(),
                        due_str,
                    ];
                    state.mode = AppMode::MultiStepForm {
                        kind: FormKind::ModifyTask {
                            original_name: task.name.clone(),
                        },
                        step: 0,
                        name: name.clone(),
                        answers: prefill.clone(),
                        current_input: String::new(),
                        warning: None,
                        in_insert_mode: false,
                    };
                }
            }
        },
        _ => {}
    }
    Ok(())
}
