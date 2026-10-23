//! Малювання панелей.
//!
//! Панелі ті самі, що в GUI: дерево проєкту | редактор | консоль/вивід.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
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
    if app.palette_visible {
        draw_palette(f, app);
    }
    if app.help_visible {
        draw_help(f, app);
    }
    if app.eco_visible {
        draw_eco(f, app);
    }
    if app.graph_visible {
        draw_graph(f, app);
    }
    if app.swarm_visible {
        draw_swarm(f, app);
    }
    if app.search_visible {
        draw_search(f, app);
    }
}

/// Прямокутники панелей тіла — ОДНА розкладка і для малювання, і для влучання
/// мишею: координати кліку не можуть розійтися з картинкою.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PaneRects {
    pub explorer: Rect,
    pub editor: Rect,
    pub ast: Option<Rect>,
    pub console: Rect,
}

pub fn pane_rects(area: Rect, ast_visible: bool) -> PaneRects {
    if area.width < 76 {
        // Вузький термінал: не стискати редактор до кількох символів.
        if ast_visible {
            let rows = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(5), Constraint::Percentage(45),
                    Constraint::Percentage(25), Constraint::Min(4)])
                .split(area);
            PaneRects { explorer: rows[0], editor: rows[1], ast: Some(rows[2]), console: rows[3] }
        } else {
            let rows = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(5), Constraint::Percentage(55), Constraint::Min(4)])
                .split(area);
            PaneRects { explorer: rows[0], editor: rows[1], ast: None, console: rows[2] }
        }
    } else {
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(28), Constraint::Min(1)])
            .split(area);
        let right = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(cols[1]);
        if ast_visible {
            let editor_ast = Layout::default().direction(Direction::Vertical)
                .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
                .split(right[0]);
            PaneRects { explorer: cols[0], editor: editor_ast[0], ast: Some(editor_ast[1]), console: right[1] }
        } else {
            PaneRects { explorer: cols[0], editor: right[0], ast: None, console: right[1] }
        }
    }
}

fn inside(rect: Rect, column: u16, row: u16) -> bool {
    column >= rect.x && column < rect.x + rect.width && row >= rect.y && row < rect.y + rect.height
}

/// Яка панель під точкою. Панель AST належить контексту редактора.
pub fn pane_at(rects: &PaneRects, column: u16, row: u16) -> Option<Pane> {
    if inside(rects.explorer, column, row) {
        return Some(Pane::Explorer);
    }
    if inside(rects.editor, column, row) {
        return Some(Pane::Editor);
    }
    if rects.ast.map(|ast| inside(ast, column, row)).unwrap_or(false) {
        return Some(Pane::Editor);
    }
    if inside(rects.console, column, row) {
        return Some(Pane::Console);
    }
    None
}

/// Індекс рядка дерева під точкою. ratatui List щоразу будується з нульовим
/// відступом і тримає вибір у видимій зоні, тож відступ = вибір + 1 − висота.
pub fn explorer_index_at(files: usize, selected: usize, rect: Rect, row: u16) -> Option<usize> {
    if row <= rect.y || row + 1 >= rect.y + rect.height {
        return None; // рамка
    }
    let inner_row = (row - rect.y - 1) as usize;
    let inner_height = rect.height.saturating_sub(2) as usize;
    let offset = selected.saturating_add(1).saturating_sub(inner_height);
    let index = offset + inner_row;
    (index < files).then_some(index)
}

fn draw_body(f: &mut Frame, app: &App, area: Rect) {
    let rects = pane_rects(area, app.ast_visible);
    draw_explorer(f, app, rects.explorer);
    draw_editor(f, app, rects.editor);
    if let Some(ast) = rects.ast {
        draw_ast(f, app, ast);
    }
    draw_console(f, app, rects.console);
}

/// Рамка панелі: округлі кути, акцент на активній панелі, неактивні — приглушені.
fn pane_block(title: String, active: bool, theme: crate::theme::Theme) -> Block<'static> {
    let (colour, modifier) = if active {
        (theme.accent(), Modifier::BOLD)
    } else {
        (theme.inactive(), Modifier::empty())
    };
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colour))
        .title(Span::styled(format!(" {title} "), Style::default().fg(colour).add_modifier(modifier)))
}

