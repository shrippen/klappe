//! What the printed sheets of the shot list and the reports share, and the title of every page
//! (plan, camera pages included): the title wrapped inside the margins, the table head, rows,
//! rules, and the sheets under construction.
//!
//! ```text
//! ┌──────────────────────────────┐
//! │ title, wrapped to the width  │  `title_band`
//! │ ──────────────────────────── │  `table_head`, `row`, `rule`
//! │ cells …                      │
//! └──────────────────────────────┘  `next_sheet` at `bottom`
//! ```

use crate::canvas::{Anchor, Dash, Geom, Prim, Sheet, Stroke, Weight};
use crate::layout::{GAP_PT, TITLE_BAND_PT, TITLE_PT};
use crate::metrics::text_width_pt;
use crate::palette::{Palette, Rgb};
use crate::table::{cell_lines, CELL_PT, HEAD_RULE_PT, LINE_PT, PAD_PT, ROW_PAD_PT};

/// Baseline to baseline of a wrapped title.
const TITLE_LINE_PT: f64 = 17.0;
const TITLE_CLASS: &str = "title";
const CELL_CLASS: &str = "cell";

/// The title at the top of a page, inside `content_w` from `margin`, and how far down the
/// band it takes reaches from the margin. One too wide wraps; `cell_lines` measures at
/// `CELL_PT`, so the width is scaled to it. One that fits stays as it is, since wrapping folds
/// the two spaces after a scene number into one.
pub(crate) fn title_band(
    title: &str,
    margin: f64,
    content_w: f64,
    pal: &Palette,
) -> (Vec<Prim>, f64) {
    let fits = text_width_pt(title, TITLE_PT, Weight::Bold) <= content_w;
    let lines = match fits {
        true => vec![title.to_owned()],
        false => cell_lines(title, content_w * CELL_PT / TITLE_PT, Weight::Bold),
    };

    let mut y = margin + TITLE_PT;
    let mut prims = Vec::new();
    for line in lines {
        prims.push(Prim::Text {
            rotation_deg: 0.0,
            class: TITLE_CLASS.into(),
            anchor: Anchor::Start,
            at: (margin, y),
            size_pt: TITLE_PT,
            weight: Weight::Bold,
            rgb: pal.foreground,
            text: line,
        });
        y += TITLE_LINE_PT;
    }

    let extra = prims.len().saturating_sub(1) as f64 * TITLE_LINE_PT;

    (prims, TITLE_BAND_PT + extra)
}

/// Sheets under construction, drawn top down.
pub(crate) struct SheetWriter<'a> {
    pal: &'a Palette,
    width_pt: f64,
    height_pt: f64,
    margin: f64,
    bottom: f64,
    /// How far a rule reaches from the margin.
    rule_w: f64,
    /// Over every sheet, until `next_sheet` is told another.
    title: String,
    sheets: Vec<Sheet>,
    prims: Vec<Prim>,
    y: f64,
    /// Table rows on this sheet.
    rows: usize,
}

impl<'a> SheetWriter<'a> {
    pub(crate) fn new(
        pal: &'a Palette,
        (width_pt, height_pt): (f64, f64),
        margin: f64,
        title: &str,
    ) -> Self {
        SheetWriter {
            pal,
            width_pt,
            height_pt,
            margin,
            bottom: height_pt - margin,
            rule_w: width_pt - 2.0 * margin,
            title: title.to_owned(),
            sheets: Vec::new(),
            prims: Vec::new(),
            y: 0.0,
            rows: 0,
        }
    }

    pub(crate) fn palette(&self) -> &'a Palette {
        self.pal
    }

    pub(crate) fn margin(&self) -> f64 {
        self.margin
    }

    pub(crate) fn y(&self) -> f64 {
        self.y
    }

    pub(crate) fn rows(&self) -> usize {
        self.rows
    }

    /// What is left of the sheet below the cursor.
    pub(crate) fn room(&self) -> f64 {
        self.bottom - self.y
    }

    pub(crate) fn fits(&self, height: f64) -> bool {
        self.y + height <= self.bottom
    }

    pub(crate) fn advance(&mut self, dy: f64) {
        self.y += dy;
    }

    pub(crate) fn reset_rows(&mut self) {
        self.rows = 0;
    }

    pub(crate) fn set_title(&mut self, title: &str) {
        self.title = title.to_owned();
    }

    pub(crate) fn set_rule_w(&mut self, rule_w: f64) {
        self.rule_w = rule_w;
    }

    pub(crate) fn push(&mut self, prim: Prim) {
        self.prims.push(prim);
    }

    pub(crate) fn content_w(&self) -> f64 {
        self.width_pt - 2.0 * self.margin
    }

    /// The title at the top of a fresh sheet.
    pub(crate) fn begin(&mut self) {
        let (prims, band) = title_band(&self.title, self.margin, self.content_w(), self.pal);
        self.prims.extend(prims);
        self.y = self.margin + band + GAP_PT;
        self.rows = 0;
    }

    /// Closes the sheet and begins another, with the title again.
    pub(crate) fn next_sheet(&mut self) {
        self.close();
        self.begin();
    }

    pub(crate) fn close(&mut self) {
        self.sheets.push(Sheet {
            width_pt: self.width_pt,
            height_pt: self.height_pt,
            prims: std::mem::take(&mut self.prims),
        });
    }

    pub(crate) fn finish(mut self) -> Vec<Sheet> {
        self.close();

        self.sheets
    }

    pub(crate) fn text(&mut self, x: f64, y: f64, size_pt: f64, weight: Weight, text: &str) {
        self.prims.push(Prim::Text {
            rotation_deg: 0.0,
            class: CELL_CLASS.into(),
            anchor: Anchor::Start,
            at: (x, y),
            size_pt,
            weight,
            rgb: self.pal.foreground,
            text: text.to_owned(),
        });
    }

    pub(crate) fn rule(&mut self, rule_pt: f64, rgb: Rgb) {
        self.prims.push(Prim::Shape {
            class: "rule".into(),
            geom: Geom::Open(vec![
                (self.margin, self.y),
                (self.margin + self.rule_w, self.y),
            ]),
            fill: None,
            stroke: Some(Stroke {
                rgb,
                width_pt: rule_pt,
                dash: Dash::Solid,
            }),
        });
    }

    pub(crate) fn table_head(&mut self, head: &[Vec<String>], widths: &[f64]) {
        self.row(
            head,
            widths,
            Weight::Bold,
            HEAD_RULE_PT,
            self.pal.foreground,
        );
        self.rows = 0;
    }

    /// A row of wrapped cells, one column each, then a rule below; none for `rule_pt` 0.
    pub(crate) fn row(
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
        if rule_pt > 0.0 {
            self.rule(rule_pt, rule);
        }
    }
}

pub(crate) fn row_height(cells: &[Vec<String>]) -> f64 {
    let lines = cells.iter().map(Vec::len).max().unwrap_or(0).max(1);

    lines as f64 * LINE_PT + 2.0 * ROW_PAD_PT
}

pub(crate) fn wrapped(texts: &[String], widths: &[f64], weight: Weight) -> Vec<Vec<String>> {
    texts
        .iter()
        .zip(widths)
        .map(|(text, width)| match *width > 0.0 {
            true => cell_lines(text, width - 2.0 * PAD_PT, weight),
            false => Vec::new(),
        })
        .collect()
}
