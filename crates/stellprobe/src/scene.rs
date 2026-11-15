//! The set as bodies at one moment (3D-KONZEPT.md, section 4.2): one description for every
//! view that is not the plan. Each body knows what it is (a role, never a colour) and what
//! a click on it hits (the same source as on the plan).
//!
//! ```text
//!   wall      box between its openings, thickness included, ends closed;
//!             below a sill and above a head the wall goes on
//!   object    its outline raised from its underside to its top; without a height
//!             see-through in the role Missing
//!   figure    an octagon as tall as its pose, its name above it
//!   camera    a box at its lens height, turned where it looks (not the one looked through)
//!   floor     the set's area
//! ```

use klappe_model::{FigureColor, FloorPlan, ObjectShape, Point, Project, SetPlan, Wall};

use crate::display::{object_corners, Source};
use crate::eye::Vec3;
use crate::figure::body_radius_m;
use crate::height::{figure_height_m, object_extent, opening_span, wall_top};
use crate::motion::{aim_at, camera_at, figure_at};
use crate::opening::solid_spans;

/// Corners of a figure's body.
const FIGURE_SIDES: usize = 8;
/// How far a figure's name stands above its head.
const LABEL_ABOVE_M: f64 = 0.15;

/// What a body is; the surface gives it its colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneRole {
    Floor,
    Wall,
    Object,
    /// An object without a height: shown see-through until it has one (LH-93).
    Missing,
    Figure(FigureColor),
    Camera,
}

const FIGURE_CODES: u8 = 8;

impl SceneRole {
    /// The role as a pixel of the frame stores it; 0 is no body (`klappe_raster::EMPTY`).
    pub fn code(self) -> u8 {
        match self {
            SceneRole::Floor => 1,
            SceneRole::Wall => 2,
            SceneRole::Object => 3,
            SceneRole::Missing => 4,
            SceneRole::Camera => 5,
            SceneRole::Figure(color) => {
                let index = FigureColor::ALL.iter().position(|c| *c == color);
                FIGURE_CODES + index.unwrap_or_default() as u8
            }
        }
    }

    /// The role a pixel stores as `code`; `None` for an empty pixel.
    pub fn of_code(code: u8) -> Option<Self> {
        match code {
            1 => Some(SceneRole::Floor),
            2 => Some(SceneRole::Wall),
            3 => Some(SceneRole::Object),
            4 => Some(SceneRole::Missing),
            5 => Some(SceneRole::Camera),
            _ => FigureColor::ALL
                .get(usize::from(code.checked_sub(FIGURE_CODES)?))
                .map(|color| SceneRole::Figure(*color)),
        }
    }
}

/// A body: flat faces, each a convex polygon in metres (x, y as on the plan, z up).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Solid {
    pub(crate) source: Option<Source>,
    pub(crate) role: SceneRole,
    pub(crate) faces: Vec<Face>,
    /// A name and the point it stands at, e.g. a figure's above its head.
    pub(crate) label: Option<(String, Vec3)>,
}

/// A flat face. Faces of one `surface` of a body show no edge between them, e.g. the sides
/// of a round table, which only stand in for its curve.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Face {
    pub(crate) corners: Vec<Vec3>,
    pub(crate) surface: usize,
}

/// Whether the sides of a prism are faces of their own or one round surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Sides {
    Flat,
    Round,
}

/// The bodies of the set, the figures and the cameras at `t_s`; not the camera `looking`
/// through, which would stand around the eye.
pub(crate) fn scene(
    project: &Project,
    set: &SetPlan,
    plan: &FloorPlan,
    t_s: f64,
    looking: Option<&str>,
) -> Vec<Solid> {
    let mut solids = vec![floor(set)];
    walls(set, &mut solids);
    objects(set, &mut solids);
    figures(project, plan, t_s, &mut solids);
    cameras(plan, t_s, looking, &mut solids);

    solids
}

/// Size of a camera's body: along its view, across it, and high.
const CAMERA_BODY_M: (f64, f64, f64) = (0.36, 0.2, 0.2);