fn draw_explorer(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .files
        .iter()
        .enumerate()
        .map(|(index, p)| {
            let indent = "  ".repeat(app.depths.get(index).copied().unwrap_or(0));
            let name = display_name(p);
            let (text, style) = if p.is_dir() {
                let marker = if app.expanded.contains(p) {
                    "\u{25be}"
                } else {
                    "\u{25b8}"
                };
                (
                    format!("{indent}{marker} {name}/"),
                    Style::default().fg(app.theme.info()).add_modifier(Modifier::BOLD),
                )
            } else {
                (format!("{indent}   {name}"), Style::default())
            };
            ListItem::new(Line::styled(text, style))
        })
        .collect();
    let mut state = ListState::default();
    if !app.files.is_empty() {
        state.select(Some(app.file_sel));
    }
    let heading = format!("{} · {}", app.t("files"), app.explorer_dir.display());
    let list = List::new(items)
        .block(pane_block(heading, app.pane == Pane::Explorer, app.theme))
        .highlight_style(
            Style::default()
                .fg(app.theme.accent_fg())
                .bg(app.theme.accent())
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("› ");
    f.render_stateful_widget(list, area, &mut state);
}

fn draw_editor(f: &mut Frame, app: &App, area: Rect) {
    let title = if app.editor_path.is_some() {
        format!("{}  {}", app.t("editor"), app.tabs_title())
    } else {
        app.t("editor").to_string()
    };
    // Підсвічування — display-only підказка (як пошук): маркер курсора
    // малюється стилем теми, а не вставкою символу в текст.
    let shown = crate::highlight::editor_lines(&app.editor_buf, app.editor_cursor, app.theme);
    // Без word-wrap курсор і прокрутка мають однозначні екранні координати.
    // Раніше довгі рядки (особливо з кирилицею) переносилися поза видимим
    // курсором: вертикальний scroll рахував лише '\n', а не переноси.
    let (vertical, horizontal) = editor_scroll_start(
        &app.editor_buf, app.editor_cursor, area.height, area.width,
    );
    let para = Paragraph::new(shown)
        .block(pane_block(title, app.pane == Pane::Editor, app.theme))
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
    // Живий AST: рядок, чий вузол містить курсор редактора, підсвічується.
    let body: Vec<Line> = app
        .ast_lines
        .iter()
        .skip(start)
        .take(visible)
        .map(|line| {
            let here = line
                .span
                .map(|(begin, end)| begin <= app.editor_cursor && app.editor_cursor <= end)
                .unwrap_or(false);
            let style = if here {
                Style::default()
                    .fg(app.theme.accent_fg())
                    .bg(app.theme.accent())
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            Line::styled(line.text.clone(), style)
        })
        .collect();
    f.render_widget(
        Paragraph::new(body).block(pane_block(app.t("ast").to_string(), false, app.theme)),
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
            Origin::Stderr => Style::default().fg(app.theme.error()),
            Origin::System => Style::default().fg(app.theme.inactive()),
        };
        for visual in console_visual_rows(&log.text, inner_width) {
            lines.push(Line::styled(visual, style));
        }
    }
    let prompt = repl_prompt(&app.input, app.input_cursor, app.pane == Pane::Console);
    for visual in console_visual_rows(&prompt, inner_width) {
        lines.push(Line::styled(visual, Style::default().fg(app.theme.warn())));
    }

    // Після явного перенесення кожен елемент lines — один екранний рядок.
    // Останній результат і промпт видимі за back=0; PageUp переглядає всю
    // історію навіть за одного довгого stderr-повідомлення.
    let visible = area.height.saturating_sub(2) as usize;
    let scroll = console_scroll_start(lines.len(), visible, app.console_scroll_back);
    let para = Paragraph::new(lines)
        .block(pane_block(app.t("console").to_string(), app.pane == Pane::Console, app.theme))
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
    let provenance = match &app.pinned_sha {
        Some(sha) if sha.len() >= 8 => format!("SENS {} \u{2502} ", &sha[..8]),
        _ => String::new(),
    };
    let head = match app.cursor_head() {
        Some(head) => format!("\u{2316} {head} \u{2502} "),
        None => String::new(),
    };
    let text = if area.width < 76 {
        format!(" my-idea-tui \u{2502} {}{} \u{2502} {} \u{2502} {}{} \u{2502} Ctrl+S F5 F6 F9",
            provenance, app.pane_label(), app.language, run, position)
    } else {
        format!(
            " my-idea-tui \u{2502} {}{} \u{2502} {} \u{2502} {}{} \u{2502} {head}:інтерфейс uk|de|en  :мова укр|ук|en|sa|core  Ctrl+S {}  F5 {}  F6 AST  F9 {} ",
            provenance, app.pane_label(), app.language, run, position, app.t("save"), app.t("run"), app.t("stop")
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

/// Палітра команд (Ctrl+P): оверлей по центру з тими самими діями, що й клавіші.
fn draw_palette(f: &mut Frame, app: &App) {
    let items = app.palette_items();
    let area = f.area();
    let width = area.width.saturating_sub(4).min(64).max(20);
    let height = (items.len() as u16 + 4).min(area.height.saturating_sub(2)).max(4);
    let rect = Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    };
    // Рахуємо внутрішню область вручну (Block::inner у різних версіях бере self за значенням).
    let inner = Rect {
        x: rect.x + 1,
        y: rect.y + 1,
        width: rect.width.saturating_sub(2),
        height: rect.height.saturating_sub(2),
    };

    let title = format!("{} \u{b7} {}", app.t("palette"), app.t("palette-hint"));
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {title} "))
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(app.theme.accent()));
    f.render_widget(block, rect);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(inner);

    f.render_widget(
        Paragraph::new(Line::from(format!("> {}", app.palette_query))),
        rows[0],
    );

    if items.is_empty() {
        f.render_widget(Paragraph::new(Line::from(app.t("palette-empty"))), rows[1]);
        return;
    }

    let list_items: Vec<ListItem> = items
        .iter()
        .map(|id| ListItem::new(Line::from(app.t(*id))))
        .collect();
    let mut state = ListState::default();
    state.select(Some(app.palette_sel.min(items.len().saturating_sub(1))));
    let list = List::new(list_items)
        .highlight_style(
            Style::default()
                .fg(app.theme.accent_fg())
                .bg(app.theme.accent())
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");
    f.render_stateful_widget(list, rows[1], &mut state);
}

/// Довідка клавіш (F1): оверлей із тим самим набором дій, що й палітра.
fn draw_help(f: &mut Frame, app: &App) {
    let lines = app.help_lines();
    let area = f.area();
    let width = area.width.saturating_sub(4).min(64).max(24);
    let height = area
        .height
        .saturating_sub(2)
        .min(lines.len() as u16 + 4)
        .max(4);
    let rect = Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    };
    let inner = Rect {
        x: rect.x + 1,
        y: rect.y + 1,
        width: rect.width.saturating_sub(2),
        height: rect.height.saturating_sub(2),
    };
    let title = format!("{} \u{b7} {}", app.t("help"), app.t("help-hint"));
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {title} "))
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(app.theme.ok()));
    f.render_widget(block, rect);

    let body: Vec<Line> = lines
        .iter()
        .skip(app.help_scroll)
        .map(|s| Line::from(s.clone()))
        .collect();
    f.render_widget(Paragraph::new(body), inner);
}

/// Панель «Екосистема» (Ctrl+E): сусідні репозиторії та їхні repo.lisp.
fn draw_eco(f: &mut Frame, app: &App) {
    let lines = app.eco_lines();
    let area = f.area();
    let width = area.width.saturating_sub(4).min(76).max(28);
    let height = area
        .height
        .saturating_sub(2)
        .min(lines.len() as u16 + 4)
        .max(4);
    let rect = Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    };
    let inner = Rect {
        x: rect.x + 1,
        y: rect.y + 1,
        width: rect.width.saturating_sub(2),
        height: rect.height.saturating_sub(2),
    };
    let title = format!("{} \u{b7} {}", app.t("eco"), app.t("eco-hint"));
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {title} "))
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(app.theme.warn()));
    f.render_widget(block, rect);

    let items: Vec<ListItem> = lines
        .iter()
        .map(|s| ListItem::new(Line::from(s.clone())))
        .collect();
    let mut state = ListState::default();
    if !app.siblings.is_empty() {
        state.select(Some(app.eco_sel.min(lines.len().saturating_sub(1))));
    }
    let list = List::new(items)
        .highlight_style(
            Style::default()
                .fg(app.theme.accent_fg())
                .bg(app.theme.warn())
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");
    f.render_stateful_widget(list, inner, &mut state);
}

