use engine::Engine;

pub enum AppContext {
    Home,
    Project { name: String, id: i64 },
}

pub enum AppMode {
    Browsing,
    Search { buffer: String },
}

pub struct AppState {
    pub context: AppContext,
    pub mode: AppMode,
    pub projects: Vec<engine::Project>,
    pub tasks: Vec<engine::Task>,
    pub selected: usize,
}

impl AppState {
    pub fn new(engine: &Engine) -> anyhow::Result<AppState> {
        let projects = engine.list_projects()?;
        let tasks = Vec::new();
        Ok(AppState {
            context: AppContext::Home,
            mode: AppMode::Browsing,
            projects,
            tasks,
            selected: 0,
        })
    }
}

pub fn handle_key(
    state: &mut AppState,
    key: crossterm::event::KeyEvent,
    engine: &Engine,
) -> anyhow::Result<bool> {
    use crossterm::event::KeyCode;
    if key.code == KeyCode::Char('q') {
        return Ok(true); // signal quit
    }
    if key.code == KeyCode::Char('j') {
        let len = match &state.context {
            AppContext::Home => state.projects.len(),
            AppContext::Project { .. } => state.tasks.len(),
        };
        if len > 0 {
            state.selected = (state.selected + 1) % len;
        }
    }
    if key.code == KeyCode::Char('k') {
        let len = match &state.context {
            AppContext::Home => state.projects.len(),
            AppContext::Project { .. } => state.tasks.len(),
        };
        if len > 0 {
            state.selected = state.selected.saturating_sub(1);
        }
    }

    match (&state.context, key.code) {
        (AppContext::Home, KeyCode::Enter) => {
            if !state.projects.is_empty() {
                let project = &state.projects[state.selected];
                let tasks = engine.list_tasks(&project.name)?;
                state.context = AppContext::Project {
                    name: project.name.clone(),
                    id: project.id,
                };
                state.tasks = tasks;
                state.selected = 0;
            }
        }
        (AppContext::Project { .. }, KeyCode::Esc) => {
            state.context = AppContext::Home;
            state.tasks = Vec::new();
            state.selected = 0;
        }
        _ => {}
    }

    Ok(false)
}