/// Each camera at `t_s` but `looking`: a box around its lens, turned where it looks.
fn cameras(plan: &FloorPlan, t_s: f64, looking: Option<&str>, solids: &mut Vec<Solid>) {
    for camera in &plan.cameras {
        if looking == Some(camera.id.as_str()) {
            continue;
        }
        let (Some(place), Some((height_m, _))) = (camera_at(camera, t_s), aim_at(camera, t_s))
        else {
            continue;
        };

        let (length, width, high) = CAMERA_BODY_M;
        let (sin, cos) = place.facing_deg.to_radians().sin_cos();
        let corner = |along: f64, across: f64| {
            Point::new(
                place.at.x_m + along * cos - across * sin,
                place.at.y_m + along * sin + across * cos,
            )
        };
        let outline = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)]
            .map(|(a, c)| corner(a * length / 2.0, c * width / 2.0));
        let low_m = (height_m - high / 2.0).max(0.0);

        solids.push(Solid {
            source: Some(Source::Camera(camera.id.clone())),
            role: SceneRole::Camera,
            faces: prism(&outline, (low_m, low_m + high), Sides::Flat),
            label: Some((
                camera.label.clone(),
                [place.at.x_m, place.at.y_m, low_m + high + LABEL_ABOVE_M],
            )),
        });
    }
}

fn floor(set: &SetPlan) -> Solid {
    let (w, d) = (set.width_m, set.depth_m);

    Solid {
        source: None,
        role: SceneRole::Floor,
        faces: vec![Face {
            corners: vec![[0.0, 0.0, 0.0], [w, 0.0, 0.0], [w, d, 0.0], [0.0, d, 0.0]],
            surface: 0,
        }],
        label: None,
    }
}

/// Each wall as boxes: between its openings to its top, under each sill and over each head.
/// Where two walls meet, their ends are mitred so the corner closes in one line; a free end
/// stops at the wall's point. The long sides and the top are one surface each, so a wall
/// shows no line where a window cuts it.
fn walls(set: &SetPlan, solids: &mut Vec<Solid>) {
    for (index, wall) in set.walls.iter().enumerate() {
        let length_m = wall.length_m();
        if length_m == 0.0 {
            continue;
        }

        let top_m = wall_top(set, wall);
        let ends = [End::From, End::To].map(|end| mitre(set, index, end));
        let mut faces = Vec::new();
        let mut next = FIRST_OWN_SURFACE;
        for (start_m, end_m) in solid_spans(wall) {
            let mut band = wall_band(wall, start_m, end_m);
            if start_m <= 0.0 {
                if let Some((plus, minus)) = ends[0] {
                    (band[0], band[3]) = (plus, minus);
                }
            }
            if end_m >= length_m {
                if let Some((plus, minus)) = ends[1] {
                    (band[1], band[2]) = (plus, minus);
                }
            }
            faces.extend(wall_box(&band, (0.0, top_m), top_m, &mut next));
        }
        for opening in &wall.openings {
            let (start_m, end_m) = opening.span_m();
            let band = wall_band(wall, start_m.max(0.0), end_m.min(length_m));
            let (sill_m, head_m) = opening_span(opening);
            if sill_m > 0.0 {
                let low = (0.0, sill_m.min(top_m));
                faces.extend(wall_box(&band, low, top_m, &mut next));
            }
            if head_m < top_m {
                let high = (head_m.max(0.0), top_m);
                faces.extend(wall_box(&band, high, top_m, &mut next));
            }
        }

        solids.push(Solid {
            source: Some(Source::Wall(index)),
            role: SceneRole::Wall,
            faces,
            label: None,
        });
    }
}

/// Surfaces of a wall: its two long sides and its top are shared by all its boxes.
const PLUS_SIDE: usize = 0;
const MINUS_SIDE: usize = 1;
const TOP_SURFACE: usize = 2;
const FIRST_OWN_SURFACE: usize = 3;
/// A mitre longer than this many thicknesses (a very sharp corner) is not drawn as one.
const MAX_MITRE: f64 = 4.0;

/// One end of a wall.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum End {
    From,
    To,
}

