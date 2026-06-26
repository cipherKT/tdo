use crate::app::{AppContext, AppMode, AppState};
use engine::Engine;

pub(super) fn handle_confirm(
    state: &mut AppState,
    key: crossterm::event::KeyEvent,
    engine: &Engine,
) -> anyhow::Result<()> {
    use crossterm::event::KeyCode;

    let target = if let AppMode::ConfirmPrompt { target_name, .. } = &state.mode {
        target_name.clone()
    } else {
        return Ok(());
    };

    match key.code {
        KeyCode::Char('y') => {
            match &state.context {
                AppContext::Home => {
                    engine.delete_project(&target)?;
                    state.projects = engine.list_projects()?;
                    state.filtered_projects = (0..state.projects.len()).collect();
                    state.selected = state.selected.min(state.projects.len().saturating_sub(1));
                }
                AppContext::Project { name, .. } => {
                    let project_name = name.clone();
                    engine.delete_task(&project_name, &target)?;
                    state.tasks = engine.list_tasks(&project_name)?;
                    state.filtered_tasks = (0..state.tasks.len()).collect();
                    state.selected = state.selected.min(state.tasks.len().saturating_sub(1));
                }
            }
            state.mode = AppMode::Browsing;
        }
        KeyCode::Char('n') | KeyCode::Esc => {
            state.mode = AppMode::Browsing;
        }
        _ => {}
    }
    Ok(())
}
