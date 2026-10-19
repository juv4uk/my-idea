//! Малювання панелей.
//!
//! Панелі ті самі, що в GUI: дерево проєкту | редактор | консоль/вивід.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::app::{display_name, App, Pane};
use crate::engine::Origin;

pub fn draw(f: &mut Frame, app: &App) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(f.area());
    draw_body(f, app, outer[0]);
    draw_status(f, app, outer[1]);
}

fn draw_body(f: &mut Frame, app: &App, area: Rect) {
    // Вузький термінал: не стискати редактор до кількох символів.
    // Всі три складники тієї самої IDE лишаються видимими.
    if area.width < 76 {
        if app.ast_visible {
            let rows = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(5), Constraint::Percentage(45),
                    Constraint::Percentage(25), Constraint::Min(4)])
                .split(area);
            draw_explorer(f, app, rows[0]);
            draw_editor(f, app, rows[1]);
            draw_ast(f, app, rows[2]);
            draw_console(f, app, rows[3]);
        } else {
            let rows = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(5), Constraint::Percentage(55), Constraint::Min(4)])
                .split(area);
            draw_explorer(f, app, rows[0]);
            draw_editor(f, app, rows[1]);
            draw_console(f, app, rows[2]);
        }
        return;
    }

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(28), Constraint::Min(1)])
        .split(area);
    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(cols[1]);

    draw_explorer(f, app, cols[0]);
    if app.ast_visible {
        let editor_ast = Layout::default().direction(Direction::Vertical)
            .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
            .split(right[0]);
        draw_editor(f, app, editor_ast[0]);
        draw_ast(f, app, editor_ast[1]);
    } else {
        draw_editor(f, app, right[0]);
    }
    draw_console(f, app, right[1]);
}

fn pane_block<'a>(title: &'a str, active: bool) -> Block<'a> {
    let mut b = Block::default().borders(Borders::ALL).title(title);
    if active {
        b = b.border_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
    }
    b
}

fn draw_explorer(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .files
        .iter()
        .map(|p| ListItem::new(if p.is_dir() { format!("{}/", display_name(p)) } else { display_name(p) }))
        .collect();
    let mut state = ListState::default();
    if !app.files.is_empty() {
        state.select(Some(app.file_sel));
    }
    let heading = format!("{} · {}", app.t("files"), app.explorer_dir.display());
    let list = List::new(items)
        .block(pane_block(&heading, app.pane == Pane::Explorer))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("› ");
    f.render_stateful_widget(list, area, &mut state);
}

fn draw_editor(f: &mut Frame, app: &App, area: Rect) {
    let title = if app.editor_path.is_some() {
        format!("{}  {}", app.t("editor"), app.tabs_title())
    } else {
        app.t("editor").to_string()
    };
    let mut shown = app.editor_buf.clone();
    if shown.is_char_boundary(app.editor_cursor) {
        shown.insert(app.editor_cursor, '▏');
    }
    // Без word-wrap курсор і прокрутка мають однозначні екранні координати.
    // Раніше довгі рядки (особливо з кирилицею) переносилися поза видимим
    // курсором: вертикальний scroll рахував лише '\n', а не переноси.
    let (vertical, horizontal) = editor_scroll_start(
        &app.editor_buf, app.editor_cursor, area.height, area.width,
    );
    let para = Paragraph::new(shown)
        .block(pane_block(&title, app.pane == Pane::Editor))
        .scroll((vertical, horizontal));
    f.render_widget(para, area);
}

/// Вікно редактора для довгих рядків, зокрема UTF-8 людської проєкції.
/// Термінальна ширина — в комірках, а не в байтах чи кількості Unicode scalar.
fn editor_scroll_start(text: &str, cursor: usize, height: u16, width: u16) -> (u16, u16) {
    let before = &text[..cursor];
    let line = before.bytes().filter(|byte| *byte == b'\n').count();
    let column = Line::from(before.rsplit('\n').next().unwrap_or("")).width();
    let inner_height = height.saturating_sub(2) as usize;
    let inner_width = width.saturating_sub(2) as usize;
    let vertical = line.saturating_sub(inner_height.saturating_sub(1));
    let horizontal = column.saturating_sub(inner_width.saturating_sub(1));
    (
        vertical.min(u16::MAX as usize) as u16,
        horizontal.min(u16::MAX as usize) as u16,
    )
}

/// У консолі scroll вимірюється фізичними рядками термінала, а не
/// повідомленнями stdout/stderr. Явне перенесення також не губить промпт,
/// коли одне повідомлення займає багато екранних рядків.
fn console_visual_rows(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![String::new()];
    }
    let mut rows = Vec::new();
    let mut row = String::new();
    let mut used = 0usize;
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        // Діагностика з внутрішнім LF/CRLF/CR — це кілька фізичних
        // рядків, а не один широкий рядок із невидимими control chars.
        // Обробляємо тільки presentation; оригінальний stdout не змінюємо.
        if ch == '\r' || ch == '\n' {
            if ch == '\r' && chars.peek() == Some(&'\n') {
                chars.next();
            }
            rows.push(std::mem::take(&mut row));
            used = 0;
            continue;
        }
        let char_width = Line::from(ch.to_string()).width();
        // Надвузький термінал (1 комірка): не віддавати Ratatui символ
        // ширший за панель. Заміна стосується лише показу, не SENS-виводу.
        let (shown, cells) = if char_width > width { ('�', 1) } else { (ch, char_width) };
        if used > 0 && used + cells > width {
            rows.push(std::mem::take(&mut row));
            used = 0;
        }
        row.push(shown);
        used += cells;
    }
    rows.push(row);
    rows
}

