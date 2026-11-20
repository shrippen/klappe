//! Writes Stellprobe's display list as SVG, PDF and PNG, the shot list as PDF, CSV, ODS and XLSX, and the views of the set (PNG, PDF pages) (planung/KONZEPT.md, section 4).
//! The GUI draws the same list, so the printout matches the screen (LH-80/81).
//!
//! ```text
//! DisplayList ─► layout (page, scale, legend) ─► Doc ─┬─► svg
//!                                                     ├─► pdf
//!                                                     └─► raster ─► png
//! ```
//!
//! Colours come from Kante's `palette.json` (`scripts/sync-kante.sh`), never from Rust.

mod camera_pages;
mod canvas;
mod deflate;
mod error;
mod font;
mod image;
mod inflate;
mod jpeg;
mod lamps;
mod layout;
mod metrics;
mod ods;
mod page;
mod palette;
mod pdf;
mod picture_png;
mod png;
mod png_write;
mod raster;
mod rgb8;
mod scan;
mod script;
mod sheet;
mod shots;
mod stroke;
mod style;
mod svg;
mod table;
mod video;
mod xlsx;
mod zip;

use klappe_model::FloorPlan;

pub use error::ExportError;
pub use klappe_model::{Orientation, Paper};
pub use klappe_stellprobe::{DisplayList, Lamp, Picture, ShotOrder, VectorView};
pub use page::{
    DrawingTexts, LampColumns, LampTexts, Page, ShotColumns, ShotTexts, ShotView, Texts,
};
pub use palette::Theme;
pub use script::{
    Face, Placed, SceneNumbers, ScriptLine, ScriptPage, ScriptSheet, SheetPage, TitlePage,
    ASCENT_PT as SCRIPT_ASCENT_PT, FONT_PT as SCRIPT_FONT_PT,
};
pub use video::{video_program_found, VideoSpec, VideoWriter, FFMPEG};

/// The plan on one page as SVG.
pub fn svg(list: &DisplayList, page: &Page, theme: Theme) -> Result<String, ExportError> {
    Ok(svg::write(&layout::compose(list, page, theme)?))
}

/// The plan on one page as PDF 1.4.
pub fn pdf(list: &DisplayList, page: &Page, theme: Theme) -> Result<Vec<u8>, ExportError> {
    pdf::write(&layout::compose(list, page, theme)?)
}

/// Every plan of `pages` on a page of its own, as PDF 1.4 (LH-87): one page per shot or camera,
/// each with its own list and sheet; an empty slice is `ExportError::EmptyPlan`.
pub fn pdf_pages(pages: &[(DisplayList, Page)], theme: Theme) -> Result<Vec<u8>, ExportError> {
    let mut docs = pages
        .iter()
        .map(|(list, page)| layout::compose(list, page, theme));
    let mut first = docs.next().ok_or(ExportError::EmptyPlan)??;
    for doc in docs {
        first.pages.push(doc?);
    }

    pdf::write(&first)
}

/// The plan on one page as PNG: RGB, 150 dpi, drawn by this crate's own rasteriser.
pub fn png(list: &DisplayList, page: &Page, theme: Theme) -> Result<Vec<u8>, ExportError> {
    let surface = raster::render(&layout::compose(list, page, theme)?)?;

    Ok(png_write::encode(
        surface.width,
        surface.height,
        &surface.pixels,
        raster::RESOLUTION_DPI,
    ))
}

/// One picture as raw pixels: RGB, row by row, `3 * width * height` bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

/// The plan on one page as pixels, as `png` draws it, for a video (LH-84).
pub fn frame(list: &DisplayList, page: &Page, theme: Theme) -> Result<Frame, ExportError> {
    let surface = raster::render(&layout::compose(list, page, theme)?)?;

    Ok(Frame {
        width: surface.width,
        height: surface.height,
        pixels: surface.pixels,
    })
}

/// The plan as PDF 1.4, followed by the plan's shot list on sheets of the page's paper and orientation (LH-80), in
/// `order`; in shooting order each setup starts under its own heading.
pub fn pdf_with_shots(
    list: &DisplayList,
    page: &Page,
    theme: Theme,
    plan: &FloorPlan,
    order: ShotOrder,
) -> Result<Vec<u8>, ExportError> {
    pdf_with_views(list, page, theme, plan, order, &[])
}

