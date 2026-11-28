//! The reports of DB-61 as files: a table as PDF, CSV, ODS or XLSX, and the extract of the
//! screenplay, one page per scene, as PDF. `services/` works out every text; this module
//! only sets them.
//!
//! ```text
//! ReportTable ─┬─► csv
//!              ├─► ods / xlsx   (one sheet, every cell text)
//!              └─► pdf          (A4 landscape, head row on every sheet)
//! ExtractPage ───► pdf          (A4 portrait, one page per scene)
//! ```

use crate::canvas::{Doc, Rect, Sheet};
use crate::error::ExportError;
use crate::palette::{palette, Theme};
use crate::shots::{csv_of, Kind, Table};
use crate::{ods, pdf, report_sheets, sheet, xlsx};

const LINE_END: &str = "\r\n";

/// A report as text: its title, the head row and the rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportTable {
    /// The title of the sheet; the name of the worksheet in ODS and XLSX.
    pub title: String,
    pub head: Vec<String>,
    /// Every row has as many cells as `head`.
    pub rows: Vec<Vec<String>>,
}

/// One scene of the extract (DB-61): a page of its own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractPage {
    /// The head of the page: the scene's number and heading.
    pub title: String,
    /// The facts under it, each as `Label: value`.
    pub facts: Vec<String>,
    pub boxes: Vec<ExtractBox>,
}

/// A box of a page: a heading and what stands under it, one entry per line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractBox {
    pub title: String,
    pub lines: Vec<String>,
}

impl ReportTable {
    /// Every cell is text: numbers of scenes are names (`12A`), lengths are fractions.
    fn table(&self) -> Table {
        Table {
            head: self.head.clone(),
            rows: self.rows.clone(),
            kinds: vec![Kind::Text; self.head.len()],
        }
    }
}

/// The report as CSV (RFC 4180, UTF-8, `\r\n`).
pub fn report_csv(report: &ReportTable) -> String {
    csv_of(&report.table())
}

/// The report as OpenDocument spreadsheet.
pub fn report_ods(report: &ReportTable) -> Result<Vec<u8>, ExportError> {
    tables_ods(std::slice::from_ref(report))
}

/// The report as Excel workbook.
pub fn report_xlsx(report: &ReportTable) -> Result<Vec<u8>, ExportError> {
    tables_xlsx(std::slice::from_ref(report))
}

/// The report as PDF 1.4 on A4 landscape sheets; the head row repeats on every sheet.
pub fn report_pdf(report: &ReportTable, theme: Theme) -> Result<Vec<u8>, ExportError> {
    tables_pdf(std::slice::from_ref(report), theme)
}

/// A report of several tables as CSV: each table is its title on a line, its head and its
/// rows, a blank line between two tables (a CSV file has no sheets).
pub fn tables_csv(tables: &[ReportTable]) -> String {
    let parts: Vec<String> = tables
        .iter()
        .map(|t| {
            let title = ReportTable {
                title: String::new(),
                head: vec![t.title.clone()],
                rows: Vec::new(),
            };

            report_csv(&title) + &report_csv(t)
        })
        .collect();

    parts.join(LINE_END)
}

/// A report of several tables as OpenDocument spreadsheet: a worksheet per table.
pub fn tables_ods(tables: &[ReportTable]) -> Result<Vec<u8>, ExportError> {
    let sheets: Vec<_> = tables.iter().map(worksheet).collect();

    ods::write(&sheets)
}

/// A report of several tables as Excel workbook: a worksheet per table.
pub fn tables_xlsx(tables: &[ReportTable]) -> Result<Vec<u8>, ExportError> {
    let sheets: Vec<_> = tables.iter().map(worksheet).collect();

    xlsx::write(&sheets)
}

/// A report of several tables as PDF 1.4: each table starts on a sheet of its own. No table is
/// `ExportError::EmptyPlan`.
pub fn tables_pdf(tables: &[ReportTable], theme: Theme) -> Result<Vec<u8>, ExportError> {
    if tables.is_empty() {
        return Err(ExportError::EmptyPlan);
    }
    let pal = palette(theme)?;
    let sheets = tables
        .iter()
        .flat_map(|t| report_sheets::table_sheets(t, pal))
        .collect();

    write_sheets(sheets, theme)
}

/// The extract as PDF 1.4: a page of A4 portrait for each of `pages`, more where a scene's
/// boxes do not fit one. An empty slice is `ExportError::EmptyPlan`.
pub fn extract_pdf(pages: &[ExtractPage], theme: Theme) -> Result<Vec<u8>, ExportError> {
    if pages.is_empty() {
        return Err(ExportError::EmptyPlan);
    }
    let pal = palette(theme)?;

    write_sheets(report_sheets::extract_sheets(pages, pal), theme)
}

fn worksheet(report: &ReportTable) -> sheet::Sheet<'_> {
    sheet::Sheet {
        name: &report.title,
        table: report.table(),
    }
}

/// The PDF writer takes a plan page and sheets after it; a report has sheets only, so its
/// first sheet stands in as the first page and the rest follow.
fn write_sheets(mut sheets: Vec<Sheet>, theme: Theme) -> Result<Vec<u8>, ExportError> {
    let first = sheets.remove(0);
    let doc = Doc {
        width_pt: first.width_pt,
        height_pt: first.height_pt,
        paper: palette(theme)?.paper,
        clip: Rect {
            x: 0.0,
            y: 0.0,
            w: first.width_pt,
            h: first.height_pt,
        },
        room: Rect {
            x: 0.0,
            y: 0.0,
            w: first.width_pt,
            h: first.height_pt,
        },
        plan: Vec::new(),
        at_room: Vec::new(),
        chrome: first.prims,
        pages: Vec::new(),
        sheets,
    };

    pdf::write(&doc)
}
