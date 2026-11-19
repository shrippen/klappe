//! Drawing a frame: header, scene list, editor, status line and the overlays, in Kante's colours.

use klappe_services::inline_runs;
use ratatui::layout::{Alignment, Constraint, Layout, Position, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph};
use ratatui::Frame;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::app::{App, Focus, Mode, Notice};
use crate::fountain::LineKind;
use crate::layout::{cols, Row, TAB_WIDTH};
use crate::theme;
use crate::ui_list::{draw_list, list_rows_area};
use crate::ui_overlay::{
    draw_backup, draw_book, draw_find, draw_numbers, draw_pdf, draw_suggest, draw_title,
};

/// Share of the width the scene list takes: 1/4, but at least `LIST_MIN` columns.
const LIST_SHARE: u16 = 4;
const LIST_MIN: u16 = 24;
/// Gap between the pieces of the status line.
const GAP: &str = "  ";
/// Marks the scene the cursor is in, in the list: a bar in the accent colour.
pub(crate) const CURRENT_MARK: &str = "▌ ";
/// Columns the text keeps free at the left and right edge of the editor.
const EDITOR_PAD: u16 = 1;
/// Columns between a row's text and its chips, and between two chips.
const CHIP_GAP: usize = 2;
/// Width of the search box and the matches it shows at most.
const GOTO_WIDTH: usize = 60;
const GOTO_ROWS: usize = 10;
/// The page break line is drawn with this character.
const PAGE_RULE: &str = "─";
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
        LineKind::Synopsis => Style::new().fg(theme::MUTED).add_modifier(Modifier::ITALIC),
        LineKind::Note => Style::new().fg(theme::NOTE).add_modifier(Modifier::ITALIC),
        LineKind::PageBreak | LineKind::Title => Style::new().fg(theme::MUTED),
        LineKind::Lyrics => Style::new()
            .fg(theme::LYRICS)
            .add_modifier(Modifier::ITALIC),
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

/// The style of a character: the line's own, then its inline marks (emphasis, a note, a
/// boneyard) and, while searching, the matches.
fn mark_style(base: Style, marks: klappe_services::Marks) -> Style {
    let mut style = base;
    if marks.bold {
        style = style.add_modifier(Modifier::BOLD);
    }
    if marks.italic {
        style = style.add_modifier(Modifier::ITALIC);
    }
    if marks.underline {
        style = style.add_modifier(Modifier::UNDERLINED);
    }
    if marks.note {
        style = style.fg(theme::NOTE).add_modifier(Modifier::ITALIC);
    }
    if marks.boneyard {
        style = style.fg(theme::MUTED).add_modifier(Modifier::DIM);
    }

    style
}

/// Characters `start..end` of a screen row as spans, styled per character.
fn row_spans(app: &App, row: Row, base: Style, end: usize) -> Vec<Span<'static>> {
    let text = app.editor.lines().get(row.line).map_or("", String::as_str);
    let open = app
        .analysis
        .boneyard
        .get(row.line)
        .copied()
        .unwrap_or(false);
    let (runs, _) = inline_runs(text, open);
    let hits = app.find_hits_on(row.line);
    let end = end.max(row.start);
    let tab = " ".repeat(TAB_WIDTH);

    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut last: Option<Style> = None;
    for (i, c) in text
        .chars()
        .enumerate()
        .skip(row.start)
        .take(end - row.start)
    {
        let marks = runs
            .iter()
            .find(|r| r.start <= i && i < r.end)
            .map(|r| r.marks)
            .unwrap_or_default();
        let mut style = mark_style(base, marks);
        for (cols, current) in &hits {
            if cols.contains(&i) {
                style = style.add_modifier(match current {
                    true => Modifier::REVERSED,
                    false => Modifier::UNDERLINED,
                });
            }
        }

        let piece = match c {
            '\t' => tab.clone(),
            _ => c.to_string(),
        };
        match (spans.last_mut(), last == Some(style)) {
            (Some(span), true) => span.content.to_mut().push_str(&piece),
            _ => spans.push(Span::styled(piece, style)),
        }
        last = Some(style);
    }

    spans
}

