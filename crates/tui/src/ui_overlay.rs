//! The boxes over the editor: the suggestion list, search and replace, the title page form
//! the open questions about the book, the PDF options and the question about a backup.

use klappe_services::{Case, Choice, SceneNumbering};
use ratatui::layout::{Position, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use crate::app::{App, Backup, FindField, PdfRow, PDF_ROWS};
use crate::layout::cols;
use crate::theme;
use crate::ui::centered;

/// Suggestions shown at most.
const SUGGEST_ROWS: usize = 6;
/// Width of the search and title boxes.
const BOX_WIDTH: usize = 72;
/// Rows of the title box besides its fields: two borders, a blank row and the hint.
const TITLE_CHROME: usize = 4;
/// Columns the label of a field takes in a form.
const LABEL_WIDTH: usize = 18;
/// Rows of the numbering box: two fields, a gap, the hint, and the frame.
const NUMBERS_HEIGHT: usize = 6;
/// Marks the chosen row.
const CHOSEN: &str = "▌";

fn framed<'a>(title: &'a str) -> Block<'a> {
    Block::new()
        .borders(Borders::ALL)
        .border_style(Style::new().fg(theme::ACCENT))
        .title(title)
        .style(Style::new().fg(theme::TEXT).bg(theme::SURFACE))
}

/// The suggestions of the cursor line, in a box under the cursor (above it at the bottom).
pub(crate) fn draw_suggest(frame: &mut Frame, app: &App, cell: Position) {
    let Some(open) = &app.suggest else {
        return;
    };
    let options = &open.items.options;
    let shown = options.len().min(SUGGEST_ROWS);
    // Keep the chosen row in the window of rows.
    let first = (open.sel + 1).saturating_sub(shown);
    let width = options.iter().map(|o| cols(o.chars())).max().unwrap_or(0) + 4;

    let screen = frame.area();
    let height = u16::try_from(shown).unwrap_or(u16::MAX);
    let width = u16::try_from(width).unwrap_or(u16::MAX).min(screen.width);
    let below = cell.y + 1 + height <= screen.bottom();
    let area = Rect {
        x: cell.x.min(screen.right().saturating_sub(width)),
        y: if below {
            cell.y + 1
        } else {
            cell.y.saturating_sub(height)
        },
        width,
        height,
    };

    let items: Vec<ListItem> = options
        .iter()
        .skip(first)
        .take(shown)
        .map(|o| ListItem::new(format!(" {o}")))
        .collect();
    let list = List::new(items)
        .style(Style::new().fg(theme::TEXT).bg(theme::SURFACE))
        .highlight_symbol(CHOSEN)
        .highlight_style(Style::new().fg(theme::ACCENT).bg(theme::RAISED));
    let mut state = ListState::default().with_selected(Some(open.sel - first));
    frame.render_widget(Clear, area);
    frame.render_stateful_widget(list, area, &mut state);
}

/// One `label  value` row of a form; the active one is lifted and shows the cursor.
fn field_row(frame: &mut Frame, area: Rect, label: &str, value: &str, active: bool) {
    let mut line = vec![
        Span::styled(
            format!("{label:<LABEL_WIDTH$}"),
            Style::new().fg(theme::MUTED),
        ),
        Span::styled(value.to_owned(), Style::new().fg(theme::STRONG)),
    ];
    if !active {
        frame.render_widget(Paragraph::new(Line::from(std::mem::take(&mut line))), area);
        return;
    }

    frame.render_widget(
        Paragraph::new(Line::from(line)).style(Style::new().bg(theme::RAISED)),
        area,
    );
    let used = LABEL_WIDTH + cols(value.chars());
    let x = area
        .x
        .saturating_add(u16::try_from(used).unwrap_or(u16::MAX))
        .min(area.right().saturating_sub(1));
    frame.set_cursor_position(Position::new(x, area.y));
}

/// The row `index` inside a box.
fn row(inner: Rect, index: u16) -> Rect {
    Rect {
        y: inner.y + index,
        height: 1,
        ..inner
    }
}