/// A box of a wall over `band` (see `wall_band`) from `low_m` to `high_m`. Its long sides and,
/// at the wall's `top_m`, its top take the wall's shared surfaces; ends and undersides each
/// their own, numbered from `next`.
fn wall_box(
    band: &[Point; 4],
    (low_m, high_m): (f64, f64),
    top_m: f64,
    next: &mut usize,
) -> Vec<Face> {
    let mut own = || {
        *next += 1;
        *next - 1
    };
    let side = |a: Point, b: Point| {
        vec![
            [a.x_m, a.y_m, low_m],
            [b.x_m, b.y_m, low_m],
            [b.x_m, b.y_m, high_m],
            [a.x_m, a.y_m, high_m],
        ]
    };
    let level = |z: f64| band.iter().map(|p| [p.x_m, p.y_m, z]).collect();
    let [a_plus, b_plus, b_minus, a_minus] = *band;

    let mut faces = vec![
        Face {
            corners: side(a_plus, b_plus),
            surface: PLUS_SIDE,
        },
        Face {
            corners: side(b_plus, b_minus),
            surface: own(),
        },
        Face {
            corners: side(b_minus, a_minus),
            surface: MINUS_SIDE,
        },
        Face {
            corners: side(a_minus, a_plus),
            surface: own(),
        },
    ];
    let top = if high_m == top_m { TOP_SURFACE } else { own() };
    faces.push(Face {
        corners: level(high_m),
        surface: top,
    });
    if low_m > 0.0 {
        faces.push(Face {
            corners: level(low_m),
            surface: own(),
        });
    }

    faces
}

/// The corners of the wall at `index` at its `end` where another wall meets it there: on its
/// plus side and its minus side (see `wall_band`). `None` at a free end, where two walls run
/// on in a line, or at a corner too sharp for a mitre.
fn mitre(set: &SetPlan, index: usize, end: End) -> Option<(Point, Point)> {
    let wall = &set.walls[index];
    let (at, into) = match end {
        End::From => (wall.from, unit(wall.to, wall.from)?),
        End::To => (wall.to, unit(wall.from, wall.to)?),
    };
    // The other wall leaving `at`, as a walk along both: into the corner, then out of it.
    let (other, out) = set.walls.iter().enumerate().find_map(|(i, w)| {
        if i == index {
            return None;
        }
        if w.from.distance_m(at) < SAME_POINT_M {
            return Some((w, unit(w.from, w.to)?));
        }
        if w.to.distance_m(at) < SAME_POINT_M {
            return Some((w, unit(w.to, w.from)?));
        }
        None
    })?;

    // Left of the walk on both walls, then right; each offset by half its wall.
    let (h1, h2) = (wall.thickness_m / 2.0, other.thickness_m / 2.0);
    let left = |d: (f64, f64)| (-d.1, d.0);
    let offset = |d: (f64, f64), h: f64, sign: f64| {
        let n = left(d);
        Point::new(at.x_m + n.0 * h * sign, at.y_m + n.1 * h * sign)
    };
    let corner = |sign: f64| meet(offset(into, h1, sign), into, offset(out, h2, sign), out);
    let (left_corner, right_corner) = (corner(1.0)?, corner(-1.0)?);
    let reach = MAX_MITRE * wall.thickness_m.max(other.thickness_m);
    if left_corner.distance_m(at) > reach || right_corner.distance_m(at) > reach {
        return None;
    }

    // The band's plus side is left of the wall from `from` to `to`.
    match end {
        End::To => Some((left_corner, right_corner)),
        End::From => Some((right_corner, left_corner)),
    }
}

/// Two points this close are one corner.
const SAME_POINT_M: f64 = 1e-6;

/// The direction from `a` to `b`, length 1; `None` when they are one point.
fn unit(a: Point, b: Point) -> Option<(f64, f64)> {
    let length = a.distance_m(b);
    (length > 0.0).then(|| ((b.x_m - a.x_m) / length, (b.y_m - a.y_m) / length))
}

/// Where the line through `p` along `d` meets the one through `q` along `e`; `None` when they
/// run side by side.
fn meet(p: Point, d: (f64, f64), q: Point, e: (f64, f64)) -> Option<Point> {
    let cross = d.0 * e.1 - d.1 * e.0;
    if cross.abs() < 1e-9 {
        return None;
    }
    let t = ((q.x_m - p.x_m) * e.1 - (q.y_m - p.y_m) * e.0) / cross;

    Some(Point::new(p.x_m + d.0 * t, p.y_m + d.1 * t))
}