/// `pdf_with_shots`, then the pages of camera views (LH-86): each shot of `views` at its
/// start, in `order`, six to a page in the page's paper and orientation; a shot without a
/// view is left out.
pub fn pdf_with_views(
    list: &DisplayList,
    page: &Page,
    theme: Theme,
    plan: &FloorPlan,
    order: ShotOrder,
    views: &[ShotView],
) -> Result<Vec<u8>, ExportError> {
    let pal = palette::palette(theme)?;
    let mut doc = layout::compose(list, page, theme)?;
    doc.sheets = table::sheets(plan, page, pal, order)?;
    doc.sheets
        .extend(camera_pages::sheets(plan, page, pal, order, views)?);

    pdf::write(&doc)
}

/// A screenplay laid out by the caller, as PDF 1.4 in Courier (a standard font, nothing
/// embedded): the title page when it has anything, then the pages.
pub fn script_pdf(sheet: &ScriptSheet) -> Vec<u8> {
    script::pdf(sheet)
}

/// The pages `script_pdf` prints, each text where it stands on the paper, for a surface that
/// shows them (D-15).
pub fn script_sheet_pages(sheet: &ScriptSheet) -> Vec<SheetPage> {
    script::sheet_pages(sheet)
}

/// `files` (name with `/`, bytes) as one ZIP archive, stored uncompressed: a folder handed on
/// whole, e.g. a project archived beside its folder. Refuses 4 GiB or more and more than 65535 files (`ZipTooBig`).
pub fn zip_files(files: &[(&str, &[u8])]) -> Result<Vec<u8>, ExportError> {
    let mut zip = zip::Zip::default();
    for (name, bytes) in files {
        zip.add(name, bytes)?;
    }

    zip.finish()
}

/// Whether `entries` files of `bytes` in all fit a ZIP archive (`zip_files`), so a caller can
/// ask before it reads them.
pub fn zip_fits(entries: usize, bytes: u64) -> Result<(), ExportError> {
    zip::fits(entries, bytes)
}

/// A view of the set as PNG (LH-86), as wide as the picture: print colours, names written in.
/// `strength` is how much of its colour pixel `i` keeps, 1 for all of it.
pub fn picture_png(
    picture: &Picture,
    theme: Theme,
    strength: &dyn Fn(usize) -> f64,
) -> Result<Vec<u8>, ExportError> {
    picture_png::write(picture, theme, strength)
}

/// The plan's shots in `order` as CSV (LH-82): RFC 4180, UTF-8, `\r\n`, numbers with a point.
pub fn shots_csv(plan: &FloorPlan, texts: &ShotTexts, order: ShotOrder) -> String {
    shots::csv(plan, texts, order)
}

/// The plan's shots in `order` as OpenDocument spreadsheet (LH-85): the CSV's columns, numbers as
/// numbers; `texts.title` names the sheet. With `lamps` a second sheet lists them (LH-50), named
/// `words.title`.
pub fn shots_ods(
    plan: &FloorPlan,
    texts: &ShotTexts,
    order: ShotOrder,
    lamps: &[Lamp],
    words: &LampTexts,
) -> Result<Vec<u8>, ExportError> {
    ods::write(&sheets(plan, texts, order, lamps, words))
}

/// The plan's shots in `order` as Excel workbook (LH-85), the same as `shots_ods`.
pub fn shots_xlsx(
    plan: &FloorPlan,
    texts: &ShotTexts,
    order: ShotOrder,
    lamps: &[Lamp],
    words: &LampTexts,
) -> Result<Vec<u8>, ExportError> {
    xlsx::write(&sheets(plan, texts, order, lamps, words))
}

/// The sheets of a workbook: the shot list, and the lamp list when the plan has lamps.
fn sheets<'a>(
    plan: &FloorPlan,
    texts: &'a ShotTexts,
    order: ShotOrder,
    lamps: &[Lamp],
    words: &'a LampTexts,
) -> Vec<sheet::Sheet<'a>> {
    let mut sheets = vec![sheet::Sheet {
        name: &texts.title,
        table: shots::table(plan, texts, order),
    }];
    if !lamps.is_empty() {
        sheets.push(sheet::Sheet {
            name: &words.title,
            table: lamps::table(lamps, words),
        });
    }

    sheets
}

#[cfg(test)]
mod png_tests;
