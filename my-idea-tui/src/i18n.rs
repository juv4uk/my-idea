//! Інтерфейсні рядки TUI.
//!
//! Той самий набір ключів, що в GUI (`src-cljs/my_idea/i18n.cljs`); порядок мов
//! **uk → de → en**. Мова інтерфейсу — не перемикач доменної/семантичної SENS-проєкції.
//! Рядки, яких ще немає, повертаються як сам ключ (щоб не вигадувати переклад).

pub const LANGUAGES: [&str; 3] = ["uk", "de", "en"];

pub fn t(lang: &str, key: &str) -> &'static str {
    match (lang, key) {
        ("uk", "files") => "Файли",
        ("uk", "editor") => "Редактор",
        ("uk", "console") => "Консоль",
        ("uk", "open") => "Відкрити папку",
        ("uk", "save") => "Зберегти",
        ("uk", "run") => "Запустити",
        ("uk", "stop") => "Зупинити",
        ("uk", "quit") => "Вихід",
        ("uk", "banner") => "my-idea-tui · термінальна поверхня SENS. :мова ук|en|de — мова інтерфейсу.",
        ("uk", "no-engine") => "Рушій не знайдено: задайте MY_LISP_BIN або зберіть pinned my-lisp CLI.",
        ("uk", "run-hint") => "Запуск поточного файла через pinned SENS…",

        ("de", "files") => "Explorer",
        ("de", "editor") => "Editor",
        ("de", "console") => "Konsole",
        ("de", "open") => "Ordner öffnen",
        ("de", "save") => "Speichern",
        ("de", "run") => "Starten",
        ("de", "stop") => "Stoppen",
        ("de", "quit") => "Beenden",
        ("de", "banner") => "my-idea-tui · SENS-Terminaloberfläche. :мова ук|en|de — Oberflächensprache.",
        ("de", "no-engine") => "Engine nicht gefunden: MY_LISP_BIN setzen oder pinned my-lisp CLI bauen.",
        ("de", "run-hint") => "Aktuelle Datei über pinned SENS ausführen…",

        ("en", "files") => "Explorer",
        ("en", "editor") => "Editor",
        ("en", "console") => "Console",
        ("en", "open") => "Open folder",
        ("en", "save") => "Save",
        ("en", "run") => "Run",
        ("en", "stop") => "Stop",
        ("en", "quit") => "Quit",
        ("en", "banner") => "my-idea-tui · SENS terminal surface. :мова ук|en|de — UI language.",
        ("en", "no-engine") => "Engine not found: set MY_LISP_BIN or build the pinned my-lisp CLI.",
        ("en", "run-hint") => "Running current file via pinned SENS…",

        _ => key,
    }
}
