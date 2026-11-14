//! Places the plan on the page: title on top, plan in the middle at one scale,
//! scale bar and legend below.
//!
//! ```text
//! ┌─────────────────────────────┐
//! │ title                       │
//! │ ┌─────────────────────────┐ │
//! │ │ plan (aspect kept)      │ │
//! │ └─────────────────────────┘ │
//! │ ├──────┤ 5 m                │
//! │ ● Figur  ▲ Kamera  ● Licht …│
//! └─────────────────────────────┘
//! ```

use klappe_model::Point;
use klappe_stellprobe::{DisplayList, Item, Role, Shape};

use crate::canvas::{Anchor, Dash, Doc, Geom, Prim, Pt, Rect, Stroke, Weight};
use crate::error::ExportError;
use crate::page::{Orientation, Page, Paper, Texts};
use crate::palette::{palette, Palette, Theme};
use crate::style::{role_style, RoleStyle};
use klappe_model::FigureColor;

pub(crate) const PT_PER_MM: f64 = 72.0 / 25.4;
/// Print text never goes below this.
pub(crate) const MIN_TEXT_PT: f64 = 8.0;
const LABEL_PT: f64 = MIN_TEXT_PT;
pub(crate) const TITLE_PT: f64 = 14.0;
const LEGEND_PT: f64 = MIN_TEXT_PT;
/// Gap between a shape and its label.
const LABEL_GAP_PT: f64 = 3.0;
/// Baseline sits this fraction of the font size below the text's middle.
const BASELINE_SHIFT: f64 = 0.35;
pub(crate) const TITLE_BAND_PT: f64 = 24.0;
const SCALE_ROW_PT: f64 = 26.0;
const LEGEND_ROW_PT: f64 = 16.0;
const LEGEND_ENTRY_PT: f64 = 84.0;
const SWATCH_PT: f64 = 18.0;
const SWATCH_RADIUS_PT: f64 = 4.0;
const SWATCH_TEXT_GAP_PT: f64 = 5.0;
pub(crate) const GAP_PT: f64 = 6.0;
/// A thin wall must still show.
const MIN_WALL_PT: f64 = 1.0;
const FRAME_PT: f64 = 0.75;
const SCALE_TICK_PT: f64 = 4.0;
const SCALE_BAR_PT: f64 = 1.5;
/// The bar takes at most this much of the page width.
const SCALE_BAR_MAX_PT: f64 = 160.0;
/// Round lengths for the scale bar, in metres.
const SCALE_STEPS_M: [f64; 14] = [
    0.01, 0.02, 0.05, 0.1, 0.2, 0.5, 1.0, 2.0, 5.0, 10.0, 20.0, 50.0, 100.0, 200.0,
];

const LEGEND_ENTRIES: usize = 6;

pub(crate) fn paper_mm(paper: Paper) -> (f64, f64) {
    match paper {
        Paper::A4 => (210.0, 297.0),
        Paper::A3 => (297.0, 420.0),
    }
}

pub(crate) fn compose(list: &DisplayList, page: &Page, theme: Theme) -> Result<Doc, ExportError> {
    let sized = |v: f64| v.is_finite() && v > 0.0;
    if !sized(list.width_m) || !sized(list.depth_m) {
        return Err(ExportError::EmptyPlan);
    }

    let pal = palette(theme)?;
    let (short_mm, long_mm) = paper_mm(page.paper);
    let (w_mm, h_mm) = match page.orientation {
        Orientation::Landscape => (long_mm, short_mm),
        Orientation::Portrait => (short_mm, long_mm),
    };
    let (width_pt, height_pt) = (w_mm * PT_PER_MM, h_mm * PT_PER_MM);
    let margin = page.margin_mm.max(0.0) * PT_PER_MM;
    let content_w = width_pt - 2.0 * margin;

    // Legend wraps into rows when the page is narrow.
    let columns = ((content_w / LEGEND_ENTRY_PT) as usize).max(1);
    let legend_rows = LEGEND_ENTRIES.div_ceil(columns);
    let footer_h = SCALE_ROW_PT + legend_rows as f64 * LEGEND_ROW_PT;

    let area = Rect {
        x: margin,
        y: margin + TITLE_BAND_PT + GAP_PT,
        w: content_w,
        h: height_pt - 2.0 * margin - TITLE_BAND_PT - footer_h - 2.0 * GAP_PT,
    };
    if area.w <= 0.0 || area.h <= 0.0 {
        return Err(ExportError::PageTooSmall);
    }

    // One scale for both axes keeps the aspect ratio; the plan is centred.
    let scale = (area.w / list.width_m).min(area.h / list.depth_m);
    let frame = Rect {
        x: area.x + (area.w - list.width_m * scale) / 2.0,
        y: area.y + (area.h - list.depth_m * scale) / 2.0,
        w: list.width_m * scale,
        h: list.depth_m * scale,
    };
    let map = Map { frame, scale };

    let mut plan = Vec::new();
    for item in &list.items {
        item_prims(item, &map, pal, &mut plan)?;
    }

    let footer_y = area.y + area.h + GAP_PT;
    let mut chrome = vec![
        frame_prim(frame, pal),
        Prim::Text {
            class: "title".into(),
            anchor: Anchor::Start,
            at: (margin, margin + TITLE_PT),
            size_pt: TITLE_PT,
            weight: Weight::Bold,
            rgb: pal.foreground,
            text: page.title.clone(),
        },
    ];
    scale_bar(&map, margin, footer_y, &page.texts, pal, &mut chrome);
    legend(
        list,
        margin,
        footer_y + SCALE_ROW_PT,
        columns,
        &page.texts,
        pal,
        &mut chrome,
    )?;

    Ok(Doc {
        width_pt,
        height_pt,
        paper: pal.paper,
        clip: frame,
        plan,
        chrome,
        sheets: Vec::new(),
    })
}

