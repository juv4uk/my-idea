//! Стан TUI і обробка подій.
//!
//! Вертикаль #85: Open workspace → Explorer → Edit → Save → Run → Output → Stop.
//! Панелі: дерево проєкту | редактор | консоль/вивід.

use std::path::{Component, Path, PathBuf};
use std::fs::OpenOptions;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::engine::{self, Engine, Line, Origin};
use crate::i18n;
use crate::ast;

#[path = "../../src-tauri/src/workspace_save.rs"]
mod workspace_save;

#[derive(Clone, Copy, PartialEq)]
pub enum Pane {
    Explorer,
    Editor,
    Console,
}

/// Знімок редагованого тексту. Зберігаємо лише стани користувацьких змін;
/// SENS-семантику не інтерпретуємо й не переписуємо.
#[derive(Clone)]
struct EditSnapshot {
    text: String,
    cursor: usize,
}

/// Один файл у вкладках. Незбережений текст та Undo/Redo належать документу,
/// а не поточному Explorer, тому перемикання не знищує зміни.
#[derive(Clone)]
struct OpenDocument {
    path: PathBuf,
    text: String,
    original: String,
    cursor: usize,
    undo: Vec<EditSnapshot>,
    redo: Vec<EditSnapshot>,
}

const MAX_EDIT_HISTORY: usize = 64;

pub struct App {
    pub language: String,
    pub pane: Pane,
    pub cwd: PathBuf, // Незмінний корінь workspace, зокрема для безпечного Save
    pub explorer_dir: PathBuf, // Поточний каталог Explorer усередині workspace
    pub files: Vec<PathBuf>,
    pub file_sel: usize,
    pub editor_path: Option<PathBuf>,
    tabs: Vec<OpenDocument>,
    active_tab: usize,
    pub editor_buf: String,
    // Текст відкритого файла на диску при останньому Open/Save.
    pub editor_original: String,
    pub editor_cursor: usize, // UTF-8 byte offset, завжди на межі символу
    undo: Vec<EditSnapshot>,
    redo: Vec<EditSnapshot>,
    pub dirty: bool,
    pub quit_armed: bool,
    pub console: Vec<Line>,
    /// Скільки логічних рядків відступити від хвоста консолі (0 = останній вивід).
    pub console_scroll_back: usize,
    pub ast_visible: bool,
    pub ast_scroll: usize,
    pub ast_lines: Vec<String>,
    ast_cached_source: String,
    pub input: String,
    /// UTF-8 byte offset усередині поточної команди REPL.
    pub input_cursor: usize,
    pub history: Vec<String>,
    pub hist_idx: usize,
    /// Ненадіслана чернетка та курсор UTF-8 до переходу в історію.
    history_draft: Option<(String, usize)>,
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
            tabs: Vec::new(),
            active_tab: 0,
            editor_buf: String::new(),
            editor_original: String::new(),
            editor_cursor: 0,
            undo: Vec::new(),
            redo: Vec::new(),
            dirty: false,
            quit_armed: false,
            console: Vec::new(),
            console_scroll_back: 0,
            ast_visible: false,
            ast_scroll: 0,
            ast_lines: Vec::new(),
            ast_cached_source: String::new(),
            input: String::new(),
            input_cursor: 0,
            history: Vec::new(),
            hist_idx: 0,
            history_draft: None,
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

    /// Логічна позиція курсора в символах, а не в байтах UTF-8.
    pub fn editor_position(&self) -> (usize, usize) {
        let before = &self.editor_buf[..self.editor_cursor];
        let line = before.bytes().filter(|&byte| byte == b'\n').count() + 1;
        let start = before.rfind('\n').map_or(0, |index| index + 1);
        (line, before[start..].chars().count() + 1)
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

    /// Поточний видимий документ у вкладці синхронізується перед перемиканням.
    fn store_active_tab(&mut self) {
        let Some(path) = self.editor_path.as_ref() else { return; };
        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            if &tab.path == path {
                tab.text = self.editor_buf.clone();
                tab.original = self.editor_original.clone();
                tab.cursor = self.editor_cursor;
                tab.undo = self.undo.clone();
                tab.redo = self.redo.clone();
            }
        }
    }

    fn show_tab(&mut self, index: usize) {
        let Some(tab) = self.tabs.get(index).cloned() else { return; };
        self.active_tab = index;
        self.editor_path = Some(tab.path.clone());
        self.editor_buf = tab.text;
        self.editor_original = tab.original;
        self.editor_cursor = tab.cursor;
        self.undo = tab.undo;
        self.redo = tab.redo;
        self.dirty = self.editor_buf != self.editor_original;
        self.pane = Pane::Editor;
        self.status = format!("open {}", tab.path.display());
    }

    fn switch_tab(&mut self, index: usize) {
        if index >= self.tabs.len() { return; }
        self.store_active_tab();
        self.show_tab(index);
    }

    fn cycle_tab(&mut self, backward: bool) {
        if self.tabs.is_empty() { return; }
        let target = if backward {
            (self.active_tab + self.tabs.len() - 1) % self.tabs.len()
        } else {
            (self.active_tab + 1) % self.tabs.len()
        };
        self.switch_tab(target);
    }

