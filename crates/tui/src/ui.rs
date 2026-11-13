//! Drawing a frame: scene list, editor, status line and the help overlay.

use ratatui::layout::{Constraint, Layout, Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use crate::app::{App, Focus, Mode, Notice};
use crate::fountain::LineKind;
use crate::i18n::{help, tr, Text};

/// Share of the width the scene list takes: 1/4, but at least `LIST_MIN` columns.
const LIST_SHARE: u16 = 4;
const LIST_MIN: u16 = 24;
/// The mark behind the file name when the text is not saved.
const MODIFIED_MARK: &str = " [+]";
/// Gap between the pieces of the status line.
const GAP: &str = "  ";
/// Marks the scene the cursor is in, in the list.
const CURRENT_MARK: &str = "▶ ";
const OTHER_MARK: &str = "  ";

/// Style of a line kind: modifiers and the terminal's own colours only.
fn style_of(kind: LineKind) -> Style {
    match kind {
        LineKind::Heading => Style::new().add_modifier(Modifier::BOLD),
        LineKind::Character => Style::new().add_modifier(Modifier::BOLD),
        LineKind::Parenthetical => Style::new().add_modifier(Modifier::ITALIC),
        LineKind::Transition => Style::new().add_modifier(Modifier::DIM),
        LineKind::Centered => Style::new().add_modifier(Modifier::ITALIC),
        LineKind::Section => Style::new().fg(Color::Blue).add_modifier(Modifier::BOLD),
        LineKind::Synopsis | LineKind::Note => Style::new().add_modifier(Modifier::DIM),
        LineKind::PageBreak => Style::new().add_modifier(Modifier::DIM),
        LineKind::Lyrics => Style::new().add_modifier(Modifier::ITALIC),
        LineKind::Blank | LineKind::Action | LineKind::Dialogue => Style::new(),
    }
}

fn border_style(focused: bool) -> Style {
    if focused {
        return Style::new().add_modifier(Modifier::BOLD);
    }
    Style::new().add_modifier(Modifier::DIM)
}

/// Characters `start..end` of a line.
fn slice(line: &str, start: usize, end: usize) -> String {
    line.chars().skip(start).take(end - start).collect()
}

pub fn draw(frame: &mut Frame, app: &mut App) {
    let [main, status] =
        Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).areas(frame.area());
    let list_width = (main.width / LIST_SHARE).max(LIST_MIN).min(main.width / 2);
    let [list, editor] =
        Layout::horizontal([Constraint::Length(list_width), Constraint::Min(1)]).areas(main);

    draw_list(frame, app, list);
    draw_editor(frame, app, editor);
    draw_status(frame, app, status);
    if app.mode == Mode::Help {
        draw_help(frame, app);
    }
}

fn draw_list(frame: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == Focus::List;
    let block = Block::new()
        .borders(Borders::ALL)
        .border_style(border_style(focused))
        .title(tr(app.lang, Text::Scenes));

    if app.analysis.scenes.is_empty() {
        let none = Paragraph::new(tr(app.lang, Text::NoScenes)).block(block);
        frame.render_widget(none, area);
        return;
    }

    let current = app.analysis.scene_at(app.cursor().line);
    let items: Vec<ListItem> = app
        .analysis
        .scenes
        .iter()
        .enumerate()
        .map(|(i, scene)| {
            let number = scene.number.clone().unwrap_or_else(|| (i + 1).to_string());
            let mark = if current == Some(i) {
                CURRENT_MARK
            } else {
                OTHER_MARK
            };
            let style = if current == Some(i) {
                Style::new().add_modifier(Modifier::BOLD)
            } else {
                Style::new()
            };
            ListItem::new(format!("{mark}{number}  {}", scene.heading)).style(style)
        })
        .collect();

    let highlight = if focused {
        Style::new().add_modifier(Modifier::REVERSED)
    } else {
        Style::new()
    };
    let widget = List::new(items).block(block).highlight_style(highlight);
    let mut state = ListState::default().with_selected(Some(app.list_sel));
    frame.render_stateful_widget(widget, area, &mut state);
}

