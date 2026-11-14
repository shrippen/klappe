//! Writes Stellprobe's display list as SVG and PDF, and the shot list as PDF and CSV (planung/KONZEPT.md, section 4).
//! The GUI draws the same list, so the printout matches the screen (LH-80/81).
//!
//! ```text
//! DisplayList ─► layout (page, scale, legend) ─► Doc ─┬─► svg
//!                                                     └─► pdf
//! ```
//!
//! Colours come from Kante's `palette.json` (`scripts/sync-kante.sh`), never from Rust.

mod canvas;
mod error;
mod layout;
mod metrics;
mod page;
mod palette;
mod pdf;
mod shots;
mod style;
mod svg;
mod table;

use klappe_model::FloorPlan;

pub use error::ExportError;
pub use klappe_stellprobe::DisplayList;
pub use page::{Orientation, Page, Paper, ShotColumns, ShotTexts, Texts};
pub use palette::Theme;

/// The plan on one page as SVG.
pub fn svg(list: &DisplayList, page: &Page, theme: Theme) -> Result<String, ExportError> {
    Ok(svg::write(&layout::compose(list, page, theme)?))
}

/// The plan on one page as PDF 1.4.
pub fn pdf(list: &DisplayList, page: &Page, theme: Theme) -> Result<Vec<u8>, ExportError> {
    Ok(pdf::write(&layout::compose(list, page, theme)?))
}

/// The plan as PDF 1.4, followed by the plan's shot list on upright sheets (LH-80).
pub fn pdf_with_shots(
    list: &DisplayList,
    page: &Page,
    theme: Theme,
    plan: &FloorPlan,
) -> Result<Vec<u8>, ExportError> {
    let mut doc = layout::compose(list, page, theme)?;
    doc.sheets = table::sheets(plan, page, palette::palette(theme)?)?;

    Ok(pdf::write(&doc))
}

/// The plan's shots in cut order as CSV (LH-82): RFC 4180, UTF-8, `\r\n`, numbers with a point.
pub fn shots_csv(plan: &FloorPlan, texts: &ShotTexts) -> String {
    shots::csv(plan, texts)
}