    fn close_current_tab(&mut self) {
        if self.editor_path.is_none() { return; }
        if self.dirty || self.editor_buf != self.editor_original {
            self.sys("Незбережені зміни: Ctrl+S або Ctrl+Z перед Ctrl+W.");
            return;
        }
        self.tabs.remove(self.active_tab);
        if self.tabs.is_empty() {
            self.editor_path = None;
            self.editor_buf.clear();
            self.editor_original.clear();
            self.editor_cursor = 0;
            self.undo.clear();
            self.redo.clear();
            self.dirty = false;
            self.active_tab = 0;
            self.pane = Pane::Explorer;
        } else {
            // Активна вкладка вже вилучена; не записувати її стан поверх іншої.
            let index = self.active_tab.min(self.tabs.len() - 1);
            self.show_tab(index);
        }
    }

    fn has_unsaved_changes(&self) -> bool {
        (self.editor_path.is_some() && (self.dirty || self.editor_buf != self.editor_original))
            || self.tabs.iter().enumerate().any(|(index, tab)| {
                index != self.active_tab && tab.text != tab.original
            })
    }

    /// Підписи вкладок GUI-подібної робочої поверхні: ● = активна, * = брудна.
    pub fn tabs_title(&self) -> String {
        self.tabs.iter().enumerate().map(|(index, tab)| {
            let name = display_name(&tab.path);
            let dirty = if index == self.active_tab {
                self.editor_buf != self.editor_original
            } else {
                tab.text != tab.original
            };
            format!("{}{}{}", if index == self.active_tab { "›" } else { "" },
                name, if dirty { "*" } else { "" })
        }).collect::<Vec<_>>().join(" | ")
    }

    fn open_selected(&mut self) {
        if let Some(p) = self.files.get(self.file_sel).cloned() {
            if p.is_dir() {
                self.explorer_dir = p;
                self.file_sel = 0;
                self.reload_files();
                self.status = format!("каталог {}", self.explorer_dir.display());
                return;
            }
            if p.extension().and_then(|ext| ext.to_str()) == Some("sens") {
                self.sys("Фізичний .sens — двійковий файл, не UTF-8 Lisp. Відкрийте джерело .lisp.");
                return;
            }
            if let Some(index) = self.tabs.iter().position(|tab| tab.path == p) {
                self.switch_tab(index);
                return;
            }
            match std::fs::read_to_string(&p) {
                Ok(text) => {
                    self.store_active_tab();
                    self.editor_original = text.clone();
                    self.editor_buf = text.clone();
                    self.editor_cursor = text.len();
                    self.editor_path = Some(p.clone());
                    self.undo.clear();
                    self.redo.clear();
                    self.ast_cached_source.clear();
                    self.ast_lines.clear();
                    self.ast_scroll = 0;
                    self.dirty = false;
                    self.active_tab = self.tabs.len();
                    self.tabs.push(OpenDocument {
                        path: p.clone(), text: text.clone(), original: text,
                        cursor: self.editor_cursor, undo: Vec::new(), redo: Vec::new(),
                    });
                    self.pane = Pane::Editor;
                    self.status = format!("open {}", p.display());
                }
                Err(error) => self.sys(&format!("не вдалось відкрити {}: {error}", p.display())),
            }
        }
    }

    /// Створити новий текстовий Lisp у поточній папці Explorer. Ніколи не
    /// перезаписувати існуючий файл й не змінювати буфер із незбереженими змінами.
    fn create_new_document(&mut self, name: &str) {
        if self.dirty {
            self.sys("Незбережені зміни: спочатку Ctrl+S, а потім :новий.");
            return;
        }
        let candidate = Path::new(name);
        if name.is_empty()
            || name.contains('\\')
            || name.chars().any(char::is_control)
            || !matches!(candidate.components().next(), Some(Component::Normal(_)))
            || candidate.components().count() != 1
            || !matches!(candidate.extension().and_then(|ext| ext.to_str()), Some("lisp" | "лісп"))
        {
            self.sys("Вкажіть одне ім'я .lisp або .лісп без шляху, наприклад :новий програма.lisp");
            return;
        }
        // Навігація Explorer не повинна створювати джерела за коренем workspace.
        let root = match self.cwd.canonicalize() {
            Ok(root) => root,
            Err(error) => { self.sys(&format!("workspace недоступний: {error}")); return; }
        };
        let folder = match self.explorer_dir.canonicalize() {
            Ok(folder) => folder,
            Err(error) => { self.sys(&format!("каталог Explorer недоступний: {error}")); return; }
        };
        if !folder.starts_with(&root) {
            self.sys("Каталог Explorer вийшов за межі workspace.");
            return;
        }
        let path = folder.join(name);
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => {
                drop(file);
                self.editor_path = Some(path.clone());
                self.editor_original.clear();
                self.editor_buf.clear();
                self.editor_cursor = 0;
                self.undo.clear();
                self.redo.clear();
                self.dirty = false;
                self.quit_armed = false;
                self.reload_files();
                if let Some(position) = self.files.iter().position(|p| p == &path) {
                    self.file_sel = position;
                }
                self.pane = Pane::Editor;
                self.status = format!("новий {}", path.display());
                self.sys(&format!("Створено {}", path.display()));
            }
            Err(error) => self.sys(&format!("Не вдалося створити {}: {error}", path.display())),
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
                    self.sys(&format!("SENS REPL surface=укр gitlink={} cli={} (SHA збірки окремо не засвідчено)", bin.pinned_sha, bin.path.display()));
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
        self.history_draft = None;

        if let Some(rest) = line.strip_prefix(':') {
            self.meta(rest);
            return;
        }
        self.forward_to_repl(&line);
    }

