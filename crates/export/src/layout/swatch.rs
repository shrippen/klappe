//! The symbols of the legend, each with its own shape so that it reads without colour.

use crate::canvas::Geom;

pub(super) const SWATCH_PT: f64 = 18.0;
const SWATCH_RADIUS_PT: f64 = 4.0;
/// The ring and chevron of the mark swatches with a sign, as shares of `SWATCH_RADIUS_PT`.
const RING_SHARE: f64 = 0.6;
const ARM_SHARE: f64 = 0.7;
const SIGN_GAP_PT: f64 = 1.5;

/// Each symbol has its own shape, so the legend reads without colour: the figure is the plan's
/// circle with its facing wedge, the lamp a circle with its cone, the camera its wedge, the mark
/// a ring, path and axis a dotted and a dashed line.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Swatch {
    Figure,
    Light,
    Camera,
    Mark,
    /// A mark with the chevron of an appearing (LH-26) or a leaving figure.
    Arrival,
    Departure,
    Line,
    /// A filled drawing: a small box.
    Area,
}

/// Which way the chevron of a mark swatch points, seen from its ring.
#[derive(Clone, Copy)]
enum Direction {
    To,
    From,
}

/// A small ring at `cx` with a chevron pointing at it or away from it, on the right of the
/// ring for `From`, on its left for `To` (the plan's signs of a mark, LH-26).
fn ringed(cx: f64, mid: f64, direction: Direction) -> Vec<Geom> {
    let ring = SWATCH_RADIUS_PT * RING_SHARE;
    let arm = SWATCH_RADIUS_PT * ARM_SHARE;
    let reach = ring + SIGN_GAP_PT;
    let tip = match direction {
        Direction::To => cx - reach,
        Direction::From => cx + reach + arm,
    };

    vec![
        Geom::Circle {
            center: (cx, mid),
            radius: ring,
        },
        Geom::Open(vec![
            (tip - arm, mid - arm),
            (tip, mid),
            (tip - arm, mid + arm),
        ]),
    ]
}

impl Swatch {
    pub(super) fn geoms(self, left: f64, mid: f64) -> Vec<Geom> {
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
                vec![
                    Geom::Circle {
                        center: (cx - r / 2.0, mid),
                        radius: body,
                    },
                    Geom::Closed(vec![
                        (cx - r / 2.0, mid),
                        (cx + r * 1.5, mid - r * 0.9),
                        (cx + r * 1.5, mid + r * 0.9),
                    ]),
                ]
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
            Swatch::Arrival => ringed(left + SWATCH_PT * 0.75, mid, Direction::To),
            Swatch::Departure => ringed(left + SWATCH_PT * 0.25, mid, Direction::From),
            Swatch::Line => vec![Geom::Open(vec![(left, mid), (left + SWATCH_PT, mid)])],
            Swatch::Area => vec![Geom::Closed(vec![
                (left, mid - r),
                (left + SWATCH_PT, mid - r),
                (left + SWATCH_PT, mid + r),
                (left, mid + r),
            ])],
        }
    }
}