/// Граф знань (Ctrl+G): можливість, хто експортує, хто імпортує.
fn draw_graph(f: &mut Frame, app: &App) {
    let lines = app.graph_lines();
    let area = f.area();
    let width = area.width.saturating_sub(4).min(80).max(28);
    let height = area
        .height
        .saturating_sub(2)
        .min(lines.len() as u16 + 4)
        .max(4);
    let rect = Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    };
    let inner = Rect {
        x: rect.x + 1,
        y: rect.y + 1,
        width: rect.width.saturating_sub(2),
        height: rect.height.saturating_sub(2),
    };
    let title = format!("{} \u{b7} {}", app.t("graph"), app.t("graph-hint"));
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(format!(" {title} "))
        .border_style(Style::default().fg(app.theme.graph()));
    f.render_widget(block, rect);

    let body: Vec<Line> = lines
        .iter()
        .skip(app.graph_scroll)
        .map(|s| Line::from(s.clone()))
        .collect();
    f.render_widget(Paragraph::new(body), inner);
}

/// Спостереження swarm-node (Ctrl+R): сира відповідь `(status)` + розбір SENS.
fn draw_swarm(f: &mut Frame, app: &App) {
    let area = f.area();
    let width = area.width.saturating_sub(4).min(84).max(28);
    let height = area
        .height
        .saturating_sub(2)
        .min(app.swarm_lines.len() as u16 + 4)
        .max(4);
    let rect = Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    };
    let inner = Rect {
        x: rect.x + 1,
        y: rect.y + 1,
        width: rect.width.saturating_sub(2),
        height: rect.height.saturating_sub(2),
    };
    let title = format!("{} \u{b7} {}", app.t("swarm"), app.t("swarm-hint"));
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(format!(" {title} "))
        .border_style(Style::default().fg(app.theme.info()));
    f.render_widget(block, rect);

    let body: Vec<Line> = app
        .swarm_lines
        .iter()
        .skip(app.swarm_scroll)
        .map(|s| Line::from(s.clone()))
        .collect();
    f.render_widget(Paragraph::new(body), inner);
}