    /// Лише транспорт для наявного SENS: не класифікуємо й не трактуємо
    /// невідомі Lisp/REPL команди на стороні Ratatui.
    fn forward_to_repl(&mut self, line: &str) {
        self.console_scroll_back = 0;
        self.console.push(Line { origin: Origin::System, text: format!("> {line}") });
        if self.repl_engine.is_none() {
            self.start_repl();
        }
        if let Some(eng) = self.repl_engine.as_mut() {
            if let Err(error) = eng.send_line(line) {
                self.console.push(Line { origin: Origin::Stderr, text: format!("REPL stdin: {error}") });
            }
        }
    }

    fn meta(&mut self, rest: &str) {
        let mut it = rest.split_whitespace();
        match it.next() {
            // Локалізація TUI не змінює мову програми.
            Some("інтерфейс") | Some("ui") | Some("language") => match it.next() {
                Some(l) if i18n::LANGUAGES.contains(&l) && it.next().is_none() => {
                    self.language = l.to_string();
                    self.sys(&format!("мова інтерфейсу: {l}"));
                }
                Some(l) => self.sys(&format!("невідомий код інтерфейсу або зайві аргументи: {l}; доступно uk|de|en")),
                None => self.sys("Мова інтерфейсу: uk|de|en; :мова — проєкція SENS"),
            },
            Some("новий") | Some("new") => {
                let name = it.next().unwrap_or("");
                if it.next().is_some() {
                    self.sys("Потрібне одне ім'я без пробілів: :новий програма.lisp");
                } else {
                    self.create_new_document(name);
                }
            },
            Some("run") => self.run_current(),
            Some("stop") => self.stop(),
            Some("save") => self.save(),
            // Усі інші :команди належать SENS (наприклад :допомога,
            // :імена, :ім\'я, :мова); TUI не приховує його можливостей.
            _ => self.forward_to_repl(&format!(":{rest}")),
        }
    }

    /// Фіксація стану перед однією дією редагування; Unicode-курсор є byte offset.
    fn remember_edit(&mut self) {
        if self.editor_path.is_none() {
            return;
        }
        if self.undo.len() >= MAX_EDIT_HISTORY {
            self.undo.remove(0);
        }
        self.undo.push(EditSnapshot {
            text: self.editor_buf.clone(),
            cursor: self.editor_cursor,
        });
        self.redo.clear();
    }

    fn undo_edit(&mut self) {
        if self.pane != Pane::Editor { return; }
        if let Some(previous) = self.undo.pop() {
            self.redo.push(EditSnapshot {
                text: std::mem::replace(&mut self.editor_buf, previous.text),
                cursor: self.editor_cursor,
            });
            self.editor_cursor = previous.cursor;
            self.dirty = self.editor_buf != self.editor_original;
        }
    }

