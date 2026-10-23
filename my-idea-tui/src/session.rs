//! Сесія TUI між запусками: мова, тема й відкриті вкладки.
//!
//! Це стан **хоста**, не мова: жодної другої семантики — зберігаємо лише те,
//! що вже відкрито. Формат — простий текст із табуляціями та екрануванням,
//! без сторонніх залежностей (контракт #31: жодного нового crate).

use std::path::{Path, PathBuf};

use crate::theme::Theme;

pub const HEADER: &str = "my-idea-tui-session 1";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionTab {
    pub path: PathBuf,
    pub text: String,
    pub cursor: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Session {
    pub language: String,
    pub theme: Theme,
    pub active: usize,
    pub tabs: Vec<SessionTab>,
}

impl Default for Session {
    fn default() -> Self {
        Session {
            language: "uk".to_string(),
            theme: Theme::Dark,
            active: 0,
            tabs: Vec::new(),
        }
    }
}

impl Session {
    /// Серіалізувати у текст: заголовок + по рядку на поле, вкладки — по рядку.
    pub fn to_text(&self) -> String {
        let mut out = String::from(HEADER);
        out.push('\n');
        out.push_str(&format!("language\t{}\n", escape(&self.language)));
        out.push_str(&format!("theme\t{}\n", self.theme.name()));
        out.push_str(&format!("active\t{}\n", self.active));
        for tab in &self.tabs {
            out.push_str(&format!(
                "tab\t{}\t{}\t{}\n",
                escape(&tab.path.to_string_lossy()),
                tab.cursor,
                escape(&tab.text)
            ));
        }
        out
    }

    /// Розібрати текст. Невідомі рядки й пошкоджені вкладки мовчки пропускаємо:
    /// сесія — це зручність, а не джерело істини, тож вона не має ламати запуск.
    pub fn from_text(text: &str) -> Session {
        let mut session = Session::default();
        let mut lines = text.lines();
        if lines.next() != Some(HEADER) {
            return session;
        }
        for line in lines {
            let mut fields = line.split('\t');
            match fields.next() {
                Some("language") => {
                    if let Some(value) = fields.next() {
                        session.language = unescape(value);
                    }
                }
                Some("theme") => {
                    if let Some(value) = fields.next() {
                        session.theme = Theme::from_name(value);
                    }
                }
                Some("active") => {
                    if let Some(value) = fields.next().and_then(|raw| raw.parse().ok()) {
                        session.active = value;
                    }
                }
                Some("tab") => {
                    let path = fields.next().map(unescape);
                    let cursor = fields.next().and_then(|raw| raw.parse().ok());
                    let body = fields.next().map(unescape);
                    if let (Some(path), Some(cursor), Some(body)) = (path, cursor, body) {
                        session.tabs.push(SessionTab {
                            path: PathBuf::from(path),
                            text: body,
                            cursor,
                        });
                    }
                }
                _ => {}
            }
        }
        session
    }

    pub fn load(path: &Path) -> Option<Session> {
        std::fs::read_to_string(path).ok().map(|text| Session::from_text(&text))
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, self.to_text())
    }
}

/// Штатне місце сесії: `$XDG_STATE_HOME/my-idea-tui/session` або
/// `$HOME/.local/state/my-idea-tui/session`. Якщо немає ні того, ні того —
/// сесію просто не зберігаємо (жодного вигаданого каталогу).
pub fn session_path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/state")))?;
    Some(base.join("my-idea-tui").join("session"))
}

fn escape(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            _ => out.push(ch),
        }
    }
    out
}

fn unescape(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some('\\') => out.push('\\'),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{session_path, Session, SessionTab, HEADER};
    use crate::theme::Theme;
    use std::path::PathBuf;

    #[test]
    fn session_round_trips_language_theme_and_tabs_with_escapes() {
        let session = Session {
            language: "de".to_string(),
            theme: Theme::Light,
            active: 1,
            tabs: vec![
                SessionTab {
                    path: PathBuf::from("/tmp/проєкт/програма.лісп"),
                    text: "(додати 2 3)\n\tдругий рядок".to_string(),
                    cursor: 7,
                },
                SessionTab {
                    path: PathBuf::from("/tmp/з пробілом.лісп"),
                    text: String::new(),
                    cursor: 0,
                },
            ],
        };
        let text = session.to_text();
        assert!(text.starts_with(HEADER));
        assert_eq!(Session::from_text(&text), session);
    }

    #[test]
    fn foreign_or_broken_text_yields_default_instead_of_panicking() {
        assert_eq!(Session::from_text("щось інше"), Session::default());
        // Заголовок є, але вкладки пошкоджені — лишаємо решту, вкладку пропускаємо.
        let text = format!("{HEADER}\nlanguage\tuk\ntab\tлише-шлях\nactive\tx\n");
        let parsed = Session::from_text(&text);
        assert_eq!(parsed.language, "uk");
        assert!(parsed.tabs.is_empty());
    }

    #[test]
    fn session_path_prefers_xdg_state_home() {
        let path = session_path().expect("у тестовому середовищі є HOME або XDG_STATE_HOME");
        assert!(path.ends_with("my-idea-tui/session"), "{path:?}");
    }
}
