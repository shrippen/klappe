//! The sheets of the reports: a table over landscape sheets with its head row repeated, and
//! the extract with a portrait page per scene.
//!
//! ```text
//! table                              extract
//! ┌────────────────────────────┐     ┌────────────────────┐
//! │ Szenen                     │     │ 12  INT. SALON     │
//! │ Nr. Kopf          Länge …  │     │ Spieltag: Tag 3 ·… │
//! │ ────────────────────────── │     │ ┌────────────────┐ │
//! │ 12  INT. SALON    1 3/8 …  │     │ │ Requisiten     │ │
//! └────────────────────────────┘     │ └────────────────┘ │
//!                                    └────────────────────┘
//! ```

use crate::canvas::{Anchor, Dash, Geom, Prim, Rect, Sheet, Stroke, Weight};
use crate::layout::{paper_mm, GAP_PT, PT_PER_MM, TITLE_BAND_PT, TITLE_PT};
use crate::metrics::text_width_pt;
use crate::palette::{Palette, Rgb};
use crate::report::{ExtractPage, ReportTable};
use crate::table::{cell_lines, CELL_PT, HEAD_RULE_PT, LINE_PT, PAD_PT, ROW_PAD_PT, ROW_RULE_PT};
use crate::Paper;

const MARGIN_MM: f64 = 10.0;
const FACT_SEPARATOR: &str = "  ·  ";
/// Baseline to baseline of a wrapped title.
const TITLE_LINE_PT: f64 = 17.0;

pub(crate) fn table_sheets(table: &ReportTable, pal: &Palette) -> Vec<Sheet> {
    let mut out = Writer::new(pal, Orientation::Landscape, &table.title);
    out.begin();
    let widths = column_widths(&table.head, &table.rows, out.content_w());
    let head = wrapped(&table.head, &widths, Weight::Bold);

    out.table_head(&head, &widths);
    for row in &table.rows {
        let lines = wrapped(row, &widths, Weight::Regular);
        if out.y + row_height(&lines) > out.bottom && out.rows > 0 {
            out.next_sheet();
            out.table_head(&head, &widths);
        }
        out.row(&lines, &widths, Weight::Regular, ROW_RULE_PT, pal.muted);
    }

    out.finish()
}

pub(crate) fn extract_sheets(pages: &[ExtractPage], pal: &Palette) -> Vec<Sheet> {
    let mut out = Writer::new(pal, Orientation::Portrait, "");

    for (i, page) in pages.iter().enumerate() {
        out.title = page.title.clone();
        match i {
            0 => out.begin(),
            _ => out.next_sheet(),
        }
        out.facts(&page.facts);
        for boxed in &page.boxes {
            out.boxed(&boxed.title, &boxed.lines);
        }
    }

    out.finish()
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Orientation {
    Landscape,
    Portrait,
}

/// Sheets under construction, drawn top down.
struct Writer<'a> {
    pal: &'a Palette,
    width_pt: f64,
    height_pt: f64,
    margin: f64,
    bottom: f64,
    /// Over every sheet, until `next_sheet` is told another.
    title: String,
    sheets: Vec<Sheet>,
    prims: Vec<Prim>,
    y: f64,
    /// Table rows on this sheet.
    rows: usize,
}

impl<'a> Writer<'a> {
    fn new(pal: &'a Palette, orientation: Orientation, title: &str) -> Self {
        let (short_mm, long_mm) = paper_mm(Paper::A4);
        let (w_mm, h_mm) = match orientation {
            Orientation::Landscape => (long_mm, short_mm),
            Orientation::Portrait => (short_mm, long_mm),
        };
        let margin = MARGIN_MM * PT_PER_MM;
        Writer {
            pal,
            width_pt: w_mm * PT_PER_MM,
            height_pt: h_mm * PT_PER_MM,
            margin,
            bottom: h_mm * PT_PER_MM - margin,
            title: title.to_owned(),
            sheets: Vec::new(),
            prims: Vec::new(),
            y: 0.0,
            rows: 0,
        }
    }

    fn content_w(&self) -> f64 {
        self.width_pt - 2.0 * self.margin
    }

