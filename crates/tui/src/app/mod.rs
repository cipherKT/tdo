use engine::Engine;

mod browsing;
mod confirm;
pub(crate) mod date_parser;
mod form;
mod search;

pub enum AppContext {
    Home,
    Project { name: String, id: i64 },
}

#[derive(Clone)]
pub enum FormKind {
    CreateProject,
    CreateTask,
    ModifyProject { original_name: String },
    ModifyTask { original_name: String },
}

pub enum AppMode {
    Browsing,
    Search {
        buffer: String,
    },
    MultiStepForm {
        kind: FormKind,
        step: usize,
        name: String,
        answers: Vec<String>,
        current_input: String,
        warning: Option<String>,
        in_insert_mode: bool,
    },
    ConfirmPrompt {
        message: String,
        target_name: String,
    },
}

pub struct AppState {
    pub context: AppContext,
    pub mode: AppMode,
    pub projects: Vec<engine::Project>,
    pub tasks: Vec<engine::Task>,
    pub selected: usize,
    pub filtered_projects: Vec<usize>,
    pub filtered_tasks: Vec<usize>,
    pub stats: engine::Stats,
    pub project_stats: engine::Stats,
    pub selected_item_tags: Vec<String>,
    pub projects_task_counts: Vec<(String, i64)>,
    pub pending_today: Vec<engine::NextTask>,
    pub theme: crate::theme::Theme,
}

impl AppState {
    pub fn new(engine: &Engine) -> anyhow::Result<AppState> {
        let projects = engine.list_projects()?;
        let filtered_projects: Vec<usize> = (0..projects.len()).collect();
        let project_stats = if let Some(proj) = projects.first() {
            engine.project_stats(&proj.name)?
        } else {
            engine::Stats::default()
        };
        let tasks = if let Some(proj) = projects.first() {
            engine.list_tasks(&proj.name)?
        } else {
            Vec::new()
        };
        let filtered_tasks = (0..tasks.len()).collect();
        let selected_item_tags = if let Some(proj) = projects.first() {
            engine
                .get_tags_for_project(&proj.name)?
                .into_iter()
                .map(|t| t.name)
                .collect()
        } else {
            Vec::new()
        };

        let mut projects_task_counts = Vec::new();
        for proj in &projects {
            let stats = engine.project_stats(&proj.name)?;
            projects_task_counts.push((proj.name.clone(), stats.total));
        }

        let pending_today = engine.list_pending_today_tasks()?;

        Ok(AppState {
            context: AppContext::Home,
            mode: AppMode::Browsing,
            projects,
            tasks,
            selected: 0,
            filtered_projects,
            filtered_tasks,
            stats: engine.global_stats()?,
            project_stats,
            selected_item_tags,
            projects_task_counts,
            pending_today,
            theme: crate::theme::Theme::load(),
        })
    }
}

pub fn update_stats(state: &mut AppState, engine: &Engine) -> anyhow::Result<()> {
    state.stats = engine.global_stats()?;
    let mut projects_task_counts = Vec::new();
    for proj in &state.projects {
        let stats = engine.project_stats(&proj.name)?;
        projects_task_counts.push((proj.name.clone(), stats.total));
    }
    state.projects_task_counts = projects_task_counts;
    state.pending_today = engine.list_pending_today_tasks()?;

    match &state.context {
        AppContext::Home => {
            if let Some(&proj_idx) = state.filtered_projects.get(state.selected) {
                if let Some(project) = state.projects.get(proj_idx) {
                    state.project_stats = engine.project_stats(&project.name)?;
                    let tasks = engine.list_tasks(&project.name)?;
                    state.tasks = tasks;
                    state.filtered_tasks = (0..state.tasks.len()).collect();
                    let tags = engine.get_tags_for_project(&project.name)?;
                    state.selected_item_tags = tags.into_iter().map(|t| t.name).collect();
                } else {
                    state.project_stats = engine::Stats::default();
                    state.tasks = Vec::new();
                    state.filtered_tasks = Vec::new();
                    state.selected_item_tags = Vec::new();
                }
            } else {
                state.project_stats = engine::Stats::default();
                state.tasks = Vec::new();
                state.filtered_tasks = Vec::new();
                state.selected_item_tags = Vec::new();
            }
        }
        AppContext::Project { name, .. } => {
            state.project_stats = engine.project_stats(name)?;
            if let Some(&task_idx) = state.filtered_tasks.get(state.selected) {
                if let Some(task) = state.tasks.get(task_idx) {
                    let tags = engine.get_tags_for_task(name, &task.name)?;
                    state.selected_item_tags = tags.into_iter().map(|t| t.name).collect();
                } else {
                    state.selected_item_tags = Vec::new();
                }
            } else {
                state.selected_item_tags = Vec::new();
            }
        }
    }
    Ok(())
}

