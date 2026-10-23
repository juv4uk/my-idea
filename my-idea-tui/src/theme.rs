//! Тема оформлення TUI. Кольори мають **ролі**, а не «синій ось тут» — тож
//! тема добирає палітру під світлий чи темний фон термінала, не чіпаючи ні
//! семантику SENS, ні розкладку.

use ratatui::style::Color;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Theme {
    Dark,
    Light,
}

impl Theme {
    pub fn toggle(self) -> Theme {
        match self {
            Theme::Dark => Theme::Light,
            Theme::Light => Theme::Dark,
        }
    }

    /// Акцент: рамка активної панелі, вибір, оверлеї-введення.
    pub fn accent(self) -> Color {
        match self {
            Theme::Dark => Color::Cyan,
            Theme::Light => Color::Blue,
        }
    }

    /// Текст на акцентному фоні (вибір у списках).
    pub fn accent_fg(self) -> Color {
        match self {
            Theme::Dark => Color::Black,
            Theme::Light => Color::White,
        }
    }

    /// Рамки неактивних панелей і системні повідомлення.
    pub fn inactive(self) -> Color {
        match self {
            Theme::Dark => Color::DarkGray,
            Theme::Light => Color::Gray,
        }
    }

    /// Каталоги в дереві файлів і панель swarm-node.
    pub fn info(self) -> Color {
        match self {
            Theme::Dark => Color::Blue,
            Theme::Light => Color::Blue,
        }
    }

    /// Довідка й успішні стани.
    pub fn ok(self) -> Color {
        match self {
            Theme::Dark => Color::Green,
            Theme::Light => Color::Green,
        }
    }

    /// Попередження, екосистема, REPL-промпт.
    pub fn warn(self) -> Color {
        match self {
            Theme::Dark => Color::Yellow,
            Theme::Light => Color::Magenta,
        }
    }

    /// Помилки (stderr).
    pub fn error(self) -> Color {
        match self {
            Theme::Dark => Color::Red,
            Theme::Light => Color::Red,
        }
    }

    /// Граф знань (окремий відтінок від акцента).
    pub fn graph(self) -> Color {
        match self {
            Theme::Dark => Color::Magenta,
            Theme::Light => Color::Cyan,
        }
    }

    /// Машинна назва (для повідомлень і тестів).
    pub fn name(self) -> &'static str {
        match self {
            Theme::Dark => "dark",
            Theme::Light => "light",
        }
    }

    /// Тема за машинною назвою (невідома → темна).
    pub fn from_name(name: &str) -> Theme {
        match name {
            "light" => Theme::Light,
            _ => Theme::Dark,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Theme;

    #[test]
    fn toggle_switches_between_the_two_themes_and_roles_differ() {
        assert_eq!(Theme::Dark.toggle(), Theme::Light);
        assert_eq!(Theme::Light.toggle(), Theme::Dark);
        assert_ne!(Theme::Dark.accent(), Theme::Light.accent());
        assert_ne!(Theme::Dark.warn(), Theme::Light.warn());
        assert_eq!(Theme::Dark.name(), "dark");
        assert_eq!(Theme::Light.name(), "light");
    }
}