/// Пошук у редакторі (Ctrl+F): запит, лічильник збігів і рядки знайденого.
fn draw_search(f: &mut Frame, app: &App) {
    let lines = app.search_lines();
    let area = f.area();
    let width = area.width.saturating_sub(4).min(76).max(24);
    let height = area
        .height
        .saturating_sub(2)
        .min(lines.len() as u16 + 4)
        .max(4);
    let rect = Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    };
    let inner = Rect {
        x: rect.x + 1,
        y: rect.y + 1,
        width: rect.width.saturating_sub(2),
        height: rect.height.saturating_sub(2),
    };
    let title = format!(
        "{}: {} \u{b7} {} {} \u{b7} {}",
        app.t("find"),
        app.search_query,
        app.search_hits.len(),
        app.t("find-hits"),
        app.t("find-hint")
    );
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(format!(" {title} "))
        .border_style(Style::default().fg(app.theme.accent()));
    f.render_widget(block, rect);

    let body: Vec<Line> = lines.iter().map(|s| Line::from(s.clone())).collect();
    f.render_widget(Paragraph::new(body), inner);
}

#[cfg(test)]
mod mouse_hit_tests {
    use super::{explorer_index_at, pane_at, pane_rects};
    use crate::app::Pane;
    use ratatui::layout::Rect;

    #[test]
    fn clicks_land_on_the_pane_that_is_drawn_there() {
        let area = Rect::new(0, 0, 120, 40);
        let rects = pane_rects(area, true);
        assert_eq!(pane_at(&rects, rects.explorer.x + 1, rects.explorer.y + 1), Some(Pane::Explorer));
        assert_eq!(pane_at(&rects, rects.editor.x + 1, rects.editor.y + 1), Some(Pane::Editor));
        assert_eq!(pane_at(&rects, rects.console.x + 1, rects.console.y + 1), Some(Pane::Console));
        // Панель AST належить контексту редактора.
        let ast = rects.ast.expect("AST видимий");
        assert_eq!(pane_at(&rects, ast.x + 1, ast.y + 1), Some(Pane::Editor));
        // Поза панелями — нічого.
        assert_eq!(pane_at(&rects, 0, 0), Some(Pane::Explorer));
    }

    #[test]
    fn explorer_row_maps_to_index_and_respects_the_border() {
        let rect = Rect::new(0, 0, 28, 10);
        // Вибір на початку: відступ 0, рядок 1 усередині рамки → індекс 0.
        assert_eq!(explorer_index_at(20, 0, rect, 1), Some(0));
        assert_eq!(explorer_index_at(20, 0, rect, 4), Some(3));
        // Рамка й поза межами файлів — нічого.
        assert_eq!(explorer_index_at(20, 0, rect, 0), None);
        assert_eq!(explorer_index_at(2, 0, rect, 9), None);
    }

    #[test]
    fn explorer_click_accounts_for_the_list_scroll_offset() {
        let rect = Rect::new(0, 0, 28, 6); // внутрішня висота 4
        // Вибір 9 при висоті 4 → відступ 6; перший видимий рядок — це індекс 6.
        assert_eq!(explorer_index_at(50, 9, rect, 1), Some(6));
    }
}