/// The floor outline of the wall from `start_m` to `end_m` along it, as thick as the wall:
/// `from` end on the plus side, `to` end on the plus side, `to` end on the minus side, `from`
/// end on the minus side. The plus side lies left of the wall walked from `from` to `to`.
fn wall_band(wall: &Wall, start_m: f64, end_m: f64) -> [Point; 4] {
    let length_m = wall.length_m();
    let (dx, dy) = (
        (wall.to.x_m - wall.from.x_m) / length_m,
        (wall.to.y_m - wall.from.y_m) / length_m,
    );
    let (nx, ny) = (-dy * wall.thickness_m / 2.0, dx * wall.thickness_m / 2.0);
    let along = |m: f64| Point::new(wall.from.x_m + dx * m, wall.from.y_m + dy * m);
    let (a, b) = (along(start_m), along(end_m));

    [
        Point::new(a.x_m + nx, a.y_m + ny),
        Point::new(b.x_m + nx, b.y_m + ny),
        Point::new(b.x_m - nx, b.y_m - ny),
        Point::new(a.x_m - nx, a.y_m - ny),
    ]
}

fn objects(set: &SetPlan, solids: &mut Vec<Solid>) {
    for (index, object) in set.objects.iter().enumerate() {
        let (base_m, top_m) = object_extent(set, index);
        let sides = match object.shape {
            ObjectShape::Rect | ObjectShape::Polygon { .. } => Sides::Flat,
            ObjectShape::Ellipse => Sides::Round,
        };
        let role = match object.height_m {
            Some(_) => SceneRole::Object,
            None => SceneRole::Missing,
        };

        solids.push(Solid {
            source: Some(Source::Object(object.id.clone())),
            role,
            faces: prism(&object_corners(object), (base_m, top_m), sides),
            label: None,
        });
    }
}

/// Each figure on the set at `t_s`: an octagon around its place, as tall as its pose.
fn figures(project: &Project, plan: &FloorPlan, t_s: f64, solids: &mut Vec<Solid>) {
    for track in &plan.figures {
        let Some(character) = project.character(&track.character) else {
            continue;
        };
        let Some(place) = figure_at(track, t_s) else {
            continue;
        };

        let radius_m = body_radius_m(character);
        let outline: Vec<Point> = (0..FIGURE_SIDES)
            .map(|i| {
                let turn = std::f64::consts::TAU * i as f64 / FIGURE_SIDES as f64;
                Point::new(
                    place.at.x_m + radius_m * turn.cos(),
                    place.at.y_m + radius_m * turn.sin(),
                )
            })
            .collect();
        let reached = track.marks.iter().rev().find(|m| m.at_s <= t_s);
        let pose = reached.or(track.marks.first()).map(|m| m.pose);
        let top_m = figure_height_m(pose.unwrap_or_default());

        solids.push(Solid {
            source: Some(Source::Figure(track.character.clone())),
            role: SceneRole::Figure(character.color),
            faces: prism(&outline, (0.0, top_m), Sides::Round),
            label: Some((
                character.title(),
                [place.at.x_m, place.at.y_m, top_m + LABEL_ABOVE_M],
            )),
        });
    }
}

/// `outline` raised from `low_m` to `high_m`: its sides, its top, and its bottom when it
/// does not lie on the floor. Round sides are one surface, top and bottom one each.
fn prism(outline: &[Point], (low_m, high_m): (f64, f64), sides: Sides) -> Vec<Face> {
    let mut faces: Vec<Face> = outline
        .iter()
        .enumerate()
        .map(|(i, a)| {
            let b = outline[(i + 1) % outline.len()];
            Face {
                corners: vec![
                    [a.x_m, a.y_m, low_m],
                    [b.x_m, b.y_m, low_m],
                    [b.x_m, b.y_m, high_m],
                    [a.x_m, a.y_m, high_m],
                ],
                surface: if sides == Sides::Round { 0 } else { i },
            }
        })
        .collect();

    let next = faces.iter().map(|f| f.surface + 1).max().unwrap_or(0);
    let level = |z: f64| outline.iter().map(|p| [p.x_m, p.y_m, z]).collect();
    faces.push(Face {
        corners: level(high_m),
        surface: next,
    });
    if low_m > 0.0 {
        faces.push(Face {
            corners: level(low_m),
            surface: next + 1,
        });
    }

    faces
}