/// Metres to page points.
struct Map {
    frame: Rect,
    scale: f64,
}

impl Map {
    fn pt(&self, p: Point) -> Pt {
        (
            self.frame.x + p.x_m * self.scale,
            self.frame.y + p.y_m * self.scale,
        )
    }
}

fn frame_prim(frame: Rect, pal: &Palette) -> Prim {
    Prim::Shape {
        class: "frame".into(),
        geom: Geom::Rect(frame),
        fill: None,
        stroke: Some(Stroke {
            rgb: pal.muted,
            width_pt: FRAME_PT,
            dash: Dash::Solid,
        }),
    }
}

fn item_prims(
    item: &Item,
    map: &Map,
    pal: &Palette,
    out: &mut Vec<Prim>,
) -> Result<(), ExportError> {
    let RoleStyle {
        class,
        fill,
        mut stroke,
    } = role_style(item.role, pal)?;

    let (geom, bounds) = match &item.shape {
        Shape::Line { from, to, width_m } => {
            let thick = width_m * map.scale;
            if item.role == Role::Wall {
                stroke = stroke.map(|s| Stroke {
                    width_pt: thick.max(MIN_WALL_PT),
                    ..s
                });
            }
            let pts = vec![map.pt(*from), map.pt(*to)];
            (Geom::Open(pts.clone()), pts)
        }
        Shape::Polyline { points } => {
            let pts: Vec<Pt> = points.iter().map(|p| map.pt(*p)).collect();
            (Geom::Open(pts.clone()), pts)
        }
        Shape::Polygon { points } => {
            let pts: Vec<Pt> = points.iter().map(|p| map.pt(*p)).collect();
            (Geom::Closed(pts.clone()), pts)
        }
        Shape::Circle { center, radius_m } => {
            let c = map.pt(*center);
            let r = radius_m * map.scale;
            let corners = vec![(c.0 - r, c.1 - r), (c.0 + r, c.1 + r)];
            (
                Geom::Circle {
                    center: c,
                    radius: r,
                },
                corners,
            )
        }
    };

    if bounds.len() < 2 {
        return Ok(());
    }

    out.push(Prim::Shape {
        class: format!("item {class}"),
        geom,
        fill,
        stroke,
    });

    let Some(label) = item.label.as_ref().filter(|l| !l.is_empty()) else {
        return Ok(());
    };
    let right = bounds.iter().map(|p| p.0).fold(f64::MIN, f64::max);
    let top = bounds.iter().map(|p| p.1).fold(f64::MAX, f64::min);
    let bottom = bounds.iter().map(|p| p.1).fold(f64::MIN, f64::max);

    // Circles carry the facing wedge on their right: label below, others right of the shape.
    let (at, anchor) = match item.shape {
        Shape::Circle { .. } => (
            (
                (bounds[0].0 + right) / 2.0,
                bottom + LABEL_GAP_PT + LABEL_PT,
            ),
            Anchor::Middle,
        ),
        _ => (
            (
                right + LABEL_GAP_PT,
                (top + bottom) / 2.0 + LABEL_PT * BASELINE_SHIFT,
            ),
            Anchor::Start,
        ),
    };

    out.push(Prim::Text {
        class: "label".into(),
        anchor,
        at,
        size_pt: LABEL_PT,
        weight: Weight::Regular,
        rgb: pal.foreground,
        text: label.clone(),
    });

    Ok(())
}

/// The longest round length that fits `SCALE_BAR_MAX_PT`.
fn scale_length_m(scale: f64) -> f64 {
    SCALE_STEPS_M
        .iter()
        .copied()
        .take_while(|m| m * scale <= SCALE_BAR_MAX_PT)
        .last()
        .unwrap_or(SCALE_STEPS_M[0])
}