pub(crate) fn draw_find(frame: &mut Frame, app: &App) {
    let area = centered(frame.area(), BOX_WIDTH, 8);
    let block = framed(&app.texts.find_title);
    let inner = block.inner(area);
    frame.render_widget(Clear, area);
    frame.render_widget(block, area);

    let find = &app.find;
    let fields = [
        (&app.texts.find_find, &find.find, FindField::Find),
        (&app.texts.find_replace, &find.replace, FindField::Replace),
        (&app.texts.find_speaker, &find.speaker, FindField::Speaker),
    ];
    for (i, (label, value, field)) in fields.into_iter().enumerate() {
        let at = row(inner, i as u16);
        field_row(frame, at, label, value, find.field == field);
    }

    let case = match find.case {
        Case::Ignore => &app.texts.find_case_off,
        Case::Match => &app.texts.find_case_on,
    };
    let count = match find.hits.is_empty() {
        true => app.texts.find_none.clone(),
        false => app.texts.match_of(find.sel + 1, find.hits.len()),
    };
    let quiet = Style::new().fg(theme::MUTED);
    frame.render_widget(
        Paragraph::new(format!("{case}   {count}")).style(quiet),
        row(inner, 3),
    );
    frame.render_widget(
        Paragraph::new(app.texts.find_hint.as_str()).style(quiet),
        row(inner, 5),
    );
}

pub(crate) fn draw_title(frame: &mut Frame, app: &App) {
    let Some(form) = &app.title else {
        return;
    };
    let rows: usize = form
        .fields
        .iter()
        .map(|f| field_lines(&f.shown).len())
        .sum();
    let area = centered(frame.area(), BOX_WIDTH, rows + TITLE_CHROME);
    let block = framed(&app.texts.title_box);
    let inner = block.inner(area);
    frame.render_widget(Clear, area);
    frame.render_widget(block, area);

    // A field takes one row per line; the label stands on its first row. Where the rows do
    // not fit, they scroll so the last line of the selected field (the cursor) stays shown.
    let mut shown: Vec<(&str, &str, bool)> = Vec::new();
    let mut cursor = 0;
    for (i, field) in form.fields.iter().enumerate() {
        let label = app.texts.field_label(&field.key);
        let lines = field_lines(&field.shown);
        let last = lines.len() - 1;
        for (n, line) in lines.into_iter().enumerate() {
            let label = if n == 0 { label } else { "" };
            shown.push((label, line, form.sel == i && n == last));
        }
        if i <= form.sel {
            cursor = shown.len() - 1;
        }
    }
    let room = (inner.height as usize)
        .saturating_sub(TITLE_CHROME - 2)
        .max(1);
    let top = (cursor + 1).saturating_sub(room);
    for (r, (label, line, active)) in shown.iter().skip(top).take(room).enumerate() {
        field_row(frame, row(inner, r as u16), label, line, *active);
    }
    let hint = row(inner, shown.len().min(room) as u16 + 1);
    frame.render_widget(
        Paragraph::new(app.texts.title_hint.as_str()).style(Style::new().fg(theme::MUTED)),
        hint,
    );
}

pub(crate) fn draw_numbers(frame: &mut Frame, app: &App) {
    let Some(form) = &app.numbers else {
        return;
    };
    let area = centered(frame.area(), BOX_WIDTH, NUMBERS_HEIGHT);
    let block = framed(&app.texts.numbers_box);
    let inner = block.inner(area);
    frame.render_widget(Clear, area);
    frame.render_widget(block, area);

    let mode = match form.numbering {
        SceneNumbering::Automatic => &app.texts.numbering_automatic,
        SceneNumbering::Locked => &app.texts.numbering_locked,
    };
    let rows = [
        (&app.texts.numbering_label, mode.as_str()),
        (&app.texts.first_scene_label, form.first.as_str()),
    ];
    for (i, (label, value)) in rows.into_iter().enumerate() {
        field_row(frame, row(inner, i as u16), label, value, form.sel == i);
    }
    frame.render_widget(
        Paragraph::new(app.texts.numbers_hint.as_str()).style(Style::new().fg(theme::MUTED)),
        row(inner, rows.len() as u16 + 1),
    );
}

/// The lines of a field value; an empty or newline-ending value still has a last line.
fn field_lines(value: &str) -> Vec<&str> {
    value.split('\n').collect()
}

