//! A screenplay as PDF: pages of text positioned in columns and rows, in Courier. The caller
//! has laid the text out; this only sets it on paper, so export knows nothing of Fountain.
//! Where each text stands (`sheet_pages`) is apart from the PDF, so a surface can show the pages
//! as they print (D-15).
//!
//! ```text
//! title page (optional)   then per page:  header · · · · · page number
//!                                         row 0   column 0 ………………… column 59
//!
//! ScriptSheet ─► sheet_pages (texts in points) ─┬─► PDF
//!                                                └─► the editor's print view
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
/// The size of the type, in points.
pub const FONT_PT: f64 = 12.0;
/// Left edge of the text: 1.5 inches.
const LEFT_PT: f64 = 108.0;
/// Top of the first row: 1 inch.
const TOP_PT: f64 = 72.0;
/// Baseline of the line of page number and header: half an inch down.
const HEADER_BASELINE_PT: f64 = 46.0;
/// Distance of the baseline from the top of its row.
pub const ASCENT_PT: f64 = 10.0;
/// Where the title starts on the title page: a third down.
const TITLE_AT: f64 = 1.0 / 3.0;
/// Rows between the parts of the title page, and below the title.
const TITLE_GAP_ROWS: usize = 2;
/// The foot of the title page: this many points above the bottom edge.
const FOOT_PT: f64 = 72.0;
/// A scene number stands this many columns away from the text.
const NUMBER_GAP_COLUMNS: usize = 1;
/// The revision mark ends this far (10 mm) from the paper's right edge, where a printer
/// still reaches.
const REVISION_MARK_EDGE_PT: f64 = 10.0 * PT_PER_MM;
/// An underline is this thick and this far below the baseline, in points.
const UNDERLINE_PT: f64 = 0.6;
const UNDERLINE_DROP_PT: f64 = 1.5;
/// What marks a line of a revision in the margin.
const REVISION_MARK: &str = "*";

/// Whether a line is new or changed in the revision (DB-41): it gets a mark in the right margin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Revised {
    #[default]
    No,
    Yes,
}

/// The Courier of a text: the four standard faces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Face {
    #[default]
    Regular,
    Bold,
    Italic,
    BoldItalic,
}

impl Face {
    pub fn is_bold(self) -> bool {
        matches!(self, Face::Bold | Face::BoldItalic)
    }

    pub fn is_italic(self) -> bool {
        matches!(self, Face::Italic | Face::BoldItalic)
    }
}

/// Whether a text has a line under it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Underline {
    #[default]
    No,
    Yes,
}

/// The characters `start..end` of a line (not bytes) in one face.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScriptRun {
    pub start: usize,
    pub end: usize,
    pub face: Face,
    pub underline: Underline,
}

/// Text at a column and row of a page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptLine {
    /// From 0 below the top margin.
    pub row: usize,
    /// From 0 at the left edge of the text.
    pub col: usize,
    pub text: String,
    /// The face of the whole text, where `runs` is empty.
    pub face: Face,
    /// Sections of the text in a face of their own, ascending; text between them has `face`.
    pub runs: Vec<ScriptRun>,
    /// The number of the scene a heading starts: printed at both sides if asked for.
    pub scene_number: Option<String>,
    pub revised: Revised,
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
    /// A line above the page number of every page but the title page, e.g. `Revision Blau`.
    pub revision: Option<String>,
}

/// A text on a page: `x_pt` from the left edge, its baseline `baseline_pt` below the top.
#[derive(Debug, Clone, PartialEq)]
pub struct Placed {
    pub x_pt: f64,
    pub baseline_pt: f64,
    pub face: Face,
    pub underline: Underline,
    pub text: String,
}

/// A page as it prints, upright, in points.
#[derive(Debug, Clone, PartialEq)]
pub struct SheetPage {
    pub width_pt: f64,
    pub height_pt: f64,
    pub texts: Vec<Placed>,
}