#[cfg(test)]
mod tests {
    use klappe_model::{Opening, OpeningKind};

    use super::*;

    #[test]
    fn every_role_comes_back_from_its_code() {
        let mut roles = vec![
            SceneRole::Floor,
            SceneRole::Wall,
            SceneRole::Object,
            SceneRole::Missing,
            SceneRole::Camera,
        ];
        roles.extend(FigureColor::ALL.map(SceneRole::Figure));

        for role in roles {
            assert_ne!(role.code(), 0);
            assert_eq!(SceneRole::of_code(role.code()), Some(role));
        }
        assert_eq!(SceneRole::of_code(0), None);
    }

    /// A round object's sides are one surface (no edges between them), its top another; a
    /// box has a surface per side.
    #[test]
    fn round_sides_are_one_surface() {
        let outline =
            [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)].map(|(x, y)| Point::new(x, y));
        let surfaces = |sides| {
            let mut s: Vec<usize> = prism(&outline, (0.0, 1.0), sides)
                .iter()
                .map(|f| f.surface)
                .collect();
            s.dedup();
            s
        };

        assert_eq!(surfaces(Sides::Round), vec![0, 1]);
        assert_eq!(surfaces(Sides::Flat), vec![0, 1, 2, 3, 4]);
    }

    /// A 4 m wall, 0.2 m thick, with a window from 1 to 2 m: two full boxes and one each
    /// under the sill and over the head. Its free ends stop at its points; its long sides
    /// and its top are one surface each, so the window draws no line across the wall.
    #[test]
    fn a_wall_is_boxes_around_its_openings() {
        let mut set = SetPlan::default();
        let mut wall = Wall::new(Point::new(0.0, 0.0), Point::new(4.0, 0.0), 0.2);
        wall.openings = vec![Opening::new(OpeningKind::Window, 1.5, 1.0)];
        set.walls = vec![wall];
        let mut solids = Vec::new();

        walls(&set, &mut solids);

        // Each box has four sides and a top; the one over the head also a bottom.
        let faces = &solids[0].faces;
        assert_eq!(faces.len(), 5 + 5 + 5 + 6);
        let xs = faces.iter().flat_map(|f| &f.corners).map(|p| p[0]);
        assert_eq!(xs.clone().fold(f64::MAX, f64::min), 0.0);
        assert_eq!(xs.fold(f64::MIN, f64::max), 4.0);
        let plus_sides = faces.iter().filter(|f| f.surface == PLUS_SIDE).count();
        assert_eq!(plus_sides, 4);
    }

    /// Two walls at a right angle, 0.2 m thick, meeting at (4, 0): their ends are mitred, so
    /// the outer corner lies at (4.1, -0.1) on both and the inner one at (3.9, 0.1).
    #[test]
    fn walls_meeting_at_a_corner_are_mitred() {
        let set = SetPlan {
            walls: vec![
                Wall::new(Point::new(0.0, 0.0), Point::new(4.0, 0.0), 0.2),
                Wall::new(Point::new(4.0, 0.0), Point::new(4.0, 3.0), 0.2),
            ],
            ..SetPlan::default()
        };

        let close = |p: Point, x: f64, y: f64| (p.x_m - x).abs() < 1e-9 && (p.y_m - y).abs() < 1e-9;
        let (plus, minus) = mitre(&set, 0, End::To).expect("first");
        let (plus2, minus2) = mitre(&set, 1, End::From).expect("second");

        // Plus lies left of the walk from `from` to `to`: y grows downwards, so left of +x is +y.
        assert!(close(plus, 3.9, 0.1), "{plus:?}");
        assert!(close(minus, 4.1, -0.1), "{minus:?}");
        // The second wall walks +y: its plus side lies at -x, the inner one, like the first's.
        assert!(close(plus2, 3.9, 0.1), "{plus2:?}");
        assert!(close(minus2, 4.1, -0.1), "{minus2:?}");
        assert_eq!(mitre(&set, 0, End::From), None);
    }
}