    fn redo_edit(&mut self) {
        if self.pane != Pane::Editor { return; }
        if let Some(next) = self.redo.pop() {
            self.undo.push(EditSnapshot {
                text: std::mem::replace(&mut self.editor_buf, next.text),
                cursor: self.editor_cursor,
            });
            self.editor_cursor = next.cursor;
            self.dirty = self.editor_buf != self.editor_original;
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
                self.remember_edit();
                self.editor_buf.insert_str(self.editor_cursor, &normalized);
                self.editor_cursor += normalized.len();
                self.dirty = true;
            }
            Pane::Console => {
                if text.contains('\r') || text.contains('\n') {
                    self.sys("Багаторядкова вставка в REPL не виконується: збережіть файл і натисніть F5.");
                } else {
                    self.input.insert_str(self.input_cursor, text);
                    self.input_cursor += text.len();
                }
            }
            Pane::Explorer => self.sys("Виберіть редактор або консоль перед вставленням."),
        }
    }

    /// AST отримуємо тільки з reader-а pinned SENS і лише коли змінився текст.
    /// Жодної інтерпретації чи дублювання правил Lisp у TUI.
    pub fn refresh_ast(&mut self) {
        if self.editor_path.is_none() {
            self.ast_lines.clear();
            self.ast_cached_source.clear();
            self.ast_scroll = 0;
        } else if self.ast_lines.is_empty() || self.ast_cached_source != self.editor_buf {
            self.ast_lines = ast::render(&self.editor_buf);
            self.ast_cached_source.clone_from(&self.editor_buf);
            self.ast_scroll = 0;
        }
    }

    /// true — вийти з TUI.
    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        if !(ctrl && matches!(key.code, KeyCode::Char('q' | 'c'))) {
            self.quit_armed = false;
        }
        match key.code {
            KeyCode::Char('n') if ctrl => {
                if self.input.is_empty() {
                    self.input = ":новий ".to_string();
                    self.pane = Pane::Console;
                    self.console_scroll_back = 0;
                } else {
                    self.sys("Консоль має незавершений ввід: завершіть або очистіть його перед Ctrl+N.");
                    self.pane = Pane::Console;
                }
            },
            KeyCode::Char('s') if ctrl => self.save(),
            KeyCode::Char('w') if ctrl => self.close_current_tab(),
            KeyCode::Char('z') if ctrl => self.undo_edit(),
            KeyCode::Char('y') if ctrl => self.redo_edit(),
            KeyCode::Char('c' | 'q') if ctrl => {
                if self.has_unsaved_changes() && !self.quit_armed {
                    self.quit_armed = true;
                    self.sys("Незбережені зміни: Ctrl+S — зберегти; Ctrl+Q повторно — вийти без збереження");
                } else {
                    return true;
                }
            },
            KeyCode::Esc => self.pane = Pane::Explorer,
            KeyCode::F(5) => self.run_current(),
            KeyCode::F(6) => { self.ast_visible = !self.ast_visible; self.refresh_ast(); },
            KeyCode::F(9) => self.stop(),
            KeyCode::PageUp if self.ast_visible && key.modifiers.contains(KeyModifiers::ALT) => {
                self.ast_scroll = self.ast_scroll.saturating_sub(6);
            },
            KeyCode::PageDown if self.ast_visible && key.modifiers.contains(KeyModifiers::ALT) => {
                self.ast_scroll = self.ast_scroll.saturating_add(6).min(self.ast_lines.len().saturating_sub(1));
            },
            KeyCode::Tab if ctrl => self.cycle_tab(key.modifiers.contains(KeyModifiers::SHIFT)),
            KeyCode::BackTab if ctrl => self.cycle_tab(true),
            KeyCode::Tab => self.cycle_pane(),
            KeyCode::BackTab => self.cycle_pane_back(),
            KeyCode::Enter => {
                if self.pane == Pane::Console {
                    let line = std::mem::take(&mut self.input);
                    self.input_cursor = 0;
                    self.submit(&line);
                } else if self.pane == Pane::Explorer {
                    self.open_selected();
                } else if self.editor_path.is_some() {
                    self.remember_edit();
                    self.editor_buf.insert(self.editor_cursor, '\n');
                    self.editor_cursor += 1;
                    self.dirty = true;
                }
            }
            KeyCode::PageUp if self.pane == Pane::Console => {
                // Прокручуємо фізичні рядки, а не число повідомлень:
                // одне довге stderr може займати десятки рядків.
                self.console_scroll_back = self.console_scroll_back.saturating_add(5);
            }
            KeyCode::PageDown if self.pane == Pane::Console => {
                self.console_scroll_back = self.console_scroll_back.saturating_sub(5);
            }
            KeyCode::Home if self.pane == Pane::Console => self.input_cursor = 0,
            KeyCode::End if self.pane == Pane::Console => {
                if ctrl { self.console_scroll_back = 0; }
                else { self.input_cursor = self.input.len(); }
            },
            KeyCode::PageUp if self.pane == Pane::Editor => {
                for _ in 0..10 { self.move_editor_line(false); }
            }
            KeyCode::PageDown if self.pane == Pane::Editor => {
                for _ in 0..10 { self.move_editor_line(true); }
            }
            KeyCode::Delete if self.pane == Pane::Console => self.delete_input_forward(),
            KeyCode::Delete if self.pane == Pane::Editor => self.delete_forward(),
            KeyCode::Backspace => self.backspace(),
            KeyCode::Up => self.up(),
            KeyCode::Down => self.down(),
            KeyCode::Left => {
                if self.pane == Pane::Editor { self.move_editor_cursor(false); }
                else if self.pane == Pane::Console { self.move_input_cursor(false); }
            }
            KeyCode::Right => {
                if self.pane == Pane::Editor { self.move_editor_cursor(true); }
                else if self.pane == Pane::Console { self.move_input_cursor(true); }
            }
            // Ctrl+Home / Ctrl+End переміщують курсор між краями всього
            // документа; звичайні Home / End залишаються межами рядка.
            KeyCode::Home if ctrl && self.pane == Pane::Editor => self.editor_cursor = 0,
            KeyCode::End if ctrl && self.pane == Pane::Editor => self.editor_cursor = self.editor_buf.len(),
            KeyCode::Home if self.pane == Pane::Editor => {
                self.editor_cursor = if ctrl { 0 } else {
                    self.editor_buf[..self.editor_cursor].rfind('\n').map_or(0, |p| p + 1)
                };
            }
            KeyCode::End if self.pane == Pane::Editor => {
                self.editor_cursor = if ctrl { self.editor_buf.len() } else {
                    self.editor_buf[self.editor_cursor..].find('\n')
                        .map_or(self.editor_buf.len(), |p| self.editor_cursor + p)
                };
            }
            KeyCode::Char(c) if !key.modifiers.intersects(
                KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER
            ) => {
                if self.pane == Pane::Console {
                    self.input.insert(self.input_cursor, c);
                    self.input_cursor += c.len_utf8();
                } else if self.pane == Pane::Editor && self.editor_path.is_some() {
                    self.remember_edit();
                    self.editor_buf.insert(self.editor_cursor, c);
                    self.editor_cursor += c.len_utf8();
                    self.dirty = true;
                }
            }
            _ => {}
        }
        false
    }

    /// Видалення символу праворуч без розриву послідовності UTF-8.
    fn delete_forward(&mut self) {
        if self.editor_path.is_none() { return; }
        if let Some(next) = self.editor_buf[self.editor_cursor..].chars().next() {
            // Поведінка Delete має бути симетрична Backspace: Ctrl+Z повертає
            // вилучений український символ, а край файла лишається чистим.
            self.remember_edit();
            self.editor_buf.drain(self.editor_cursor..self.editor_cursor + next.len_utf8());
            self.dirty = self.editor_buf != self.editor_original;
        }
    }

    fn backspace(&mut self) {
        match self.pane {
            Pane::Console => {
                if let Some((previous, _)) = self.input[..self.input_cursor].char_indices().last() {
                    self.input.drain(previous..self.input_cursor);
                    self.input_cursor = previous;
                }
            }
            Pane::Editor if self.editor_cursor > 0 => {
                if let Some((previous, _)) = self.editor_buf[..self.editor_cursor].char_indices().last() {
                    self.remember_edit();
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
                    if self.hist_idx == self.history.len() {
                        self.history_draft = Some((self.input.clone(), self.input_cursor));
                    }
                    self.hist_idx -= 1;
                    self.input = self.history[self.hist_idx].clone();
                    self.input_cursor = self.input.len();
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
                    self.input_cursor = self.input.len();
                } else if self.hist_idx < self.history.len() {
                    self.hist_idx = self.history.len();
                    let (draft, cursor) = self.history_draft.take()
                        .unwrap_or_else(|| (String::new(), 0));
                    self.input = draft;
                    self.input_cursor = cursor;
                }
            }
            Pane::Editor => self.move_editor_line(true),
        }
    }

    /// Редагування REPL-команди: лише її ввід, жодної Lisp-семантики.
    fn move_input_cursor(&mut self, forward: bool) {
        if forward {
            if let Some(ch) = self.input[self.input_cursor..].chars().next() {
                self.input_cursor += ch.len_utf8();
            }
        } else if let Some((previous, _)) = self.input[..self.input_cursor].char_indices().last() {
            self.input_cursor = previous;
        }
    }

    fn delete_input_forward(&mut self) {
        if let Some(ch) = self.input[self.input_cursor..].chars().next() {
            self.input.drain(self.input_cursor..self.input_cursor + ch.len_utf8());
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

#[cfg(test)]
mod edit_history_tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    #[test]
    fn unicode_paste_undo_redo_and_saved_state_are_real_edits() {
        static NONCE: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "my-idea-edit-history-{}-{}", std::process::id(),
            NONCE.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&root).unwrap();
        let first = root.join("а.lisp");
        let second = root.join("б.lisp");
        std::fs::write(&first, "(цитата мама)\n").unwrap();
        std::fs::write(&second, "(цитата тато)\n").unwrap();
        let mut app = App::new_in(root.clone());
        assert_eq!(app.files[0], first);
        // Explorer actually opens a file, not a synthesized testing-only buffer.
        app.open_selected();
        let original = app.editor_buf.clone();
        app.paste("\n(цитата дім)");
        assert!(app.dirty);
        assert_eq!(app.undo.len(), 1);
        assert!(app.editor_buf.ends_with("(цитата дім)"));
        app.handle_key(KeyEvent::new(KeyCode::Char('z'), KeyModifiers::CONTROL));
        assert_eq!(app.editor_buf, original);
        assert!(!app.dirty);
        assert!(app.editor_buf.is_char_boundary(app.editor_cursor));
        app.handle_key(KeyEvent::new(KeyCode::Char('y'), KeyModifiers::CONTROL));
        assert!(app.dirty);
        assert!(app.editor_buf.ends_with("(цитата дім)"));
        app.save();
        assert!(!app.dirty);
        assert_eq!(std::fs::read_to_string(&first).unwrap(), app.editor_buf);
        app.handle_key(KeyEvent::new(KeyCode::Char('z'), KeyModifiers::CONTROL));
        assert!(app.dirty, "undo of saved change must mark buffer unsaved");
        app.handle_key(KeyEvent::new(KeyCode::Char('y'), KeyModifiers::CONTROL));
        assert!(!app.dirty);

        // History belongs to the currently opened file only.
        app.pane = Pane::Explorer;
        app.file_sel = 1;
        app.open_selected();
        assert_eq!(app.editor_path.as_ref(), Some(&second));
        assert!(app.undo.is_empty() && app.redo.is_empty());
        app.handle_key(KeyEvent::new(KeyCode::Char('z'), KeyModifiers::CONTROL));
        assert_eq!(app.editor_buf, "(цитата тато)\n");
        assert!(!app.dirty);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn delete_does_not_split_utf8_codepoint() {
        let root = std::env::temp_dir().join(format!(
            "my-idea-delete-{}-{:?}", std::process::id(), std::thread::current().id()
        ));
        std::fs::create_dir(&root).unwrap();
        let source = root.join("а.lisp");
        std::fs::write(&source, "дім").unwrap();
        let mut app = App::new_in(root.clone());
        app.open_selected();
        app.editor_cursor = 0;
        app.handle_key(KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE));
        assert_eq!(app.editor_buf, "ім");
        app.handle_key(KeyEvent::new(KeyCode::Char('z'), KeyModifiers::CONTROL));
        assert_eq!(app.editor_buf, "дім");
        std::fs::remove_dir_all(root).unwrap();
    }
}


