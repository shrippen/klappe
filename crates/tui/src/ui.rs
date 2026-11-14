//! Drawing a frame: header, scene list, editor, status line and the overlays, in Kante's colours.

use ratatui::layout::{Alignment, Constraint, Layout, Position, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use crate::app::{App, Focus, Mode, Notice};
use crate::fountain::LineKind;
use crate::layout::{cols, Row, TAB_WIDTH};
use crate::theme;

/// Share of the width the scene list takes: 1/4, but at least `LIST_MIN` columns.
const LIST_SHARE: u16 = 4;
const LIST_MIN: u16 = 24;
/// Gap between the pieces of the status line.
const GAP: &str = "  ";
/// Marks the scene the cursor is in, in the list: a bar in the accent colour.
const CURRENT_MARK: &str = "▌ ";
const OTHER_MARK: &str = "  ";
/// Columns the text keeps free at the left and right edge of the editor.
const EDITOR_PAD: u16 = 1;
/// Columns between a row's text and its chips, and between two chips.
const CHIP_GAP: usize = 2;
/// Width of the search box and the matches it shows at most.
const GOTO_WIDTH: usize = 60;
const GOTO_ROWS: usize = 10;
/// Prompt of the search box.
const GOTO_PROMPT: &str = "› ";

/// Style of a line kind, in Kante's colours.
fn style_of(kind: LineKind) -> Style {
    let base = Style::new().fg(theme::TEXT);
    match kind {
        LineKind::Heading => Style::new()
            .fg(theme::ACCENT)
            .bg(theme::SURFACE)
            .add_modifier(Modifier::BOLD),
        LineKind::Character => Style::new().fg(theme::STRONG).add_modifier(Modifier::BOLD),
        LineKind::Parenthetical => Style::new().fg(theme::SOFT).add_modifier(Modifier::ITALIC),
        LineKind::Transition => Style::new().fg(theme::MUTED),
        LineKind::Centered => base.add_modifier(Modifier::ITALIC),
        LineKind::Section => Style::new().fg(theme::BLUE).add_modifier(Modifier::BOLD),
        LineKind::Synopsis | LineKind::Note => {
            Style::new().fg(theme::MUTED).add_modifier(Modifier::ITALIC)
        }
        LineKind::PageBreak => Style::new().fg(theme::MUTED),
        LineKind::Lyrics => base.add_modifier(Modifier::ITALIC),
        LineKind::Blank | LineKind::Action | LineKind::Dialogue => base,
    }
}

/// Characters `start..end` of a line, tabs as spaces.
fn slice(line: &str, start: usize, end: usize) -> String {
    let tab = " ".repeat(TAB_WIDTH);

    line.chars()
        .skip(start)
        .take(end - start)
        .map(|c| {
            if c == '\t' {
                tab.clone()
            } else {
                c.to_string()
            }
        })
        .collect()
}

pub fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    app.set_screen(area.width);
    frame.render_widget(
        Block::new().style(Style::new().fg(theme::TEXT).bg(theme::GROUND)),
        area,
    );

    let [header, main, status] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(1),
    ])
    .areas(area);

    draw_header(frame, app, header);
    if app.list_shown() {
        let list_width = (main.width / LIST_SHARE).max(LIST_MIN).min(main.width / 2);
        let [list, editor] =
            Layout::horizontal([Constraint::Length(list_width), Constraint::Min(1)]).areas(main);
        draw_list(frame, app, list);
        draw_editor(frame, app, editor);
    } else {
        draw_editor(frame, app, main);
    }
    draw_status(frame, app, status);

    match app.mode {
        Mode::Help => draw_help(frame, app),
        Mode::Goto => draw_goto(frame, app),
        Mode::Normal | Mode::ConfirmQuit => {}
    }
}

