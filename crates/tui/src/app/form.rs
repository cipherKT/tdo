use crate::app::{AppMode, AppState, FormKind, form_total_steps};
use engine::Engine;

pub(super) fn handle_form(
    state: &mut AppState,
    key: crossterm::event::KeyEvent,
    engine: &Engine,
) -> anyhow::Result<()> {
    use crossterm::event::KeyCode;

    let mut submit_data = None;

    {
        let (form_kind, step, answers, current_input, warning, in_insert_mode, name) =
            if let AppMode::MultiStepForm {
                kind,
                step,
                answers,
                current_input,
                warning,
                in_insert_mode,
                name,
            } = &mut state.mode
            {
                (
                    kind.clone(),
                    step,
                    answers,
                    current_input,
                    warning,
                    in_insert_mode,
                    name.clone(),
                )
            } else {
                return Ok(());
            };

        let total = form_total_steps(&form_kind);

        if *in_insert_mode {
            match key.code {
                KeyCode::Esc => {
                    let check = if *step == 2 {
                        validate_tags(current_input)
                    } else {
                        Ok(())
                    };
                    if let Err(err_msg) = check {
                        *warning = Some(err_msg);
                    } else {
                        *warning = None;
                    }
                    if *step < answers.len() {
                        answers[*step] = current_input.clone();
                    } else {
                        answers.push(current_input.clone());
                    }
                    *in_insert_mode = false;
                }
                KeyCode::Char(c) => {
                    current_input.push(c);
                    *warning = None;
                }
                KeyCode::Backspace => {
                    current_input.pop();
                    *warning = None;
                }
                _ => {}
            }
        } else {
            match key.code {
                KeyCode::Esc => {
                    state.mode = AppMode::Browsing;
                    state.filtered_projects = (0..state.projects.len()).collect();
                    state.filtered_tasks = (0..state.tasks.len()).collect();
                }
                KeyCode::Char('j') => {
                    *warning = None;
                    *step = (*step + 1) % total;
                }
                KeyCode::Char('k') => {
                    *warning = None;
                    *step = if *step == 0 { total - 1 } else { *step - 1 };
                }
                KeyCode::Char('i') => {
                    let initial = answers.get(*step).cloned().unwrap_or_default();
                    *current_input = initial;
                    *in_insert_mode = true;
                    *warning = None;
                }
                KeyCode::Enter => {
                    let tags_val = answers.get(2).map(|s| s.as_str()).unwrap_or("");
                    if let Err(err_msg) = validate_tags(tags_val) {
                        *warning = Some(err_msg);
                        *step = 2;
                        return Ok(());
                    }
                    if total > 3 {
                        let prio_val = answers.get(3).map(|s| s.as_str()).unwrap_or("");
                        if !prio_val.is_empty() {
                            if let Ok(val) = prio_val.parse::<i64>() {
                                if !(1..=3).contains(&val) {
                                    *warning = Some("Priority must be 1, 2, or 3".to_string());
                                    *step = 3;
                                    return Ok(());
                                }
                            } else {
                                *warning =
                                    Some("Priority must be a number (1, 2, or 3)".to_string());
                                *step = 3;
                                return Ok(());
                            }
                        }
                    }
                    if total > 4 {
                        let due_val = answers.get(4).map(|s| s.as_str()).unwrap_or("");
                        if !due_val.is_empty()
                            && chrono::NaiveDate::parse_from_str(due_val, "%Y-%m-%d").is_err()
                        {
                            *warning =
                                Some("Due date must be in YYYY-MM-DD format or blank".to_string());
                            *step = 4;
                            return Ok(());
                        }
                    }

                    submit_data = Some((form_kind, answers.clone(), name));
                }
                _ => {}
            }
        }
    }

    if let Some((form_kind, answers, name)) = submit_data {
        match &form_kind {
            FormKind::CreateProject => {
                submit_create_project(state, engine, &answers)?;
            }
            FormKind::CreateTask => {
                submit_create_task(state, engine, &answers, &name)?;
            }
            FormKind::ModifyProject { original_name } => {
                submit_modify_project(state, engine, &answers, original_name)?;
            }
            FormKind::ModifyTask { original_name } => {
                submit_modify_task(state, engine, &answers, original_name, &name)?;
            }
        }
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

fn submit_modify_project(
    state: &mut AppState,
    engine: &Engine,
    answers: &[String],
    original_name: &str,
) -> anyhow::Result<()> {
    let new_name = answers.get(0).map(|s| s.as_str()).unwrap_or("");
    let description = answers.get(1).map(|s| s.as_str()).unwrap_or("");
    let tags_raw = answers.get(2).map(|s| s.as_str()).unwrap_or("");

    let patch = engine::ProjectPatch {
        name: if new_name != original_name {
            Some(new_name.to_string())
        } else {
            None
        },
        description: Some(description.to_string()),
    };

    match engine.modify_project(original_name, patch) {
        Ok(_) => {
            if !tags_raw.is_empty() {
                let lookup_name = if new_name != original_name {
                    new_name
                } else {
                    original_name
                };
                let tags: Vec<&str> = tags_raw
                    .split_whitespace()
                    .map(|t| t.trim_start_matches('#'))
                    .collect();
                let _ = engine.add_tags_to_project(lookup_name, &tags);
            }
            state.projects = engine.list_projects()?;
            state.filtered_projects = (0..state.projects.len()).collect();
            state.mode = AppMode::Browsing;
            state.selected = 0;
        }
        Err(e) => {
            state.mode = AppMode::Browsing;
            eprintln!("error modifying project: {}", e);
        }
    }
    Ok(())
}

fn submit_modify_task(
    state: &mut AppState,
    engine: &Engine,
    answers: &[String],
    original_name: &str,
    project_name: &str,
) -> anyhow::Result<()> {
    let new_name = answers.get(0).map(|s| s.as_str()).unwrap_or("");
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

    let patch = engine::TaskPatch {
        name: if new_name != original_name {
            Some(new_name.to_string())
        } else {
            None
        },
        description: Some(description.to_string()),
        priority: Some(priority),
        due_date: Some(due_date),
        done: None,
    };

    match engine.modify_task(project_name, original_name, patch) {
        Ok(_) => {
            if !tags_raw.is_empty() {
                let lookup_name = if new_name != original_name {
                    new_name
                } else {
                    original_name
                };
                let tags: Vec<&str> = tags_raw
                    .split_whitespace()
                    .map(|t| t.trim_start_matches('#'))
                    .collect();
                let _ = engine.add_tags_to_task(project_name, lookup_name, &tags);
            }
            state.tasks = engine.list_tasks(project_name)?;
            state.filtered_tasks = (0..state.tasks.len()).collect();
            state.mode = AppMode::Browsing;
            state.selected = 0;
        }
        Err(e) => {
            state.mode = AppMode::Browsing;
            eprintln!("error modifying task: {}", e);
        }
    }
    Ok(())
}

fn validate_tags(tags_raw: &str) -> Result<(), String> {
    if tags_raw.trim().is_empty() {
        return Ok(());
    }
    for tag in tags_raw.split_whitespace() {
        if !tag.starts_with('#') {
            return Err(format!("Tag '{}' must start with '#'", tag));
        }
        let content = &tag[1..];
        if content.is_empty() {
            return Err(format!("Tag '{}' cannot be empty after '#'", tag));
        }
        for c in content.chars() {
            if !c.is_ascii_alphanumeric() && c != '_' && c != '-' {
                return Err(format!(
                    "Tag '{}' contains invalid character '{}' (only alphanumeric, '_' and '-' are allowed)",
                    tag, c
                ));
            }
        }
    }
    Ok(())
}
