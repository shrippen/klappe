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
//!   floor     the set's area
//! ```

use klappe_model::{FigureColor, FloorPlan, ObjectShape, Point, Project, SetPlan, Wall};

use crate::display::{object_corners, Source};
use crate::eye::Vec3;
use crate::figure::body_radius_m;
use crate::height::{figure_height_m, object_extent, opening_span, wall_top};
use crate::motion::figure_at;
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

/// The bodies of the set and of the figures at `t_s`.
pub(crate) fn scene(project: &Project, set: &SetPlan, plan: &FloorPlan, t_s: f64) -> Vec<Solid> {
    let mut solids = vec![floor(set)];
    walls(set, &mut solids);
    objects(set, &mut solids);
    figures(project, plan, t_s, &mut solids);

    solids
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
/// The outer ends reach half the thickness further, so two walls meeting at a corner close it.
fn walls(set: &SetPlan, solids: &mut Vec<Solid>) {
    for (index, wall) in set.walls.iter().enumerate() {
        let length_m = wall.length_m();
        if length_m == 0.0 {
            continue;
        }

        let top_m = wall_top(set, wall);
        let half = wall.thickness_m / 2.0;
        let mut faces = Vec::new();
        for (start_m, end_m) in solid_spans(wall) {
            let start_m = if start_m <= 0.0 { -half } else { start_m };
            let end_m = if end_m >= length_m {
                length_m + half
            } else {
                end_m
            };
            let band = wall_band(wall, start_m, end_m);
            add(&mut faces, prism(&band, (0.0, top_m), Sides::Flat));
        }
        for opening in &wall.openings {
            let (start_m, end_m) = opening.span_m();
            let band = wall_band(wall, start_m.max(0.0), end_m.min(length_m));
            let (sill_m, head_m) = opening_span(opening);
            if sill_m > 0.0 {
                add(
                    &mut faces,
                    prism(&band, (0.0, sill_m.min(top_m)), Sides::Flat),
                );
            }
            if head_m < top_m {
                add(
                    &mut faces,
                    prism(&band, (head_m.max(0.0), top_m), Sides::Flat),
                );
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

/// The floor outline of the wall from `start_m` to `end_m` along it, as thick as the wall.
fn wall_band(wall: &Wall, start_m: f64, end_m: f64) -> Vec<Point> {
    let length_m = wall.length_m();
    let (dx, dy) = (
        (wall.to.x_m - wall.from.x_m) / length_m,
        (wall.to.y_m - wall.from.y_m) / length_m,
    );
    let (nx, ny) = (-dy * wall.thickness_m / 2.0, dx * wall.thickness_m / 2.0);
    let along = |m: f64| Point::new(wall.from.x_m + dx * m, wall.from.y_m + dy * m);
    let (a, b) = (along(start_m), along(end_m));

    vec![
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
            ObjectShape::Rect => Sides::Flat,
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

/// Appends `more` to `faces`, its surfaces after those already there.
fn add(faces: &mut Vec<Face>, more: Vec<Face>) {
    let first = faces.iter().map(|f| f.surface + 1).max().unwrap_or(0);
    faces.extend(more.into_iter().map(|f| Face {
        surface: first + f.surface,
        ..f
    }));
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
    /// under the sill and over the head; the first box starts 0.1 m before the wall.
    #[test]
    fn a_wall_is_boxes_around_its_openings() {
        let mut set = SetPlan::default();
        let mut wall = Wall::new(Point::new(0.0, 0.0), Point::new(4.0, 0.0), 0.2);
        wall.openings = vec![Opening::new(OpeningKind::Window, 1.5, 1.0)];
        set.walls = vec![wall];
        let mut solids = Vec::new();

        walls(&set, &mut solids);

        // Each box has four sides and a top; the one over the head also a bottom.
        assert_eq!(solids[0].faces.len(), 5 + 5 + 5 + 6);
        let xs = solids[0]
            .faces
            .iter()
            .flat_map(|f| &f.corners)
            .map(|p| p[0]);
        assert!((xs.clone().fold(f64::MAX, f64::min) + 0.1).abs() < 1e-9);
        assert!((xs.fold(f64::MIN, f64::max) - 4.1).abs() < 1e-9);
    }
}
