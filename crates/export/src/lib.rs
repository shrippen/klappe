//! Writes Stellprobe's display list as SVG and PDF (planung/KONZEPT.md, section 4).
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
mod page;
mod palette;
mod pdf;
mod style;
mod svg;

pub use error::ExportError;
pub use klappe_stellprobe::DisplayList;
pub use page::{Orientation, Page, Paper, Texts};
pub use palette::Theme;

/// The plan on one page as SVG.
pub fn svg(list: &DisplayList, page: &Page, theme: Theme) -> Result<String, ExportError> {
    Ok(svg::write(&layout::compose(list, page, theme)?))
}

/// The plan on one page as PDF 1.4.
pub fn pdf(list: &DisplayList, page: &Page, theme: Theme) -> Result<Vec<u8>, ExportError> {
    Ok(pdf::write(&layout::compose(list, page, theme)?))
}