#[cfg(test)]
mod console_paging_tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn page_up_is_not_limited_by_number_of_logical_messages() {
        let mut app = App::new_in(std::env::temp_dir());
        app.pane = Pane::Console;
        app.console.clear();
        app.console.push(Line { origin: Origin::Stderr, text: "помилка ".repeat(100) });
        app.handle_key(KeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE));
        assert_eq!(app.console_scroll_back, 5);
        app.handle_key(KeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE));
        assert_eq!(app.console_scroll_back, 10);
        app.handle_key(KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE));
        assert_eq!(app.console_scroll_back, 5);
        app.handle_key(KeyEvent::new(KeyCode::End, KeyModifiers::NONE));
        assert_eq!(app.console_scroll_back, 0);
    }
}

#[cfg(test)]
mod document_navigation_tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::sync::atomic::{AtomicU64, Ordering};

    #[test]
    fn ctrl_home_end_navigate_document_without_modifying_unicode_source() {
        static NONCE: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "my-idea-tui-nav-{}-{}", std::process::id(),
            NONCE.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&root).unwrap();
        let path = root.join("програма.lisp");
        let source = "(цитата дім)\n(цитата мова)\n";
        std::fs::write(&path, source).unwrap();
        let mut app = App::new_in(root.clone());
        app.open_selected();
        assert_eq!(app.editor_path.as_ref(), Some(&path));
        assert_eq!(app.editor_cursor, source.len());

        app.handle_key(KeyEvent::new(KeyCode::Home, KeyModifiers::CONTROL));
        assert_eq!(app.editor_cursor, 0);
        app.handle_key(KeyEvent::new(KeyCode::End, KeyModifiers::NONE));
        assert_eq!(app.editor_cursor, source.find('\n').unwrap());
        app.handle_key(KeyEvent::new(KeyCode::End, KeyModifiers::CONTROL));
        assert_eq!(app.editor_cursor, source.len());
        app.handle_key(KeyEvent::new(KeyCode::Home, KeyModifiers::NONE));
        assert_eq!(app.editor_cursor, source.len()); // останній порожній рядок
        assert_eq!(app.editor_buf, source);
        assert!(!app.dirty);
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[cfg(test)]
mod editor_navigation_tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn unicode_delete_and_document_navigation_preserve_utf8() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let workspace = std::env::temp_dir()
            .join(format!("my-idea-navigation-{}-{nonce}", std::process::id()));
        std::fs::create_dir(&workspace).unwrap();
        let source = workspace.join("програма.lisp");
        std::fs::write(&source, "абв\nїжак\nкінець").unwrap();
        let mut editor = App::new_in(workspace.clone());
        editor.open_selected(); // реальний файл, а не вигаданий буфер
        editor.editor_cursor = "абв\nї".len();
        assert_eq!(editor.editor_position(), (2, 2));
        editor.handle_key(KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE));
        assert_eq!(editor.editor_buf, "абв\nїак\nкінець");
        assert_eq!(editor.editor_position(), (2, 2));
        assert!(editor.dirty);
        assert!(editor.editor_buf.is_char_boundary(editor.editor_cursor));

        editor.handle_key(KeyEvent::new(KeyCode::Home, KeyModifiers::CONTROL));
        assert_eq!(editor.editor_cursor, 0);
        editor.handle_key(KeyEvent::new(KeyCode::End, KeyModifiers::CONTROL));
        assert_eq!(editor.editor_cursor, editor.editor_buf.len());
        assert_eq!(editor.editor_position(), (3, 7));
        editor.handle_key(KeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE));
        assert_eq!(editor.editor_position().0, 1);
        editor.handle_key(KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE));
        assert_eq!(editor.editor_position().0, 3);
        assert!(editor.editor_buf.is_char_boundary(editor.editor_cursor));
        std::fs::remove_dir_all(workspace).unwrap();
    }

    #[test]
    fn delete_at_eof_does_not_mark_clean_buffer_dirty() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let workspace = std::env::temp_dir()
            .join(format!("my-idea-eof-{}-{nonce}", std::process::id()));
        std::fs::create_dir(&workspace).unwrap();
        std::fs::write(workspace.join("програма.lisp"), "лісп").unwrap();
        let mut editor = App::new_in(workspace.clone());
        editor.open_selected();
        editor.handle_key(KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE));
        assert_eq!(editor.editor_buf, "лісп");
        assert!(!editor.dirty);
        std::fs::remove_dir_all(workspace).unwrap();
    }
}

