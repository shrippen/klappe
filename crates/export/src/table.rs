//! The shot list as PDF sheets: a table with the head row repeated on every sheet.
//!
//! ```text
//! ┌──────────────────────────────────────────┐
//! │ Szene 12 · Sängersaal · Plan A · Shotliste│
//! │ Nr. Kam Obj. Größe … Beschreibung  Notiz │
//! │ ───────────────────────────────────────  │
//! │ 12.1 A  35  Totale … Elisabeth tritt …   │
//! │ ───────────────────────────────────────  │
//! └──────────────────────────────────────────┘
//! ```
//!
//! Short columns (number, camera, lens, size, movement, times) take what their words need;
//! the free text columns share the rest and wrap. Sheets are always landscape, whatever the
//! plan page: thirteen columns need the width.

use klappe_model::{FloorPlan, Shot};

use crate::canvas::{Anchor, Dash, Geom, Prim, Sheet, Stroke, Weight};
use crate::error::ExportError;
use crate::layout::{paper_mm, GAP_PT, MIN_TEXT_PT, PT_PER_MM, TITLE_BAND_PT, TITLE_PT};
use crate::metrics::text_width_pt;
use crate::page::{Page, ShotTexts};
use crate::palette::Palette;
use crate::shots::{cells, heads, COLUMNS};

const CELL_PT: f64 = MIN_TEXT_PT;
const LINE_PT: f64 = 10.5;
const PAD_PT: f64 = 2.5;
/// Rounding slack when a width comes back from `+ 2 * PAD_PT - 2 * PAD_PT`.
const FIT_SLACK_PT: f64 = 1e-6;
const ROW_PAD_PT: f64 = 3.0;
const HEAD_RULE_PT: f64 = 0.75;
const ROW_RULE_PT: f64 = 0.5;
/// Longer cells are cut, so one row always fits a sheet.
const MAX_CELL_LINES: usize = 8;
const ELLIPSIS: char = '…';
/// Columns before this index are short (number … duration); the rest are free text.
const FIRST_TEXT_COLUMN: usize = 8;
const DESCRIPTION_COLUMN: usize = 8;
/// Share of the table a short column may take at most.
const SHORT_COLUMN_MAX: f64 = 0.2;
/// The description gets twice the room of the other free text columns.
const DESCRIPTION_WEIGHT: f64 = 2.0;
const TEXT_WEIGHT: f64 = 1.0;
const TITLE_SEPARATOR: &str = " · ";

pub(crate) fn sheets(
    plan: &FloorPlan,
    page: &Page,
    pal: &Palette,
) -> Result<Vec<Sheet>, ExportError> {
    if plan.shots.is_empty() {
        return Ok(Vec::new());
    }

    // Always landscape (see above); upright A4 broke words.
    let (short_mm, long_mm) = paper_mm(page.paper);
    let (width_pt, height_pt) = (long_mm * PT_PER_MM, short_mm * PT_PER_MM);
    let margin = page.margin_mm.max(0.0) * PT_PER_MM;
    let content_w = width_pt - 2.0 * margin;
    let top = margin + TITLE_BAND_PT + GAP_PT;
    let bottom = height_pt - margin;
    if content_w <= 0.0 || bottom <= top {
        return Err(ExportError::PageTooSmall);
    }

    let texts = &page.texts.shots;
    let rows: Vec<[String; COLUMNS]> = plan
        .shots
        .iter()
        .map(|s: &Shot| cells(s, plan, texts, page.texts.decimal_separator))
        .collect();
    let head = heads(&texts.columns).map(str::to_owned);
    let widths = column_widths(&head, &rows, content_w);
    let head_cells = wrapped(&head, &widths, Weight::Bold);

    let title = sheet_title(page, texts);
    let mut ctx = Ctx {
        pal,
        margin,
        widths: &widths,
        title: &title,
        head: &head_cells,
        sheets: Vec::new(),
        prims: Vec::new(),
        y: top,
        rows: 0,
    };

    ctx.begin();
    for row in &rows {
        let lines = wrapped(row, &widths, Weight::Regular);
        let height = row_height(&lines);
        if ctx.y + height > bottom && ctx.rows > 0 {
            ctx.finish(width_pt, height_pt);
            ctx.begin();
        }
        ctx.row(&lines, height, Weight::Regular, ROW_RULE_PT, pal.muted);
    }
    ctx.finish(width_pt, height_pt);

    Ok(ctx.sheets)
}

fn sheet_title(page: &Page, texts: &ShotTexts) -> String {
    if texts.title.is_empty() {
        return page.title.clone();
    }

    format!("{}{TITLE_SEPARATOR}{}", page.title, texts.title)
}