pub fn form_prompt(kind: &FormKind, step: usize) -> &'static str {
    match kind {
        FormKind::CreateProject | FormKind::ModifyProject { .. } => match step {
            0 => "name",
            1 => "description",
            2 => "tags",
            _ => "",
        },
        FormKind::CreateTask | FormKind::ModifyTask { .. } => match step {
            0 => "name",
            1 => "description",
            2 => "tags",
            3 => "priority",
            4 => "due date",
            _ => "",
        },
    }
}

pub fn form_total_steps(kind: &FormKind) -> usize {
    match kind {
        FormKind::CreateProject | FormKind::ModifyProject { .. } => 3,
        FormKind::CreateTask | FormKind::ModifyTask { .. } => 5,
    }
}

pub fn recompute_filter(state: &mut AppState) {
    let buffer = match &state.mode {
        AppMode::Search { buffer } => buffer.to_lowercase(),
        _ => return,
    };
    match &state.context {
        AppContext::Home => {
            state.filtered_projects = state
                .projects
                .iter()
                .enumerate()
                .filter(|(_, p)| p.name.to_lowercase().contains(&buffer))
                .map(|(i, _)| i)
                .collect();
        }
        AppContext::Project { .. } => {
            state.filtered_tasks = state
                .tasks
                .iter()
                .enumerate()
                .filter(|(_, t)| t.name.to_lowercase().contains(&buffer))
                .map(|(i, _)| i)
                .collect();
        }
    }
    state.selected = 0;
}

