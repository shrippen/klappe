//! A screenplay as PDF: pages of text positioned in columns and rows, in Courier. The caller
//! has laid the text out; this only sets it on paper, so export knows nothing of Fountain.
//!
//! ```text
//! title page (optional)   then per page:  header · · · · · page number
//!                                         row 0   column 0 ………………… column 59
//! ```
//!
//! One column is 1/10 inch (7.2 pt, Courier 12 pt), one row 1/6 inch (12 pt).

use std::fmt::Write;

use klappe_model::Paper;

use crate::canvas::num;
use crate::layout::{paper_mm, PT_PER_MM};
use crate::pdf::{pdf_string, write_text, TextPage};

const COLUMN_PT: f64 = 7.2;
const ROW_PT: f64 = 12.0;
const FONT_PT: f64 = 12.0;
/// Left edge of the text: 1.5 inches.
const LEFT_PT: f64 = 108.0;
/// Top of the first row: 1 inch.
const TOP_PT: f64 = 72.0;
/// Baseline of the line of page number and header: half an inch down.
const HEADER_BASELINE_PT: f64 = 46.0;
/// Distance of the baseline from the top of its row.
const ASCENT_PT: f64 = 10.0;
/// Where the title starts on the title page: a third down.
const TITLE_AT: f64 = 1.0 / 3.0;
/// Rows between the parts of the title page, and below the title.
const TITLE_GAP_ROWS: usize = 2;
/// The foot of the title page: this many points above the bottom edge.
const FOOT_PT: f64 = 72.0;
/// A scene number stands this many columns away from the text.
const NUMBER_GAP_COLUMNS: usize = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Face {
    #[default]
    Regular,
    Bold,
}

/// Text at a column and row of a page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptLine {
    /// From 0 below the top margin.
    pub row: usize,
    /// From 0 at the left edge of the text.
    pub col: usize,
    pub text: String,
    pub face: Face,
    /// The number of the scene a heading starts: printed at both sides if asked for.
    pub scene_number: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ScriptPage {
    /// Shown top right, e.g. `2.`; the first page has none.
    pub label: Option<String>,
    pub lines: Vec<ScriptLine>,
}

/// The title page, from the title page keys of the screenplay. Empty parts are left out.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TitlePage {
    pub title: String,
    pub credit: String,
    pub author: String,
    pub source: String,
    pub draft_date: String,
    pub contact: String,
}