/// Sheet under construction.
struct Ctx<'a> {
    pal: &'a Palette,
    margin: f64,
    widths: &'a [f64; COLUMNS],
    title: &'a str,
    head: &'a [Vec<String>; COLUMNS],
    sheets: Vec<Sheet>,
    prims: Vec<Prim>,
    y: f64,
    /// Shot rows on this sheet.
    rows: usize,
}

impl Ctx<'_> {
    /// Title and head row at the top of a new sheet.
    fn begin(&mut self) {
        self.prims.push(Prim::Text {
            class: "title".into(),
            anchor: Anchor::Start,
            at: (self.margin, self.margin + TITLE_PT),
            size_pt: TITLE_PT,
            weight: Weight::Bold,
            rgb: self.pal.foreground,
            text: self.title.to_owned(),
        });

        self.y = self.margin + TITLE_BAND_PT + GAP_PT;
        self.rows = 0;
        let head = self.head;
        let height = row_height(head);
        self.row(
            head,
            height,
            Weight::Bold,
            HEAD_RULE_PT,
            self.pal.foreground,
        );
        self.rows = 0;
    }

    fn row(
        &mut self,
        lines: &[Vec<String>; COLUMNS],
        height: f64,
        weight: Weight,
        rule_pt: f64,
        rule: crate::palette::Rgb,
    ) {
        let mut x = self.margin;
        for (cell, width) in lines.iter().zip(self.widths) {
            for (k, text) in cell.iter().enumerate() {
                self.prims.push(Prim::Text {
                    class: "shot".into(),
                    anchor: Anchor::Start,
                    at: (
                        x + PAD_PT,
                        self.y + ROW_PAD_PT + k as f64 * LINE_PT + CELL_PT,
                    ),
                    size_pt: CELL_PT,
                    weight,
                    rgb: self.pal.foreground,
                    text: text.clone(),
                });
            }
            x += width;
        }

        self.rows += 1;
        self.y += height;
        let table_w: f64 = self.widths.iter().sum();
        self.prims.push(Prim::Shape {
            class: "rule".into(),
            geom: Geom::Open(vec![(self.margin, self.y), (self.margin + table_w, self.y)]),
            fill: None,
            stroke: Some(Stroke {
                rgb: rule,
                width_pt: rule_pt,
                dash: Dash::Solid,
            }),
        });
    }

    fn finish(&mut self, width_pt: f64, height_pt: f64) {
        self.sheets.push(Sheet {
            width_pt,
            height_pt,
            prims: std::mem::take(&mut self.prims),
        });
    }
}

fn row_height(cells: &[Vec<String>; COLUMNS]) -> f64 {
    let lines = cells.iter().map(Vec::len).max().unwrap_or(0).max(1);

    lines as f64 * LINE_PT + 2.0 * ROW_PAD_PT
}

/// Short columns as wide as their words, free text columns share the rest by need. When
/// even the narrowest layout is too wide, everything shrinks alike (long words then break).
fn column_widths(
    head: &[String; COLUMNS],
    rows: &[[String; COLUMNS]],
    content_w: f64,
) -> [f64; COLUMNS] {
    let mut widths = [0.0; COLUMNS];
    let mut used = [false; COLUMNS];

    for (i, title) in head.iter().enumerate() {
        let widest_head = title
            .split_whitespace()
            .map(|w| text_width_pt(w, CELL_PT, Weight::Bold))
            .fold(0.0, f64::max);
        let widest_cell = rows.iter().map(|r| widest_line(&r[i])).fold(0.0, f64::max);
        used[i] = rows.iter().any(|r| !r[i].is_empty());

        widths[i] = if i < FIRST_TEXT_COLUMN {
            widest_head
                .max(widest_cell)
                .min(content_w * SHORT_COLUMN_MAX)
                + 2.0 * PAD_PT
        } else {
            widest_head + 2.0 * PAD_PT
        };
    }

    let total: f64 = widths.iter().sum();
    if total >= content_w {
        return widths.map(|w| w * content_w / total);
    }

    let weight = |i: usize| match (used[i], i) {
        (false, _) => 0.0,
        (true, DESCRIPTION_COLUMN) => DESCRIPTION_WEIGHT,
        (true, _) => TEXT_WEIGHT,
    };
    let weights: f64 = (FIRST_TEXT_COLUMN..COLUMNS).map(weight).sum();
    if weights == 0.0 {
        return widths;
    }

    let free = content_w - total;
    for (i, width) in widths.iter_mut().enumerate().skip(FIRST_TEXT_COLUMN) {
        *width += free * weight(i) / weights;
    }

    widths
}

fn widest_line(text: &str) -> f64 {
    text.lines()
        .map(|l| text_width_pt(l, CELL_PT, Weight::Regular))
        .fold(0.0, f64::max)
}