/// The PDF options: one on/off row each.
pub(crate) fn draw_pdf(frame: &mut Frame, app: &App) {
    let Some(form) = &app.pdf else {
        return;
    };
    let area = centered(frame.area(), BOX_WIDTH, PDF_ROWS.len() + 4);
    let block = framed(&app.texts.pdf_title);
    let inner = block.inner(area);
    frame.render_widget(Clear, area);
    frame.render_widget(block, area);

    for (i, kind) in PDF_ROWS.into_iter().enumerate() {
        let label = match kind {
            PdfRow::Numbers => &app.texts.pdf_numbers,
            PdfRow::Header => &app.texts.pdf_header,
            PdfRow::Contd => &app.texts.pdf_contd,
        };
        let value = match form.is_on(kind) {
            true => &app.texts.pdf_on,
            false => &app.texts.pdf_off,
        };
        let line = Line::from(vec![
            Span::styled(
                format!("{label:<LABEL_WIDTH$}"),
                Style::new().fg(theme::MUTED),
            ),
            Span::styled(value.as_str(), Style::new().fg(theme::STRONG)),
        ]);
        let style = match form.sel == i {
            true => Style::new().bg(theme::RAISED),
            false => Style::new(),
        };
        frame.render_widget(Paragraph::new(line).style(style), row(inner, i as u16));
    }
    frame.render_widget(
        Paragraph::new(app.texts.pdf_hint.as_str()).style(Style::new().fg(theme::MUTED)),
        row(inner, PDF_ROWS.len() as u16 + 1),
    );
}

pub(crate) fn draw_backup(frame: &mut Frame, app: &App) {
    let Some(open) = app.backup else {
        return;
    };
    let (message, hint) = match open {
        Backup::Found => (&app.texts.backup_found, &app.texts.backup_hint),
        Backup::Unreadable => (
            &app.texts.backup_unreadable,
            &app.texts.backup_hint_unreadable,
        ),
    };
    let width = cols(message.chars()).max(cols(hint.chars())) + 4;
    let area = centered(frame.area(), width, 4);
    let block = framed(&app.texts.backup_title);
    let inner = block.inner(area);
    frame.render_widget(Clear, area);
    frame.render_widget(block, area);

    let strong = Style::new().fg(theme::STRONG).add_modifier(Modifier::BOLD);
    frame.render_widget(
        Paragraph::new(message.as_str()).style(strong),
        row(inner, 0),
    );
    frame.render_widget(
        Paragraph::new(hint.as_str()).style(Style::new().fg(theme::MUTED)),
        row(inner, 1),
    );
}

/// The question about the book being asked and its lines: a scene first, then a location
/// of the headings (DB-51).
fn book_lines(app: &App) -> Option<(&str, String, Vec<String>)> {
    if let Some(question) = app.book_question() {
        let ask = app.texts.book_ask(&question.number, &question.heading);
        let options = app
            .book_answers()
            .iter()
            .map(|a| match a.choice {
                Choice::NotInBook => app.texts.book_none.clone(),
                Choice::Scene(_) => format!("{} {}", a.number, a.heading),
            })
            .collect();

        return Some((&app.texts.book_title, ask, options));
    }

    let question = app.place_question()?;
    let ask = app.texts.place_ask(&question.name, question.headings);
    let options = app
        .place_answers()
        .iter()
        .map(|a| app.texts.place_line(a, &question.name))
        .collect();

    Some((&app.texts.place_title, ask, options))
}

/// One open question about the book: the entry and the scenes it may be, or a location of
/// the headings and the play locations it may be.
pub(crate) fn draw_book(frame: &mut Frame, app: &App) {
    let Some((title, ask, options)) = book_lines(app) else {
        return;
    };

    let width = BOX_WIDTH.max(cols(ask.chars()) + 4);
    let area = centered(frame.area(), width, options.len() + 4);
    let block = framed(title);
    let inner = block.inner(area);
    frame.render_widget(Clear, area);
    frame.render_widget(block, area);

    let strong = Style::new().fg(theme::STRONG).add_modifier(Modifier::BOLD);
    frame.render_widget(Paragraph::new(ask).style(strong), row(inner, 0));
    for (i, option) in options.iter().enumerate() {
        let chosen = app.book_sel == i;
        let (mark, style) = match chosen {
            true => (CHOSEN, Style::new().fg(theme::STRONG).bg(theme::RAISED)),
            false => (" ", Style::new().fg(theme::TEXT)),
        };
        let line = format!("{mark}{option}");
        frame.render_widget(Paragraph::new(line).style(style), row(inner, i as u16 + 1));
    }
    let hint = row(inner, options.len() as u16 + 1);
    frame.render_widget(
        Paragraph::new(app.texts.book_hint.as_str()).style(Style::new().fg(theme::MUTED)),
        hint,
    );
}