#[cfg(test)]
mod repl_input_cursor_tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn press(app: &mut App, code: KeyCode) {
        assert!(!app.handle_key(KeyEvent::new(code, KeyModifiers::NONE)));
        assert!(app.input.is_char_boundary(app.input_cursor));
    }

    #[test]
    fn ukrainian_repl_input_can_be_edited_in_the_middle() {
        let mut app = App::new_in(std::path::PathBuf::from("."));
        app.pane = Pane::Console;
        app.paste("дм");
        press(&mut app, KeyCode::Left);
        press(&mut app, KeyCode::Char('і'));
        assert_eq!(app.input, "дім");
        assert_eq!(app.input_cursor, "ді".len());

        press(&mut app, KeyCode::Home);
        press(&mut app, KeyCode::Right);
        press(&mut app, KeyCode::Delete);
        assert_eq!(app.input, "дм");
        assert_eq!(app.input_cursor, "д".len());
        press(&mut app, KeyCode::Backspace);
        assert_eq!(app.input, "м");
        assert_eq!(app.input_cursor, 0);
        press(&mut app, KeyCode::End);
        assert_eq!(app.input_cursor, app.input.len());
        assert!(app.repl_engine.is_none(), "редагування не виконує SENS до Enter");
    }

    #[test]
    fn history_and_paste_keep_unicode_cursor_valid() {
        let mut app = App::new_in(std::path::PathBuf::from("."));
        app.pane = Pane::Console;
        app.history = vec!["(атом мама)".into(), "(атом дім)".into()];
        app.hist_idx = app.history.len();
        press(&mut app, KeyCode::Up);
        assert_eq!(app.input, "(атом дім)");
        assert_eq!(app.input_cursor, app.input.len());
        press(&mut app, KeyCode::Home);
        app.paste("укр ");
        assert_eq!(app.input, "укр (атом дім)");
        assert_eq!(app.input_cursor, "укр ".len());
        press(&mut app, KeyCode::Down);
        assert!(app.input.is_empty());
        assert_eq!(app.input_cursor, 0);
        assert!(app.repl_engine.is_none());
    }
}

