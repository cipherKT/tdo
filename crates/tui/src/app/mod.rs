use engine::Engine;

mod browsing;
mod confirm;
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
        Ok(AppState {
            context: AppContext::Home,
            mode: AppMode::Browsing,
            projects,
            tasks: Vec::new(),
            selected: 0,
            filtered_projects,
            filtered_tasks: Vec::new(),
            stats: engine.global_stats()?,
            project_stats,
        })
    }
}

pub fn update_stats(state: &mut AppState, engine: &Engine) -> anyhow::Result<()> {
    state.stats = engine.global_stats()?;
    match &state.context {
        AppContext::Home => {
            if let Some(&proj_idx) = state.filtered_projects.get(state.selected) {
                if let Some(project) = state.projects.get(proj_idx) {
                    state.project_stats = engine.project_stats(&project.name)?;
                } else {
                    state.project_stats = engine::Stats::default();
                }
            } else {
                state.project_stats = engine::Stats::default();
            }
        }
        AppContext::Project { name, .. } => {
            state.project_stats = engine.project_stats(name)?;
        }
    }
    Ok(())
}

pub fn form_prompt(kind: &FormKind, step: usize) -> &'static str {
    match kind {
        FormKind::CreateProject | FormKind::ModifyProject { .. } => match step {
            0 => "name",
            1 => "description",
            2 => "tags (space-separated, e.g. #security #recon)",
            _ => "",
        },
        FormKind::CreateTask | FormKind::ModifyTask { .. } => match step {
            0 => "name",
            1 => "description",
            2 => "tags (space-separated)",
            3 => "priority (1/2/3)",
            4 => "due date (YYYY-MM-DD or leave blank)",
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
                step: 1,
                ..
            }
        ));

        // Fill description step (step 1)
        for c in "desc".chars() {
            handle_key(&mut state, make_key(KeyCode::Char(c)), &engine).unwrap();
        }
        handle_key(&mut state, make_key(KeyCode::Enter), &engine).unwrap();

        // Now step 2 (tags)
        assert!(matches!(
            state.mode,
            AppMode::MultiStepForm {
                kind: FormKind::CreateProject,
                step: 2,
                ..
            }
        ));
        for c in "#t1 #t2".chars() {
            handle_key(&mut state, make_key(KeyCode::Char(c)), &engine).unwrap();
        }
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
                step: 1,
                ..
            }
        ));

        // Step 1: description
        for c in "task desc".chars() {
            handle_key(&mut state, make_key(KeyCode::Char(c)), &engine).unwrap();
        }
        handle_key(&mut state, make_key(KeyCode::Enter), &engine).unwrap();

        // Step 2: tags
        for c in "#task_tag".chars() {
            handle_key(&mut state, make_key(KeyCode::Char(c)), &engine).unwrap();
        }
        handle_key(&mut state, make_key(KeyCode::Enter), &engine).unwrap();

        // Step 3: priority (1/2/3)
        handle_key(&mut state, make_key(KeyCode::Char('1')), &engine).unwrap();
        handle_key(&mut state, make_key(KeyCode::Enter), &engine).unwrap();

        // Step 4: due date
        handle_key(&mut state, make_key(KeyCode::Enter), &engine).unwrap();

        // Completed task creation! Mode back to browsing
        assert!(matches!(state.mode, AppMode::Browsing));
        assert_eq!(state.tasks.len(), 1);
        assert_eq!(state.tasks[0].name, "task_test");
        assert_eq!(state.tasks[0].priority, 1);
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
        // Description
        handle_key(&mut state, make_key(KeyCode::Enter), &engine).unwrap();
        // Tags
        handle_key(&mut state, make_key(KeyCode::Enter), &engine).unwrap();
        // Priority
        handle_key(&mut state, make_key(KeyCode::Char('2')), &engine).unwrap();
        handle_key(&mut state, make_key(KeyCode::Enter), &engine).unwrap();
        // Due date
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

        // Fill description step
        handle_key(&mut state, make_key(KeyCode::Enter), &engine).unwrap();

        // Now in MultiStepForm at step 2 (tags)
        assert!(matches!(
            state.mode,
            AppMode::MultiStepForm {
                kind: FormKind::CreateProject,
                step: 2,
                ..
            }
        ));

        // Let's type an invalid tag: "invalid_tag" (missing '#')
        for c in "invalid_tag".chars() {
            handle_key(&mut state, make_key(KeyCode::Char(c)), &engine).unwrap();
        }
        // Hit Enter
        handle_key(&mut state, make_key(KeyCode::Enter), &engine).unwrap();

        // Should still be at step 2, with a warning
        if let AppMode::MultiStepForm { step, warning, .. } = &state.mode {
            assert_eq!(*step, 2);
            assert!(warning.is_some());
            assert!(warning.as_ref().unwrap().contains("must start with '#'"));
        } else {
            panic!("Expected MultiStepForm");
        }

        // Type backspace (clears warning)
        handle_key(&mut state, make_key(KeyCode::Backspace), &engine).unwrap();
        if let AppMode::MultiStepForm { warning, .. } = &state.mode {
            assert!(warning.is_none());
        }

        // Clear the buffer by backspacing the rest
        for _ in 0..10 {
            handle_key(&mut state, make_key(KeyCode::Backspace), &engine).unwrap();
        }

        // Type another invalid tag: "#tag$name" (invalid char '$')
        for c in "#tag$name".chars() {
            handle_key(&mut state, make_key(KeyCode::Char(c)), &engine).unwrap();
        }
        handle_key(&mut state, make_key(KeyCode::Enter), &engine).unwrap();

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

        // Clear the buffer
        for _ in 0..9 {
            handle_key(&mut state, make_key(KeyCode::Backspace), &engine).unwrap();
        }

        // Type valid tags: "#valid-tag #valid_tag2 #valid3"
        for c in "#valid-tag #valid_tag2 #valid3".chars() {
            handle_key(&mut state, make_key(KeyCode::Char(c)), &engine).unwrap();
        }
        handle_key(&mut state, make_key(KeyCode::Enter), &engine).unwrap();

        // Completed!
        assert!(matches!(state.mode, AppMode::Browsing));
        assert_eq!(state.projects.len(), 1);

        let tags = engine.get_tags_for_project("new_proj").unwrap();
        assert_eq!(tags.len(), 3);
    }
}
