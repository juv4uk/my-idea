//! Інтерфейсні рядки TUI.
//!
//! Той самий набір ключів, що в GUI (`src-cljs/my_idea/i18n.cljs`); порядок мов
//! **uk → de → en**. Мова інтерфейсу — не перемикач доменної/семантичної SENS-проєкції.
//! Рядки, яких ще немає, повертаються як сам ключ (щоб не вигадувати переклад).

pub const LANGUAGES: [&str; 3] = ["uk", "de", "en"];

pub fn t<'a>(lang: &str, key: &'a str) -> &'a str {
    match (lang, key) {
        ("uk", "files") => "Файли",
        ("uk", "editor") => "Редактор",
        ("uk", "ast") => "Дерево SENS",
        ("uk", "console") => "Консоль",
        ("uk", "open") => "Відкрити папку",
        ("uk", "save") => "Зберегти",
        ("uk", "run") => "Запустити",
        ("uk", "stop") => "Зупинити",
        ("uk", "quit") => "Вихід",
        ("uk", "banner") => "my-idea-tui · термінальна поверхня SENS. :інтерфейс uk|de|en — UI; :мова укр|ук|en|sa|core — SENS.",
        ("uk", "no-engine") => "Рушій не знайдено: зберіть pinned SENS CLI в external/sens/target.",
        ("uk", "run-hint") => "Запуск поточного файла через pinned SENS…",

        ("de", "files") => "Explorer",
        ("de", "editor") => "Editor",
        ("de", "ast") => "SENS-Baum",
        ("de", "console") => "Konsole",
        ("de", "open") => "Ordner öffnen",
        ("de", "save") => "Speichern",
        ("de", "run") => "Starten",
        ("de", "stop") => "Stoppen",
        ("de", "quit") => "Beenden",
        ("de", "banner") => "my-idea-tui · SENS-Terminaloberfläche. :інтерфейс uk|de|en — UI; :мова укр|ук|en|sa|core — SENS.",
        ("de", "no-engine") => "Engine nicht gefunden: pinned SENS CLI unter external/sens/target bauen.",
        ("de", "run-hint") => "Aktuelle Datei über pinned SENS ausführen…",

        ("en", "files") => "Explorer",
        ("en", "editor") => "Editor",
        ("en", "ast") => "SENS AST",
        ("en", "console") => "Console",
        ("en", "open") => "Open folder",
        ("en", "save") => "Save",
        ("en", "run") => "Run",
        ("en", "stop") => "Stop",
        ("en", "quit") => "Quit",
        ("en", "banner") => "my-idea-tui · SENS terminal surface. :інтерфейс uk|de|en — UI; :мова укр|ук|en|sa|core — SENS.",
        ("en", "no-engine") => "Engine not found: build the pinned SENS CLI under external/sens/target.",
        ("en", "run-hint") => "Running current file via pinned SENS…",

        _ => key,
    }
}