#[cfg(test)]
mod document_tabs_tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn tabs_preserve_ukrainian_edits_and_independent_undo_history() {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let root = std::env::temp_dir()
            .join(format!("my-idea-real-tabs-{}-{stamp}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        let first = root.join("а.lisp");
        let second = root.join("б.lisp");
        std::fs::write(&first, "(атом а)\n").unwrap();
        std::fs::write(&second, "(атом б)\n").unwrap();

        let mut app = App::new_in(root.clone());
        app.file_sel = app.files.iter().position(|path| path == &first).unwrap();
        app.open_selected();
        app.paste("(цитата ї)\n");
        let first_edited = app.editor_buf.clone();
        assert!(app.dirty);

        app.pane = Pane::Explorer;
        app.file_sel = app.files.iter().position(|path| path == &second).unwrap();
        app.open_selected(); // не відкидає незбережений текст першого файла
        assert_eq!(app.editor_path.as_ref(), Some(&second));
        assert_eq!(app.editor_buf, "(атом б)\n");
        assert!(!app.dirty);
        app.paste("(цитата є)\n");
        let second_edited = app.editor_buf.clone();

        app.handle_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::CONTROL));
        assert_eq!(app.editor_path.as_ref(), Some(&first));
        assert_eq!(app.editor_buf, first_edited);
        assert!(app.dirty);
        assert!(app.tabs_title().contains("›а.lisp*"));

        app.handle_key(KeyEvent::new(KeyCode::Char('z'), KeyModifiers::CONTROL));
        assert_eq!(app.editor_buf, "(атом а)\n");
        assert!(!app.dirty);
        app.handle_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::CONTROL));
        assert_eq!(app.editor_buf, second_edited);
        assert!(app.dirty);

        // Не дозволяти закрити брудну вкладку або вийти із чистої,
        // поки в іншій вкладці лишилися незбережені зміни.
        app.handle_key(KeyEvent::new(KeyCode::Char('w'), KeyModifiers::CONTROL));
        assert_eq!(app.tabs.len(), 2);
        app.handle_key(KeyEvent::new(KeyCode::Char('z'), KeyModifiers::CONTROL));
        assert!(!app.dirty);
        app.handle_key(KeyEvent::new(KeyCode::Char('w'), KeyModifiers::CONTROL));
        assert_eq!(app.tabs.len(), 1);
        assert_eq!(app.editor_path.as_ref(), Some(&first));
        assert_eq!(std::fs::read_to_string(&first).unwrap(), "(атом а)\n");
        assert_eq!(std::fs::read_to_string(&second).unwrap(), "(атом б)\n");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn background_dirty_tab_requires_explicit_quit_confirmation() {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let root = std::env::temp_dir().join(format!(
            "my-idea-quit-tabs-{}-{stamp}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        let first = root.join("а.lisp");
        let second = root.join("б.lisp");
        std::fs::write(&first, "(цитата а)").unwrap();
        std::fs::write(&second, "(цитата б)").unwrap();
        let mut app = App::new_in(root.clone());
        app.file_sel = app.files.iter().position(|p| p == &first).unwrap();
        app.open_selected();
        app.paste("ї");
        app.file_sel = app.files.iter().position(|p| p == &second).unwrap();
        app.open_selected();
        assert!(!app.dirty);
        assert!(app.has_unsaved_changes());
        let quit = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL);
        assert!(!app.handle_key(quit));
        assert!(app.quit_armed);
        assert!(app.handle_key(quit));
        std::fs::remove_dir_all(root).unwrap();
    }
}