pub fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    app.set_screen(area.width);
    app.clamp_help(usize::from(area.height));
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
        app.set_list_area(list_rows_area(list));
        draw_list(frame, app, list);
        draw_editor(frame, app, editor);
    } else {
        app.set_list_area(Rect::default());
        draw_editor(frame, app, main);
    }
    draw_status(frame, app, status);

    match app.mode {
        Mode::Help => draw_help(frame, app),
        Mode::Goto => draw_goto(frame, app),
        Mode::Find => draw_find(frame, app),
        Mode::Title => draw_title(frame, app),
        Mode::Numbers => draw_numbers(frame, app),
        Mode::Book => draw_book(frame, app),
        Mode::Backup => draw_backup(frame, app),
        Mode::Pdf => draw_pdf(frame, app),
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

    let shown = focused && app.mode == Mode::Normal;
    if !shown && !(focused && app.mode == Mode::Find) {
        return;
    }
    let Some(cell) = cursor_cell(app, inner) else {
        return;
    };
    if shown {
        frame.set_cursor_position(cell);
        draw_suggest(frame, app, cell);
    }
}

/// One screen row of text; scene headings get a band and the number at the right, other
/// lines the chips of their links.
fn draw_row(frame: &mut Frame, app: &App, row: Row, band: Rect, rect: Rect) {
    if let Some(page) = row.page {
        draw_page_break(frame, app, page, rect);
        return;
    }

    let text = app.editor.lines().get(row.line).map_or("", String::as_str);
    let kind = app.analysis.kind(row.line);
    let style = style_of(kind);

    if kind != LineKind::Heading {
        let mut spans = vec![Span::raw(" ".repeat(row.indent))];
        spans.extend(row_spans(
            app,
            row,
            dual_style(style, app, row.line),
            row.end,
        ));
        frame.render_widget(Paragraph::new(Line::from(spans)), rect);
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
    let shown_cols = cols(slice(text, row.start, end.max(row.start)).chars());
    frame.render_widget(
        Paragraph::new(Line::from(row_spans(app, row, style, end))),
        rect,
    );

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

/// The thin line between two pages: `─ Page 3 ───────`, in the quiet colour; italic while it
/// belongs to an older text.
fn draw_page_break(frame: &mut Frame, app: &App, page: usize, rect: Rect) {
    let mut label = format!("{PAGE_RULE} {} ", app.texts.page(page));
    if app.pages_stale() {
        label.push_str(&format!("({}) ", app.texts.stale));
    }

    let fill = usize::from(rect.width).saturating_sub(cols(label.chars()));
    let line = format!("{label}{}", PAGE_RULE.repeat(fill));
    let mut style = Style::new().fg(theme::MUTED);
    if app.pages_stale() {
        style = style.add_modifier(Modifier::ITALIC);
    }

    frame.render_widget(Paragraph::new(line).style(style), rect);
}

/// The cue and the speech of a dual dialogue in their own colour.
fn dual_style(style: Style, app: &App, line: usize) -> Style {
    if app.analysis.is_dual(line) {
        return style.fg(theme::DUAL);
    }

    style
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

/// The screen cell of the text cursor, if it is in view.
fn cursor_cell(app: &App, inner: Rect) -> Option<Position> {
    let cursor = app.cursor();
    let index = app.layout.row_of(cursor);
    let row = app.layout.row(index)?;
    let y = index.checked_sub(app.scroll)?;
    if y >= usize::from(inner.height) {
        return None;
    }

    let text = app.editor.lines().get(row.line).map_or("", String::as_str);
    let before = text.chars().skip(row.start).take(cursor.col - row.start);
    let x = row.indent + cols(before);
    let (Ok(x), Ok(y)) = (u16::try_from(x), u16::try_from(y)) else {
        return None;
    };

    Some(Position::new(inner.x + x, inner.y + y))
}

/// What the status line's mode badge says.
fn mode_label(app: &App) -> &str {
    match (app.mode, app.focus) {
        (Mode::Help, _) => &app.texts.mode_help,
        (Mode::Goto, _) => &app.texts.mode_goto,
        (Mode::Find, _) => &app.texts.mode_find,
        (Mode::Title, _) => &app.texts.mode_title,
        (Mode::Numbers, _) => &app.texts.mode_numbers,
        (Mode::Book, _) => &app.texts.mode_book,
        (Mode::Backup, _) => &app.texts.mode_backup,
        (Mode::Pdf, _) => &app.texts.mode_pdf,
        (_, Focus::List) => &app.texts.mode_list,
        (_, Focus::Editor) => &app.texts.mode_text,
    }
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

    let mode = mode_label(app);
    let mode_style = Style::new()
        .fg(theme::GROUND)
        .bg(theme::ACCENT)
        .add_modifier(Modifier::BOLD);
    let chip = Span::styled(format!(" {mode} "), mode_style);

    let message = match &app.notice {
        Some(Notice::Saved) => app.texts.saved.clone(),
        Some(Notice::SaveFailed(why)) => format!("{}: {why}", app.texts.save_failed),
        Some(Notice::Replaced(count)) => app.texts.replaced(*count),
        Some(Notice::Loaded(text)) => text.clone(),
        Some(Notice::SaveFirst) => app.texts.book_save_first.clone(),
        Some(Notice::Pdf(text)) => text.clone(),
        None => app.texts.hints.clone(),
    };
    let message = Span::styled(message, Style::new().fg(theme::MUTED));

    // Without a notice the hints simply follow, cut at the end of the line.
    if app.notice.is_none() {
        let mut line = status_left(app, chip, StatusNote::Show, app.file_name());
        line.push_span(message);
        frame.render_widget(Paragraph::new(line).style(bar), area);
        return;
    }

    // The mode word and the notice stay. The book note gives way first, then the file name,
    // then the rest of the left part. At least one space separates the two parts.
    let width = usize::from(area.width);
    let notice_width = message.width().min(width.saturating_sub(chip.width() + 1));
    let room = width.saturating_sub(notice_width + 1);
    let mut left = status_left(app, chip.clone(), StatusNote::Show, app.file_name());
    if left.width() > room {
        left = status_left(app, chip.clone(), StatusNote::Hide, app.file_name());
    }
    let excess = left.width().saturating_sub(room);
    if excess > 0 {
        let name = shorten(app.file_name(), excess);
        left = status_left(app, chip, StatusNote::Hide, &name);
    }
    let notice_at = left.width().min(room + 1);
    let [left_area, _] = Layout::horizontal([
        Constraint::Length(notice_at.min(room) as u16),
        Constraint::Min(0),
    ])
    .areas(area);
    let [_, notice_area] =
        Layout::horizontal([Constraint::Length(notice_at as u16), Constraint::Min(0)]).areas(area);
    frame.render_widget(Paragraph::new(left).style(bar), left_area);
    frame.render_widget(Paragraph::new(message).style(bar), notice_area);
}

#[derive(Clone, Copy, PartialEq)]
enum StatusNote {
    Show,
    Hide,
}

/// The left part of the status line: mode word, file name, scene, book note, cursor.
fn status_left<'a>(app: &App, chip: Span<'a>, note: StatusNote, name: &str) -> Line<'a> {
    let cursor = app.cursor();
    let mut spans = vec![chip, Span::raw(format!(" {name}"))];
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
    if note == StatusNote::Show && !app.book_note.is_empty() {
        spans.push(Span::styled(
            format!("{GAP}{}", app.book_note),
            Style::new().fg(theme::ACCENT),
        ));
    }
    spans.push(Span::raw(format!(
        "{GAP}{}:{}{GAP}",
        cursor.line + 1,
        cursor.col + 1
    )));

    Line::from(spans)
}

/// `text` without its last `cut` columns (at least one character stays), closed by an ellipsis.
fn shorten(text: &str, cut: usize) -> String {
    let keep = text.width().saturating_sub(cut + 1);
    let mut head = String::new();
    let mut used = 0;
    for ch in text.chars() {
        let next = used + ch.width().unwrap_or(0);
        if next > keep && !head.is_empty() {
            break;
        }
        head.push(ch);
        used = next;
    }

    format!("{head}…")
}

/// Rows of the help box: every line, or as many as the screen has.
pub(crate) fn help_height(lines: usize, rows: usize) -> usize {
    (lines + 2).min(rows)
}

fn draw_help(frame: &mut Frame, app: &App) {
    let lines = &app.texts.help;
    let width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0) + 4;
    let height = help_height(lines.len(), usize::from(frame.area().height));
    let area = centered(frame.area(), width, height);

    // A screen too low for every line scrolls.
    let room = height.saturating_sub(2);
    let first = app.help_scroll.min(lines.len().saturating_sub(room));
    let block = Block::new()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(theme::ACCENT))
        .title(app.texts.help_title.as_str())
        .style(Style::new().fg(theme::TEXT).bg(theme::SURFACE));
    let body = Paragraph::new(
        lines
            .iter()
            .skip(first)
            .take(room)
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

/// The text cut to `room` columns.
pub(crate) fn fit(text: &str, room: usize) -> String {
    let mut used = 0;

    text.chars()
        .take_while(|&c| {
            used += cols([c]);
            used <= room
        })
        .collect()
}

/// A box of the size, centred in the area and cut to it.
pub(crate) fn centered(area: Rect, width: usize, height: usize) -> Rect {
    let width = u16::try_from(width).unwrap_or(u16::MAX).min(area.width);
    let height = u16::try_from(height).unwrap_or(u16::MAX).min(area.height);

    Rect {
        x: area.x + (area.width - width) / 2,
        y: area.y + (area.height - height) / 2,
        width,
        height,
    }
}