/// File, the scene the cursor is in and its counts.
fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
    let bar = Style::new().fg(theme::MUTED).bg(theme::SURFACE);
    frame.render_widget(Block::new().style(bar), area);

    let mut left = vec![Span::raw(format!(" {}", app.file_name()))];
    let mut right = String::new();
    let current = app.analysis.scene_at(app.cursor().line);
    if let Some((i, scene)) = current.and_then(|i| Some((i, app.analysis.scenes.get(i)?))) {
        let accent = Style::new()
            .fg(theme::ACCENT)
            .bg(theme::SURFACE)
            .add_modifier(Modifier::BOLD);
        left.push(Span::raw(GAP));
        left.push(Span::styled(
            format!("{}  {}", scene.display_number(i), scene.heading),
            accent,
        ));

        let facts = scene.number.as_deref().and_then(|n| app.links.facts(n));
        if let Some(facts) = facts {
            right = format!(
                "{}{GAP}{} ",
                app.texts.shots(facts.shots),
                app.texts.figures(facts.figures)
            );
        }
    }

    frame.render_widget(Paragraph::new(Line::from(left)).style(bar), area);
    frame.render_widget(
        Paragraph::new(right).alignment(Alignment::Right).style(bar),
        area,
    );
}

fn draw_list(frame: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == Focus::List;
    let block = Block::new()
        .borders(Borders::RIGHT)
        .border_style(Style::new().fg(theme::RAISED));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let [title, rows] = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(inner);
    let heading = Style::new().fg(theme::MUTED).add_modifier(Modifier::BOLD);
    frame.render_widget(
        Paragraph::new(format!(" {}", app.texts.scenes)).style(heading),
        title,
    );

    if app.analysis.scenes.is_empty() {
        let none =
            Paragraph::new(app.texts.no_scenes.as_str()).style(Style::new().fg(theme::MUTED));
        frame.render_widget(none, rows);
        return;
    }

    let current = app.analysis.scene_at(app.cursor().line);
    let items: Vec<ListItem> = app
        .analysis
        .scenes
        .iter()
        .enumerate()
        .map(|(i, scene)| {
            let number = scene.display_number(i);
            if current == Some(i) {
                return ListItem::new(Line::from(vec![
                    Span::styled(CURRENT_MARK, Style::new().fg(theme::ACCENT)),
                    Span::styled(
                        format!("{number}  {}", scene.heading),
                        Style::new().fg(theme::STRONG).add_modifier(Modifier::BOLD),
                    ),
                ]));
            }
            ListItem::new(Line::from(vec![
                Span::raw(OTHER_MARK),
                Span::styled(format!("{number}  "), Style::new().fg(theme::MUTED)),
                Span::styled(scene.heading.as_str(), Style::new().fg(theme::SOFT)),
            ]))
        })
        .collect();

    let highlight = if focused {
        Style::new().bg(theme::RAISED)
    } else {
        Style::new()
    };
    let widget = List::new(items).highlight_style(highlight);
    let mut state = ListState::default().with_selected(Some(app.list_sel));
    frame.render_stateful_widget(widget, rows, &mut state);
}

fn draw_editor(frame: &mut Frame, app: &mut App, area: Rect) {
    let focused = app.focus == Focus::Editor;
    let inner = Rect {
        x: area.x + EDITOR_PAD,
        width: area.width.saturating_sub(2 * EDITOR_PAD),
        ..area
    };
    app.set_view(usize::from(inner.width), usize::from(inner.height));

    for y in 0..inner.height {
        let index = app.scroll + usize::from(y);
        let Some(row) = app.layout.row(index) else {
            break;
        };
        let band = Rect {
            y: area.y + y,
            height: 1,
            ..area
        };
        let rect = Rect {
            y: inner.y + y,
            height: 1,
            ..inner
        };
        draw_row(frame, app, *row, band, rect);
    }

    if !focused || app.mode != Mode::Normal {
        return;
    }
    place_cursor(frame, app, inner);
}

