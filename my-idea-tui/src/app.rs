//! Стан TUI і обробка подій.
//!
//! Вертикаль #85: Open workspace → Explorer → Edit → Save → Run → Output → Stop.
//! Панелі: дерево проєкту | редактор | консоль/вивід.

use std::path::{Component, Path, PathBuf};
use std::fs::OpenOptions;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::engine::{self, Engine, Line, Origin};
use crate::eco;
use crate::swarm;
use crate::i18n;
use crate::ast;

#[path = "../../src-tauri/src/workspace_save.rs"]
mod workspace_save;

#[derive(Clone, Copy, Debug, PartialEq)]
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
    /// Глибина кожного видимого рядка дерева Explorer (для відступу).
    pub depths: Vec<usize>,
    /// Розгорнуті каталоги дерева Explorer.
    pub expanded: std::collections::HashSet<PathBuf>,
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
    pub input: String,
    /// UTF-8 byte offset усередині поточної команди REPL.
    pub input_cursor: usize,
    pub history: Vec<String>,
    pub hist_idx: usize,
    pub repl_engine: Option<Engine>,
    pub run_engine: Option<Engine>,
    pub engine_running: bool,
    pub quit: bool,
    pub status: String,
    /// Точний pinned SENS gitlink (provenance), якщо відомий.
    pub pinned_sha: Option<String>,
    /// Довідка клавіш (F1) — лише перелік, без другої семантики.
    pub help_visible: bool,
    pub help_scroll: usize,
    /// Сусідні репозиторії екосистеми (їхні repo.lisp через pinned SENS).
    pub siblings: Vec<eco::Sibling>,
    pub eco_visible: bool,
    pub eco_sel: usize,
    /// Граф знань: звʼязки сусідів за спільними можливостями.
    pub graph_visible: bool,
    pub graph_scroll: usize,
    /// Спостереження власного swarm-node (той самий клієнт, що в GUI).
    pub swarm_visible: bool,
    pub swarm_scroll: usize,
    pub swarm_lines: Vec<String>,
    pub ast_visible: bool,
    pub ast_scroll: usize,
    pub ast_lines: Vec<ast::AstLine>,
    ast_cached_source: String,
    /// Палітра команд (Ctrl+P): той самий набір дій, що й клавіші.
    pub palette_visible: bool,
    pub palette_query: String,
    pub palette_sel: usize,
    /// Пошук у редакторі (Ctrl+F): запит, знайдені зсуви, вибір.
    pub search_visible: bool,
    pub search_query: String,
    pub search_hits: Vec<usize>,
    pub search_sel: usize,
}

/// Команди палітри (Ctrl+P). Ключі ті самі, що в i18n; жодної другої семантики.
const PALETTE: [&str; 19] = [
    "cmd-save", "cmd-run", "cmd-stop", "cmd-close-tab", "cmd-undo", "cmd-redo",
    "cmd-toggle-ast", "cmd-cycle-tab", "cmd-provenance", "cmd-find", "cmd-eco",
    "cmd-graph", "cmd-swarm", "cmd-ui-uk", "cmd-ui-de", "cmd-ui-en",
    "cmd-surface-укр", "cmd-surface-en", "cmd-quit",
];

/// Клавіші для F1: (підпис, i18n-ключ). Той самий набір, що в палітрі.
const HELP_KEYS: [(&str, &str); 12] = [
    ("Ctrl+S", "cmd-save"),
    ("F5", "cmd-run"),
    ("F9", "cmd-stop"),
    ("F6", "cmd-toggle-ast"),
    ("Ctrl+N", "cmd-new"),
    ("Ctrl+W", "cmd-close-tab"),
    ("Ctrl+Z", "cmd-undo"),
    ("Ctrl+Y", "cmd-redo"),
    ("Ctrl+P", "cmd-palette"),
    ("Tab", "cmd-cycle-pane"),
    ("Ctrl+Tab", "cmd-cycle-tab"),
    ("Ctrl+Q", "cmd-quit"),
];