/// The paper upright, in points.
fn size_pt(paper: Paper) -> (f64, f64) {
    let (width_mm, height_mm) = paper_mm(paper);

    (width_mm * PT_PER_MM, height_mm * PT_PER_MM)
}

/// The pages the PDF prints: the title page if it has a part, then the pages of text; at
/// least one page.
pub(crate) fn sheet_pages(sheet: &ScriptSheet) -> Vec<SheetPage> {
    let (width_pt, height_pt) = size_pt(sheet.paper);
    let page = |texts| SheetPage {
        width_pt,
        height_pt,
        texts,
    };
    let mut pages = Vec::new();

    if !sheet.title_page.is_empty() {
        pages.push(page(title_texts(
            &sheet.title_page,
            sheet.columns,
            height_pt,
        )));
    }
    for one in &sheet.pages {
        pages.push(page(page_texts(sheet, one)));
    }
    if pages.is_empty() {
        pages.push(page(Vec::new()));
    }

    pages
}

pub(crate) fn pdf(sheet: &ScriptSheet) -> Vec<u8> {
    let pages: Vec<TextPage> = sheet_pages(sheet)
        .into_iter()
        .map(|page| TextPage {
            width_pt: page.width_pt,
            height_pt: page.height_pt,
            content: content(&page).into_bytes(),
        })
        .collect();

    write_text(&pages)
}

/// The texts of a page as PDF operators.
fn content(page: &SheetPage) -> String {
    let mut out = String::new();
    for placed in &page.texts {
        text_ops(&mut out, placed, page.height_pt);
    }

    out
}

/// `BT /F1 12 Tf x y Td (text) Tj ET`: `at_pt` is the baseline's distance from the top.
fn text_ops(out: &mut String, placed: &Placed, height_pt: f64) {
    let font = match placed.face {
        Face::Regular => "F1",
        Face::Bold => "F2",
        Face::Italic => "F3",
        Face::BoldItalic => "F4",
    };
    let y_pt = height_pt - placed.baseline_pt;
    let _ = writeln!(
        out,
        "BT /{font} {} Tf {} {} Td ({}) Tj ET",
        num(FONT_PT),
        num(placed.x_pt),
        num(y_pt),
        pdf_string(&placed.text)
    );

    if placed.underline == Underline::No {
        return;
    }

    // A line under the characters, not under the spaces after them.
    let under_pt = y_pt - UNDERLINE_DROP_PT;
    let _ = writeln!(
        out,
        "{} w {} {} m {} {} l S",
        num(UNDERLINE_PT),
        num(placed.x_pt),
        num(under_pt),
        num(placed.x_pt + width_pt(placed.text.trim_end())),
        num(under_pt)
    );
}

/// Adds `text` at `x_pt` with its baseline `at_pt` below the top; blank text prints nothing.
fn place(out: &mut Vec<Placed>, face: Face, x_pt: f64, at_pt: f64, text: &str) {
    place_run(out, (face, Underline::No), x_pt, at_pt, text);
}

fn place_run(
    out: &mut Vec<Placed>,
    (face, underline): (Face, Underline),
    x_pt: f64,
    at_pt: f64,
    text: &str,
) {
    if text.trim().is_empty() {
        return;
    }

    out.push(Placed {
        x_pt,
        baseline_pt: at_pt,
        face,
        underline,
        text: text.to_owned(),
    });
}

/// The text of a line: one piece per run, each at its column.
fn place_line(out: &mut Vec<Placed>, line: &ScriptLine, x_pt: f64, at_pt: f64) {
    if line.runs.is_empty() {
        place(out, line.face, x_pt, at_pt, &line.text);
        return;
    }

    for run in &line.runs {
        let part: String = line
            .text
            .chars()
            .skip(run.start)
            .take(run.end - run.start)
            .collect();
        let run_pt = x_pt + run.start as f64 * COLUMN_PT;
        place_run(out, (run.face, run.underline), run_pt, at_pt, &part);
    }
}