fn draw_ast(f: &mut Frame, app: &App, area: Rect) {
    let visible = area.height.saturating_sub(2) as usize;
    let start = app.ast_scroll.min(app.ast_lines.len().saturating_sub(visible));
    let lines = app.ast_lines.iter().skip(start).take(visible)
        .cloned().collect::<Vec<_>>().join("\n");
    f.render_widget(
        Paragraph::new(lines)
            .block(pane_block(app.t("ast"), false)),
        area,
    );
}

/// Курсор консольної команди — лише представлення; `App.input` без
/// додаткового символу й надсилається у справжній pinned SENS без змін.
/// UTF-8 offset відповідає App::move_input_cursor / bracketed paste.
fn repl_prompt(input: &str, cursor: usize, focused: bool) -> String {
    let mut shown = String::with_capacity(input.len() + 12);
    shown.push_str("SENS> ");
    shown.push_str(input);
    if focused && input.is_char_boundary(cursor) {
        shown.insert("SENS> ".len() + cursor, '▏');
    }
    shown
}

fn draw_console(f: &mut Frame, app: &App, area: Rect) {
    let inner_width = area.width.saturating_sub(2) as usize;
    let mut lines: Vec<Line> = Vec::new();
    for log in &app.console {
        let style = match log.origin {
            Origin::Stdout => Style::default(),
            Origin::Stderr => Style::default().fg(Color::Red),
            Origin::System => Style::default().fg(Color::DarkGray),
        };
        for visual in console_visual_rows(&log.text, inner_width) {
            lines.push(Line::styled(visual, style));
        }
    }
    let prompt = repl_prompt(&app.input, app.input_cursor, app.pane == Pane::Console);
    for visual in console_visual_rows(&prompt, inner_width) {
        lines.push(Line::styled(visual, Style::default().fg(Color::Yellow)));
    }

    // Після явного перенесення кожен елемент lines — один екранний рядок.
    // Останній результат і промпт видимі за back=0; PageUp переглядає всю
    // історію навіть за одного довгого stderr-повідомлення.
    let visible = area.height.saturating_sub(2) as usize;
    let scroll = console_scroll_start(lines.len(), visible, app.console_scroll_back);
    let para = Paragraph::new(lines)
        .block(pane_block(app.t("console"), app.pane == Pane::Console))
        .scroll((scroll, 0));
    f.render_widget(para, area);
}

fn draw_status(f: &mut Frame, app: &App, area: Rect) {
    let run = if app.engine_running { "RUN" } else { "idle" };
    let position = if app.editor_path.is_some() {
        let (line, column) = app.editor_position();
        format!(" | {}:{}", line, column)
    } else {
        String::new()
    };
    let text = if area.width < 76 {
        format!(" {} | {} | {}{} | Ctrl+S  F5  F6 AST  F9",
            app.pane_label(), app.language, run, position)
    } else {
        format!(
            " {} | {} | {}{} | :інтерфейс uk|de|en  :мова укр|ук|en|sa|core  Ctrl+S {}  F5 {}  F6 AST  F9 {} ",
            app.pane_label(), app.language, run, position, app.t("save"), app.t("run"), app.t("stop")
        )
    };
    let para = Paragraph::new(text).style(Style::default().add_modifier(Modifier::REVERSED));
    f.render_widget(para, area);
}

// Коли відступ дорівнює нулю, промпт і останній результат завжди видимі.
// Прокрутка не стирає stderr/stdout і не втручається у SENS-сесію.
fn console_scroll_start(lines: usize, visible: usize, back: usize) -> u16 {
    lines.saturating_sub(visible)
        .saturating_sub(back)
        .min(u16::MAX as usize) as u16
}

#[cfg(test)]
mod console_visual_tests {
    use super::{console_scroll_start, console_visual_rows, draw, repl_prompt};
    use crate::app::App;
    use crate::engine::{Line as EngineLine, Origin};
    use ratatui::{backend::TestBackend, Terminal};
    use std::path::PathBuf;