    /// The title at the top of a fresh sheet. One too wide wraps to the content width;
    /// `cell_lines` measures at `CELL_PT`, so the width is scaled to it. One that fits stays
    /// as it is, since wrapping folds the two spaces after a scene number into one.
    fn begin(&mut self) {
        let fits = text_width_pt(&self.title, TITLE_PT, Weight::Bold) <= self.content_w();
        let lines = match fits {
            true => vec![self.title.clone()],
            false => {
                let max_w = self.content_w() * CELL_PT / TITLE_PT;
                cell_lines(&self.title, max_w, Weight::Bold)
            }
        };
        let mut y = self.margin + TITLE_PT;
        for line in &lines {
            self.text(self.margin, y, TITLE_PT, Weight::Bold, line);
            y += TITLE_LINE_PT;
        }

        let extra = lines.len().saturating_sub(1) as f64 * TITLE_LINE_PT;
        self.y = self.margin + TITLE_BAND_PT + extra + GAP_PT;
        self.rows = 0;
    }

    /// Closes the sheet and begins another, with the title again.
    fn next_sheet(&mut self) {
        self.close();
        self.begin();
    }

    fn close(&mut self) {
        self.sheets.push(Sheet {
            width_pt: self.width_pt,
            height_pt: self.height_pt,
            prims: std::mem::take(&mut self.prims),
        });
    }

    fn finish(mut self) -> Vec<Sheet> {
        self.close();

        self.sheets
    }

    fn text(&mut self, x: f64, y: f64, size_pt: f64, weight: Weight, text: &str) {
        self.prims.push(Prim::Text {
            rotation_deg: 0.0,
            class: "report".into(),
            anchor: Anchor::Start,
            at: (x, y),
            size_pt,
            weight,
            rgb: self.pal.foreground,
            text: text.to_owned(),
        });
    }

    fn rule(&mut self, rule_pt: f64, rgb: Rgb) {
        self.prims.push(Prim::Shape {
            class: "rule".into(),
            geom: Geom::Open(vec![
                (self.margin, self.y),
                (self.margin + self.content_w(), self.y),
            ]),
            fill: None,
            stroke: Some(Stroke {
                rgb,
                width_pt: rule_pt,
                dash: Dash::Solid,
            }),
        });
    }

    fn table_head(&mut self, head: &[Vec<String>], widths: &[f64]) {
        self.row(
            head,
            widths,
            Weight::Bold,
            HEAD_RULE_PT,
            self.pal.foreground,
        );
        self.rows = 0;
    }

    fn row(
        &mut self,
        lines: &[Vec<String>],
        widths: &[f64],
        weight: Weight,
        rule_pt: f64,
        rule: Rgb,
    ) {
        let mut x = self.margin;
        for (cell, width) in lines.iter().zip(widths) {
            for (k, text) in cell.iter().enumerate() {
                let y = self.y + ROW_PAD_PT + k as f64 * LINE_PT + CELL_PT;
                self.text(x + PAD_PT, y, CELL_PT, weight, text);
            }
            x += width;
        }

        self.rows += 1;
        self.y += row_height(lines);
        self.rule(rule_pt, rule);
    }

    /// The facts of a scene on lines under its head, `·` between them.
    fn facts(&mut self, facts: &[String]) {
        let joined = facts.join(FACT_SEPARATOR);
        for line in cell_lines(&joined, self.content_w() - 2.0 * PAD_PT, Weight::Regular) {
            self.text(
                self.margin + PAD_PT,
                self.y + CELL_PT,
                CELL_PT,
                Weight::Regular,
                &line,
            );
            self.y += LINE_PT;
        }
        self.y += GAP_PT;
    }

    /// A box with `title` over `lines`; one that does not fit what is left of the sheet goes on
    /// in a box of the next sheet, which has the same title.
    fn boxed(&mut self, title: &str, entries: &[String]) {
        let max_w = self.content_w() - 4.0 * PAD_PT;
        let lines: Vec<String> = entries
            .iter()
            .flat_map(|e| cell_lines(e, max_w, Weight::Regular))
            .collect();
        let mut rest = lines.as_slice();

        loop {
            let room = self.bottom - self.y - 2.0 * ROW_PAD_PT - LINE_PT;
            let fits = (room / LINE_PT).floor().max(0.0) as usize;
            if fits < rest.len().min(1) {
                self.next_sheet();
                continue;
            }

            let (now, later) = rest.split_at(fits.min(rest.len()));
            self.draw_box(title, now);
            rest = later;
            if rest.is_empty() {
                return;
            }
            self.next_sheet();
        }
    }

