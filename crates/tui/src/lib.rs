use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use engine::Engine;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;

mod app;
mod theme;
mod ui;

pub use app::AppState;

struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
    }
}

pub fn run(engine: Engine) -> anyhow::Result<()> {
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    let _guard = TerminalGuard;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    let mut state = AppState::new(&engine)?;

    loop {
        terminal.draw(|frame| ui::render(frame, &state))?;

        if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
            if app::handle_key(&mut state, key, &engine)? {
                break;
            }
        }
    }

    Ok(())
}
