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

use crate::canvas::{Dash, Geom, Prim, Rect, Sheet, Stroke, Weight};
use crate::layout::{paper_mm, GAP_PT, PT_PER_MM};
use crate::metrics::text_width_pt;
use crate::palette::Palette;
use crate::report::{ExtractPage, ReportTable};
use crate::sheet_writer::{row_height, wrapped, SheetWriter};
use crate::table::{cell_lines, CELL_PT, LINE_PT, PAD_PT, ROW_PAD_PT, ROW_RULE_PT};
use crate::Paper;

const MARGIN_MM: f64 = 10.0;
const FACT_SEPARATOR: &str = "  ·  ";

pub(crate) fn table_sheets(table: &ReportTable, pal: &Palette) -> Vec<Sheet> {
    let mut out = writer(pal, Orientation::Landscape, &table.title);
    out.begin();
    let widths = column_widths(&table.head, &table.rows, out.content_w());
    let head = wrapped(&table.head, &widths, Weight::Bold);

    out.table_head(&head, &widths);
    for row in &table.rows {
        let lines = wrapped(row, &widths, Weight::Regular);
        if !out.fits(row_height(&lines)) && out.rows() > 0 {
            out.next_sheet();
            out.table_head(&head, &widths);
        }
        out.row(&lines, &widths, Weight::Regular, ROW_RULE_PT, pal.muted);
    }

    out.finish()
}

pub(crate) fn extract_sheets(pages: &[ExtractPage], pal: &Palette) -> Vec<Sheet> {
    let mut out = writer(pal, Orientation::Portrait, "");

    for (i, page) in pages.iter().enumerate() {
        out.set_title(&page.title);
        match i {
            0 => out.begin(),
            _ => out.next_sheet(),
        }
        facts(&mut out, &page.facts);
        for entry in &page.boxes {
            boxed(&mut out, &entry.title, &entry.lines);
        }
    }

    out.finish()
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Orientation {
    Landscape,
    Portrait,
}

fn writer<'a>(pal: &'a Palette, orientation: Orientation, title: &str) -> SheetWriter<'a> {
    let (short_mm, long_mm) = paper_mm(Paper::A4);
    let (w_mm, h_mm) = match orientation {
        Orientation::Landscape => (long_mm, short_mm),
        Orientation::Portrait => (short_mm, long_mm),
    };
    let size = (w_mm * PT_PER_MM, h_mm * PT_PER_MM);

    SheetWriter::new(pal, size, MARGIN_MM * PT_PER_MM, title)
}

/// The facts of a scene on lines under its head, `·` between them.
fn facts(out: &mut SheetWriter, facts: &[String]) {
    let joined = facts.join(FACT_SEPARATOR);
    for line in cell_lines(&joined, out.content_w() - 2.0 * PAD_PT, Weight::Regular) {
        out.text(
            out.margin() + PAD_PT,
            out.y() + CELL_PT,
            CELL_PT,
            Weight::Regular,
            &line,
        );
        out.advance(LINE_PT);
    }
    out.advance(GAP_PT);
}

/// A box with `title` over `lines`; one that does not fit what is left of the sheet goes on
/// in a box of the next sheet, which has the same title.
fn boxed(out: &mut SheetWriter, title: &str, entries: &[String]) {
    let max_w = out.content_w() - 4.0 * PAD_PT;
    let lines: Vec<String> = entries
        .iter()
        .flat_map(|e| cell_lines(e, max_w, Weight::Regular))
        .collect();
    let mut rest = lines.as_slice();

    loop {
        let room = out.room() - 2.0 * ROW_PAD_PT - LINE_PT;
        let fits = (room / LINE_PT).floor().max(0.0) as usize;
        if fits < rest.len().min(1) {
            out.next_sheet();
            continue;
        }

        let (now, later) = rest.split_at(fits.min(rest.len()));
        draw_box(out, title, now);
        rest = later;
        if rest.is_empty() {
            return;
        }
        out.next_sheet();
    }
}

fn draw_box(out: &mut SheetWriter, title: &str, lines: &[String]) {
    let x = out.margin() + 2.0 * PAD_PT;
    let height = (lines.len() + 1) as f64 * LINE_PT + 2.0 * ROW_PAD_PT;
    out.push(Prim::Shape {
        class: "box".into(),
        geom: Geom::Rect(Rect {
            x: out.margin(),
            y: out.y(),
            w: out.content_w(),
            h: height,
        }),
        fill: None,
        stroke: Some(Stroke {
            rgb: out.palette().muted,
            width_pt: ROW_RULE_PT,
            dash: Dash::Solid,
        }),
    });

    let top = out.y() + ROW_PAD_PT + CELL_PT;
    out.text(x, top, CELL_PT, Weight::Bold, title);
    for (k, line) in lines.iter().enumerate() {
        out.text(
            x,
            top + (k + 1) as f64 * LINE_PT,
            CELL_PT,
            Weight::Regular,
            line,
        );
    }
    out.advance(height + GAP_PT);
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
