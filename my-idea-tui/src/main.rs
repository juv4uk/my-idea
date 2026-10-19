//! my-idea-tui — термінальна поверхня my-idea (Ratatui).
//!
//! Той самий продукт, що GUI, лише в терміналі. НЕ новий рушій і НЕ друга мова:
//! виконання йде через той самий pinned SENS CLI з `external/sens`.
//!
//! Вертикаль (#85): Open workspace → Explorer → Edit → Save → Run → Output → Stop.

mod app;
mod engine;
mod i18n;
mod ui;

use std::io;
use std::path::PathBuf;

use crossterm::{
    event::{self, DisableBracketedPaste, EnableBracketedPaste, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::App;

/// Гарантоване відновлення термінала (raw/alternate-screen) навіть при помилці.
struct TerminalGuard { bracketed_paste: bool }

impl TerminalGuard {
    fn enter() -> io::Result<Self> {
        enable_raw_mode()?;
        if let Err(error) = execute!(io::stdout(), EnterAlternateScreen) {
            // Raw-режим уже активний: не залишати термінал з вимкненим echo,
            // якщо альтернативний екран недоступний.
            let _ = disable_raw_mode();
            return Err(error);
        }
        // Legacy Windows terminals may not support bracketed paste.
        let bracketed_paste = execute!(io::stdout(), EnableBracketedPaste).is_ok();
        Ok(TerminalGuard { bracketed_paste })
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        if self.bracketed_paste {
            let _ = execute!(io::stdout(), DisableBracketedPaste);
        }
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}

/// Позиційний аргумент задає робочий простір; відмова до входу в raw mode.
fn workspace_from_args() -> io::Result<PathBuf> {
    let mut args = std::env::args_os().skip(1);
    let requested = args.next().map(PathBuf::from);
    if args.next().is_some() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput,
            "використання: my-idea-tui [каталог-проєкту]"));
    }
    let root = match requested {
        Some(path) => path,
        None => std::env::current_dir()?,
    }.canonicalize()?;
    if !root.is_dir() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput,
            format!("не каталог робочого простору: {}", root.display())));
    }
    Ok(root)
}

fn main() -> io::Result<()> {
    let workspace = workspace_from_args()?;
    let _guard = TerminalGuard::enter()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut app = App::new_in(workspace);
    run(&mut terminal, &mut app)
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> io::Result<()> {
    loop {
        app.drain_engine();
        terminal.draw(|f| ui::draw(f, app))?;

        if event::poll(std::time::Duration::from_millis(200))? {
            match event::read()? {
                Event::Paste(text) => app.paste(&text),
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    if app.handle_key(key) {
                        return Ok(());
                    }
                }
                _ => {}
            }
        }
        if app.quit {
            return Ok(());
        }
    }
}
