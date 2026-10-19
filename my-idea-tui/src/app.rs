//! Стан TUI і обробка подій.
//!
//! Вертикаль #85: Open workspace → Explorer → Edit → Save → Run → Output → Stop.
//! Панелі: дерево проєкту | редактор | консоль/вивід.

use std::path::{Path, PathBuf};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::engine::{self, Engine, Line, Origin};
use crate::i18n;

#[derive(Clone, Copy, PartialEq)]
pub enum Pane {
    Explorer,
    Editor,
    Console,
}

pub struct App {
    pub language: String,
    pub pane: Pane,
    pub cwd: PathBuf,
    pub files: Vec<PathBuf>,
    pub file_sel: usize,
    pub editor_path: Option<PathBuf>,
    pub editor_buf: String,
    pub editor_cursor: usize, // UTF-8 byte offset, завжди на межі символу
    pub dirty: bool,
    pub quit_armed: bool,
    pub console: Vec<Line>,
    pub input: String,
    pub history: Vec<String>,
    pub hist_idx: usize,
    pub engine: Option<Engine>,
    pub engine_running: bool,
    pub quit: bool,
    pub status: String,
}

impl App {
    pub fn new() -> App {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut app = App {
            language: "uk".to_string(),
            pane: Pane::Explorer,
            cwd,
            files: Vec::new(),
            file_sel: 0,
            editor_path: None,
            editor_buf: String::new(),
            editor_cursor: 0,
            dirty: false,
            quit_armed: false,
            console: Vec::new(),
            input: String::new(),
            history: Vec::new(),
            hist_idx: 0,
            engine: None,
            engine_running: false,
            quit: false,
            status: String::new(),
        };
        app.sys(i18n::t(&app.language, "banner"));
        app.reload_files();
        app.start_engine();
        app
    }

