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
        .map(|p| ListItem::new(display_name(p)))
        .collect();
    let mut state = ListState::default();
    if !app.files.is_empty() {
        state.select(Some(app.file_sel));
    }
    let list = List::new(items)
        .block(pane_block(app.t("files"), app.pane == Pane::Explorer))
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
    let prompt = format!("my-idea[{}]> {}", app.language, app.input);
    lines.push(Line::from(prompt).style(Style::default().fg(Color::Yellow)));

    let para = Paragraph::new(lines)
        .block(pane_block(app.t("console"), app.pane == Pane::Console))
        .wrap(Wrap { trim: false });
    f.render_widget(para, area);
}

fn draw_status(f: &mut Frame, app: &App, area: Rect) {
    let run = if app.engine_running { "RUN" } else { "idle" };
    let text = format!(
        " {} | {} | {} | :мова ук|de|en  Tab  F5 Run  F9 Stop  Ctrl-C Quit ",
        app.pane_label(),
        app.language,
        run
    );
    let para = Paragraph::new(text).style(Style::default().add_modifier(Modifier::REVERSED));
    f.render_widget(para, area);
}
