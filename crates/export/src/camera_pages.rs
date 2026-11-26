//! The camera views as PDF pages (LH-86): one view per shot at its start, vector faces far to
//! near, six to a page, each with the shot's number, size and lens above it.
//!
//! ```text
//! ┌──────────────────────────────────────┐
//! │ Szene 12 · Sängersaal · Plan A · Kamerablicke
//! │ 12.1 · Totale      35 │ 12.2 · Nah   85 │ …   3 × 2 (landscape),
//! │ ┌──────────────┐      │ ┌───────────┐     │     2 × 3 (portrait)
//! │ │  faces       │      │ │           │     │
//! │ └──────────────┘      │ └───────────┘     │
//! └──────────────────────────────────────┘
//! ```

use klappe_model::{FloorPlan, Shot};
use klappe_stellprobe::{axis_jumps, ordered_shots, ShotOrder};

use crate::canvas::{Anchor, Dash, Fill, Geom, Prim, Rect, Sheet, Stroke, Weight};
use crate::error::ExportError;
use crate::layout::{sheet_size_pt, GAP_PT, MIN_TEXT_PT, PT_PER_MM};
use crate::metrics::text_width_pt;
use crate::page::{Page, ShotView};
use crate::palette::Palette;
use crate::sheet_writer::title_band;
use crate::shots::cells;
use crate::style::{scene_style, shaded};
use klappe_model::Orientation;

/// Views to a page, in a grid of `LONG_SIDE` along the longer edge and `SHORT_SIDE` across.
const LONG_SIDE: usize = 3;
const SHORT_SIDE: usize = 2;
const CAPTION_PT: f64 = MIN_TEXT_PT;
/// Room the caption takes above its view.
const CAPTION_BAND_PT: f64 = CAPTION_PT + 4.0;
const FRAME_PT: f64 = 0.5;
const FACE_LINE_PT: f64 = 0.25;
const LENS_UNIT: &str = "mm";
const CAPTION_SEPARATOR: &str = " · ";
const TITLE_SEPARATOR: &str = " · ";
/// Columns of the shot list's cells: number, lens and size (`shots::heads`).
const NUMBER_CELL: usize = 0;
const LENS_CELL: usize = 2;
const SIZE_CELL: usize = 3;

pub(crate) fn sheets(
    plan: &FloorPlan,
    page: &Page,
    pal: &Palette,
    order: ShotOrder,
    views: &[ShotView],
) -> Result<Vec<Sheet>, ExportError> {
    let ordered = ordered_shots(plan, order);
    let shown: Vec<(&Shot, &ShotView)> = ordered
        .iter()
        .filter_map(|(_, shot)| {
            let view = views.iter().find(|v| v.shot == shot.id)?;

            Some((*shot, view))
        })
        .collect();
    if shown.is_empty() {
        return Ok(Vec::new());
    }

    let (width_pt, height_pt) = sheet_size_pt(page);
    let (columns, rows) = match page.orientation {
        Orientation::Landscape => (LONG_SIDE, SHORT_SIDE),
        Orientation::Portrait => (SHORT_SIDE, LONG_SIDE),
    };
    let margin = page.margin_mm.max(0.0) * PT_PER_MM;
    let texts = &page.texts;
    let title = match texts.shots.views.is_empty() {
        true => page.title.clone(),
        false => format!("{}{TITLE_SEPARATOR}{}", page.title, texts.shots.views),
    };
    let (title_prims, title_h) = title_band(&title, margin, width_pt - 2.0 * margin, pal);
    let top = margin + title_h + GAP_PT;
    let cell_w = (width_pt - 2.0 * margin - (columns - 1) as f64 * GAP_PT) / columns as f64;
    let cell_h = (height_pt - margin - top - (rows - 1) as f64 * GAP_PT) / rows as f64;
    if cell_w <= 0.0 || cell_h <= CAPTION_BAND_PT {
        return Err(ExportError::PageTooSmall);
    }

    let jumps = axis_jumps(plan);

    let mut sheets = Vec::new();
    for group in shown.chunks(columns * rows) {
        let mut prims = title_prims.clone();

        for (i, (shot, view)) in group.iter().enumerate() {
            let cell = Rect {
                x: margin + (i % columns) as f64 * (cell_w + GAP_PT),
                y: top + (i / columns) as f64 * (cell_h + GAP_PT),
                w: cell_w,
                h: cell_h,
            };
            let row = cells(shot, plan, &jumps, &texts.shots, texts.decimal_separator);
            let (name, lens) = (
                format!("{}{CAPTION_SEPARATOR}{}", row[NUMBER_CELL], row[SIZE_CELL]),
                format!("{} {LENS_UNIT}", row[LENS_CELL]),
            );
            caption(&mut prims, cell, &name, &lens, pal);
            picture(&mut prims, cell, view, pal)?;
        }

        sheets.push(Sheet {
            width_pt,
            height_pt,
            prims,
        });
    }

    Ok(sheets)
}

/// The shot's number and size on the left, its lens on the right, above the view.
fn caption(prims: &mut Vec<Prim>, cell: Rect, name: &str, lens: &str, pal: &Palette) {
    let text = |x: f64, text: &str| Prim::Text {
        rotation_deg: 0.0,
        class: "caption".into(),
        anchor: Anchor::Start,
        at: (x, cell.y + CAPTION_PT),
        size_pt: CAPTION_PT,
        weight: Weight::Bold,
        rgb: pal.foreground,
        text: text.to_owned(),
    };

    let lens_w = text_width_pt(lens, CAPTION_PT, Weight::Bold);
    prims.push(text(cell.x, name));
    prims.push(text(cell.x + cell.w - lens_w, lens));
}

/// The view's faces in a frame as large as `cell` below the caption allows, centred.
fn picture(
    prims: &mut Vec<Prim>,
    cell: Rect,
    shot_view: &ShotView,
    pal: &Palette,
) -> Result<(), ExportError> {
    let view = &shot_view.view;
    let room = (cell.w, cell.h - CAPTION_BAND_PT);
    let width = room.0.min(room.1 / view.height);
    let (x, y) = (cell.x + (room.0 - width) / 2.0, cell.y + CAPTION_BAND_PT);

    prims.push(Prim::Shape {
        class: "view-frame".into(),
        geom: Geom::Rect(Rect {
            x,
            y,
            w: width,
            h: view.height * width,
        }),
        fill: None,
        stroke: Some(Stroke {
            rgb: pal.muted,
            width_pt: FRAME_PT,
            dash: Dash::Solid,
        }),
    });

    for (i, face) in view.faces.iter().enumerate() {
        let strength = shot_view.strength.get(i).copied().unwrap_or(1.0);
        let style = scene_style(face.role, pal)?;
        let rgb = shaded(style.face, face.shade);
        let corners = face
            .corners
            .iter()
            .map(|c| (x + c[0] * width, y + c[1] * width))
            .collect();

        prims.push(Prim::Shape {
            class: "view-face".into(),
            geom: Geom::Closed(corners),
            fill: Some(Fill {
                rgb,
                opacity: strength,
            }),
            // A hairline in the role's line colour: outlines, and no seams between faces.
            stroke: Some(Stroke {
                rgb: style.line.faded(pal.paper, strength),
                width_pt: FACE_LINE_PT,
                dash: Dash::Solid,
            }),
        });
    }

    Ok(())
}