impl TitlePage {
    fn is_empty(&self) -> bool {
        *self == TitlePage::default()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SceneNumbers {
    #[default]
    Hidden,
    Shown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptSheet {
    pub paper: Paper,
    /// Columns of the text area; the right edge and the middle of the title page follow it.
    pub columns: usize,
    pub title_page: TitlePage,
    pub pages: Vec<ScriptPage>,
    pub scene_numbers: SceneNumbers,
    /// A line at the top left of every page but the title page, e.g. the title and draft.
    pub header: Option<String>,
}

/// The paper upright, in points.
fn size_pt(paper: Paper) -> (f64, f64) {
    let (width_mm, height_mm) = paper_mm(paper);

    (width_mm * PT_PER_MM, height_mm * PT_PER_MM)
}

pub(crate) fn pdf(sheet: &ScriptSheet) -> Vec<u8> {
    let (width_pt, height_pt) = size_pt(sheet.paper);
    let mut pages = Vec::new();

    if !sheet.title_page.is_empty() {
        pages.push(TextPage {
            width_pt,
            height_pt,
            content: title_content(&sheet.title_page, sheet.columns, height_pt).into_bytes(),
        });
    }
    for page in &sheet.pages {
        pages.push(TextPage {
            width_pt,
            height_pt,
            content: page_content(sheet, page).into_bytes(),
        });
    }
    // A PDF has at least one page.
    if pages.is_empty() {
        pages.push(TextPage {
            width_pt,
            height_pt,
            content: Vec::new(),
        });
    }

    write_text(&pages)
}

/// `BT /F1 12 Tf x y Td (text) Tj ET`: `at_pt` is the baseline's distance from the top.
fn text_ops(out: &mut String, face: Face, x_pt: f64, at_pt: f64, height_pt: f64, text: &str) {
    if text.trim().is_empty() {
        return;
    }

    let font = match face {
        Face::Regular => "F1",
        Face::Bold => "F2",
    };
    let _ = writeln!(
        out,
        "BT /{font} {} Tf {} {} Td ({}) Tj ET",
        num(FONT_PT),
        num(x_pt),
        num(height_pt - at_pt),
        pdf_string(text)
    );
}

fn width_pt(text: &str) -> f64 {
    text.chars().count() as f64 * COLUMN_PT
}

/// The right edge of the text.
fn right_pt(columns: usize) -> f64 {
    LEFT_PT + columns as f64 * COLUMN_PT
}

fn page_content(sheet: &ScriptSheet, page: &ScriptPage) -> String {
    let height_pt = size_pt(sheet.paper).1;
    let mut out = String::new();

    if let Some(header) = &sheet.header {
        text_ops(
            &mut out,
            Face::Regular,
            LEFT_PT,
            HEADER_BASELINE_PT,
            height_pt,
            header,
        );
    }
    if let Some(label) = &page.label {
        let x_pt = right_pt(sheet.columns) - width_pt(label);
        text_ops(
            &mut out,
            Face::Regular,
            x_pt,
            HEADER_BASELINE_PT,
            height_pt,
            label,
        );
    }

    for line in &page.lines {
        let at_pt = TOP_PT + line.row as f64 * ROW_PT + ASCENT_PT;
        let x_pt = LEFT_PT + line.col as f64 * COLUMN_PT;
        text_ops(&mut out, line.face, x_pt, at_pt, height_pt, &line.text);

        let Some(number) = line
            .scene_number
            .as_ref()
            .filter(|_| sheet.scene_numbers == SceneNumbers::Shown)
        else {
            continue;
        };
        let gap_pt = NUMBER_GAP_COLUMNS as f64 * COLUMN_PT;
        text_ops(
            &mut out,
            line.face,
            LEFT_PT - gap_pt - width_pt(number),
            at_pt,
            height_pt,
            number,
        );
        text_ops(
            &mut out,
            line.face,
            right_pt(sheet.columns) + gap_pt,
            at_pt,
            height_pt,
            number,
        );
    }

    out
}

/// Title, credit, author and source centred a third down; contact bottom left, draft date
/// bottom right.
fn title_content(title: &TitlePage, columns: usize, height_pt: f64) -> String {
    let mut out = String::new();
    let centre_pt = LEFT_PT + width_pt(&" ".repeat(columns)) / 2.0;
    let mut at_pt = height_pt * TITLE_AT;

    let parts = [
        (title.title.to_uppercase(), Face::Bold),
        (title.credit.clone(), Face::Regular),
        (title.author.clone(), Face::Regular),
        (title.source.clone(), Face::Regular),
    ];
    for (text, face) in parts.iter().filter(|(t, _)| !t.is_empty()) {
        for line in text.lines() {
            let x_pt = centre_pt - width_pt(line) / 2.0;
            text_ops(&mut out, *face, x_pt, at_pt, height_pt, line);
            at_pt += ROW_PT;
        }
        at_pt += TITLE_GAP_ROWS as f64 * ROW_PT;
    }

    let contact: Vec<&str> = title.contact.lines().collect();
    let foot_pt = height_pt - FOOT_PT - contact.len().saturating_sub(1) as f64 * ROW_PT;
    for (i, line) in contact.iter().enumerate() {
        text_ops(
            &mut out,
            Face::Regular,
            LEFT_PT,
            foot_pt + i as f64 * ROW_PT,
            height_pt,
            line,
        );
    }

    let date = &title.draft_date;
    let x_pt = right_pt(columns) - width_pt(date);
    text_ops(
        &mut out,
        Face::Regular,
        x_pt,
        height_pt - FOOT_PT,
        height_pt,
        date,
    );

    out
}
