//! Малювання панелей.
//!
//! Панелі ті самі, що в GUI: дерево проєкту | редактор | консоль/вивід.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
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
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(5), Constraint::Percentage(55), Constraint::Min(4)])
            .split(area);
        draw_explorer(f, app, rows[0]);
        draw_editor(f, app, rows[1]);
        draw_console(f, app, rows[2]);
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
    draw_editor(f, app, right[0]);
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
    let title = match &app.editor_path {
        Some(p) => {
            let star = if app.dirty { " •" } else { "" };
            format!("{}{star}  {}", app.t("editor"), p.display())
        }
        None => app.t("editor").to_string(),
    };
    let mut shown = app.editor_buf.clone();
    if shown.is_char_boundary(app.editor_cursor) {
        shown.insert(app.editor_cursor, '▏');
    }
    let cursor_line = app.editor_buf[..app.editor_cursor].bytes().filter(|ch| *ch == b'\n').count();
    let scroll = cursor_line.saturating_sub(area.height.saturating_sub(3) as usize).min(u16::MAX as usize) as u16;
    let para = Paragraph::new(shown)
        .block(pane_block(&title, app.pane == Pane::Editor))
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));
    f.render_widget(para, area);
}

fn draw_console(f: &mut Frame, app: &App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();
    for l in &app.console {
        let styled = match l.origin {
            Origin::Stdout => Line::from(l.text.clone()),
            Origin::Stderr => Line::from(l.text.clone()).style(Style::default().fg(Color::Red)),
            Origin::System => Line::from(l.text.clone()).style(Style::default().fg(Color::DarkGray)),
        };
        lines.push(styled);
    }
    let prompt = format!("SENS> {}", app.input);
    lines.push(Line::from(prompt).style(Style::default().fg(Color::Yellow)));

    // Ratatui за замовчуванням показує перші рядки Paragraph; для REPL
    // найважливіші останні stdout/stderr і поточний промпт.
    let visible = area.height.saturating_sub(2) as usize; // рамка панелі
    let scroll = console_scroll_start(lines.len(), visible, app.console_scroll_back);
    let para = Paragraph::new(lines)
        .block(pane_block(app.t("console"), app.pane == Pane::Console))
        .wrap(Wrap { trim: false })
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
        format!(" {} | {} | {}{} | Ctrl+S  F5  F9",
            app.pane_label(), app.language, run, position)
    } else {
        format!(
            " {} | {} | {}{} | :інтерфейс uk|de|en  :мова укр|ук|en|sa|core  Ctrl+S Save  F5 Run  F9 Stop ",
            app.pane_label(), app.language, run, position
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
