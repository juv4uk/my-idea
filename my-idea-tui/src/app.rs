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
    pub dirty: bool,
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
            dirty: false,
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

    pub fn t(&self, key: &'static str) -> &'static str {
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
        if let Some(p) = self.files.get(self.file_sel).cloned() {
            match std::fs::read_to_string(&p) {
                Ok(text) => {
                    self.editor_buf = text;
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
            // Кожну форму — окремим рядком у stdin рушія; це механічний транспорт,
            // а не вигадана семантика. Точний вивід приходить у консоль.
            for line in body.lines() {
                let _ = eng.send_line(line);
            }
            self.engine_running = true;
        }
    }

    fn stop(&mut self) {
        if let Some(eng) = self.engine.as_mut() {
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
            let _ = eng.send_line(&line);
        }
    }

    fn meta(&mut self, rest: &str) {
        let mut it = rest.split_whitespace();
        match it.next() {
            Some("мова") | Some("language") => match it.next() {
                Some(l) if i18n::LANGUAGES.contains(&l) => {
                    self.language = l.to_string();
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
        match key.code {
            KeyCode::Char('c') if ctrl => return true,
            KeyCode::Char('q') if ctrl => return true,
            KeyCode::Esc => return true,
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
                    self.editor_buf.push('\n');
                    self.dirty = true;
                }
            }
            KeyCode::Backspace => self.backspace(),
            KeyCode::Up => self.up(),
            KeyCode::Down => self.down(),
            KeyCode::Left => {
                if self.pane == Pane::Editor {
                    self.move_editor_cursor(false);
                }
            }
            KeyCode::Right => {
                if self.pane == Pane::Editor {
                    self.move_editor_cursor(true);
                }
            }
            KeyCode::Char(c) => {
                if self.pane == Pane::Console {
                    self.input.push(c);
                } else if self.pane == Pane::Editor {
                    self.editor_buf.push(c);
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
            Pane::Editor => {
                self.editor_buf.pop();
                self.dirty = true;
            }
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
            Pane::Editor => self.move_editor_cursor(false),
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
            Pane::Editor => self.move_editor_cursor(true),
        }
    }

    // Найпростіший рух «курсора» — вставка/видалення в кінці рядка.
    // Повноцінний редактор-віджет — наступний крок вертикалі, не зараз.
    fn move_editor_cursor(&mut self, _forward: bool) {}

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