pub fn handle_key(
    state: &mut AppState,
    key: crossterm::event::KeyEvent,
    engine: &Engine,
) -> anyhow::Result<bool> {
    use crossterm::event::KeyCode;

    state.theme = crate::theme::Theme::load();

    if matches!(state.mode, AppMode::Browsing) && key.code == KeyCode::Char('q') {
        return Ok(true);
    }

    match &state.mode {
        AppMode::Browsing => browsing::handle_browsing(state, key, engine)?,
        AppMode::Search { .. } => search::handle_search(state, key, engine)?,
        AppMode::MultiStepForm { .. } => form::handle_form(state, key, engine)?,
        AppMode::ConfirmPrompt { .. } => confirm::handle_confirm(state, key, engine)?,
    }

    update_stats(state, engine)?;

    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn make_key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn test_app_state_initialization() {
        let engine = Engine::open(":memory:").unwrap();
        engine.create_project("p1", "desc").unwrap();

        let state = AppState::new(&engine).unwrap();
        assert_eq!(state.projects.len(), 1);
        assert_eq!(state.projects[0].name, "p1");
        assert!(matches!(state.context, AppContext::Home));
        assert!(matches!(state.mode, AppMode::Browsing));
    }

    #[test]
    fn test_app_navigation_and_browsing() {
        let engine = Engine::open(":memory:").unwrap();
        engine.create_project("p1", "desc1").unwrap();
        engine.create_project("p2", "desc2").unwrap();

        let mut state = AppState::new(&engine).unwrap();
        assert_eq!(state.selected, 0);

        // Move down
        handle_key(&mut state, make_key(KeyCode::Char('j')), &engine).unwrap();
        assert_eq!(state.selected, 1);

        // Move up
        handle_key(&mut state, make_key(KeyCode::Char('k')), &engine).unwrap();
        assert_eq!(state.selected, 0);

        // Quit key returns true
        let quit = handle_key(&mut state, make_key(KeyCode::Char('q')), &engine).unwrap();
        assert!(quit);
    }

    #[test]
    fn test_app_search_and_filter() {
        let engine = Engine::open(":memory:").unwrap();
        engine.create_project("alpha", "desc").unwrap();
        engine.create_project("beta", "desc").unwrap();

        let mut state = AppState::new(&engine).unwrap();

        // Enter search mode
        handle_key(&mut state, make_key(KeyCode::Char('/')), &engine).unwrap();
        assert!(matches!(state.mode, AppMode::Search { .. }));

        // Type 'l'
        handle_key(&mut state, make_key(KeyCode::Char('l')), &engine).unwrap();
        if let AppMode::Search { buffer } = &state.mode {
            assert_eq!(buffer, "l");
        } else {
            panic!("Expected search mode");
        }

        // Filtering should match "alpha" (selected 0) but not "beta" (length of filtered is 1)
        assert_eq!(state.filtered_projects.len(), 1);
        assert_eq!(state.projects[state.filtered_projects[0]].name, "alpha");

        // Backspace
        handle_key(&mut state, make_key(KeyCode::Backspace), &engine).unwrap();
        if let AppMode::Search { buffer } = &state.mode {
            assert_eq!(buffer, "");
        }
        assert_eq!(state.filtered_projects.len(), 2);

        // Esc exits search
        handle_key(&mut state, make_key(KeyCode::Esc), &engine).unwrap();
        assert!(matches!(state.mode, AppMode::Browsing));
    }

    #[test]
    fn test_app_project_creation_form() {
        let engine = Engine::open(":memory:").unwrap();
        let mut state = AppState::new(&engine).unwrap();

        // Search for "new_proj" and hit Enter to trigger project creation form
        handle_key(&mut state, make_key(KeyCode::Char('/')), &engine).unwrap();
        for c in "new_proj".chars() {
            handle_key(&mut state, make_key(KeyCode::Char(c)), &engine).unwrap();
        }
        handle_key(&mut state, make_key(KeyCode::Enter), &engine).unwrap();

        // Now in MultiStepForm for project
        assert!(matches!(
            state.mode,
            AppMode::MultiStepForm {
                kind: FormKind::CreateProject,
                step: 0,
                ..
            }
        ));

        // Move to step 1 (Description) and edit it
        handle_key(&mut state, make_key(KeyCode::Char('j')), &engine).unwrap();
        handle_key(&mut state, make_key(KeyCode::Char('i')), &engine).unwrap();
        for c in "desc".chars() {
            handle_key(&mut state, make_key(KeyCode::Char(c)), &engine).unwrap();
        }
        handle_key(&mut state, make_key(KeyCode::Esc), &engine).unwrap();

        // Move to step 2 (Tags) and edit it
        handle_key(&mut state, make_key(KeyCode::Char('j')), &engine).unwrap();
        handle_key(&mut state, make_key(KeyCode::Char('i')), &engine).unwrap();
        for c in "#t1 #t2".chars() {
            handle_key(&mut state, make_key(KeyCode::Char(c)), &engine).unwrap();
        }
        handle_key(&mut state, make_key(KeyCode::Esc), &engine).unwrap();

        // Save form by pressing Enter
        handle_key(&mut state, make_key(KeyCode::Enter), &engine).unwrap();

        // Completed! App mode goes back to browsing
        assert!(matches!(state.mode, AppMode::Browsing));
        assert_eq!(state.projects.len(), 1);
        assert_eq!(state.projects[0].name, "new_proj");

        let tags = engine.get_tags_for_project("new_proj").unwrap();
        assert_eq!(tags.len(), 2);
    }

    #[test]
    fn test_app_task_creation_form() {
        let engine = Engine::open(":memory:").unwrap();
        engine.create_project("proj", "desc").unwrap();

        let mut state = AppState::new(&engine).unwrap();

        // Enter project "proj" by hitting Enter on it
        handle_key(&mut state, make_key(KeyCode::Enter), &engine).unwrap();
        assert!(matches!(state.context, AppContext::Project { .. }));

        // Enter search mode
        handle_key(&mut state, make_key(KeyCode::Char('/')), &engine).unwrap();

        // Search for task "task_test"
        for c in "task_test".chars() {
            handle_key(&mut state, make_key(KeyCode::Char(c)), &engine).unwrap();
        }

        // Hit enter to start task creation form
        handle_key(&mut state, make_key(KeyCode::Enter), &engine).unwrap();
        assert!(matches!(
            state.mode,
            AppMode::MultiStepForm {
                kind: FormKind::CreateTask,
                step: 0,
                ..
            }
        ));

        // Step 1: description
        handle_key(&mut state, make_key(KeyCode::Char('j')), &engine).unwrap();
        handle_key(&mut state, make_key(KeyCode::Char('i')), &engine).unwrap();
        for c in "task desc".chars() {
            handle_key(&mut state, make_key(KeyCode::Char(c)), &engine).unwrap();
        }
        handle_key(&mut state, make_key(KeyCode::Esc), &engine).unwrap();

        // Step 2: tags
        handle_key(&mut state, make_key(KeyCode::Char('j')), &engine).unwrap();
        handle_key(&mut state, make_key(KeyCode::Char('i')), &engine).unwrap();
        for c in "#task_tag".chars() {
            handle_key(&mut state, make_key(KeyCode::Char(c)), &engine).unwrap();
        }
        handle_key(&mut state, make_key(KeyCode::Esc), &engine).unwrap();

        // Step 3: priority (1/2/3)
        handle_key(&mut state, make_key(KeyCode::Char('j')), &engine).unwrap();
        handle_key(&mut state, make_key(KeyCode::Char('i')), &engine).unwrap();
        handle_key(&mut state, make_key(KeyCode::Backspace), &engine).unwrap(); // clear default "3"
        handle_key(&mut state, make_key(KeyCode::Char('1')), &engine).unwrap();
        handle_key(&mut state, make_key(KeyCode::Esc), &engine).unwrap();

        // Step 4: due date (leave blank)
        handle_key(&mut state, make_key(KeyCode::Char('j')), &engine).unwrap();

        // Submit form
        handle_key(&mut state, make_key(KeyCode::Enter), &engine).unwrap();

        // Completed task creation! Mode back to browsing
        assert!(matches!(state.mode, AppMode::Browsing));
        assert_eq!(state.tasks.len(), 1);
        assert_eq!(state.tasks[0].name, "task_test");
        assert_eq!(state.tasks[0].priority, 1);
    }

    #[test]
    fn test_app_task_creation_form_with_relative_due_date() {
        let engine = Engine::open(":memory:").unwrap();
        engine.create_project("proj", "desc").unwrap();

        let mut state = AppState::new(&engine).unwrap();

        // Enter project "proj" by hitting Enter on it
        handle_key(&mut state, make_key(KeyCode::Enter), &engine).unwrap();
        assert!(matches!(state.context, AppContext::Project { .. }));

        // Enter search mode
        handle_key(&mut state, make_key(KeyCode::Char('/')), &engine).unwrap();

        // Search for task "task_test"
        for c in "task_test".chars() {
            handle_key(&mut state, make_key(KeyCode::Char(c)), &engine).unwrap();
        }

        // Hit enter to start task creation form
        handle_key(&mut state, make_key(KeyCode::Enter), &engine).unwrap();
        assert!(matches!(
            state.mode,
            AppMode::MultiStepForm {
                kind: FormKind::CreateTask,
                step: 0,
                ..
            }
        ));

        // Go to Step 4: due date
        for _ in 0..4 {
            handle_key(&mut state, make_key(KeyCode::Char('j')), &engine).unwrap();
        }

        // Edit due date field
        handle_key(&mut state, make_key(KeyCode::Char('i')), &engine).unwrap();
        // Type "+3"
        handle_key(&mut state, make_key(KeyCode::Char('+')), &engine).unwrap();
        handle_key(&mut state, make_key(KeyCode::Char('3')), &engine).unwrap();
        // Exit insert mode - this should normalize the date
        handle_key(&mut state, make_key(KeyCode::Esc), &engine).unwrap();

        // Verify it was normalized in AppMode answers
        if let AppMode::MultiStepForm { answers, .. } = &state.mode {
            let normalized = &answers[4];
            let expected_date = chrono::Local::now().date_naive() + chrono::Days::new(3);
            assert_eq!(normalized, &expected_date.format("%Y-%m-%d").to_string());
        } else {
            panic!("expected MultiStepForm mode");
        }

        // Submit form
        handle_key(&mut state, make_key(KeyCode::Enter), &engine).unwrap();

        // Verify task was created with correct date
        assert!(matches!(state.mode, AppMode::Browsing));
        assert_eq!(state.tasks.len(), 1);
        let task = &state.tasks[0];
        let expected_date = (chrono::Local::now().date_naive() + chrono::Days::new(3))
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();
        assert_eq!(task.due_date, Some(expected_date));
    }

    #[test]
    fn test_app_project_deletion_confirm() {
        let engine = Engine::open(":memory:").unwrap();
        engine.create_project("p1", "desc").unwrap();

        let mut state = AppState::new(&engine).unwrap();

        // Trigger delete prompt with 'd'
        handle_key(&mut state, make_key(KeyCode::Char('d')), &engine).unwrap();
        assert!(matches!(state.mode, AppMode::ConfirmPrompt { .. }));

        // Confirm with 'y'
        handle_key(&mut state, make_key(KeyCode::Char('y')), &engine).unwrap();
        assert!(matches!(state.mode, AppMode::Browsing));
        assert_eq!(state.projects.len(), 0);
    }

    #[test]
    fn test_app_stats_tracking() {
        let engine = Engine::open(":memory:").unwrap();
        engine.create_project("p1", "desc1").unwrap();
        engine.create_project("p2", "desc2").unwrap();

        // Add a task to p1
        engine.create_task("p1", "t1", "d", 1, None).unwrap();

        // Initialize state
        let mut state = AppState::new(&engine).unwrap();

        // Initially global stats should show 1 task, done 0, pending 1
        assert_eq!(state.stats.total, 1);
        assert_eq!(state.stats.pending, 1);
        // And project stats should be for "p1" (selected = 0)
        assert_eq!(state.project_stats.total, 1);
        assert_eq!(state.project_stats.pending, 1);

        // Move to p2 (selected = 1)
        handle_key(&mut state, make_key(KeyCode::Char('j')), &engine).unwrap();
        assert_eq!(state.selected, 1);
        // project stats should update for "p2", which has 0 tasks
        assert_eq!(state.project_stats.total, 0);

        // Enter project p2
        handle_key(&mut state, make_key(KeyCode::Enter), &engine).unwrap();
        assert!(matches!(state.context, AppContext::Project { .. }));

        // Trigger search and type a non-existent task to create it
        handle_key(&mut state, make_key(KeyCode::Char('/')), &engine).unwrap();
        for c in "new_task".chars() {
            handle_key(&mut state, make_key(KeyCode::Char(c)), &engine).unwrap();
        }
        handle_key(&mut state, make_key(KeyCode::Enter), &engine).unwrap(); // Start form

        // Fill form steps:
        // Description (skip)
        // Tags (skip)
        // Priority (set to 2)
        // Due date (skip)
        handle_key(&mut state, make_key(KeyCode::Char('j')), &engine).unwrap();
        handle_key(&mut state, make_key(KeyCode::Char('j')), &engine).unwrap();
        handle_key(&mut state, make_key(KeyCode::Char('j')), &engine).unwrap();
        handle_key(&mut state, make_key(KeyCode::Char('i')), &engine).unwrap();
        handle_key(&mut state, make_key(KeyCode::Backspace), &engine).unwrap(); // clear default "3"
        handle_key(&mut state, make_key(KeyCode::Char('2')), &engine).unwrap();
        handle_key(&mut state, make_key(KeyCode::Esc), &engine).unwrap();
        handle_key(&mut state, make_key(KeyCode::Enter), &engine).unwrap();

        // Now stats should be updated!
        // Global stats total should be 2 (1 in p1, 1 in p2)
        assert_eq!(state.stats.total, 2);
        // Project stats for current project (p2) should be 1
        assert_eq!(state.project_stats.total, 1);
        assert_eq!(state.project_stats.p2, 1);

        // Toggle task done
        assert_eq!(state.selected, 0);
        handle_key(&mut state, make_key(KeyCode::Char(' ')), &engine).unwrap();
        // Now pending should be 0, done 1 for p2 project stats, and 1 done in global
        assert_eq!(state.project_stats.done, 1);
        assert_eq!(state.project_stats.pending, 0);
        assert_eq!(state.stats.done, 1);
        assert_eq!(state.stats.pending, 1); // 1 pending in p1, 0 in p2
    }

    #[test]
    fn test_app_tag_validation() {
        let engine = Engine::open(":memory:").unwrap();
        let mut state = AppState::new(&engine).unwrap();

        // Start project creation form
        handle_key(&mut state, make_key(KeyCode::Char('/')), &engine).unwrap();
        for c in "new_proj".chars() {
            handle_key(&mut state, make_key(KeyCode::Char(c)), &engine).unwrap();
        }
        handle_key(&mut state, make_key(KeyCode::Enter), &engine).unwrap();

        // Move to step 2 (tags)
        handle_key(&mut state, make_key(KeyCode::Char('j')), &engine).unwrap(); // step 1
        handle_key(&mut state, make_key(KeyCode::Char('j')), &engine).unwrap(); // step 2

        assert!(matches!(
            state.mode,
            AppMode::MultiStepForm {
                kind: FormKind::CreateProject,
                step: 2,
                ..
            }
        ));

        // Enter insert mode to edit tags
        handle_key(&mut state, make_key(KeyCode::Char('i')), &engine).unwrap();
        for c in "invalid_tag".chars() {
            handle_key(&mut state, make_key(KeyCode::Char(c)), &engine).unwrap();
        }
        // Hit Esc to exit insert mode (validates and sets warning)
        handle_key(&mut state, make_key(KeyCode::Esc), &engine).unwrap();

        // Should still be at step 2, with a warning
        if let AppMode::MultiStepForm { step, warning, .. } = &state.mode {
            assert_eq!(*step, 2);
            assert!(warning.is_some());
            assert!(warning.as_ref().unwrap().contains("must start with '#'"));
        } else {
            panic!("Expected MultiStepForm");
        }

        // Enter insert mode again (warning is cleared, backspacing all clears it on Esc)
        handle_key(&mut state, make_key(KeyCode::Char('i')), &engine).unwrap();
        for _ in 0..11 {
            handle_key(&mut state, make_key(KeyCode::Backspace), &engine).unwrap();
        }
        handle_key(&mut state, make_key(KeyCode::Esc), &engine).unwrap();
        if let AppMode::MultiStepForm { warning, .. } = &state.mode {
            assert!(warning.is_none());
        }

        // Enter insert mode to type invalid tag
        handle_key(&mut state, make_key(KeyCode::Char('i')), &engine).unwrap();
        for c in "#tag$name".chars() {
            handle_key(&mut state, make_key(KeyCode::Char(c)), &engine).unwrap();
        }
        handle_key(&mut state, make_key(KeyCode::Esc), &engine).unwrap();

        // Should still be at step 2, with a warning
        if let AppMode::MultiStepForm { step, warning, .. } = &state.mode {
            assert_eq!(*step, 2);
            assert!(warning.is_some());
            assert!(
                warning
                    .as_ref()
                    .unwrap()
                    .contains("contains invalid character")
            );
        } else {
            panic!("Expected MultiStepForm");
        }

        // Enter insert mode, clear buffer, and type valid tags
        handle_key(&mut state, make_key(KeyCode::Char('i')), &engine).unwrap();
        for _ in 0..9 {
            handle_key(&mut state, make_key(KeyCode::Backspace), &engine).unwrap();
        }
        for c in "#valid-tag #valid_tag2 #valid3".chars() {
            handle_key(&mut state, make_key(KeyCode::Char(c)), &engine).unwrap();
        }
        handle_key(&mut state, make_key(KeyCode::Esc), &engine).unwrap();

        // Submit form
        handle_key(&mut state, make_key(KeyCode::Enter), &engine).unwrap();

        // Completed!
        assert!(matches!(state.mode, AppMode::Browsing));
        assert_eq!(state.projects.len(), 1);

        let tags = engine.get_tags_for_project("new_proj").unwrap();
        assert_eq!(tags.len(), 3);
    }

    #[test]
    fn test_app_pending_today() {
        let engine = Engine::open(":memory:").unwrap();
        engine.create_project("proj", "desc").unwrap();
        let today = chrono::Utc::now();
        engine
            .create_task("proj", "task1", "desc", 1, Some(today))
            .unwrap();

        let mut state = AppState::new(&engine).unwrap();
        assert_eq!(state.pending_today.len(), 1);
        assert_eq!(state.pending_today[0].task.name, "task1");
        assert_eq!(state.pending_today[0].project_name, "proj");

        // Complete the task and see if it's removed from pending_today
        engine.toggle_done("proj", "task1").unwrap();
        update_stats(&mut state, &engine).unwrap();
        assert_eq!(state.pending_today.len(), 0);
    }
}
