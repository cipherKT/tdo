use crate::app::{AppMode, AppState, FormKind, form_total_steps};
use engine::Engine;

pub(super) fn handle_form(
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
        }
        KeyCode::Char(c) => {
            if let AppMode::MultiStepForm { current_input, .. } = &mut state.mode {
                current_input.push(c);
            }
        }
        KeyCode::Backspace => {
            if let AppMode::MultiStepForm { current_input, .. } = &mut state.mode {
                current_input.pop();
            }
        }
        KeyCode::Enter => {
            // extract what we need before mutating state
            let (kind_is_project, step, total, answers, current, name) =
                if let AppMode::MultiStepForm {
                    kind,
                    step,
                    answers,
                    current_input,
                    name,
                } = &state.mode
                {
                    let is_project = matches!(kind, FormKind::CreateProject);
                    let total = form_total_steps(kind);
                    (
                        is_project,
                        *step,
                        total,
                        answers.clone(),
                        current_input.clone(),
                        name.clone(),
                    )
                } else {
                    return Ok(());
                };

            let mut new_answers = answers.clone();
            new_answers.push(current.clone());
            let next_step = step + 1;

            if next_step >= total {
                // all fields collected — submit
                if kind_is_project {
                    submit_create_project(state, engine, &new_answers)?;
                } else {
                    submit_create_task(state, engine, &new_answers, &name)?;
                }
            } else {
                // advance to next step
                if let AppMode::MultiStepForm {
                    step,
                    answers,
                    current_input,
                    ..
                } = &mut state.mode
                {
                    *step = next_step;
                    *answers = new_answers;
                    *current_input = String::new();
                }
            }
        }
        _ => {}
    }
    Ok(())
}

fn submit_create_project(
    state: &mut AppState,
    engine: &Engine,
    answers: &[String],
) -> anyhow::Result<()> {
    let name = answers.get(0).map(|s| s.as_str()).unwrap_or("");
    let description = answers.get(1).map(|s| s.as_str()).unwrap_or("");
    let tags_raw = answers.get(2).map(|s| s.as_str()).unwrap_or("");

    match engine.create_project(name, description) {
        Ok(_) => {
            // add tags if any were provided
            if !tags_raw.is_empty() {
                let tags: Vec<&str> = tags_raw
                    .split_whitespace()
                    .map(|t| t.trim_start_matches('#'))
                    .collect();
                let _ = engine.add_tags_to_project(name, &tags);
            }
            state.projects = engine.list_projects()?;
            state.filtered_projects = (0..state.projects.len()).collect();
            state.mode = AppMode::Browsing;
            state.selected = 0;
        }
        Err(e) => {
            // for now just go back to browsing — later we can show the error
            state.mode = AppMode::Browsing;
            eprintln!("error creating project: {}", e);
        }
    }
    Ok(())
}

fn submit_create_task(
    state: &mut AppState,
    engine: &Engine,
    answers: &[String],
    project_name: &str,
) -> anyhow::Result<()> {
    let name = answers.get(0).map(|s| s.as_str()).unwrap_or("");
    let description = answers.get(1).map(|s| s.as_str()).unwrap_or("");
    let tags_raw = answers.get(2).map(|s| s.as_str()).unwrap_or("");
    let priority: i64 = answers.get(3).and_then(|s| s.parse().ok()).unwrap_or(3);
    let due_date = answers.get(4).and_then(|s| {
        if s.is_empty() {
            None
        } else {
            chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .ok()
                .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc())
        }
    });

    match engine.create_task(project_name, name, description, priority, due_date) {
        Ok(_) => {
            if !tags_raw.is_empty() {
                let tags: Vec<&str> = tags_raw
                    .split_whitespace()
                    .map(|t| t.trim_start_matches('#'))
                    .collect();
                let _ = engine.add_tags_to_task(project_name, name, &tags);
            }
            state.tasks = engine.list_tasks(project_name)?;
            state.filtered_tasks = (0..state.tasks.len()).collect();
            state.mode = AppMode::Browsing;
            state.selected = 0;
        }
        Err(e) => {
            state.mode = AppMode::Browsing;
            eprintln!("error creating task: {}", e);
        }
    }
    Ok(())
}