    #[test]
    fn wraps_long_ukrainian_diagnostics_without_splitting_utf8() {
        assert_eq!(console_visual_rows("їжак", 3), vec!["їжа", "к"]);
        assert_eq!(console_visual_rows("абвгд", 3), vec!["абв", "гд"]);
        assert_eq!(console_visual_rows("", 3), vec![""]);
        assert_eq!(console_visual_rows("мама\nїжак", 3), vec!["мам", "а", "їжа", "к"]);
        assert_eq!(console_visual_rows("а\r\nб\rв", 3), vec!["а", "б", "в"]);
        assert_eq!(console_visual_rows("а\n", 3), vec!["а", ""]);
        assert_eq!(console_visual_rows("а\n\nв", 3), vec!["а", "", "в"]);
        let rows = console_visual_rows(&"п".repeat(120), 10);
        assert_eq!(rows.len(), 12);
        assert_eq!(console_scroll_start(rows.len() + 1, 4, 0), 9);
        assert_eq!(console_scroll_start(rows.len() + 1, 4, 5), 4);
    }

    #[test]
    fn repl_cursor_renders_at_unicode_boundary_without_mutating_input() {
        let input = "атом? 'їжак";
        let mid = "атом? 'ї".len();
        assert_eq!(repl_prompt(input, mid, true), "SENS> атом? 'ї▏жак");
        assert_eq!(repl_prompt(input, 0, true), "SENS> ▏атом? 'їжак");
        assert_eq!(repl_prompt(input, input.len(), true), "SENS> атом? 'їжак▏");
        assert_eq!(repl_prompt(input, mid, false), "SENS> атом? 'їжак");
        // Невалідний byte offset від зовнішнього виклику не рве UTF-8.
        assert_eq!(repl_prompt(input, 1, true), "SENS> атом? 'їжак");
        assert_eq!(input, "атом? 'їжак");
    }

    #[test]
    fn console_displays_real_repl_cursor_in_the_middle_of_ukrainian_input() {
        let mut app = App::new_in(PathBuf::from("."));
        app.pane = crate::app::Pane::Console;
        app.console.clear();
        app.input = "атом? 'їжак".to_owned();
        app.input_cursor = "атом? 'ї".len();
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        terminal.draw(|frame| draw(frame, &app)).unwrap();
        let visible = terminal.backend().buffer().content().iter()
            .map(|cell| cell.symbol()).collect::<Vec<_>>().join("");
        assert!(visible.contains("SENS> атом? 'ї▏жак"), "UTF-8 REPL caret: {visible}");
        assert_eq!(app.input, "атом? 'їжак", "рендер не редагує SENS-команду");
    }

    #[test]
    fn prompt_remains_visible_below_multirow_stderr_in_narrow_terminal() {
        let mut app = App::new_in(PathBuf::from("."));
        app.console.clear();
        app.console.push(EngineLine { origin: Origin::Stderr, text: "помилка ".repeat(50) });
        let mut terminal = Terminal::new(TestBackend::new(40, 24)).unwrap();
        terminal.draw(|frame| draw(frame, &app)).unwrap();
        let screen = terminal.backend().buffer().content().iter()
            .map(|cell| cell.symbol()).collect::<Vec<_>>().join("");
        assert!(screen.contains("SENS>"), "видимий промпт після довгого stderr");
    }
}

#[cfg(test)]
mod console_scrolling_tests {
    use super::console_scroll_start;

    #[test]
    fn shows_latest_output_and_allows_reviewing_prior_lines() {
        assert_eq!(console_scroll_start(30, 8, 0), 22);
        assert_eq!(console_scroll_start(30, 8, 5), 17);
        assert_eq!(console_scroll_start(30, 8, 999), 0);
        assert_eq!(console_scroll_start(4, 8, 0), 0);
    }
}


#[cfg(test)]
mod editor_scrolling_tests {
    use super::editor_scroll_start;

    #[test]
    fn keeps_unicode_cursor_visible_without_byte_based_scrolling() {
        let text = "привіт"; // шість термінальних комірок, дванадцять байтів
        assert_eq!(editor_scroll_start(text, text.len(), 6, 6), (0, 3));
        assert_eq!(editor_scroll_start(text, "при".len(), 6, 6), (0, 0));
    }

    #[test]
    fn follows_cursor_through_lines_and_tiny_windows() {
        let text = "а\nб\nв\nг".replace("\n", "\n");
        assert_eq!(editor_scroll_start(&text, text.len(), 4, 8), (2, 0));
        assert_eq!(editor_scroll_start(&text, text.len(), 1, 1), (3, 1));
    }
}

#[cfg(test)]
mod adaptive_layout_tests {
    use super::draw;
    use crate::app::App;
    use ratatui::{backend::TestBackend, Terminal};
    use std::path::PathBuf;

    #[test]
    fn narrow_and_wide_terminals_render_all_three_panels() {
        let app = App::new_in(PathBuf::from("."));
        for (width, height) in [(40, 24), (100, 30)] {
            let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
            terminal.draw(|frame| draw(frame, &app)).unwrap();
            let output = terminal.backend().buffer().content().iter()
                .map(|cell| cell.symbol()).collect::<Vec<_>>().join("");
            assert!(output.contains("Файли"), "Explorer {width}x{height}");
            assert!(output.contains("Редактор"), "Editor {width}x{height}");
            assert!(output.contains("Консоль"), "Console {width}x{height}");
        }
    }
}