#[cfg(test)]
mod create_document_tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::sync::atomic::{AtomicU64, Ordering};
    static NONCE: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn create_edit_save_new_ukrainian_lisp_file_without_overwrite() {
        let root = std::env::temp_dir().join(format!(
            "my-idea-new-lisp-{}-{}", std::process::id(),
            NONCE.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&root).unwrap();
        let mut app = App::new_in(root.clone());

        // Той самий ввід консолі, що доступний користувачу, не тестова підміна стану.
        app.handle_key(KeyEvent::new(KeyCode::Char('n'), KeyModifiers::CONTROL));
        assert_eq!(app.pane, Pane::Console);
        assert_eq!(app.input, ":новий ");
        app.submit(":новий програма.lisp");
        let source = root.join("програма.lisp");
        assert_eq!(app.editor_path.as_ref(), Some(&source));
        assert_eq!(std::fs::read_to_string(&source).unwrap(), "");
        assert_eq!(app.pane, Pane::Editor);
        assert!(!app.dirty);

        app.paste("(атом? 'дім)\n");
        assert!(app.dirty);
        app.handle_key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL));
        assert!(!app.dirty);
        assert_eq!(std::fs::read_to_string(&source).unwrap(), "(атом? 'дім)\n");

        // Існуючі дані не можна випадково обнулити повторним :новий.
        app.submit(":новий програма.lisp");
        assert_eq!(std::fs::read_to_string(&source).unwrap(), "(атом? 'дім)\n");
        assert_eq!(app.editor_path.as_ref(), Some(&source));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn cannot_escape_workspace_or_discard_unsaved_source() {
        let root = std::env::temp_dir().join(format!(
            "my-idea-new-safe-{}-{}", std::process::id(),
            NONCE.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&root).unwrap();
        let mut app = App::new_in(root.clone());

        for name in ["../поза.lisp", "/tmp/поза.lisp", "під/поза.lisp", r"під\поза.lisp",
                     "без_розширення", "порожньо.sens"] {
            app.create_new_document(name);
            assert!(app.editor_path.is_none(), "небезпечний шлях: {name}");
        }
        app.create_new_document("перший.лісп");
        let first = root.join("перший.лісп");
        assert_eq!(app.editor_path.as_ref(), Some(&first));
        app.paste("(цитата дім)");
        assert!(app.dirty);
        app.create_new_document("другий.lisp");
        assert!(!root.join("другий.lisp").exists());
        assert_eq!(app.editor_path.as_ref(), Some(&first));
        assert!(app.dirty);
        assert_eq!(std::fs::read_to_string(first).unwrap(), "");
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[cfg(test)]
mod repl_history_draft_tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn up_down_restores_ukrainian_draft_and_utf8_cursor() {
        let mut app = App::new_in(std::env::temp_dir());
        app.pane = Pane::Console;
        // Це команди IDE, без дочірнього SENS; тут перевіряється реальна
        // клавіатурна поведінка редактора вводу, не імітація evaluator.
        app.submit(":інтерфейс de");
        app.submit(":інтерфейс uk");
        let draft = "(атом? 'чернетка)".to_owned();
        let cursor = "(атом? '".len();
        app.input = draft.clone();
        app.input_cursor = cursor;
        let up = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        let down = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        app.handle_key(up);
        assert_eq!(app.input, ":інтерфейс uk");
        app.handle_key(up);
        assert_eq!(app.input, ":інтерфейс de");
        app.handle_key(down);
        assert_eq!(app.input, ":інтерфейс uk");
        app.handle_key(down);
        assert_eq!(app.input, draft);
        assert_eq!(app.input_cursor, cursor);
        assert!(app.input.is_char_boundary(app.input_cursor));
        assert!(app.history_draft.is_none());
        app.handle_key(down);
        assert_eq!(app.input, draft);
        // Зміна запису історії не змінює збережену чернетку.
        app.handle_key(up);
        app.input.push('!');
        app.input_cursor = app.input.len();
        app.handle_key(down);
        assert_eq!(app.input, draft);
        assert_eq!(app.input_cursor, cursor);
        assert_eq!(app.history[1], ":інтерфейс uk");
    }

    #[test]
    fn empty_history_does_not_clear_input() {
        let mut app = App::new_in(std::env::temp_dir());
        app.pane = Pane::Console;
        app.input = "чернетка".to_owned();
        app.input_cursor = "чер".len();
        app.handle_key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        app.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(app.input, "чернетка");
        assert_eq!(app.input_cursor, "чер".len());
        assert!(app.history_draft.is_none());
    }
}