fn wrapped(
    texts: &[String; COLUMNS],
    widths: &[f64; COLUMNS],
    weight: Weight,
) -> [Vec<String>; COLUMNS] {
    std::array::from_fn(|i| cell_lines(&texts[i], widths[i] - 2.0 * PAD_PT, weight))
}

/// Greedy word wrap to `max_w`; words wider than the column break by letter. Cut at
/// `MAX_CELL_LINES` with an ellipsis.
fn cell_lines(text: &str, max_w: f64, weight: Weight) -> Vec<String> {
    let fits = |t: &str| text_width_pt(t, CELL_PT, weight) <= max_w + FIT_SLACK_PT;
    let mut lines = Vec::new();

    if text.trim().is_empty() {
        return lines;
    }

    for paragraph in text.lines() {
        let mut line = String::new();
        for word in paragraph.split_whitespace() {
            for piece in pieces(word, max_w, weight) {
                let joined = if line.is_empty() {
                    piece.clone()
                } else {
                    format!("{line} {piece}")
                };
                if fits(&joined) {
                    line = joined;
                    continue;
                }
                lines.push(std::mem::replace(&mut line, piece));
            }
        }
        lines.push(line);
    }

    if lines.len() > MAX_CELL_LINES {
        lines.truncate(MAX_CELL_LINES);
        if let Some(last) = lines.last_mut() {
            ellipsise(last, max_w, weight);
        }
    }

    lines
}

/// The word, or chunks of it that each fit `max_w` (at least one letter each).
fn pieces(word: &str, max_w: f64, weight: Weight) -> Vec<String> {
    if text_width_pt(word, CELL_PT, weight) <= max_w + FIT_SLACK_PT {
        return vec![word.to_owned()];
    }

    let mut out = Vec::new();
    let mut chunk = String::new();
    for c in word.chars() {
        chunk.push(c);
        if chunk.chars().count() > 1 && text_width_pt(&chunk, CELL_PT, weight) > max_w {
            chunk.pop();
            out.push(std::mem::take(&mut chunk));
            chunk.push(c);
        }
    }
    out.push(chunk);

    out
}

/// Ends the line with "…", dropping letters until it fits.
fn ellipsise(line: &mut String, max_w: f64, weight: Weight) {
    line.push(ELLIPSIS);
    while line.chars().count() > 1 && text_width_pt(line, CELL_PT, weight) > max_w {
        line.pop();
        line.pop();
        line.push(ELLIPSIS);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NARROW_PT: f64 = 60.0;

    /// A short column is as wide as its widest cell: "12.1" stays on one line.
    #[test]
    fn short_cells_fit_their_column() {
        let head: [String; COLUMNS] = std::array::from_fn(|_| String::new());
        let numbers = ["12.1", "12.2", "12.3"];
        let rows: Vec<[String; COLUMNS]> = numbers
            .iter()
            .map(|n| std::array::from_fn(|i| if i == 0 { n.to_string() } else { String::new() }))
            .collect();

        let widths = column_widths(&head, &rows, 700.0);
        for row in &rows {
            let lines = wrapped(row, &widths, Weight::Regular);
            assert_eq!(lines[0], [row[0].clone()]);
        }
    }

    #[test]
    fn words_wrap_at_the_width() {
        let lines = cell_lines("Elisabeth tritt an den Tisch", NARROW_PT, Weight::Regular);

        assert!(lines.len() > 1);
        for line in &lines {
            assert!(
                text_width_pt(line, CELL_PT, Weight::Regular) <= NARROW_PT,
                "{line}"
            );
        }
        assert_eq!(lines.join(" "), "Elisabeth tritt an den Tisch");
    }

    #[test]
    fn a_long_word_breaks_by_letter() {
        let word = "Donaudampfschifffahrtsgesellschaft";
        let lines = cell_lines(word, NARROW_PT, Weight::Regular);

        assert!(lines.len() > 1);
        assert_eq!(lines.concat(), word);
        for line in &lines {
            assert!(
                text_width_pt(line, CELL_PT, Weight::Regular) <= NARROW_PT,
                "{line}"
            );
        }
    }

    #[test]
    fn line_breaks_stay_and_long_text_is_cut() {
        assert_eq!(
            cell_lines("eins\nzwei", NARROW_PT, Weight::Regular),
            ["eins", "zwei"]
        );

        let lines = cell_lines(&"wort ".repeat(500), NARROW_PT, Weight::Regular);
        assert_eq!(lines.len(), MAX_CELL_LINES);
        assert!(lines[MAX_CELL_LINES - 1].ends_with(ELLIPSIS));
    }

    #[test]
    fn an_empty_cell_has_no_lines() {
        assert!(cell_lines("  ", NARROW_PT, Weight::Regular).is_empty());
    }
}