    pub fn t(&self, key: &str) -> &'static str {
        i18n::t(&self.language, key)
    }

    pub fn pane_label(&self) -> &'static str {
        match self.pane {
            Pane::Explorer => self.t("files"),
            Pane::Editor => self.t("editor"),
            Pane::Console => self.t("console"),
        }
    }

    fn sys(&mut self, text: &str) {
        self.console.push(Line { origin: Origin::System, text: text.to_string() });
    }

    // ---- робочий простір / файли -------------------------------------------

    pub fn reload_files(&mut self) {
        self.files.clear();
        if let Ok(rd) = std::fs::read_dir(&self.cwd) {
            for e in rd.flatten() {
                let p = e.path();
                if p.is_file() {
                    self.files.push(p);
                }
            }
        }
        self.files.sort();
        if self.file_sel >= self.files.len() {
            self.file_sel = self.files.len().saturating_sub(1);
        }
    }

    fn open_selected(&mut self) {
        if self.dirty {
            self.sys("Незбережені зміни: Ctrl+S перед відкриттям іншого файлу");
            return;
        }
        if let Some(p) = self.files.get(self.file_sel).cloned() {
            match std::fs::read_to_string(&p) {
                Ok(text) => {
                    self.editor_buf = text;
                    self.editor_cursor = self.editor_buf.len();
                    self.editor_path = Some(p.clone());
                    self.dirty = false;
                    self.pane = Pane::Editor;
                    self.status = format!("open {}", p.display());
                }
                Err(e) => self.sys(&format!("не вдалось відкрити {}: {e}", p.display())),
            }
        }
    }

    fn save(&mut self) {
        if let Some(p) = self.editor_path.clone() {
            match std::fs::write(&p, &self.editor_buf) {
                Ok(_) => {
                    self.dirty = false;
                    self.status = format!("saved {}", p.display());
                }
                Err(e) => self.sys(&format!("не вдалось зберегти {}: {e}", p.display())),
            }
        }
    }

    // ---- рушій -------------------------------------------------------------

    fn start_engine(&mut self) {
        match engine::resolve_bin() {
            Some(bin) => match Engine::spawn(&bin) {
                Ok(eng) => {
                    self.engine = Some(eng);
                    self.sys(&format!("SENS engine: {bin}"));
                }
                Err(e) => {
                    let msg = format!("{} ({e})", self.t("no-engine"));
                    self.sys(&msg);
                }
            },
            None => {
                let msg = self.t("no-engine").to_string();
                self.sys(&msg);
            }
        }
    }

    /// Run: надіслати вміст поточного файла рушію (реальне виконання pinned SENS).
    fn run_current(&mut self) {
        let Some(path) = self.editor_path.clone() else {
            self.sys("немає відкритого файла для запуску");
            return;
        };
        if self.engine.is_none() {
            self.start_engine();
        }
        self.sys(self.t("run-hint"));
        self.sys(&format!("run {}", path.display()));
        let body = self.editor_buf.clone();
        if let Some(eng) = self.engine.as_mut() {
            // Передаємо весь буфер із точними LF, без обрізання/ігнорування IO-помилок.
            // REPL залишається власником інтерпретації введених форм.
            match eng.send_source(&body) {
                Ok(()) => self.engine_running = true,
                Err(error) => {
                    self.engine_running = false;
                    self.console.push(Line { origin: Origin::Stderr, text: format!("stdin SENS: {error}") });
                }
            }
        }
    }

    fn stop(&mut self) {
        if let Some(mut eng) = self.engine.take() {
            eng.stop();
        }
        self.engine_running = false;
        self.sys(self.t("stop"));
    }

    /// Забрати все, що рушій уже вивів (викликається в циклі малювання).
    pub fn drain_engine(&mut self) {
        if let Some(eng) = self.engine.as_ref() {
            while let Ok(line) = eng.rx.try_recv() {
                self.console.push(line);
            }
        }
    }

    // ---- ввід --------------------------------------------------------------

    pub fn submit(&mut self, line: &str) {
        let line = line.trim().to_string();
        if line.is_empty() {
            return;
        }
        self.history.push(line.clone());
        self.hist_idx = self.history.len();

        if let Some(rest) = line.strip_prefix(':') {
            self.meta(rest);
            return;
        }
        // Звичайний рядок — у консоль і в рушій.
        self.console.push(Line { origin: Origin::System, text: format!("> {line}") });
        if self.engine.is_none() {
            self.start_engine();
        }
        if let Some(eng) = self.engine.as_mut() {
            if let Err(error) = eng.send_line(&line) {
                self.console.push(Line { origin: Origin::Stderr, text: format!("stdin SENS: {error}") });
            }
        }
    }

    fn meta(&mut self, rest: &str) {
        let mut it = rest.split_whitespace();
        match it.next() {
            Some("мова") | Some("language") => match it.next() {
                Some(l) if i18n::LANGUAGES.contains(&l) || l == "ук" => {
                    self.language = if l == "ук" { "uk" } else { l }.to_string();
                    self.sys(&format!("мова інтерфейсу: {l}"));
                }
                Some(l) => self.sys(&format!("невідома мова: {l}")),
                None => self.sys(&format!("мови: {}", i18n::LANGUAGES.join(", "))),
            },
            Some("run") => self.run_current(),
            Some("stop") => self.stop(),
            Some("save") => self.save(),
            _ => self.sys(&format!("невідома команда: :{rest}")),
        }
    }

    /// true — вийти з TUI.
    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        if !(ctrl && matches!(key.code, KeyCode::Char('q' | 'c'))) {
            self.quit_armed = false;
        }
        match key.code {
            KeyCode::Char('s') if ctrl => self.save(),
            KeyCode::Char('c' | 'q') if ctrl => {
                if self.dirty && !self.quit_armed {
                    self.quit_armed = true;
                    self.sys("Незбережені зміни: Ctrl+S — зберегти; Ctrl+Q повторно — вийти без збереження");
                } else {
                    return true;
                }
            },
            KeyCode::Esc => self.pane = Pane::Explorer,
            KeyCode::F(5) => self.run_current(),
            KeyCode::F(9) => self.stop(),
            KeyCode::Tab => self.cycle_pane(),
            KeyCode::BackTab => self.cycle_pane_back(),
            KeyCode::Enter => {
                if self.pane == Pane::Console {
                    let line = std::mem::take(&mut self.input);
                    self.submit(&line);
                } else if self.pane == Pane::Explorer {
                    self.open_selected();
                } else {
                    self.editor_buf.insert(self.editor_cursor, '\n');
                    self.editor_cursor += 1;
                    self.dirty = true;
                }
            }
            KeyCode::Backspace => self.backspace(),
            KeyCode::Up => self.up(),
            KeyCode::Down => self.down(),
            KeyCode::Left => {
                if self.pane == Pane::Editor { self.move_editor_cursor(false); }
            }
            KeyCode::Right => {
                if self.pane == Pane::Editor {
                    self.move_editor_cursor(true);
                }
            }
            KeyCode::Home if self.pane == Pane::Editor => {
                self.editor_cursor = self.editor_buf[..self.editor_cursor].rfind('\n').map_or(0, |p| p + 1);
            }
            KeyCode::End if self.pane == Pane::Editor => {
                self.editor_cursor = self.editor_buf[self.editor_cursor..].find('\n').map_or(self.editor_buf.len(), |p| self.editor_cursor + p);
            }
            KeyCode::Char(c) => {
                if self.pane == Pane::Console {
                    self.input.push(c);
                } else if self.pane == Pane::Editor {
                    self.editor_buf.insert(self.editor_cursor, c);
                    self.editor_cursor += c.len_utf8();
                    self.dirty = true;
                }
            }
            _ => {}
        }
        false
    }

    fn backspace(&mut self) {
        match self.pane {
            Pane::Console => {
                self.input.pop();
            }
            Pane::Editor if self.editor_cursor > 0 => {
                if let Some((previous, _)) = self.editor_buf[..self.editor_cursor].char_indices().last() {
                    self.editor_buf.drain(previous..self.editor_cursor);
                    self.editor_cursor = previous;
                    self.dirty = true;
                }
            }
            Pane::Editor => {}
            Pane::Explorer => {}
        }
    }

    fn up(&mut self) {
        match self.pane {
            Pane::Explorer => self.file_sel = self.file_sel.saturating_sub(1),
            Pane::Console => {
                if self.hist_idx > 0 {
                    self.hist_idx -= 1;
                    self.input = self.history.get(self.hist_idx).cloned().unwrap_or_default();
                }
            }
            Pane::Editor => self.move_editor_line(false),
        }
    }

    fn down(&mut self) {
        match self.pane {
            Pane::Explorer => {
                if self.file_sel + 1 < self.files.len() {
                    self.file_sel += 1;
                }
            }
            Pane::Console => {
                if self.hist_idx + 1 < self.history.len() {
                    self.hist_idx += 1;
                    self.input = self.history[self.hist_idx].clone();
                } else {
                    self.hist_idx = self.history.len();
                    self.input.clear();
                }
            }
            Pane::Editor => self.move_editor_line(true),
        }
    }

    /// Рух по Unicode-символах, без порушення UTF-8 меж.
    fn move_editor_cursor(&mut self, forward: bool) {
        if forward {
            if let Some(ch) = self.editor_buf[self.editor_cursor..].chars().next() {
                self.editor_cursor += ch.len_utf8();
            }
        } else if let Some((i, _)) = self.editor_buf[..self.editor_cursor].char_indices().last() {
            self.editor_cursor = i;
        }
    }

    fn move_editor_line(&mut self, forward: bool) {
        let start = self.editor_buf[..self.editor_cursor].rfind('\n').map_or(0, |p| p + 1);
        let column = self.editor_buf[start..self.editor_cursor].chars().count();
        let (line_start, line_end) = if forward {
            let Some(offset) = self.editor_buf[self.editor_cursor..].find('\n') else { return; };
            let next_start = self.editor_cursor + offset + 1;
            let next_end = self.editor_buf[next_start..].find('\n').map_or(self.editor_buf.len(), |i| next_start + i);
            (next_start, next_end)
        } else {
            if start == 0 { return; }
            let prev_end = start - 1;
            let prev_start = self.editor_buf[..prev_end].rfind('\n').map_or(0, |p| p + 1);
            (prev_start, prev_end)
        };
        let line = &self.editor_buf[line_start..line_end];
        let byte_column = line.char_indices().nth(column).map_or(line.len(), |(i, _)| i);
        self.editor_cursor = line_start + byte_column;
    }

    fn cycle_pane(&mut self) {
        self.pane = match self.pane {
            Pane::Explorer => Pane::Editor,
            Pane::Editor => Pane::Console,
            Pane::Console => Pane::Explorer,
        };
    }

    fn cycle_pane_back(&mut self) {
        self.pane = match self.pane {
            Pane::Explorer => Pane::Console,
            Pane::Editor => Pane::Explorer,
            Pane::Console => Pane::Editor,
        };
    }
}

pub fn display_name(p: &Path) -> String {
    p.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| p.display().to_string())
}