/// One screen row of text; scene headings get a band and the number at the right, other
/// lines the chips of their links.
fn draw_row(frame: &mut Frame, app: &App, row: Row, band: Rect, rect: Rect) {
    let text = app.editor.lines().get(row.line).map_or("", String::as_str);
    let kind = app.analysis.kind(row.line);
    let style = style_of(kind);

    if kind != LineKind::Heading {
        let line = Line::from(vec![
            Span::raw(" ".repeat(row.indent)),
            Span::styled(slice(text, row.start, row.end), style),
        ]);
        frame.render_widget(Paragraph::new(line), rect);
        draw_chips(frame, app, row, rect);
        return;
    }

    frame.render_widget(Block::new().style(Style::new().bg(theme::SURFACE)), band);

    // The line being edited shows its `#12#`; the others show the number at the right.
    let number = app
        .analysis
        .scenes
        .iter()
        .find(|s| s.line == row.line)
        .and_then(|s| s.number.as_deref())
        .filter(|_| row.line != app.cursor().line);
    let end = number.map_or(row.end, |n| row.end.min(shown_chars(text, n)));
    let shown = slice(text, row.start, end.max(row.start));
    let shown_cols = cols(shown.chars());
    frame.render_widget(Paragraph::new(Span::styled(shown, style)), rect);

    let Some(number) = number.filter(|_| row.last) else {
        return;
    };
    if shown_cols + number.chars().count() + CHIP_GAP > usize::from(rect.width) {
        return;
    }
    // Only the number's own cells get the quiet style, not the heading text before it.
    let width = u16::try_from(number.chars().count()).unwrap_or(u16::MAX);
    let at = Rect {
        x: rect.right().saturating_sub(width),
        width,
        ..rect
    };
    let quiet = Style::new().fg(theme::MUTED).bg(theme::SURFACE);
    frame.render_widget(Paragraph::new(number).style(quiet), at);
}

/// Characters of a heading line left of its `#number#` and the space before it.
fn shown_chars(text: &str, number: &str) -> usize {
    let marker = format!("#{number}#");
    let Some(before) = text.trim_end().strip_suffix(&marker) else {
        return text.chars().count();
    };

    before.trim_end().chars().count()
}

/// The shots and marks linked to the element starting on this row, at its right edge, if
/// the text leaves room.
fn draw_chips(frame: &mut Frame, app: &App, row: Row, rect: Rect) {
    let Some(pos) = app.analysis.element_at(row.line).filter(|_| row.start == 0) else {
        return;
    };
    let Some(scene) = pos.scene.as_deref() else {
        return;
    };
    let chips = app.links.chips(scene, pos.index);
    if chips.is_empty() {
        return;
    }

    let text = app.editor.lines().get(row.line).map_or("", String::as_str);
    let text_end = row.indent + cols(slice(text, row.start, row.end).chars());
    let width: usize =
        chips.iter().map(|c| c.text.chars().count()).sum::<usize>() + CHIP_GAP * (chips.len() - 1);
    if text_end + CHIP_GAP + width > usize::from(rect.width) {
        return;
    }

    let mut spans = Vec::new();
    for (i, chip) in chips.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw(" ".repeat(CHIP_GAP)));
        }
        spans.push(Span::styled(
            chip.text.as_str(),
            Style::new().fg(theme::label(&chip.label)),
        ));
    }
    frame.render_widget(
        Paragraph::new(Line::from(spans)).alignment(Alignment::Right),
        rect,
    );
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

    let text = app.editor.lines().get(row.line).map_or("", String::as_str);
    let before = text.chars().skip(row.start).take(cursor.col - row.start);
    let x = row.indent + cols(before);
    let (Ok(x), Ok(y)) = (u16::try_from(x), u16::try_from(y)) else {
        return;
    };
    frame.set_cursor_position(Position::new(inner.x + x, inner.y + y));
}