fn draw_editor(frame: &mut Frame, app: &mut App, area: Rect) {
    let focused = app.focus == Focus::Editor;
    let block = Block::new()
        .borders(Borders::ALL)
        .border_style(border_style(focused))
        .title(tr(app.lang, Text::Script));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    app.set_view(usize::from(inner.width), usize::from(inner.height));

    let lines = app.editor.lines();
    let rows: Vec<Line> = (app.scroll..app.scroll + usize::from(inner.height))
        .map_while(|i| app.layout.row(i))
        .map(|row| {
            let text = lines.get(row.line).map_or("", String::as_str);
            let style = style_of(app.analysis.kind(row.line));
            Line::from(vec![
                Span::raw(" ".repeat(row.indent)),
                Span::styled(slice(text, row.start, row.end), style),
            ])
        })
        .collect();
    frame.render_widget(Paragraph::new(rows), inner);

    if !focused || app.mode != Mode::Normal {
        return;
    }
    place_cursor(frame, app, inner);
}

/// Puts the terminal cursor on the text cursor, if it is in view.
fn place_cursor(frame: &mut Frame, app: &App, inner: Rect) {
    let cursor = app.cursor();
    let index = app.layout.row_of(cursor);
    let Some(row) = app.layout.row(index) else {
        return;
    };
    let Some(y) = index.checked_sub(app.scroll) else {
        return;
    };
    if y >= usize::from(inner.height) {
        return;
    }

    let x = row.indent + cursor.col - row.start;
    let (Ok(x), Ok(y)) = (u16::try_from(x), u16::try_from(y)) else {
        return;
    };
    frame.set_cursor_position(Position::new(inner.x + x, inner.y + y));
}

fn draw_status(frame: &mut Frame, app: &App, area: Rect) {
    if app.mode == Mode::ConfirmQuit {
        let ask = Paragraph::new(tr(app.lang, Text::ConfirmQuit))
            .style(Style::new().add_modifier(Modifier::REVERSED | Modifier::BOLD));
        frame.render_widget(ask, area);
        return;
    }

    let mark = if app.is_modified() { MODIFIED_MARK } else { "" };
    let cursor = app.cursor();
    let message = match &app.notice {
        Some(Notice::Saved) => tr(app.lang, Text::Saved).to_owned(),
        Some(Notice::SaveFailed(why)) => format!("{}: {why}", tr(app.lang, Text::SaveFailed)),
        None => tr(app.lang, Text::Hints).to_owned(),
    };
    let text = format!(
        " {}{mark}{GAP}{}:{}{GAP}{message}",
        app.file_name(),
        cursor.line + 1,
        cursor.col + 1
    );
    let bar = Paragraph::new(text).style(Style::new().add_modifier(Modifier::REVERSED));
    frame.render_widget(bar, area);
}

fn draw_help(frame: &mut Frame, app: &App) {
    let lines = help(app.lang);
    let width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0) + 4;
    let height = lines.len() + 2;
    let area = centered(frame.area(), width, height);

    let block = Block::new()
        .borders(Borders::ALL)
        .title(tr(app.lang, Text::HelpTitle));
    let body =
        Paragraph::new(lines.iter().map(|l| Line::from(*l)).collect::<Vec<_>>()).block(block);
    frame.render_widget(Clear, area);
    frame.render_widget(body, area);
}

/// A box of the size, centred in the area and cut to it.
fn centered(area: Rect, width: usize, height: usize) -> Rect {
    let width = u16::try_from(width).unwrap_or(u16::MAX).min(area.width);
    let height = u16::try_from(height).unwrap_or(u16::MAX).min(area.height);

    Rect {
        x: area.x + (area.width - width) / 2,
        y: area.y + (area.height - height) / 2,
        width,
        height,
    }
}