/// Усі входження `needle` у `haystack` (без урахування регістру) як байтові зсуви.
fn find_all(haystack: &str, needle: &str) -> Vec<usize> {
    let needle: Vec<char> = needle.chars().flat_map(char::to_lowercase).collect();
    if needle.is_empty() {
        return Vec::new();
    }
    let hay: Vec<char> = haystack.chars().collect();
    let offsets: Vec<usize> = haystack.char_indices().map(|(b, _)| b).collect();
    let mut hits = Vec::new();
    if needle.len() > hay.len() {
        return hits;
    }
    for start in 0..=(hay.len() - needle.len()) {
        let same = needle.iter().enumerate().all(|(i, nc)| {
            hay[start + i].to_lowercase().next().unwrap_or(hay[start + i]) == *nc
        });
        if same {
            hits.push(offsets[start]);
        }
    }
    hits
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
            depths: Vec::new(),
            expanded: std::collections::HashSet::new(),
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
            input: String::new(),
            input_cursor: 0,
            history: Vec::new(),
            hist_idx: 0,
            repl_engine: None,
            run_engine: None,
            engine_running: false,
            quit: false,
            status: String::new(),
            pinned_sha: engine::pinned_revision().ok(),
            help_visible: false,
            help_scroll: 0,
            siblings: Vec::new(),
            eco_visible: false,
            eco_sel: 0,
            graph_visible: false,
            graph_scroll: 0,
            swarm_visible: false,
            swarm_scroll: 0,
            swarm_lines: Vec::new(),
            ast_visible: false,
            ast_scroll: 0,
            ast_lines: Vec::new(),
            ast_cached_source: String::new(),
            palette_visible: false,
            palette_query: String::new(),
            palette_sel: 0,
            search_visible: false,
            search_query: String::new(),
            search_hits: Vec::new(),
            search_sel: 0,
        };
        app.sys(i18n::t(&app.language, "banner"));
        app.reload_files();
        app
    }

    pub fn t<'a>(&self, key: &'a str) -> &'a str {
        i18n::t(&self.language, key)
    }

    /// Відфільтровані команди палітри за запитом (підрядок без регістру).
    pub fn palette_items(&self) -> Vec<&'static str> {
        let q = self.palette_query.to_lowercase();
        PALETTE
            .iter()
            .copied()
            .filter(|id| {
                q.is_empty() || self.t(*id).to_lowercase().contains(q.as_str()) || id.contains(q.as_str())
            })
            .collect()
    }

    fn palette_toggle(&mut self) {
        self.palette_visible = !self.palette_visible;
        self.palette_query.clear();
        self.palette_sel = 0;
    }

    /// Виконати команду палітри за її i18n-ключем.
    fn palette_run(&mut self, id: &str) {
        self.palette_visible = false;
        self.palette_query.clear();
        self.palette_sel = 0;
        match id {
            "cmd-save" => self.save(),
            "cmd-run" => self.run_current(),
            "cmd-stop" => self.stop(),
            "cmd-close-tab" => self.close_current_tab(),
            "cmd-undo" => self.undo_edit(),
            "cmd-redo" => self.redo_edit(),
            "cmd-toggle-ast" => {
                self.ast_visible = !self.ast_visible;
                self.refresh_ast();
            }
            "cmd-cycle-tab" => self.cycle_tab(false),
            "cmd-provenance" => self.show_provenance(),
            "cmd-eco" => self.eco_toggle(),
            "cmd-graph" => self.graph_toggle(),
            "cmd-find" => self.search_toggle(),
            "cmd-swarm" => self.swarm_toggle(),
            "cmd-ui-uk" => self.set_ui_language("uk"),
            "cmd-ui-de" => self.set_ui_language("de"),
            "cmd-ui-en" => self.set_ui_language("en"),
            "cmd-surface-укр" => self.meta("мова укр"),
            "cmd-surface-en" => self.meta("мова en"),
            "cmd-quit" => {
                if self.has_unsaved_changes() && !self.quit_armed {
                    self.quit_armed = true;
                    self.sys(self.t("quit-dirty"));
                } else {
                    self.quit = true;
                }
            }
            _ => {}
        }
    }

    /// Мова інтерфейсу з інвалідацією локалізованого AST-кеша (як `:інтерфейс`).
    fn set_ui_language(&mut self, lang: &str) {
        if !i18n::LANGUAGES.contains(&lang) {
            return;
        }
        if self.language != lang {
            self.language = lang.to_string();
            self.ast_lines.clear();
            self.ast_cached_source.clear();
            self.ast_scroll = 0;
        }
        self.sys(&format!("{} {lang}", self.t("ui-lang-set")));
    }

    /// Provenance: точний pinned SENS на вимогу (паритет із GUI).
    fn show_provenance(&mut self) {
        let sha = self.pinned_sha.clone().or_else(|| engine::pinned_revision().ok());
        self.pinned_sha = sha.clone();
        self.pane = Pane::Console;
        match sha {
            Some(sha) => self.sys(&format!("SENS pinned gitlink={sha}")),
            None => self.sys(self.t("provenance-unknown")),
        }
    }

    fn help_toggle(&mut self) {
        self.help_visible = !self.help_visible;
        self.help_scroll = 0;
    }

    /// Рядки довідки F1: підпис клавіші + локалізована назва дії.
    pub fn help_lines(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for (key, label) in HELP_KEYS {
            out.push(format!("{:<12} {}", key, self.t(label)));
        }
        out.push(String::new());
        out.push(self.t("help-meta").to_string());
        out
    }

    fn eco_toggle(&mut self) {
        self.eco_visible = !self.eco_visible;
        self.eco_sel = 0;
        if self.eco_visible {
            self.refresh_siblings();
        }
    }

    /// Перечитати сусідні репозиторії з їхніх `repo.lisp` (reader pinned SENS).
    pub fn refresh_siblings(&mut self) {
        self.siblings = engine::repo_root()
            .map(|root| eco::siblings(&root))
            .unwrap_or_default();
    }

    /// Рядки панелі «Екосистема»: ім'я, роль (або стан), id.
    pub fn eco_lines(&self) -> Vec<String> {
        if self.siblings.is_empty() {
            return vec![self.t("eco-empty").to_string()];
        }
        self.siblings
            .iter()
            .map(|s| {
                let status = if s.note.is_empty() {
                    s.role.clone()
                } else {
                    self.t(s.note).to_string()
                };
                format!("{:<18} {:<26} {}", s.name, status, s.id)
            })
            .collect()
    }

    fn graph_toggle(&mut self) {
        self.graph_visible = !self.graph_visible;
        self.graph_scroll = 0;
        if self.graph_visible {
            self.refresh_siblings();
        }
    }

    /// Рядки графа знань: можливість, експортери, імпортери.
    pub fn graph_lines(&self) -> Vec<String> {
        let edges = eco::capabilities(&self.siblings);
        if edges.is_empty() {
            return vec![self.t("graph-empty").to_string()];
        }
        edges
            .iter()
            .map(|edge| {
                let exporters = if edge.exporters.is_empty() { "\u{2014}".to_string() } else { edge.exporters.join(", ") };
                let importers = if edge.importers.is_empty() { "\u{2014}".to_string() } else { edge.importers.join(", ") };
                format!("{:<26} {} \u{2192} {}", edge.name, exporters, importers)
            })
            .collect()
    }

    fn swarm_toggle(&mut self) {
        self.swarm_visible = !self.swarm_visible;
        self.swarm_scroll = 0;
        if self.swarm_visible {
            self.refresh_swarm();
        }
    }

    /// Спитати власний swarm-node `(status)` і показати сиру відповідь + розбір SENS.
    pub fn refresh_swarm(&mut self) {
        self.swarm_lines.clear();
        self.swarm_lines.push(format!("{} 127.0.0.1:9104", self.t("swarm-endpoint")));
        match swarm::query("(status)", None) {
            Ok(raw) => {
                self.swarm_lines.push(raw.clone());
                self.swarm_lines.push(String::new());
                self.swarm_lines.extend(ast::render(&raw, &self.language));
            }
            Err(error) => {
                self.swarm_lines.push(format!("{}: {error}", self.t("swarm-unavailable")));
            }
        }
    }

    fn search_toggle(&mut self) {
        self.search_visible = !self.search_visible;
        if self.search_visible {
            self.search_query.clear();
            self.search_recompute();
        }
    }

    fn search_recompute(&mut self) {
        self.search_hits = find_all(&self.editor_buf, &self.search_query);
        self.search_sel = 0;
        self.search_jump();
    }

    fn search_jump(&mut self) {
        if let Some(offset) = self.search_hits.get(self.search_sel) {
            self.editor_cursor = *offset;
        }
    }

    /// Рядки результатів пошуку: маркер, номер рядка, вміст рядка.
    pub fn search_lines(&self) -> Vec<String> {
        if self.search_query.is_empty() {
            return vec![self.t("find-prompt").to_string()];
        }
        if self.search_hits.is_empty() {
            return vec![self.t("find-none").to_string()];
        }
        let text = &self.editor_buf;
        self.search_hits
            .iter()
            .enumerate()
            .map(|(index, offset)| {
                let line = text[..*offset].matches('\n').count() + 1;
                let start = text[..*offset].rfind('\n').map(|p| p + 1).unwrap_or(0);
                let end = text[*offset..]
                    .find('\n')
                    .map(|p| *offset + p)
                    .unwrap_or(text.len());
                let marker = if index == self.search_sel { "\u{203a}" } else { " " };
                format!("{marker} {line:>4}: {}", text[start..end].trim_end())
            })
            .collect()
    }

    /// Голова найглибшої форми під курсором (живий AST, reader pinned SENS).
    pub fn cursor_head(&self) -> Option<String> {
        ast::head_at_cursor(&self.editor_buf, self.editor_cursor)
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
        let mut files = Vec::new();
        let mut depths = Vec::new();
        walk_tree(&self.explorer_dir, 0, &self.expanded, &mut files, &mut depths);
        self.files = files;
        self.depths = depths;
        if self.file_sel >= self.files.len() {
            self.file_sel = self.files.len().saturating_sub(1);
        }
    }

    /// Розгорнути/згорнути каталог у дереві Explorer.
    pub(crate) fn toggle_dir(&mut self, path: PathBuf) {
        if !self.expanded.insert(path.clone()) {
            self.expanded.remove(&path);
        }
        self.reload_files();
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
            self.sys(self.t("dirty-close"));
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

    pub(crate) fn open_selected(&mut self) {
        if let Some(p) = self.files.get(self.file_sel).cloned() {
            if p.is_dir() {
                self.toggle_dir(p);
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
                    self.dirty = false;
                    self.active_tab = self.tabs.len();
                    self.tabs.push(OpenDocument {
                        path: p.clone(), text: text.clone(), original: text,
                        cursor: self.editor_cursor, undo: Vec::new(), redo: Vec::new(),
                    });
                    self.pane = Pane::Editor;
                    self.status = format!("{} {}", self.t("open-status"), p.display());
                }
                Err(error) => self.sys(&format!("{} {}: {error}", self.t("open-failed"), p.display())),
            }
        }
    }

    /// Створити новий Lisp у поточній папці Explorer без перезапису.
    /// Незбережені чернетки зберігаються незалежно в інших вкладках.
    fn create_new_document(&mut self, name: &str) {
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
            Err(error) => { self.sys(&format!("{}: {error}", self.t("ws-unavailable"))); return; }
        };
        let folder = match self.explorer_dir.canonicalize() {
            Ok(folder) => folder,
            Err(error) => { self.sys(&format!("{}: {error}", self.t("explorer-unavailable"))); return; }
        };
        if !folder.starts_with(&root) {
            self.sys(self.t("explorer-escape"));
            return;
        }
        let path = folder.join(name);
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => {
                drop(file);
                // Новий файл — окрема вкладка: зберігаємо старий буфер, Undo
                // і курсор, щоб Ctrl+Tab не втрачав стан попереднього файла.
                self.store_active_tab();
                self.active_tab = self.tabs.len();
                self.tabs.push(OpenDocument {
                    path: path.clone(),
                    text: String::new(),
                    original: String::new(),
                    cursor: 0,
                    undo: Vec::new(),
                    redo: Vec::new(),
                });
                self.ast_cached_source.clear();
                self.ast_lines.clear();
                self.ast_scroll = 0;
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
                self.status = format!("{} {}", self.t("new-status"), path.display());
                self.sys(&format!("{} {}", self.t("created"), path.display()));
            }
            Err(error) => self.sys(&format!("{} {}: {error}", self.t("create-failed"), path.display())),
        }
    }

    fn save(&mut self) {
        let Some(path) = self.editor_path.clone() else { return; };
        if !self.dirty { return; }
        // Захист від паралельного редагування + спільний GUI/TUI атомарний запис.
        let result = save_loaded_file(&self.language, &self.cwd, &path, &self.editor_original, &self.editor_buf);
        match result {
            Ok(()) => {
                self.editor_original = self.editor_buf.clone();
                self.dirty = false;
                self.status = format!("saved {}", path.display());
            }
            Err(error) => self.sys(&format!("{} {}: {error}. {}", self.t("save-failed"), path.display(), self.t("buffer-kept"))),
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
            self.sys(self.t("no-file"));
            return;
        };
        if self.dirty {
            self.sys(self.t("save-first"));
            return;
        }
        if self.engine_running {
            self.sys(self.t("already-running"));
            return;
        }
        match std::fs::read_to_string(&path) {
            Ok(on_disk) if on_disk == self.editor_buf => {}
            Ok(_) => {
                self.sys(self.t("changed-externally"));
                return;
            }
            Err(error) => {
                self.sys(&format!("{}: {error}", self.t("run-read-failed")));
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
                    text: format!("{}: {error}", self.t("run-failed")),
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
        self.sys(self.t("stopped"));
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
                    // Закрити pipe-и нащадків CLI до приєднання reader-потоків.
                    eng.stop();
                    while let Ok(line) = eng.rx.try_recv() { self.console.push(line); }
                }
                self.sys(&format!("{}={status}; {}", self.t("repl-exit-status"), self.t("repl-next")));
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
                        // Закрити pipe-и нащадків CLI до приєднання reader-потоків.
                    eng.stop();
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
                    if self.language != l {
                        self.language = l.to_string();
                        // Переклад змінює лише UI; кеш AST містить локалізовані рядки.
                        self.ast_lines.clear();
                        self.ast_cached_source.clear();
                        self.ast_scroll = 0;
                    }
                    self.sys(&format!("{} {l}", self.t("ui-lang-set")));
                }
                Some(l) => self.sys(&format!("{} {l}; {}", self.t("ui-lang-bad"), self.t("ui-lang-avail"))),
                None => self.sys(self.t("ui-lang-help")),
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
                    self.sys(self.t("open-file-first"));
                    return;
                }
                // Paste може надходити з CRLF (Windows) або CR (старий Mac).
                // Вміст Lisp зберігаємо як LF, не залишаючи невидимих CR
                // у редакторі та не змінюючи жодних SENS-символів.
                let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
                self.remember_edit();
                self.editor_buf.insert_str(self.editor_cursor, &normalized);
                self.editor_cursor += normalized.len();
                self.dirty = self.editor_buf != self.editor_original;
            }
            Pane::Console => {
                if text.contains('\r') || text.contains('\n') {
                    self.sys(self.t("paste-multiline"));
                } else {
                    self.input.insert_str(self.input_cursor, text);
                    self.input_cursor += text.len();
                }
            }
            Pane::Explorer => self.sys(self.t("paste-no-target")),
        }
    }

    /// AST створює тільки reader pinned SENS, коли панель відкрита.
    /// Прихована панель не парсить Lisp на кожне натискання клавіші.
    pub fn refresh_ast(&mut self) {
        if !self.ast_visible { return; }
        if self.editor_path.is_none() {
            self.ast_lines.clear();
            self.ast_cached_source.clear();
            self.ast_scroll = 0;
        } else if self.ast_lines.is_empty() || self.ast_cached_source != self.editor_buf {
            self.ast_lines = ast::render_spans(&self.editor_buf, &self.language);
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
        if self.swarm_visible {
            match key.code {
                KeyCode::Esc => self.swarm_toggle(),
                KeyCode::Char('r') if ctrl => self.swarm_toggle(),
                KeyCode::Up => self.swarm_scroll = self.swarm_scroll.saturating_sub(1),
                KeyCode::Down => self.swarm_scroll = self.swarm_scroll.saturating_add(1),
                KeyCode::PageUp => self.swarm_scroll = self.swarm_scroll.saturating_sub(6),
                KeyCode::PageDown => self.swarm_scroll = self.swarm_scroll.saturating_add(6),
                _ => {}
            }
            return false;
        }
        if self.graph_visible {
            match key.code {
                KeyCode::Esc => self.graph_toggle(),
                KeyCode::Char('g') if ctrl => self.graph_toggle(),
                KeyCode::Up => self.graph_scroll = self.graph_scroll.saturating_sub(1),
                KeyCode::Down => self.graph_scroll = self.graph_scroll.saturating_add(1),
                KeyCode::PageUp => self.graph_scroll = self.graph_scroll.saturating_sub(6),
                KeyCode::PageDown => self.graph_scroll = self.graph_scroll.saturating_add(6),
                _ => {}
            }
            return false;
        }
        if self.eco_visible {
            match key.code {
                KeyCode::Esc => self.eco_toggle(),
                KeyCode::Char('e') if ctrl => self.eco_toggle(),
                KeyCode::Up => self.eco_sel = self.eco_sel.saturating_sub(1),
                KeyCode::Down => {
                    if self.eco_sel + 1 < self.siblings.len() {
                        self.eco_sel += 1;
                    }
                }
                _ => {}
            }
            return false;
        }
        if self.help_visible {
            match key.code {
                KeyCode::Esc | KeyCode::F(1) => self.help_toggle(),
                KeyCode::Up => self.help_scroll = self.help_scroll.saturating_sub(1),
                KeyCode::Down => self.help_scroll = self.help_scroll.saturating_add(1),
                KeyCode::PageUp => self.help_scroll = self.help_scroll.saturating_sub(6),
                KeyCode::PageDown => self.help_scroll = self.help_scroll.saturating_add(6),
                _ => {}
            }
            return false;
        }
        if self.search_visible {
            match key.code {
                KeyCode::Esc => self.search_toggle(),
                KeyCode::Char('f') if ctrl => self.search_toggle(),
                KeyCode::Enter => self.search_jump(),
                KeyCode::Up => {
                    self.search_sel = self.search_sel.saturating_sub(1);
                    self.search_jump();
                }
                KeyCode::Down => {
                    if self.search_sel + 1 < self.search_hits.len() {
                        self.search_sel += 1;
                        self.search_jump();
                    }
                }
                KeyCode::Backspace => {
                    self.search_query.pop();
                    self.search_recompute();
                }
                KeyCode::Char(c) if !ctrl => {
                    self.search_query.push(c);
                    self.search_recompute();
                }
                _ => {}
            }
            return false;
        }
        if self.palette_visible {
            match key.code {
                KeyCode::Esc => self.palette_toggle(),
                KeyCode::Enter => {
                    let chosen = self.palette_items().get(self.palette_sel).copied();
                    if let Some(id) = chosen {
                        self.palette_run(id);
                    }
                }
                KeyCode::Up => self.palette_sel = self.palette_sel.saturating_sub(1),
                KeyCode::Down => {
                    if self.palette_sel + 1 < self.palette_items().len() {
                        self.palette_sel += 1;
                    }
                }
                KeyCode::Backspace => {
                    self.palette_query.pop();
                    self.palette_sel = 0;
                }
                KeyCode::Char(c) if !ctrl => {
                    self.palette_query.push(c);
                    self.palette_sel = 0;
                }
                KeyCode::Char('p') if ctrl => self.palette_toggle(),
                _ => {}
            }
            return false;
        }
        if ctrl && matches!(key.code, KeyCode::Char('p')) {
            self.palette_toggle();
            return false;
        }
        match key.code {
            KeyCode::Char('f') if ctrl => self.search_toggle(),
            KeyCode::Char('n') if ctrl => {
                if self.input.is_empty() {
                    self.input = ":новий ".to_string();
                    self.input_cursor = self.input.len();
                    self.pane = Pane::Console;
                    self.console_scroll_back = 0;
                } else {
                    self.sys(self.t("console-unfinished"));
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
                    self.sys(self.t("quit-dirty"));
                } else {
                    return true;
                }
            },
            KeyCode::Esc => self.pane = Pane::Explorer,
            KeyCode::Char('e') if ctrl => self.eco_toggle(),
            KeyCode::Char('g') if ctrl => self.graph_toggle(),
            KeyCode::Char('r') if ctrl => self.swarm_toggle(),
            KeyCode::F(1) => self.help_toggle(),
            KeyCode::F(5) => self.run_current(),
            KeyCode::F(6) => {
                self.ast_visible = !self.ast_visible;
                self.refresh_ast();
            },
            KeyCode::F(9) => self.stop(),
            KeyCode::PageUp if self.ast_visible && key.modifiers.contains(KeyModifiers::ALT) => {
                self.ast_scroll = self.ast_scroll.saturating_sub(6);
            },
            KeyCode::PageDown if self.ast_visible && key.modifiers.contains(KeyModifiers::ALT) => {
                self.ast_scroll = self.ast_scroll.saturating_add(6)
                    .min(self.ast_lines.len().saturating_sub(1));
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
                    self.dirty = self.editor_buf != self.editor_original;
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
                // End завжди повертає до останнього реального stdout/stderr,
                // а також до кінця чернетки REPL (не втрачаючи UTF-8 ввід).
                self.console_scroll_back = 0;
                self.input_cursor = self.input.len();
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
                    self.dirty = self.editor_buf != self.editor_original;
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
                    self.dirty = self.editor_buf != self.editor_original;
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
                            self.status = format!("{} {}", self.t("dir"), self.explorer_dir.display());
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
                } else {
                    self.hist_idx = self.history.len();
                    self.input.clear();
                }
                self.input_cursor = self.input.len();
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
/// Рекурсивно збирає видимі рядки дерева Explorer: шлях + глибина.
/// Розгорнуті каталоги показують вміст, згорнуті — лише себе.
fn walk_tree(
    dir: &Path,
    depth: usize,
    expanded: &std::collections::HashSet<PathBuf>,
    files: &mut Vec<PathBuf>,
    depths: &mut Vec<usize>,
) {
    for entry in directory_entries(dir) {
        let is_dir = entry.is_dir();
        files.push(entry.clone());
        depths.push(depth);
        if is_dir && expanded.contains(&entry) {
            walk_tree(&entry, depth + 1, expanded, files, depths);
        }
    }
}

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
fn save_loaded_file(lang: &str, root: &Path, path: &Path, original: &str, updated: &str) -> Result<(), String> {
    let disk = std::fs::read_to_string(path)
        .map_err(|error| format!("{}: {error}", i18n::t(lang, "verify-failed")))?;
    if disk != original {
        return Err(i18n::t(lang, "save-conflict").to_string());
    }
    let relative = path.strip_prefix(root)
        .map_err(|error| format!("{}: {error}", i18n::t(lang, "outside-ws")))?;
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
        save_loaded_file("uk", &workspace, &source, "(атом а)\n", "(атом б)\n").unwrap();
        assert_eq!(fs::read_to_string(&source).unwrap(), "(атом б)\n");

        // Інший редактор змінив файл: TUI не знищує його правки.
        fs::write(&source, "(атом в)\n").unwrap();
        assert!(save_loaded_file("uk", &workspace, &source, "(атом б)\n", "(атом г)\n").is_err());
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
        // Поодинокий CR не повинен непомітно залишатись у вихідному Lisp.
        app.paste("г\rґ");
        assert_eq!(app.editor_buf, "(цитата б\nвг\nґа)");
        assert!(!app.editor_buf.contains('\r'));
        assert!(app.editor_buf.is_char_boundary(app.editor_cursor));
        // Undo повертає лише останню вставку (одна дія), навіть для CR-переносу.
        app.undo_edit();
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
    fn ctrl_n_allows_typing_filename_then_switching_back_to_previous_tab() {
        let root = std::env::temp_dir().join(format!(
            "my-idea-tui-new-tab-input-{}-{}", std::process::id(),
            NONCE.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&root).unwrap();
        let first = root.join("початковий.lisp");
        std::fs::write(&first, "(цитата початок)").unwrap();
        let mut app = App::new_in(root.clone());
        app.file_sel = app.files.iter().position(|p| p == &first).unwrap();
        app.open_selected();
        assert_eq!(app.tabs.len(), 1);

        app.handle_key(KeyEvent::new(KeyCode::Char('n'), KeyModifiers::CONTROL));
        assert_eq!(app.pane, Pane::Console);
        assert_eq!(app.input, ":новий ");
        assert_eq!(app.input_cursor, app.input.len());
        app.paste("наступний.lisp");
        assert_eq!(app.input, ":новий наступний.lisp");
        app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

        let second = root.join("наступний.lisp");
        assert_eq!(app.editor_path.as_ref(), Some(&second));
        assert_eq!(app.tabs.len(), 2);
        assert_eq!(app.active_tab, 1);
        app.paste("(цитата другий)");
        app.handle_key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL));
        assert_eq!(std::fs::read_to_string(&second).unwrap(), "(цитата другий)");

        app.handle_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::CONTROL));
        assert_eq!(app.editor_path.as_ref(), Some(&first));
        assert_eq!(app.editor_buf, "(цитата початок)");
        app.handle_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::CONTROL));
        assert_eq!(app.editor_path.as_ref(), Some(&second));
        assert_eq!(app.editor_buf, "(цитата другий)");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn cannot_escape_workspace_and_creating_new_tab_preserves_unsaved_source() {
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
        // Створення нового документа не губить чернетку попередньої вкладки.
        app.create_new_document("другий.lisp");
        let second = root.join("другий.lisp");
        assert!(second.is_file());
        assert_eq!(app.editor_path.as_ref(), Some(&second));
        assert_eq!(app.tabs.len(), 2);
        assert_eq!(app.editor_buf, "");
        assert!(!app.dirty);

        app.paste("(цитата другий)");
        app.save();
        assert_eq!(std::fs::read_to_string(&second).unwrap(), "(цитата другий)");
        app.cycle_tab(true);
        assert_eq!(app.editor_path.as_ref(), Some(&first));
        assert_eq!(app.editor_buf, "(цитата дім)");
        assert!(app.dirty);
        assert_eq!(std::fs::read_to_string(&first).unwrap(), "");

        // Повернувшись до нової вкладки, бачимо раніше збережений код.
        app.cycle_tab(false);
        assert_eq!(app.editor_path.as_ref(), Some(&second));
        assert_eq!(app.editor_buf, "(цитата другий)");
        assert!(!app.dirty);
        assert!(app.has_unsaved_changes(), "чернетка іншої вкладки не повинна загубитися");
        std::fs::remove_dir_all(root).unwrap();
    }
}