fn width_pt(text: &str) -> f64 {
    text.chars().count() as f64 * COLUMN_PT
}

/// The right edge of the text.
fn right_pt(columns: usize) -> f64 {
    LEFT_PT + columns as f64 * COLUMN_PT
}

/// Where a revision mark stands: right of the text, and of the scene number if the line has
/// one. Where that would end within 10 mm of the paper's edge, it stands left of the number
/// on the left instead, so it is printed and covers nothing.
fn mark_x_pt(sheet: &ScriptSheet, number: Option<&String>) -> f64 {
    let gap_pt = NUMBER_GAP_COLUMNS as f64 * COLUMN_PT;
    let beside_pt = right_pt(sheet.columns) + gap_pt + number.map_or(0.0, |n| width_pt(n) + gap_pt);
    let limit_pt = size_pt(sheet.paper).0 - REVISION_MARK_EDGE_PT;

    match number {
        Some(n) if beside_pt + COLUMN_PT > limit_pt => {
            LEFT_PT - gap_pt - width_pt(n) - gap_pt - COLUMN_PT
        }
        _ => beside_pt,
    }
}

fn page_texts(sheet: &ScriptSheet, page: &ScriptPage) -> Vec<Placed> {
    let mut out = Vec::new();

    if let Some(header) = &sheet.header {
        place(&mut out, Face::Regular, LEFT_PT, HEADER_BASELINE_PT, header);
    }
    if let Some(revision) = &sheet.revision {
        let x_pt = right_pt(sheet.columns) - width_pt(revision);
        let at_pt = HEADER_BASELINE_PT - ROW_PT;
        place(&mut out, Face::Regular, x_pt, at_pt, revision);
    }
    if let Some(label) = &page.label {
        let x_pt = right_pt(sheet.columns) - width_pt(label);
        place(&mut out, Face::Regular, x_pt, HEADER_BASELINE_PT, label);
    }

    for line in &page.lines {
        let at_pt = TOP_PT + line.row as f64 * ROW_PT + ASCENT_PT;
        let x_pt = LEFT_PT + line.col as f64 * COLUMN_PT;
        place_line(&mut out, line, x_pt, at_pt);

        let number = line
            .scene_number
            .as_ref()
            .filter(|_| sheet.scene_numbers == SceneNumbers::Shown);
        if line.revised == Revised::Yes {
            let mark_pt = mark_x_pt(sheet, number);
            place(&mut out, Face::Regular, mark_pt, at_pt, REVISION_MARK);
        }

        let Some(number) = number else {
            continue;
        };
        let gap_pt = NUMBER_GAP_COLUMNS as f64 * COLUMN_PT;
        place(
            &mut out,
            line.face,
            LEFT_PT - gap_pt - width_pt(number),
            at_pt,
            number,
        );
        place(
            &mut out,
            line.face,
            right_pt(sheet.columns) + gap_pt,
            at_pt,
            number,
        );
    }

    out
}

/// Title, credit, author and source centred a third down; contact bottom left, draft date
/// bottom right.
fn title_texts(title: &TitlePage, columns: usize, height_pt: f64) -> Vec<Placed> {
    let mut out = Vec::new();
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
            place(&mut out, *face, x_pt, at_pt, line);
            at_pt += ROW_PT;
        }
        at_pt += TITLE_GAP_ROWS as f64 * ROW_PT;
    }

    let contact: Vec<&str> = title.contact.lines().collect();
    let foot_pt = height_pt - FOOT_PT - contact.len().saturating_sub(1) as f64 * ROW_PT;
    for (i, line) in contact.iter().enumerate() {
        place(
            &mut out,
            Face::Regular,
            LEFT_PT,
            foot_pt + i as f64 * ROW_PT,
            line,
        );
    }

    let date = &title.draft_date;
    let x_pt = right_pt(columns) - width_pt(date);
    place(&mut out, Face::Regular, x_pt, height_pt - FOOT_PT, date);

    out
}
