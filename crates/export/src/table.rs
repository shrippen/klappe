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
//! Short columns (number, camera, lens, size, angle, movement, times) take what their words need;
//! the free text columns share the rest and wrap. Sheets have the plan page's paper and
//! orientation. Upright, fifteen columns do not fit at a readable size, so each shot takes two
//! lines of the table: the short columns on top, and under them, indented, the free text
//! columns as "Name: text" over the full width.
//!
//! ```text
//! │ Nr.  Kam Obj. Größe  Persp.  Bewegung Von Bis Dauer │
//! │ Inhalt · Requisiten · Technik · Team · Notiz · …    │
//! │ 12.1 A   35   Totale Normal. Statisch 0   6   6     │
//! │      Inhalt: Elisabeth tritt durch die Tür, bleibt  │
//! │      am Fenster stehen …                            │
//! ```

use klappe_model::{FloorPlan, Orientation, Shot};
use klappe_stellprobe::{axis_jumps, ordered_shots, ShotOrder};

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
const FIRST_TEXT_COLUMN: usize = 9;
const DESCRIPTION_COLUMN: usize = 9;
/// Share of the table a short column may take at most.
const SHORT_COLUMN_MAX: f64 = 0.2;
/// The description gets twice the room of the other free text columns.
const DESCRIPTION_WEIGHT: f64 = 2.0;
const TEXT_WEIGHT: f64 = 1.0;
/// A setup's heading row in shooting order.
const BAND_PT: f64 = LINE_PT + 2.0 * ROW_PAD_PT;
const HEAD_SEPARATOR: &str = " · ";
const TITLE_SEPARATOR: &str = " · ";

pub(crate) fn sheets(
    plan: &FloorPlan,
    page: &Page,
    pal: &Palette,
    order: ShotOrder,
) -> Result<Vec<Sheet>, ExportError> {
    if plan.shots.is_empty() {
        return Ok(Vec::new());
    }

    let upright = page.orientation == Orientation::Portrait;
    let (width_pt, height_pt) = sheet_size_pt(page);
    let margin = page.margin_mm.max(0.0) * PT_PER_MM;
    let content_w = width_pt - 2.0 * margin;
    let top = margin + TITLE_BAND_PT + GAP_PT;
    let bottom = height_pt - margin;
    if content_w <= 0.0 || bottom <= top {
        return Err(ExportError::PageTooSmall);
    }

    let texts = &page.texts.shots;
    let ordered = ordered_shots(plan, order);
    let rows = shot_cells(plan, page, &ordered);
    let head = heads(&texts.columns).map(str::to_owned);
    let widths = if upright {
        short_widths(&head, &rows, content_w)
    } else {
        column_widths(&head, &rows, content_w)
    };
    let head_cells = wrapped(&head, &widths, Weight::Bold);
    let indent = widths[0];
    let extra_w = content_w - indent - 2.0 * PAD_PT;
    let head_extra = head_extra(&head, extra_w, upright);

    let table_w = table_width(&widths, content_w, upright);
    let title = sheet_title(page, texts);
    let mut ctx = Ctx {
        pal,
        margin,
        widths: &widths,
        title: &title,
        head: &head_cells,
        head_extra: &head_extra,
        indent,
        table_w,
        sheets: Vec::new(),
        prims: Vec::new(),
        y: top,
        rows: 0,
    };

    ctx.begin();
    let mut setup = 0;
    for (row, (shot_setup, _)) in rows.iter().zip(&ordered) {
        let lines = wrapped(row, &widths, Weight::Regular);
        let extra = extra_lines(row, &head, extra_w, upright);
        let height = row_height(&lines) + extra.len() as f64 * LINE_PT;

        // A setup's heading never stands alone at the foot of a sheet, and a setup that
        // runs onto a new sheet gets its heading again there.
        let shooting = order == ShotOrder::Shooting;
        let mut heading = shooting && *shot_setup != setup;
        let band_h = if heading { BAND_PT } else { 0.0 };
        if ctx.y + band_h + height > bottom && ctx.rows > 0 {
            ctx.finish(width_pt, height_pt);
            ctx.begin();
            heading = shooting;
        }
        if heading {
            setup = *shot_setup;
            ctx.band(&format!("{} {setup}", texts.columns.setup));
        }
        ctx.shot(&lines, &extra, Weight::Regular, ROW_RULE_PT, pal.muted);
    }
    ctx.finish(width_pt, height_pt);

    Ok(ctx.sheets)
}

/// Upright, the free text lines run under the short columns: the table is as wide as the page.
fn table_width(widths: &[f64; COLUMNS], content_w: f64, upright: bool) -> f64 {
    match upright {
        true => content_w,
        false => widths.iter().sum(),
    }
}

/// The paper in the page's orientation.
fn sheet_size_pt(page: &Page) -> (f64, f64) {
    let (short_mm, long_mm) = paper_mm(page.paper);
    let (w_mm, h_mm) = match page.orientation {
        Orientation::Portrait => (short_mm, long_mm),
        Orientation::Landscape => (long_mm, short_mm),
    };

    (w_mm * PT_PER_MM, h_mm * PT_PER_MM)
}

