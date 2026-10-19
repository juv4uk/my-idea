//! Стан TUI і обробка подій.
//!
//! Вертикаль #85: Open workspace → Explorer → Edit → Save → Run → Output → Stop.
//! Панелі: дерево проєкту | редактор | консоль/вивід.

use std::path::{Path, PathBuf};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::engine::{self, Engine, Line, Origin};
use crate::i18n;

#[path = "../../src-tauri/src/workspace_save.rs"]
mod workspace_save;

#[derive(Clone, Copy, PartialEq)]
pub enum Pane {
    Explorer,
    Editor,
    Console,
}

pub struct App {
    pub language: String,
    pub pane: Pane,
    pub cwd: PathBuf, // Незмінний корінь workspace, зокрема для безпечного Save
    pub explorer_dir: PathBuf, // Поточний каталог Explorer усередині workspace
    pub files: Vec<PathBuf>,
    pub file_sel: usize,
    pub editor_path: Option<PathBuf>,
    pub editor_buf: String,
    // Текст відкритого файла на диску при останньому Open/Save.
    pub editor_original: String,
    pub editor_cursor: usize, // UTF-8 byte offset, завжди на межі символу
    pub dirty: bool,
    pub quit_armed: bool,
    pub console: Vec<Line>,
    /// Скільки логічних рядків відступити від хвоста консолі (0 = останній вивід).
    pub console_scroll_back: usize,
    pub input: String,
    pub history: Vec<String>,
    pub hist_idx: usize,
    pub repl_engine: Option<Engine>,
    pub run_engine: Option<Engine>,
    pub engine_running: bool,
    pub quit: bool,
    pub status: String,
}

impl App {
    pub fn new() -> App {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self::new_in(cwd)
    }

    /// Ініціалізація обраного каталогу без зміни cwd процесу та SENS-оточення.
    pub fn new_in(cwd: PathBuf) -> App {
        let mut app = App {
            language: "uk".to_string(),
            pane: Pane::Explorer,
            explorer_dir: cwd.clone(),
            cwd,
            files: Vec::new(),
            file_sel: 0,
            editor_path: None,
            editor_buf: String::new(),
            editor_original: String::new(),
            editor_cursor: 0,
            dirty: false,
            quit_armed: false,
            console: Vec::new(),
            console_scroll_back: 0,
            input: String::new(),
            history: Vec::new(),
            hist_idx: 0,
            repl_engine: None,
            run_engine: None,
            engine_running: false,
            quit: false,
            status: String::new(),
        };
        app.sys(i18n::t(&app.language, "banner"));
        app.reload_files();
        app
    }

