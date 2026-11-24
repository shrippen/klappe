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
    ods::write(&[worksheet(report)])
}

/// The report as Excel workbook.
pub fn report_xlsx(report: &ReportTable) -> Result<Vec<u8>, ExportError> {
    xlsx::write(&[worksheet(report)])
}

/// The report as PDF 1.4 on A4 landscape sheets; the head row repeats on every sheet.
pub fn report_pdf(report: &ReportTable, theme: Theme) -> Result<Vec<u8>, ExportError> {
    let pal = palette(theme)?;

    write_sheets(report_sheets::table_sheets(report, pal), theme)
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
        plan: Vec::new(),
        chrome: first.prims,
        pages: Vec::new(),
        sheets,
    };

    pdf::write(&doc)
}
