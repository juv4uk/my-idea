//! Завантаження `init.lisp` і `plugins/*.lisp` у персистентну my-lisp
//! сесію (issue #10, IDE-LISP-PLUGINS-1) — локальна розширюваність у стилі
//! Emacs, без пакетного менеджера: детермінований порядок, ізольований
//! збій одного плагіна, явний reload.
//!
//! Loads `init.lisp` and `plugins/*.lisp` into the persistent my-lisp
//! session (issue #10, IDE-LISP-PLUGINS-1) — local, Emacs-style
//! extensibility without a package manager: deterministic order, isolated
//! per-plugin failure, explicit reload.

use std::fs;
use std::path::{Path, PathBuf};

use crate::ReplSession;

/// One plugin (or `init.lisp`) that failed to load, kept separate from a
/// hard error so the rest of the load can continue.
/// Один плагін (або `init.lisp`), що не завантажився — окремо від жорсткої
/// помилки, щоб решта завантаження продовжилась.
#[derive(Debug, Clone)]
pub struct PluginFailure {
    pub path: PathBuf,
    pub message: String,
}

/// Result of one load/reload pass: every file that evaluated successfully,
/// in the order it was evaluated, plus every failure, isolated.
/// Результат одного проходу завантаження/reload: усі успішно обчислені
/// файли в порядку обчислення, плюс усі ізольовані збої.
#[derive(Debug, Clone, Default)]
pub struct PluginLoadReport {
    pub loaded: Vec<PathBuf>,
    pub failures: Vec<PluginFailure>,
}

/// Resolves the default plugin config directory, `$HOME/.config/my-idea`.
/// Визначає типовий каталог конфігурації плагінів, `$HOME/.config/my-idea`.
pub fn default_config_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config").join("my-idea"))
}

/// Loads `<config_dir>/init.lisp` (if present), then every
/// `<config_dir>/plugins/*.lisp` file in deterministic (sorted by
/// filename) order, into `repl`. A file that fails to read or evaluate is
/// recorded in `failures` and skipped -- it never aborts the rest of the
/// load and never changes the relative order the remaining files load in.
/// Calling this again later is exactly what "explicit reload" means: no
/// hidden file-watcher, no second evaluator.
///
/// Завантажує `<config_dir>/init.lisp` (якщо є), тоді кожен
/// `<config_dir>/plugins/*.lisp` у детермінованому (за іменем файлу)
/// порядку в `repl`. Файл, що не читається чи не обчислюється, фіксується
/// в `failures` і пропускається — це не перериває решту завантаження й не
/// змінює відносний порядок інших файлів. Повторний виклик пізніше і є
/// "явним reload": без прихованого file-watcher, без другого евалюатора.
pub fn load_plugins(repl: &mut ReplSession, config_dir: &Path) -> PluginLoadReport {
    let mut report = PluginLoadReport::default();

    let init_path = config_dir.join("init.lisp");
    if init_path.is_file() {
        eval_one(repl, &init_path, &mut report);
    }

    let plugins_dir = config_dir.join("plugins");
    if let Ok(entries) = fs::read_dir(&plugins_dir) {
        let mut paths: Vec<PathBuf> = entries
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("lisp"))
            .collect();
        paths.sort();
        for path in paths {
            eval_one(repl, &path, &mut report);
        }
    }

    report
}

fn eval_one(repl: &mut ReplSession, path: &Path, report: &mut PluginLoadReport) {
    match fs::read_to_string(path) {
        Ok(source) => match repl.evaluate(&source) {
            Ok(_) => report.loaded.push(path.to_path_buf()),
            Err(message) => report.failures.push(PluginFailure {
                path: path.to_path_buf(),
                message,
            }),
        },
        Err(error) => report.failures.push(PluginFailure {
            path: path.to_path_buf(),
            message: error.to_string(),
        }),
    }
}
