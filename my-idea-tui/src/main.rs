//! my-idea-tui — термінальна поверхня my-idea (Ratatui).
//!
//! Той самий продукт, що GUI, лише в терміналі. НЕ новий рушій і НЕ друга мова:
//! виконання йде через той самий pinned SENS CLI з `external/sens`.
//!
//! Вертикаль (#85): Open workspace → Explorer → Edit → Save → Run → Output → Stop.

mod app;
mod ast;
mod eco;
mod engine;
mod theme;
// Той самий клієнт swarm-node, що й у GUI — жодної другої реалізації.
#[path = "../../src-tauri/src/swarm.rs"]
mod swarm;
mod i18n;
mod ui;

use std::io;
use std::path::{Path, PathBuf};

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

/// Як і GUI, TUI приймає і робочий каталог, і конкретний Lisp-файл.
/// Фізичний .sens — двійковий T5M, його не можна відкривати як UTF-8.
struct StartupPath {
    workspace: PathBuf,
    file: Option<PathBuf>,
}

fn workspace_from_requested(requested: PathBuf) -> io::Result<StartupPath> {
    let path = requested.canonicalize()?;
    if path.is_dir() {
        return Ok(StartupPath { workspace: path, file: None });
    }
    if !path.is_file() || path.extension().and_then(|ext| ext.to_str()) == Some("sens") {
        return Err(io::Error::new(io::ErrorKind::InvalidInput,
            format!("очікується каталог або текстове Lisp-джерело, не двійковий .sens: {}", path.display())));
    }
    let workspace = path.parent().ok_or_else(|| io::Error::new(
        io::ErrorKind::InvalidInput, "немає батьківського каталогу файла",
    ))?.to_path_buf();
    Ok(StartupPath { workspace, file: Some(path) })
}

/// Аргументи та файл перевіряємо ДО входу в raw/alternate-screen термінал.
fn workspace_from_args() -> io::Result<StartupPath> {
    let mut args = std::env::args_os().skip(1);
    let requested = args.next().map(PathBuf::from);
    if args.next().is_some() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput,
            "використання: my-idea-tui [каталог-проєкту | файл.lisp]"));
    }
    workspace_from_requested(match requested {
        Some(path) => path,
        None => std::env::current_dir()?,
    })
}

fn open_startup_file(app: &mut App, file: &Path) -> io::Result<()> {
    let index = app.files.iter().position(|entry| entry == file)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound,
            format!("Explorer не знайшов файл: {}", file.display())))?;
    app.file_sel = index;
    // Той самий код відкриття, що й Enter в Explorer: вкладка, курсор, Undo.
    app.open_selected();
    if app.editor_path.as_deref() != Some(file) {
        return Err(io::Error::new(io::ErrorKind::InvalidData,
            format!("не вдалося прочитати Lisp-файл у UTF-8: {}", file.display())));
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let startup = workspace_from_args()?;
    let mut app = App::new_in(startup.workspace);
    if let Some(file) = startup.file {
        open_startup_file(&mut app, &file)?;
    }

    let _guard = TerminalGuard::enter()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    run(&mut terminal, &mut app)
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> io::Result<()> {
    loop {
        app.drain_engine();
        app.refresh_ast();
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

#[cfg(test)]
mod startup_file_tests {
    use super::*;

    #[test]
    fn explicit_ukrainian_lisp_path_opens_editor_before_terminal_raw_mode() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "my-idea-tui-startup-{}-{nonce}", std::process::id(),
        ));
        std::fs::create_dir(&directory).unwrap();
        let file = directory.join("програма.лісп");
        let contents = "(додати 2 3)\n";
        std::fs::write(&file, contents).unwrap();

        let startup = workspace_from_requested(file.clone()).unwrap();
        assert_eq!(startup.workspace, directory.canonicalize().unwrap());
        let selected = startup.file.expect("файл для запуску");
        let mut app = App::new_in(startup.workspace);
        open_startup_file(&mut app, &selected).unwrap();
        assert_eq!(app.editor_path.as_deref(), Some(selected.as_path()));
        assert_eq!(app.editor_buf, contents);
        assert_eq!(app.pane, crate::app::Pane::Editor);
        assert!(!app.dirty);
        assert_eq!(app.editor_cursor, contents.len());

        let folder = workspace_from_requested(directory.clone()).unwrap();
        assert_eq!(folder.workspace, directory.canonicalize().unwrap());
        assert!(folder.file.is_none());
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn binary_sens_and_invalid_utf8_fail_before_entering_terminal() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "my-idea-tui-startup-binary-{}-{nonce}", std::process::id(),
        ));
        std::fs::create_dir(&directory).unwrap();
        let binary = directory.join("машина.sens");
        std::fs::write(&binary, [0, 1, 255]).unwrap();
        assert_eq!(workspace_from_requested(binary).err().unwrap().kind(),
            io::ErrorKind::InvalidInput);

        let invalid = directory.join("не-текст.lisp");
        std::fs::write(&invalid, [255, 254]).unwrap();
        let startup = workspace_from_requested(invalid).unwrap();
        let selected = startup.file.unwrap();
        let mut app = App::new_in(startup.workspace);
        assert_eq!(open_startup_file(&mut app, &selected).err().unwrap().kind(),
            io::ErrorKind::InvalidData);
        assert!(app.editor_path.is_none());
        std::fs::remove_dir_all(directory).unwrap();
    }
}
