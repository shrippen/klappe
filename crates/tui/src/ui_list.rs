//! The outline pane (DB-20, DB-21): sections, scenes and synopses as a tree with the
//! length of each in eighths, the filter on top.

use klappe_services::OutlineKind;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use crate::app::{list_top, App, Focus, INDENT};
use crate::layout::cols;
use crate::theme;
use crate::ui::{fit, CURRENT_MARK};

const OTHER_MARK: &str = "  ";
/// Columns between a row's text and its length.
const LENGTH_GAP: usize = 1;
const OPEN: &str = "▾ ";
const FOLDED: &str = "▸ ";
const SYNOPSIS_MARK: &str = "= ";

fn pane() -> Block<'static> {
    Block::new()
        .borders(Borders::RIGHT)
        .border_style(Style::new().fg(theme::RAISED))
}

/// The rows below the title line of the pane in `area`.
pub(crate) fn list_rows_area(area: Rect) -> Rect {
    let [_, rows] =
        Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(pane().inner(area));

    rows
}

pub(crate) fn draw_list(frame: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == Focus::List;
    let block = pane();
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let [title, _] = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(inner);
    let rows = list_rows_area(area);
    let heading = Style::new().fg(theme::MUTED).add_modifier(Modifier::BOLD);
    let label = match app.list_filter.is_empty() {
        true => format!(" {}", app.texts.scenes),
        false => format!(" {}: {}", app.texts.filter, app.list_filter),
    };
    frame.render_widget(Paragraph::new(label).style(heading), title);
    // The whole screenplay in pages, eighths and minutes (DB-32); italic while the text has changed.
    if let Some(pages) = app.pages() {
        let mut style = Style::new().fg(theme::MUTED);
        if app.pages_stale() {
            style = style.add_modifier(Modifier::ITALIC);
        }
        let total = Paragraph::new(format!(
            "{} · {} ",
            pages.length,
            app.texts.minutes(pages.minutes)
        ))
        .alignment(Alignment::Right)
        .style(style);
        frame.render_widget(total, title);
    }

    let shown = app.list_rows();
    if shown.is_empty() {
        let none = match app.list_filter.is_empty() {
            true => app.texts.no_scenes.as_str(),
            false => app.texts.goto_none.as_str(),
        };
        let none = Paragraph::new(none).style(Style::new().fg(theme::MUTED));
        frame.render_widget(none, rows);
        return;
    }

    let pos = app.list_pos(&shown);
    let top = list_top(pos, usize::from(rows.height));
    let current = app.analysis.scene_at(app.cursor().line);
    let items: Vec<ListItem> = shown
        .iter()
        .skip(top)
        .take(usize::from(rows.height))
        .map(|&i| item(app, i, current, usize::from(rows.width)))
        .collect();

    let highlight = if focused {
        Style::new().bg(theme::RAISED)
    } else {
        Style::new()
    };
    let widget = List::new(items).highlight_style(highlight);
    let mut state = ListState::default().with_selected(Some(pos - top));
    frame.render_stateful_widget(widget, rows, &mut state);
}

/// One row: bar for the scene the cursor is in, indent, the entry and its length at the
/// right edge.
fn item(app: &App, i: usize, current: Option<usize>, width: usize) -> ListItem<'_> {
    let entry = &app.analysis.outline[i];
    let indent = " ".repeat(entry.depth * INDENT);
    let length = &app.outline_info[i].length;
    let here = entry.scene.is_some() && entry.scene == current;
    let mark = match here {
        true => Span::styled(CURRENT_MARK, Style::new().fg(theme::ACCENT)),
        false => Span::raw(OTHER_MARK),
    };

    let (lead, text, lead_style, text_style) = match entry.kind {
        OutlineKind::Section(_) => {
            let fold = if app.is_collapsed(i) { FOLDED } else { OPEN };
            let style = Style::new().fg(theme::STRONG).add_modifier(Modifier::BOLD);
            (fold.to_string(), entry.text.as_str(), style, style)
        }
        OutlineKind::Scene => {
            let number = entry.scene.map_or_else(String::new, |n| {
                entry.number.clone().unwrap_or_else(|| (n + 1).to_string())
            });
            let (lead, text) = match here {
                true => (
                    Style::new().fg(theme::STRONG).add_modifier(Modifier::BOLD),
                    Style::new().fg(theme::STRONG).add_modifier(Modifier::BOLD),
                ),
                false => (Style::new().fg(theme::MUTED), Style::new().fg(theme::SOFT)),
            };
            (format!("{number}  "), entry.text.as_str(), lead, text)
        }
        OutlineKind::Synopsis => {
            let style = Style::new().fg(theme::MUTED).add_modifier(Modifier::ITALIC);
            (SYNOPSIS_MARK.to_string(), entry.text.as_str(), style, style)
        }
    };

    let used = cols(OTHER_MARK.chars()) + cols(indent.chars()) + cols(lead.chars());
    let room = width
        .saturating_sub(used + cols(length.chars()) + LENGTH_GAP)
        .max(1);
    let text = fit(text, room);
    let pad = width.saturating_sub(used + cols(text.chars()) + cols(length.chars()));
    let tail = format!("{}{length}", " ".repeat(pad));

    ListItem::new(Line::from(vec![
        mark,
        Span::raw(indent),
        Span::styled(lead, lead_style),
        Span::styled(text, text_style),
        Span::styled(tail, Style::new().fg(theme::MUTED)),
    ]))
}