fn scale_bar(map: &Map, x: f64, y: f64, texts: &Texts, pal: &Palette, out: &mut Vec<Prim>) {
    let length_m = scale_length_m(map.scale);
    let length_pt = length_m * map.scale;
    let mid = y + SCALE_ROW_PT / 2.0;
    let stroke = Some(Stroke {
        rgb: pal.foreground,
        width_pt: SCALE_BAR_PT,
        dash: Dash::Solid,
    });
    let part = |pts: Vec<Pt>| Prim::Shape {
        class: "scale".into(),
        geom: Geom::Open(pts),
        fill: None,
        stroke,
    };

    out.push(part(vec![(x, mid), (x + length_pt, mid)]));
    for tick_x in [x, x + length_pt] {
        out.push(part(vec![
            (tick_x, mid - SCALE_TICK_PT),
            (tick_x, mid + SCALE_TICK_PT),
        ]));
    }

    // The caller's separator and unit: "0,5 m" or "0.5 m".
    let number = length_m
        .to_string()
        .replace('.', &texts.decimal_separator.to_string());
    let text = format!("{number} {}", texts.unit);
    out.push(Prim::Text {
        class: "scale-label".into(),
        anchor: Anchor::Start,
        at: (
            x + length_pt + LABEL_GAP_PT,
            mid + LEGEND_PT * BASELINE_SHIFT,
        ),
        size_pt: LEGEND_PT,
        weight: Weight::Regular,
        rgb: pal.foreground,
        text,
    });
}

/// The first figure colour in the list, so the legend matches the plan.
fn figure_color(list: &DisplayList) -> FigureColor {
    list.items
        .iter()
        .find_map(|i| match i.role {
            Role::Figure(c) => Some(c),
            _ => None,
        })
        .unwrap_or(FigureColor::Blue)
}

fn legend(
    list: &DisplayList,
    x: f64,
    y: f64,
    columns: usize,
    texts: &Texts,
    pal: &Palette,
    out: &mut Vec<Prim>,
) -> Result<(), ExportError> {
    let color = figure_color(list);
    let entries = [
        ("figure", &texts.figure, Role::Figure(color), Swatch::Figure),
        ("camera", &texts.camera, Role::Camera, Swatch::Camera),
        ("light", &texts.light, Role::Light, Swatch::Light),
        ("axis", &texts.axis, Role::Axis, Swatch::Line),
        ("path", &texts.path, Role::FigurePath(color), Swatch::Line),
        ("mark", &texts.mark, Role::Mark(color), Swatch::Mark),
    ];

    for (i, (key, name, role, swatch)) in entries.into_iter().enumerate() {
        let col = (i % columns) as f64;
        let row = (i / columns) as f64;
        let left = x + col * LEGEND_ENTRY_PT;
        let mid = y + row * LEGEND_ROW_PT + LEGEND_ROW_PT / 2.0;
        let style = role_style(role, pal)?;

        for geom in swatch.geoms(left, mid) {
            // Rays and lines have no inside; an open mark shows the ring only.
            let open = matches!(geom, Geom::Open(_)) || swatch == Swatch::Mark;
            out.push(Prim::Shape {
                class: format!("legend {key}"),
                geom,
                fill: style.fill.filter(|_| !open),
                stroke: style.stroke,
            });
        }
        out.push(Prim::Text {
            class: "legend-label".into(),
            anchor: Anchor::Start,
            at: (
                left + SWATCH_PT + SWATCH_TEXT_GAP_PT,
                mid + LEGEND_PT * BASELINE_SHIFT,
            ),
            size_pt: LEGEND_PT,
            weight: Weight::Regular,
            rgb: pal.foreground,
            text: name.to_string(),
        });
    }

    Ok(())
}

/// Each symbol has its own shape, so the legend reads without colour: the figure is the plan's
/// circle with its facing wedge, the lamp a circle with rays, the camera its wedge, the mark
/// a ring, path and axis a dotted and a dashed line.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Swatch {
    Figure,
    Light,
    Camera,
    Mark,
    Line,
}

impl Swatch {
    fn geoms(self, left: f64, mid: f64) -> Vec<Geom> {
        let cx = left + SWATCH_PT / 2.0;
        let r = SWATCH_RADIUS_PT;

        match self {
            Swatch::Figure => {
                let c = cx - r / 2.0;
                vec![
                    Geom::Circle {
                        center: (c, mid),
                        radius: r,
                    },
                    Geom::Closed(vec![
                        (c + r * 0.8, mid - r * 0.6),
                        (c + r * 1.5, mid),
                        (c + r * 0.8, mid + r * 0.6),
                    ]),
                ]
            }
            Swatch::Light => {
                let body = r * 0.75;
                let (from, to) = (r * 1.2, r * 1.8);
                let mut geoms = vec![Geom::Circle {
                    center: (cx, mid),
                    radius: body,
                }];
                for (dx, dy) in [(1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0)] {
                    geoms.push(Geom::Open(vec![
                        (cx + dx * from, mid + dy * from),
                        (cx + dx * to, mid + dy * to),
                    ]));
                }
                geoms
            }
            Swatch::Camera => vec![Geom::Closed(vec![
                (cx - r, mid - r),
                (cx + r + 2.0, mid),
                (cx - r, mid + r),
            ])],
            Swatch::Mark => vec![Geom::Circle {
                center: (cx, mid),
                radius: r,
            }],
            Swatch::Line => vec![Geom::Open(vec![(left, mid), (left + SWATCH_PT, mid)])],
        }
    }
}
