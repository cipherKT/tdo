use crate::app::{AppMode, AppState, FormKind, form_total_steps};
use engine::Engine;

pub(super) fn handle_form(
    state: &mut AppState,
    key: crossterm::event::KeyEvent,
    engine: &Engine,
) -> anyhow::Result<()> {
    use crossterm::event::KeyCode;

    let mut submit_data = None;
    let mut exit_form = false;

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
                    let mut val = current_input.clone();
                    if *step == 4 && !val.trim().is_empty() {
                        let today = chrono::Local::now().date_naive();
                        if let Ok(d) = super::date_parser::parse_due_date(&val, today) {
                            val = d.format("%Y-%m-%d").to_string();
                            *current_input = val.clone();
                        }
                    }
                    if *step < answers.len() {
                        answers[*step] = val;
                    } else {
                        answers.push(val);
                    }
                    *in_insert_mode = false;
                }
                KeyCode::Char(c) => {
                    current_input.push(c);
                }
                KeyCode::Backspace => {
                    current_input.pop();
                }
                _ => {}
            }
        } else {
            match key.code {
                KeyCode::Esc => {
                    exit_form = true;
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    *step = (*step + 1) % total;
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    *step = if *step == 0 { total - 1 } else { *step - 1 };
                }
                KeyCode::Char('i') => {
                    let initial = answers.get(*step).cloned().unwrap_or_default();
                    *current_input = initial;
                    *in_insert_mode = true;
                }
                KeyCode::Enter => {
                    if let Some((err_msg, err_step)) =
                        get_form_error(&form_kind, *step, answers, current_input, *in_insert_mode)
                    {
                        *warning = Some(err_msg);
                        *step = err_step;
                        return Ok(());
                    }

                    submit_data = Some((form_kind.clone(), answers.clone(), name));
                }
                _ => {}
            }
        }

        // Automatically update the warning state based on all form inputs.
        if !exit_form {
            if let Some((err_msg, _)) =
                get_form_error(&form_kind, *step, answers, current_input, *in_insert_mode)
            {
                *warning = Some(err_msg);
            } else {
                *warning = None;
            }
        }
    }

    if exit_form {
        state.mode = AppMode::Browsing;
        state.filtered_projects = (0..state.projects.len()).collect();
        state.filtered_tasks = (0..state.tasks.len()).collect();
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
            let today = chrono::Local::now().date_naive();
            super::date_parser::parse_due_date(s, today)
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
            let today = chrono::Local::now().date_naive();
            super::date_parser::parse_due_date(s, today)
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

fn validate_priority(prio_val: &str) -> Result<(), String> {
    if prio_val.is_empty() {
        return Ok(());
    }
    if let Ok(val) = prio_val.parse::<i64>() {
        if !(1..=3).contains(&val) {
            return Err("Priority must be 1, 2, or 3".to_string());
        }
    } else {
        return Err("Priority must be a number (1, 2, or 3)".to_string());
    }
    Ok(())
}

fn validate_due_date(due_val: &str) -> Result<(), String> {
    if due_val.is_empty() {
        return Ok(());
    }
    let today = chrono::Local::now().date_naive();
    super::date_parser::parse_due_date(due_val, today).map(|_| ())
}

fn get_form_error(
    form_kind: &FormKind,
    step: usize,
    answers: &[String],
    current_input: &str,
    in_insert_mode: bool,
) -> Option<(String, usize)> {
    let get_val = |idx: usize| -> String {
        if idx == step && in_insert_mode {
            current_input.to_string()
        } else {
            answers.get(idx).cloned().unwrap_or_default()
        }
    };

    // 1. Validate tags (index 2)
    if !(in_insert_mode && step == 2) {
        let tags_val = get_val(2);
        if let Err(err_msg) = validate_tags(&tags_val) {
            return Some((err_msg, 2));
        }
    }

    let total = form_total_steps(form_kind);

    // 2. Validate priority (index 3)
    if total > 3 && !(in_insert_mode && step == 3) {
        let prio_val = get_val(3);
        if let Err(err_msg) = validate_priority(&prio_val) {
            return Some((err_msg, 3));
        }
    }

    // 3. Validate due date (index 4)
    if total > 4 && !(in_insert_mode && step == 4) {
        let due_val = get_val(4);
        if let Err(err_msg) = validate_due_date(&due_val) {
            return Some((err_msg, 4));
        }
    }

    None
}