fn shot_cells(plan: &FloorPlan, page: &Page, ordered: &[(usize, &Shot)]) -> Vec<[String; COLUMNS]> {
    let jumps = axis_jumps(plan);

    ordered
        .iter()
        .map(|(_, s)| {
            cells(
                s,
                plan,
                &jumps,
                &page.texts.shots,
                page.texts.decimal_separator,
            )
        })
        .collect()
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
    /// Upright: the second head line, over the free text columns.
    head_extra: &'a [String],
    /// Upright: how far the free text lines are indented.
    indent: f64,
    table_w: f64,
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
        let (head, extra) = (self.head, self.head_extra);
        self.shot(head, extra, Weight::Bold, HEAD_RULE_PT, self.pal.foreground);
        self.rows = 0;
    }

    /// A row, and under it the free text lines of an upright table, with one rule below both.
    fn shot(
        &mut self,
        lines: &[Vec<String>; COLUMNS],
        extra: &[String],
        weight: Weight,
        rule_pt: f64,
        rule: crate::palette::Rgb,
    ) {
        if extra.is_empty() {
            self.row(lines, row_height(lines), weight, rule_pt, rule);
            return;
        }

        self.row(lines, row_height(lines), weight, 0.0, rule);
        let x = self.margin + self.indent + PAD_PT;
        let top = self.y - ROW_PAD_PT;
        for (k, text) in extra.iter().enumerate() {
            self.prims.push(Prim::Text {
                class: "shot".into(),
                anchor: Anchor::Start,
                at: (x, top + k as f64 * LINE_PT + CELL_PT),
                size_pt: CELL_PT,
                weight,
                rgb: self.pal.foreground,
                text: text.clone(),
            });
        }
        self.y += extra.len() as f64 * LINE_PT;
        self.rule(rule_pt, rule);
    }

    fn rule(&mut self, rule_pt: f64, rgb: crate::palette::Rgb) {
        self.prims.push(Prim::Shape {
            class: "rule".into(),
            geom: Geom::Open(vec![
                (self.margin, self.y),
                (self.margin + self.table_w, self.y),
            ]),
            fill: None,
            stroke: Some(Stroke {
                rgb,
                width_pt: rule_pt,
                dash: Dash::Solid,
            }),
        });
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
        if rule_pt > 0.0 {
            self.rule(rule_pt, rule);
        }
    }

    /// A heading over the rows that follow, e.g. "Setup 2".
    fn band(&mut self, text: &str) {
        self.prims.push(Prim::Text {
            class: "setup".into(),
            anchor: Anchor::Start,
            at: (self.margin + PAD_PT, self.y + ROW_PAD_PT + CELL_PT),
            size_pt: CELL_PT,
            weight: Weight::Bold,
            rgb: self.pal.foreground,
            text: text.to_owned(),
        });

        self.y += BAND_PT;
        self.rule(ROW_RULE_PT, self.pal.muted);
    }

    fn finish(&mut self, width_pt: f64, height_pt: f64) {
        self.sheets.push(Sheet {
            width_pt,
            height_pt,
            prims: std::mem::take(&mut self.prims),
        });
    }
}

/// Upright: the short columns as wide as their words; the free text columns have no width of
/// their own, they run under the row (see `extra_lines`).
fn short_widths(
    head: &[String; COLUMNS],
    rows: &[[String; COLUMNS]],
    content_w: f64,
) -> [f64; COLUMNS] {
    let keep = |i: usize| i < FIRST_TEXT_COLUMN;
    let head: [String; COLUMNS] = std::array::from_fn(|i| {
        if keep(i) {
            head[i].clone()
        } else {
            String::new()
        }
    });
    let rows: Vec<[String; COLUMNS]> = rows
        .iter()
        .map(|r| std::array::from_fn(|i| if keep(i) { r[i].clone() } else { String::new() }))
        .collect();
    let widths = column_widths(&head, &rows, content_w);

    std::array::from_fn(|i| if keep(i) { widths[i] } else { 0.0 })
}

/// Upright: the free text cells of a shot as "Name: text" lines, empty cells left out.
fn extra_lines(
    row: &[String; COLUMNS],
    head: &[String; COLUMNS],
    max_w: f64,
    upright: bool,
) -> Vec<String> {
    let mut lines = Vec::new();
    if !upright {
        return lines;
    }

    for i in FIRST_TEXT_COLUMN..COLUMNS {
        if row[i].trim().is_empty() {
            continue;
        }

        lines.extend(cell_lines(
            &format!("{}: {}", head[i], row[i]),
            max_w,
            Weight::Regular,
        ));
    }

    lines
}

/// Upright: the names of the free text columns on one line under the head row.
fn head_extra(head: &[String; COLUMNS], max_w: f64, upright: bool) -> Vec<String> {
    if !upright {
        return Vec::new();
    }

    let names: Vec<&str> = head[FIRST_TEXT_COLUMN..]
        .iter()
        .map(String::as_str)
        .collect();

    cell_lines(&names.join(HEAD_SEPARATOR), max_w, Weight::Bold)
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
    std::array::from_fn(|i| match widths[i] > 0.0 {
        true => cell_lines(&texts[i], widths[i] - 2.0 * PAD_PT, weight),
        false => Vec::new(),
    })
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