    pub fn t<'a>(&self, key: &'a str) -> &'a str {
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
        self.files = directory_entries(&self.explorer_dir);
        if self.file_sel >= self.files.len() {
            self.file_sel = self.files.len().saturating_sub(1);
        }
    }

    fn open_selected(&mut self) {
        if let Some(p) = self.files.get(self.file_sel).cloned() {
            // Навігація каталогами не змінює корінь workspace для Save/Run.
            if p.is_dir() {
                self.explorer_dir = p;
                self.file_sel = 0;
                self.reload_files();
                self.status = format!("каталог {}", self.explorer_dir.display());
                return;
            }
            if self.dirty {
                self.sys("Незбережені зміни: Ctrl+S перед відкриттям іншого файлу");
                return;
            }
            if p.extension().and_then(|ext| ext.to_str()) == Some("sens") {
                self.sys("Фізичний .sens — двійковий файл, не UTF-8 Lisp. Відкрийте джерело .lisp.");
                return;
            }
            match std::fs::read_to_string(&p) {
                Ok(text) => {
                    self.editor_original = text.clone();
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
        let Some(path) = self.editor_path.clone() else { return; };
        if !self.dirty { return; }
        // Захист від паралельного редагування + спільний GUI/TUI атомарний запис.
        let result = save_loaded_file(&self.cwd, &path, &self.editor_original, &self.editor_buf);
        match result {
            Ok(()) => {
                self.editor_original = self.editor_buf.clone();
                self.dirty = false;
                self.status = format!("saved {}", path.display());
            }
            Err(error) => self.sys(&format!(
                "Не збережено {}: {error}. Буфер редактора не втрачено.",
                path.display()
            )),
        }
    }

    // ---- рушій -------------------------------------------------------------

    /// REPL створюється лише при першому введенні, без другого evaluator.
    fn start_repl(&mut self) {
        match engine::resolve_bin() {
            Ok(bin) => match Engine::spawn_repl(&bin.path, &self.cwd) {
                Ok(eng) => {
                    self.repl_engine = Some(eng);
                    self.sys(&format!("SENS REPL: {} gitlink={}", bin.path.display(), bin.pinned_sha));
                }
                Err(error) => self.sys(&format!("{} ({error})", self.t("no-engine"))),
            },
            Err(error) => self.sys(&format!("SENS REPL: {error}")),
        }
    }

    /// F5 = CLI file mode. Незбережений буфер не видаємо за файл на диску.
    fn run_current(&mut self) {
        let Some(path) = self.editor_path.clone() else {
            self.sys("немає відкритого файла для запуску");
            return;
        };
        if self.dirty {
            self.sys("Спершу збережіть зміни (Ctrl+S): SENS запускає файл із диска.");
            return;
        }
        if self.engine_running {
            self.sys("Файл уже виконується. F9 — зупинити поточний запуск.");
            return;
        }
        match std::fs::read_to_string(&path) {
            Ok(on_disk) if on_disk == self.editor_buf => {}
            Ok(_) => {
                self.sys("Файл змінено ззовні: повторно відкрийте перед F5.");
                return;
            }
            Err(error) => {
                self.sys(&format!("SENS Run: не вдалося прочитати файл: {error}"));
                return;
            }
        }
        let bin = match engine::resolve_bin() {
            Ok(bin) => bin,
            Err(error) => {
                self.sys(&format!("SENS Run: {error}"));
                return;
            }
        };
        self.sys(&format!("SENS gitlink={} exec={} file={}",
            bin.pinned_sha, bin.path.display(), path.display()));
        match Engine::spawn_file(&bin.path, &path, &self.cwd) {
            Ok(eng) => {
                self.run_engine = Some(eng);
                self.engine_running = true;
                self.status = format!("run {}", path.display());
            }
            Err(error) => {
                self.console.push(Line {
                    origin: Origin::Stderr,
                    text: format!("не вдалося запустити SENS: {error}"),
                });
            }
        }
    }

    fn stop(&mut self) {
        if let Some(mut eng) = self.run_engine.take() {
            eng.stop();
            while let Ok(line) = eng.rx.try_recv() { self.console.push(line); }
        }
        if let Some(mut eng) = self.repl_engine.take() {
            eng.stop();
            while let Ok(line) = eng.rx.try_recv() { self.console.push(line); }
        }
        self.engine_running = false;
        self.sys("SENS: процес зупинено (F5 — повторний запуск, REPL — наступна команда).");
    }

    /// Консоль читає stdout/stderr процесів і показує exit-status файлу.
    pub fn drain_engine(&mut self) {
        if let Some(eng) = self.repl_engine.as_ref() {
            while let Ok(line) = eng.rx.try_recv() {
                self.console.push(line);
            }
        }
        // REPL може завершитися сам (EOF, помилка рушія або команда виходу).
        // Не залишати закритий stdin у стані Some: наступна команда має
        // створити нову сесію з того самого pinned SENS.
        let repl_result = self.repl_engine.as_mut().map(|eng| eng.try_wait());
        match repl_result {
            Some(Ok(Some(status))) => {
                if let Some(mut eng) = self.repl_engine.take() {
                    eng.finish_readers();
                    while let Ok(line) = eng.rx.try_recv() { self.console.push(line); }
                }
                self.sys(&format!("SENS REPL exit-status={status}; наступна команда відкриє нову сесію."));
            }
            Some(Err(error)) => {
                if let Some(mut eng) = self.repl_engine.take() {
                    eng.stop();
                    while let Ok(line) = eng.rx.try_recv() { self.console.push(line); }
                }
                self.console.push(Line {
                    origin: Origin::Stderr,
                    text: format!("SENS REPL wait: {error}"),
                });
            }
            _ => {}
        }
        if let Some(eng) = self.run_engine.as_ref() {
            while let Ok(line) = eng.rx.try_recv() {
                self.console.push(line);
            }
        }
        if self.engine_running {
            let result = self.run_engine.as_mut().map(|eng| eng.try_wait());
            match result {
                Some(Ok(Some(status))) => {
                    if let Some(mut eng) = self.run_engine.take() {
                        eng.finish_readers();
                        while let Ok(line) = eng.rx.try_recv() { self.console.push(line); }
                    }
                    self.engine_running = false;
                    self.status = format!("SENS exit-status={status}");
                    self.sys(&self.status.clone());
                }
                Some(Err(error)) => {
                    if let Some(mut eng) = self.run_engine.take() {
                        eng.stop();
                        while let Ok(line) = eng.rx.try_recv() { self.console.push(line); }
                    }
                    self.engine_running = false;
                    self.console.push(Line {
                        origin: Origin::Stderr,
                        text: format!("SENS wait: {error}"),
                    });
                }
                _ => {}
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
        if self.repl_engine.is_none() {
            self.start_repl();
        }
        if let Some(eng) = self.repl_engine.as_mut() {
            if let Err(error) = eng.send_line(&line) {
                self.console.push(Line { origin: Origin::Stderr, text: format!("REPL stdin: {error}") });
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

    /// Bracketed paste зберігає багаторядковий український Lisp як одне редагування.
    /// У REPL заборонено перетворювати вставлені переноси рядків на окремі eval.
    pub fn paste(&mut self, text: &str) {
        if text.is_empty() { return; }
        match self.pane {
            Pane::Editor => {
                if self.editor_path.is_none() {
                    self.sys("Спочатку відкрийте файл у редакторі.");
                    return;
                }
                let normalized = text.replace("\r\n", "\n");
                self.editor_buf.insert_str(self.editor_cursor, &normalized);
                self.editor_cursor += normalized.len();
                self.dirty = true;
            }
            Pane::Console => {
                if text.contains('\r') || text.contains('\n') {
                    self.sys("Багаторядкова вставка в REPL не виконується: збережіть файл і натисніть F5.");
                } else {
                    self.input.push_str(text);
                }
            }
            Pane::Explorer => self.sys("Виберіть редактор або консоль перед вставленням."),
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
            KeyCode::PageUp if self.pane == Pane::Console => {
                self.console_scroll_back = self.console_scroll_back.saturating_add(5)
                    .min(self.console.len());
            }
            KeyCode::PageDown if self.pane == Pane::Console => {
                self.console_scroll_back = self.console_scroll_back.saturating_sub(5);
            }
            KeyCode::End if self.pane == Pane::Console => self.console_scroll_back = 0,
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
            KeyCode::Char(c) if !key.modifiers.intersects(
                KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER
            ) => {
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
            Pane::Explorer => {
                if self.explorer_dir != self.cwd {
                    if let Some(parent) = self.explorer_dir.parent() {
                        if parent.starts_with(&self.cwd) {
                            self.explorer_dir = parent.to_path_buf();
                            self.file_sel = 0;
                            self.reload_files();
                            self.status = format!("каталог {}", self.explorer_dir.display());
                        }
                    }
                }
            }
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

    
/// Explorer показує звичайні каталоги і файли; символьні посилання не
/// використовуємо як прихований шлях за межі вибраного workspace.
fn directory_entries(dir: &Path) -> Vec<PathBuf> {
    let mut entries = Vec::new();
    if let Ok(read_dir) = std::fs::read_dir(dir) {
        for entry in read_dir.flatten() {
            if let Ok(kind) = entry.file_type() {
                if kind.is_dir() || kind.is_file() {
                    entries.push(entry.path());
                }
            }
        }
    }
    entries.sort_by(|a, b| b.is_dir().cmp(&a.is_dir()).then_with(|| a.cmp(b)));
    entries
}

#[cfg(test)]
mod directory_tests {
    use super::*;

    #[test]
    fn explorer_lists_directories_before_files() {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NONCE: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "my-idea-tui-explorer-{}-{}", std::process::id(),
            NONCE.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&root).unwrap();
        std::fs::write(root.join("а.lisp"), "дані").unwrap();
        std::fs::create_dir(root.join("б")).unwrap();
        let entries = directory_entries(&root);
        assert_eq!(entries.len(), 2);
        assert!(entries[0].is_dir());
        assert!(entries[1].is_file());
        std::fs::remove_dir_all(root).unwrap();
    }
}

 
/// Захищає джерело іншого процесу від перезапису, а потім атомарно зберігає.
/// Чисто файловий контракт; жодних імітаторів SENS-семантики.
fn save_loaded_file(root: &Path, path: &Path, original: &str, updated: &str) -> Result<(), String> {
    let disk = std::fs::read_to_string(path)
        .map_err(|error| format!("неможливо перевірити поточний файл: {error}"))?;
    if disk != original {
        return Err("файл змінено ззовні після Open/Save; нові дані не перезаписано".into());
    }
    let relative = path.strip_prefix(root)
        .map_err(|error| format!("файл поза workspace: {error}"))?;
    workspace_save::save_existing_workspace_file(root, relative, updated)
}

#[cfg(test)]
mod save_conflict_tests {
    use super::save_loaded_file;
    use std::fs;

    #[test]
    fn preserves_external_source_and_saves_when_unchanged() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let workspace = std::env::temp_dir().join(format!(
            "my-idea-tui-conflict-{}-{unique}", std::process::id()
        ));
        fs::create_dir(&workspace).unwrap();
        let source = workspace.join("програма.lisp");
        fs::write(&source, "(атом а)\n").unwrap();

        // Перший Ctrl+S зберігає реальне Unicode-джерело.
        save_loaded_file(&workspace, &source, "(атом а)\n", "(атом б)\n").unwrap();
        assert_eq!(fs::read_to_string(&source).unwrap(), "(атом б)\n");

        // Інший редактор змінив файл: TUI не знищує його правки.
        fs::write(&source, "(атом в)\n").unwrap();
        assert!(save_loaded_file(&workspace, &source, "(атом б)\n", "(атом г)\n").is_err());
        assert_eq!(fs::read_to_string(&source).unwrap(), "(атом в)\n");
        fs::remove_dir_all(workspace).unwrap();
    }
}

#[cfg(test)]
mod paste_tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    #[test]
    fn utf8_multiline_paste_is_single_edit_and_repl_cannot_execute_it() {
        static NONCE: AtomicU64 = AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "my-idea-paste-{}-{}", std::process::id(), NONCE.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&dir).unwrap();
        let source = dir.join("програма.lisp");
        std::fs::write(&source, "(цитата а)").unwrap();
        let mut app = App::new_in(dir.clone());
        app.open_selected();
        assert_eq!(app.editor_path.as_ref(), Some(&source));
        app.editor_cursor = "(цитата ".len();
        app.paste("б\r\nв");
        assert_eq!(app.editor_buf, "(цитата б\nва)");
        assert!(app.editor_buf.is_char_boundary(app.editor_cursor));
        assert!(app.dirty);
        app.pane = Pane::Console;
        let before = app.console.len();
        app.paste("(цитата а)\n(цитата б)");
        assert!(app.input.is_empty());
        assert!(app.repl_engine.is_none());
        assert_eq!(app.console.len(), before + 1);
        app.paste("(цитата г)");
        assert_eq!(app.input, "(цитата г)");
        std::fs::remove_dir_all(dir).unwrap();
    }
}