    fn draw_box(&mut self, title: &str, lines: &[String]) {
        let x = self.margin + 2.0 * PAD_PT;
        let height = (lines.len() + 1) as f64 * LINE_PT + 2.0 * ROW_PAD_PT;
        self.prims.push(Prim::Shape {
            class: "box".into(),
            geom: Geom::Rect(Rect {
                x: self.margin,
                y: self.y,
                w: self.content_w(),
                h: height,
            }),
            fill: None,
            stroke: Some(Stroke {
                rgb: self.pal.muted,
                width_pt: ROW_RULE_PT,
                dash: Dash::Solid,
            }),
        });

        let top = self.y + ROW_PAD_PT + CELL_PT;
        self.text(x, top, CELL_PT, Weight::Bold, title);
        for (k, line) in lines.iter().enumerate() {
            self.text(
                x,
                top + (k + 1) as f64 * LINE_PT,
                CELL_PT,
                Weight::Regular,
                line,
            );
        }
        self.y += height + GAP_PT;
    }
}

fn row_height(cells: &[Vec<String>]) -> f64 {
    let lines = cells.iter().map(Vec::len).max().unwrap_or(0).max(1);

    lines as f64 * LINE_PT + 2.0 * ROW_PAD_PT
}

fn wrapped(texts: &[String], widths: &[f64], weight: Weight) -> Vec<Vec<String>> {
    texts
        .iter()
        .zip(widths)
        .map(|(text, width)| cell_lines(text, width - 2.0 * PAD_PT, weight))
        .collect()
}

/// Each column as wide as its text wants. Too wide for the sheet, a column gives up what it
/// has over its longest word, all alike; when even the words do not fit, everything shrinks
/// alike (long words then break). Room left over is shared in proportion.
fn column_widths(head: &[String], rows: &[Vec<String>], content_w: f64) -> Vec<f64> {
    let widest_word = |text: &str, weight| {
        text.split_whitespace()
            .map(|w| text_width_pt(w, CELL_PT, weight))
            .fold(0.0, f64::max)
    };
    let widest_line = |text: &str, weight| {
        text.lines()
            .map(|l| text_width_pt(l, CELL_PT, weight))
            .fold(0.0, f64::max)
    };

    let mut shortest = Vec::new();
    let mut natural = Vec::new();
    for (i, title) in head.iter().enumerate() {
        let cells = rows.iter().filter_map(|r| r.get(i));
        let word = cells
            .clone()
            .map(|c| widest_word(c, Weight::Regular))
            .fold(widest_word(title, Weight::Bold), f64::max);
        let line = cells
            .map(|c| widest_line(c, Weight::Regular))
            .fold(widest_line(title, Weight::Bold), f64::max);
        shortest.push(word + 2.0 * PAD_PT);
        natural.push(line.max(word) + 2.0 * PAD_PT);
    }

    let (short_sum, natural_sum): (f64, f64) = (shortest.iter().sum(), natural.iter().sum());
    if natural_sum <= content_w {
        return natural
            .iter()
            .map(|w| w * content_w / natural_sum)
            .collect();
    }
    if short_sum >= content_w {
        return shortest.iter().map(|w| w * content_w / short_sum).collect();
    }

    let share = (content_w - short_sum) / (natural_sum - short_sum);

    shortest
        .iter()
        .zip(&natural)
        .map(|(s, n)| s + (n - s) * share)
        .collect()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::palette::palette;
    use crate::Theme;

    /// A scene head longer than the sheet is wide wraps inside the right margin.
    #[test]
    fn a_long_extract_head_wraps() {
        let pal = palette(Theme::Light).unwrap();
        let head = "12  INT. ALTES HERRENHAUS AM SEE – GROSSER SALON MIT KAMIN – NACHT";
        let page = ExtractPage {
            title: head.repeat(2),
            facts: Vec::new(),
            boxes: Vec::new(),
        };

        let sheets = extract_sheets(&[page], pal);

        let sheet = &sheets[0];
        let right = sheet.width_pt - MARGIN_MM * PT_PER_MM;
        for prim in &sheet.prims {
            let Prim::Text {
                at,
                size_pt,
                weight,
                text,
                ..
            } = prim
            else {
                continue;
            };
            let end = at.0 + text_width_pt(text, *size_pt, *weight);
            assert!(end <= right, "{text:?} ends at {end}, margin at {right}");
        }
    }

    /// A head that fits keeps its spacing: two spaces after the number.
    #[test]
    fn a_short_extract_head_keeps_its_spacing() {
        let pal = palette(Theme::Light).unwrap();
        let page = ExtractPage {
            title: "12  INT. SALON".into(),
            facts: Vec::new(),
            boxes: Vec::new(),
        };

        let sheets = extract_sheets(&[page], pal);

        let titles: Vec<&str> = sheets[0]
            .prims
            .iter()
            .filter_map(|p| match p {
                Prim::Text { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(titles, ["12  INT. SALON"]);
    }
}
