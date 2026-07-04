use crate::app::{
    AppContext, AppMode, AppState, CalendarState, FormKind, RightPane, recompute_filter,
};
use engine::Engine;

pub(super) fn handle_browsing(
    state: &mut AppState,
    key: crossterm::event::KeyEvent,
    engine: &Engine,
) -> anyhow::Result<()> {
    use crossterm::event::KeyCode;

    // Tab always toggles the active right pane.
    if key.code == KeyCode::Tab {
        state.right_pane = match state.right_pane {
            RightPane::PendingToday => {
                // Refresh day_tasks when entering the calendar.
                refresh_calendar_day(state, engine)?;
                RightPane::Calendar
            }
            RightPane::Calendar => RightPane::PendingToday,
        };
        return Ok(());
    }

    // When the calendar pane is focused, route vim motion keys to it.
    if state.right_pane == RightPane::Calendar {
        handle_calendar_key(state, key, engine)?;
        return Ok(());
    }

    // --- Normal browsing ---
    let list_len = match &state.context {
        AppContext::Home => state.projects.len(),
        AppContext::Project { .. } => state.tasks.len(),
    };

    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            if list_len > 0 {
                state.selected = (state.selected + 1) % list_len;
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
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

// ---------------------------------------------------------------------------
// Calendar navigation
// ---------------------------------------------------------------------------

/// Handle a keypress when the Calendar pane is focused.
fn handle_calendar_key(
    state: &mut AppState,
    key: crossterm::event::KeyEvent,
    engine: &Engine,
) -> anyhow::Result<()> {
    use crossterm::event::KeyCode;

    match key.code {
        KeyCode::Char('h') | KeyCode::Left => {
            // Move cursor left (previous day).
            move_cursor(state, -1, 0);
            refresh_calendar_day(state, engine)?;
        }
        KeyCode::Char('l') | KeyCode::Right => {
            // Move cursor right (next day).
            move_cursor(state, 1, 0);
            refresh_calendar_day(state, engine)?;
        }
        KeyCode::Char('j') | KeyCode::Down => {
            // Move cursor down (next week).
            move_cursor(state, 0, 1);
            refresh_calendar_day(state, engine)?;
        }
        KeyCode::Char('k') | KeyCode::Up => {
            // Move cursor up (previous week).
            move_cursor(state, 0, -1);
            refresh_calendar_day(state, engine)?;
        }
        KeyCode::Enter => {
            // Already shows tasks in the pane; pressing Enter re-fetches.
            refresh_calendar_day(state, engine)?;
        }
        KeyCode::Esc => {
            // Switch back to PendingToday.
            state.right_pane = RightPane::PendingToday;
        }
        _ => {}
    }

    // Update cursor_day after movement.
    let cal = &mut state.calendar;
    if let Some(d) = day_for_cursor(cal.year, cal.month, cal.cursor_row, cal.cursor_col) {
        cal.cursor_day = d;
    }

    Ok(())
}

/// Move the calendar cursor by (dcol, drow), wrapping month boundaries.
fn move_cursor(state: &mut AppState, dcol: i32, drow: i32) {
    use chrono::{Datelike, NaiveDate};

    let cal = &mut state.calendar;

    // Compute absolute day index from current cursor position.
    let first = NaiveDate::from_ymd_opt(cal.year, cal.month, 1).unwrap();
    let offset = first.weekday().num_days_from_monday() as i32; // padding before day 1
    let current_cell = (cal.cursor_row as i32) * 7 + (cal.cursor_col as i32);
    let current_day_idx = current_cell - offset; // 0-based day index in month

    // Apply delta.
    let total_delta = dcol + drow * 7;
    let new_day_idx = current_day_idx + total_delta;

    let month_days = days_in_month(cal.year, cal.month) as i32;

    if new_day_idx < 0 {
        // Navigate to previous month.
        let (prev_year, prev_month) = prev_month(cal.year, cal.month);
        let prev_days = days_in_month(prev_year, prev_month) as i32;
        // Place cursor on the last few days of the previous month.
        let target_day = (prev_days + new_day_idx + 1).max(1);
        cal.year = prev_year;
        cal.month = prev_month;
        set_cursor_to_day(cal, target_day as u32);
    } else if new_day_idx >= month_days {
        // Navigate to next month.
        let (next_year, next_month) = next_month(cal.year, cal.month);
        let target_day = (new_day_idx - month_days + 1).max(1);
        let next_days = days_in_month(next_year, next_month) as i32;
        let target_day = target_day.min(next_days);
        cal.year = next_year;
        cal.month = next_month;
        set_cursor_to_day(cal, target_day as u32);
    } else {
        let target_day = new_day_idx + 1; // 1-based day
        set_cursor_to_day(cal, target_day as u32);
    }
}

/// Set the cursor (row/col) so it sits on `day` of the current year/month.
fn set_cursor_to_day(cal: &mut CalendarState, day: u32) {
    use chrono::{Datelike, NaiveDate};
    let first = NaiveDate::from_ymd_opt(cal.year, cal.month, 1).unwrap();
    let offset = first.weekday().num_days_from_monday();
    let cell = (day - 1) + offset;
    cal.cursor_row = cell / 7;
    cal.cursor_col = cell % 7;
    cal.cursor_day = day;
}

/// Return the day-of-month for a given grid cell, or None if it's a padding cell.
fn day_for_cursor(year: i32, month: u32, row: u32, col: u32) -> Option<u32> {
    use chrono::{Datelike, NaiveDate};
    let first = NaiveDate::from_ymd_opt(year, month, 1)?;
    let offset = first.weekday().num_days_from_monday();
    let cell = row * 7 + col;
    if cell < offset {
        return None;
    }
    let day = cell - offset + 1;
    if day > days_in_month(year, month) {
        return None;
    }
    Some(day)
}

/// Fetch tasks for the day the cursor is on and store them in `state.calendar.day_tasks`.
pub(crate) fn refresh_calendar_day(state: &mut AppState, engine: &Engine) -> anyhow::Result<()> {
    let (year, month, day) = (
        state.calendar.year,
        state.calendar.month,
        state.calendar.cursor_day,
    );
    if day == 0 {
        state.calendar.day_tasks = Some(Vec::new());
        return Ok(());
    }
    let tasks = engine.list_tasks_due_on(year, month, day)?;
    state.calendar.day_tasks = Some(tasks);
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn days_in_month(year: i32, month: u32) -> u32 {
    use chrono::{Datelike, NaiveDate};
    // First day of next month minus one day.
    let (ny, nm) = next_month(year, month);
    NaiveDate::from_ymd_opt(ny, nm, 1)
        .unwrap()
        .pred_opt()
        .unwrap()
        .day()
}

fn prev_month(year: i32, month: u32) -> (i32, u32) {
    if month == 1 {
        (year - 1, 12)
    } else {
        (year, month - 1)
    }
}

fn next_month(year: i32, month: u32) -> (i32, u32) {
    if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    }
}