fn draw_status(frame: &mut Frame, app: &App, area: Rect) {
    let bar = Style::new().fg(theme::SOFT).bg(theme::SURFACE);
    if app.mode == Mode::ConfirmQuit {
        let ask = Paragraph::new(app.texts.confirm_quit.as_str()).style(
            Style::new()
                .fg(theme::GROUND)
                .bg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        );
        frame.render_widget(ask, area);
        return;
    }
    frame.render_widget(Block::new().style(bar), area);

    let mode = match (app.mode, app.focus) {
        (Mode::Help, _) => &app.texts.mode_help,
        (Mode::Goto, _) => &app.texts.mode_goto,
        (_, Focus::List) => &app.texts.mode_list,
        (_, Focus::Editor) => &app.texts.mode_text,
    };
    let mode_style = Style::new()
        .fg(theme::GROUND)
        .bg(theme::ACCENT)
        .add_modifier(Modifier::BOLD);

    let cursor = app.cursor();
    let message = match &app.notice {
        Some(Notice::Saved) => app.texts.saved.clone(),
        Some(Notice::SaveFailed(why)) => format!("{}: {why}", app.texts.save_failed),
        None => app.texts.hints.clone(),
    };
    let mut spans = vec![
        Span::styled(format!(" {mode} "), mode_style),
        Span::raw(format!(" {}", app.file_name())),
    ];
    if app.is_modified() {
        spans.push(Span::styled(
            format!("{GAP}{}", app.texts.modified),
            Style::new().fg(theme::ACCENT),
        ));
    }
    if let Some(i) = app.analysis.scene_at(cursor.line) {
        let total = app.analysis.scenes.len();
        spans.push(Span::raw(format!(
            "{GAP}{}",
            app.texts.scene_of(i + 1, total)
        )));
    }
    spans.push(Span::raw(format!(
        "{GAP}{}:{}{GAP}",
        cursor.line + 1,
        cursor.col + 1
    )));
    spans.push(Span::styled(message, Style::new().fg(theme::MUTED)));

    frame.render_widget(Paragraph::new(Line::from(spans)).style(bar), area);
}

fn draw_help(frame: &mut Frame, app: &App) {
    let lines = &app.texts.help;
    let width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0) + 4;
    let height = lines.len() + 2;
    let area = centered(frame.area(), width, height);

    let block = Block::new()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(theme::ACCENT))
        .title(app.texts.help_title.as_str())
        .style(Style::new().fg(theme::TEXT).bg(theme::SURFACE));
    let body = Paragraph::new(
        lines
            .iter()
            .map(|l| Line::from(l.as_str()))
            .collect::<Vec<_>>(),
    )
    .block(block);
    frame.render_widget(Clear, area);
    frame.render_widget(body, area);
}

/// The search box of `Ctrl+P`: the query and the scenes that match it.
fn draw_goto(frame: &mut Frame, app: &App) {
    let matches = app.goto_matches();
    let shown = matches.len().clamp(1, GOTO_ROWS);
    let area = centered(frame.area(), GOTO_WIDTH, shown + 3);

    let block = Block::new()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(theme::ACCENT))
        .title(app.texts.goto_title.as_str())
        .style(Style::new().fg(theme::TEXT).bg(theme::SURFACE));
    let inner = block.inner(area);
    frame.render_widget(Clear, area);
    frame.render_widget(block, area);

    let [prompt, rows] = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(inner);
    let typed = format!("{GOTO_PROMPT}{}", app.goto_query);
    frame.render_widget(
        Paragraph::new(typed.as_str()).style(Style::new().fg(theme::STRONG)),
        prompt,
    );
    let typed_cols = u16::try_from(cols(typed.chars())).unwrap_or(u16::MAX);
    let at_x = prompt
        .x
        .saturating_add(typed_cols)
        .min(prompt.right().saturating_sub(1));
    frame.set_cursor_position(Position::new(at_x, prompt.y));

    if matches.is_empty() {
        let none =
            Paragraph::new(app.texts.goto_none.as_str()).style(Style::new().fg(theme::MUTED));
        frame.render_widget(none, rows);
        return;
    }

    let items: Vec<ListItem> = matches
        .iter()
        .map(|&i| {
            let scene = &app.analysis.scenes[i];
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{}  ", scene.display_number(i)),
                    Style::new().fg(theme::MUTED),
                ),
                Span::styled(scene.heading.as_str(), Style::new().fg(theme::TEXT)),
            ]))
        })
        .collect();
    let widget = List::new(items)
        .highlight_symbol(CURRENT_MARK)
        .highlight_style(Style::new().fg(theme::ACCENT).bg(theme::RAISED));
    let mut state = ListState::default().with_selected(Some(app.goto_sel));
    frame.render_stateful_widget(widget, rows, &mut state);
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