#[cfg(test)]
mod ast_visibility_tests {
    use super::*;

    #[test]
    fn ui_language_switch_rebuilds_ast_and_keeps_ukrainian_file_untouched() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let root = std::env::temp_dir().join(format!("my-idea-ast-ui-{}-{nonce}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        let file = root.join("джерело.lisp");
        let source = "(умова (атом? їжак) (цитата мама))";
        std::fs::write(&file, source).unwrap();
        let mut app = App::new_in(root.clone());
        app.open_selected();
        app.ast_visible = true;
        app.refresh_ast();
        assert!(app.ast_lines.iter().any(|line| line.text.contains("Список (3) · умова")));
        app.submit(":інтерфейс de");
        app.refresh_ast();
        assert!(
            app.ast_lines.iter().any(|line| line.text.contains("Liste (3) · умова")),
            "{:?}",
            app.ast_lines
        );
        app.submit(":інтерфейс en");
        app.refresh_ast();
        assert!(app.ast_lines.iter().any(|l| l.text.contains("List (3) · умова")), "{:?}", app.ast_lines);
        assert_eq!(app.editor_buf, source);
        assert_eq!(std::fs::read_to_string(&file).unwrap(), source);
        assert!(app.repl_engine.is_none(), "UI-only switch cannot invoke SENS");
        std::fs::remove_dir_all(root).unwrap();
    }


    #[test]
    fn hidden_ast_never_parses_and_visible_ast_uses_pinned_sens() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let root = std::env::temp_dir().join(format!("my-idea-ast-{}-{nonce}",std::process::id()));
        std::fs::create_dir(&root).unwrap();
        let file = root.join("програма.lisp");
        std::fs::write(&file, "(атом? 'мама)").unwrap();
        let mut app = App::new_in(root.clone());
        app.open_selected();
        app.refresh_ast();
        assert!(app.ast_lines.is_empty(), "прихований AST не повинен парсити");

        let _ = app.handle_key(KeyEvent::new(KeyCode::F(6), KeyModifiers::NONE));
        assert!(app.ast_visible);
        assert!(app.ast_lines.iter().any(|l| l.text.contains("мама")), "{:?}", app.ast_lines);
        let cached = app.ast_cached_source.clone();

        let _ = app.handle_key(KeyEvent::new(KeyCode::F(6), KeyModifiers::NONE));
        app.paste("(цитата батько)");
        app.refresh_ast();
        assert_eq!(app.ast_cached_source, cached, "прихований AST не перепарсить джерело");

        let _ = app.handle_key(KeyEvent::new(KeyCode::F(6), KeyModifiers::NONE));
        assert!(app.ast_lines.iter().any(|l| l.text.contains("батько")), "{:?}", app.ast_lines);
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[cfg(test)]
mod actual_dirty_state_tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn manually_reverted_ukrainian_edits_allow_clean_tab_close() {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let root = std::env::temp_dir().join(format!(
            "my-idea-dirty-content-{}-{stamp}", std::process::id(),
        ));
        std::fs::create_dir(&root).unwrap();
        let path = root.join("текст.lisp");
        let original = "(цитата їжак)\n";
        std::fs::write(&path, original).unwrap();
        let mut app = App::new_in(root.clone());
        app.file_sel = app.files.iter().position(|p| p == &path).unwrap();
        app.open_selected();
        assert_eq!(app.editor_buf, original);

        // Додавання та ручне видалення UTF-8 символу відновлює чистий стан
        // без Ctrl+Z або зайвого запису у файлову систему.
        app.handle_key(KeyEvent::new(KeyCode::Char('ґ'), KeyModifiers::NONE));
        assert!(app.dirty);
        app.handle_key(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
        assert_eq!(app.editor_buf, original);
        assert!(!app.dirty);
        assert!(!app.has_unsaved_changes());

        app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert!(app.dirty);
        app.handle_key(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
        assert_eq!(app.editor_buf, original);
        assert!(!app.dirty);

        app.paste("є");
        assert!(app.dirty);
        app.handle_key(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
        assert_eq!(app.editor_buf, original);
        assert!(!app.dirty);
        assert!(!app.tabs_title().contains("текст.lisp*"));

        app.handle_key(KeyEvent::new(KeyCode::Char('w'), KeyModifiers::CONTROL));
        assert!(app.editor_path.is_none(), "чисту вкладку можна закрити");
        assert!(app.tabs.is_empty());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), original,
            "ручне скасування не перезаписує файл");
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[cfg(test)]
mod search_tests {
    use super::find_all;

    #[test]
    fn finds_utf8_offsets_case_insensitively_without_splitting_chars() {
        assert_eq!(find_all("abcABC", "a"), vec![0, 3]);
        assert_eq!(find_all("їжак їжак", "їж"), vec![0, 9]);
        assert_eq!(find_all("abc", "abc"), vec![0]);
        assert_eq!(find_all("abc", "x"), Vec::<usize>::new());
        assert_eq!(find_all("abc", ""), Vec::<usize>::new());
        // Кожен зсув — межа символу, тож зріз тексту безпечний.
        for offset in find_all("їжак їжак", "їж") {
            assert!("їжак їжак".is_char_boundary(offset));
        }
    }
}

#[cfg(test)]
mod tree_tests {
    use super::walk_tree;
    use std::collections::HashSet;
    use std::path::PathBuf;

    fn temp_tree(tag: &str) -> PathBuf {
        let base = std::env::temp_dir().join(format!("my-idea-tui-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(base.join("sub")).unwrap();
        std::fs::write(base.join("a.lisp"), "()").unwrap();
        std::fs::write(base.join("sub").join("b.lisp"), "()").unwrap();
        base
    }

    #[test]
    fn collapsed_tree_shows_top_level_and_expands_on_demand() {
        let base = temp_tree("tree");
        let mut files = Vec::new();
        let mut depths = Vec::new();
        walk_tree(&base, 0, &HashSet::new(), &mut files, &mut depths);
        assert!(files.iter().all(|p| p.parent() == Some(base.as_path())));
        assert!(depths.iter().all(|d| *d == 0));

        let mut expanded = HashSet::new();
        expanded.insert(base.join("sub"));
        let mut files = Vec::new();
        let mut depths = Vec::new();
        walk_tree(&base, 0, &expanded, &mut files, &mut depths);
        let child = base.join("sub").join("b.lisp");
        let index = files
            .iter()
            .position(|p| p == &child)
            .expect("дитина розгорнутого каталогу");
        assert_eq!(depths[index], 1);
        let _ = std::fs::remove_dir_all(&base);
    }
}
