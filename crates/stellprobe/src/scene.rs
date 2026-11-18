//! The set as bodies at one moment (3D-KONZEPT.md, section 4.2): one description for every
//! view that is not the plan. Each body knows what it is (a role, never a colour) and what
//! a click on it hits (the same source as on the plan).
//!
//! ```text
//!   wall      box between its openings, thickness included, ends closed;
//!             below a sill and above a head the wall goes on
//!   object    its outline raised from its underside to its top; without a height
//!             see-through in the role Missing; with a body form one box per part
//!   figure    an octagon as tall as its pose, its name above it
//!   camera    a box at its lens height, turned where it looks (not the one looked through)
//!   floor     the set's area
//! ```

use klappe_model::{
    FigureColor, FloorPlan, FormBox, ObjectShape, Point, Project, SetObject, SetPlan,
};

mod walls;

use self::walls::walls;
use crate::display::{object_corners, Source};
use crate::eye::Vec3;
use crate::figure::body_radius_m;
use crate::forms::Forms;
use crate::height::{figure_height_m, object_extent};
use crate::motion::{aim_at, camera_at, figure_at, figure_opacity_at};

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
    /// How strongly it is drawn, 0 to 1: a figure fading in or out (LH-26).
    pub(crate) opacity: f64,
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

/// How far the walls are cut down, a setting of the view, not of the model (LH-98).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Cut {
    #[default]
    Off,
    /// Walls end at chest height, so one sees into the room.
    Chest,
}

/// Where `Cut::Chest` cuts the walls.
const CHEST_M: f64 = 1.20;

impl Cut {
    /// The height a wall of `top_m` ends at.
    fn wall_top(self, top_m: f64) -> f64 {
        match self {
            Cut::Off => top_m,
            Cut::Chest => top_m.min(CHEST_M),
        }
    }
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
    forms: &Forms,
    t_s: f64,
    looking: Option<&str>,
    cut: Cut,
) -> Vec<Solid> {
    let mut solids = vec![floor(set)];
    walls(set, cut, &mut solids);
    objects(&crate::prop::set_at(set, plan, t_s), forms, &mut solids);
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
            opacity: 1.0,
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
        opacity: 1.0,
        source: None,
        role: SceneRole::Floor,
        faces: vec![Face {
            corners: vec![[0.0, 0.0, 0.0], [w, 0.0, 0.0], [w, d, 0.0], [0.0, d, 0.0]],
            surface: 0,
        }],
        label: None,
    }
}

fn objects(set: &SetPlan, forms: &Forms, solids: &mut Vec<Solid>) {
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
        let source = Some(Source::Object(object.id.clone()));
        let form = forms
            .get(&object.kind)
            .filter(|_| object.shape == ObjectShape::Rect);
        let Some(boxes) = form else {
            solids.push(Solid {
                opacity: 1.0,
                source,
                role,
                faces: prism(&object_corners(object), (base_m, top_m), sides),
                label: None,
            });
            continue;
        };

        for part in boxes {
            let (outline, levels) = form_box(object, part, (base_m, top_m));
            solids.push(Solid {
                opacity: 1.0,
                source: source.clone(),
                role,
                faces: prism(&outline, levels, Sides::Flat),
                label: None,
            });
        }
    }
}

/// The floor outline and the underside and top of `part` of `object`'s body form: its
/// fractions scaled to the object's size and turned about its centre like its outline; a
/// mirrored object's left and right are swapped.
fn form_box(
    object: &SetObject,
    part: &FormBox,
    (base_m, top_m): (f64, f64),
) -> ([Point; 4], (f64, f64)) {
    let corner =
        |fx: f64, fy: f64| object.placed((fx - 0.5) * object.width_m, (fy - 0.5) * object.depth_m);
    let (from, to) = (part.from, part.to);
    // Mirrored (LH-115): the box lies across the middle, its corners in the same order.
    let (x0, x1) = match object.mirrored {
        true => (1.0 - to[0], 1.0 - from[0]),
        false => (from[0], to[0]),
    };
    let outline = [
        corner(x0, from[1]),
        corner(x1, from[1]),
        corner(x1, to[1]),
        corner(x0, to[1]),
    ];
    let level = |f: f64| base_m + f * (top_m - base_m);

    (outline, (level(from[2]), level(to[2])))
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
        let opacity = figure_opacity_at(track, t_s);
        if opacity <= 0.0 {
            continue;
        }

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
            opacity,
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

    fn forms_of(kind: &str, boxes: &[([f64; 3], [f64; 3])]) -> Forms {
        let mut forms = Forms::default();
        let boxes: Vec<FormBox> = boxes
            .iter()
            .map(|&(from, to)| FormBox { from, to })
            .collect();
        forms.insert(kind, &boxes);

        forms
    }

    fn placed(kind: &str, rotation_deg: f64, height_m: f64) -> SetPlan {
        SetPlan {
            objects: vec![SetObject {
                id: "o".into(),
                kind: kind.into(),
                label: String::new(),
                at: Point::new(10.0, 10.0),
                width_m: 1.6,
                depth_m: 0.9,
                rotation_deg,
                shape: ObjectShape::Rect,
                height_m: Some(height_m),
                base_m: None,
                mirrored: false,
            }],
            ..SetPlan::default()
        }
    }

    /// The range of axis `axis` (0 x, 1 y, 2 z) over the corners of a solid.
    fn range(solid: &Solid, axis: usize) -> (f64, f64) {
        let values = solid.faces.iter().flat_map(|f| &f.corners).map(|c| c[axis]);

        (
            values.clone().fold(f64::MAX, f64::min),
            values.fold(f64::MIN, f64::max),
        )
    }

    fn close(a: (f64, f64), b: (f64, f64)) -> bool {
        (a.0 - b.0).abs() < 1e-9 && (a.1 - b.1).abs() < 1e-9
    }

    /// A table of 1.6 x 0.9 x 0.75 m at (10, 10) turned by 90 degrees is a top 0.06 m thick and
    /// four legs: a point (fx, fy) of the table lies at (10 - (fy - 0.5) 0.9, 10 + (fx - 0.5) 1.6).
    /// The top covers x 9.55..10.45, y 9.2..10.8, z 0.69..0.75; the first leg (fractions
    /// 0..0.04 by 0..0.07 by 0..0.92) x 10.387..10.45, y 9.2..9.264, z 0..0.69. Without a form
    /// there is one solid; a free form never gets one.
    #[test]
    fn a_table_is_a_top_on_four_legs() {
        let leg = |x: f64, y: f64| ([x, y, 0.0], [x + 0.04, y + 0.07, 0.92]);
        let forms = forms_of(
            "table",
            &[
                ([0.0, 0.0, 0.92], [1.0, 1.0, 1.0]),
                leg(0.0, 0.0),
                leg(0.96, 0.0),
                leg(0.0, 0.93),
                leg(0.96, 0.93),
            ],
        );
        let set = placed("table", 90.0, 0.75);
        let mut solids = Vec::new();

        objects(&set, &forms, &mut solids);

        assert_eq!(solids.len(), 5);
        let top = &solids[0];
        assert!(close(range(top, 0), (9.55, 10.45)), "{:?}", range(top, 0));
        assert!(close(range(top, 1), (9.2, 10.8)));
        assert!(close(range(top, 2), (0.69, 0.75)));
        let leg = &solids[1];
        assert!(close(range(leg, 0), (10.387, 10.45)), "{:?}", range(leg, 0));
        assert!(close(range(leg, 1), (9.2, 9.264)));
        assert!(close(range(leg, 2), (0.0, 0.69)));
        let source = Some(Source::Object("o".into()));
        assert!(solids
            .iter()
            .all(|s| s.source == source && s.role == SceneRole::Object));

        let mut plain = Vec::new();
        objects(&set, &Forms::default(), &mut plain);
        assert_eq!(plain.len(), 1);

        let mut free = placed("table", 90.0, 0.75);
        free.objects[0].shape = ObjectShape::Ellipse;
        let mut round = Vec::new();
        objects(&free, &forms, &mut round);
        assert_eq!(round.len(), 1);
    }

    /// Three steps of a 1.6 m long, 0.9 m wide flight 0.9 m high rise along x by 0.3 m each.
    #[test]
    fn stairs_rise_step_by_step() {
        let step = |i: f64| ([i / 3.0, 0.0, 0.0], [(i + 1.0) / 3.0, 1.0, (i + 1.0) / 3.0]);
        let forms = forms_of("stairs", &[step(0.0), step(1.0), step(2.0)]);
        let mut solids = Vec::new();

        objects(&placed("stairs", 0.0, 0.9), &forms, &mut solids);

        let tops: Vec<f64> = solids.iter().map(|s| range(s, 2).1).collect();
        assert_eq!(solids.len(), 3);
        assert!(close((tops[0], tops[1]), (0.3, 0.6)) && (tops[2] - 0.9).abs() < 1e-9);
        let from_x: Vec<f64> = solids.iter().map(|s| range(s, 0).0).collect();
        assert!(from_x[0] < from_x[1] && from_x[1] < from_x[2]);
    }

    /// Mirrored (LH-115), the flight rises the other way: its first step lies at the right end.
    #[test]
    fn a_mirrored_flight_rises_the_other_way() {
        let step = |i: f64| ([i / 3.0, 0.0, 0.0], [(i + 1.0) / 3.0, 1.0, (i + 1.0) / 3.0]);
        let forms = forms_of("stairs", &[step(0.0), step(1.0), step(2.0)]);
        let mut set = placed("stairs", 0.0, 0.9);
        let mut solids = Vec::new();
        set.objects[0].mirrored = true;

        objects(&set, &forms, &mut solids);

        let tops: Vec<f64> = solids.iter().map(|s| range(s, 2).1).collect();
        assert!(close((tops[0], tops[1]), (0.3, 0.6)) && (tops[2] - 0.9).abs() < 1e-9);
        assert!(close(range(&solids[0], 0), (9.2 + 1.6 * 2.0 / 3.0, 10.8)));
        assert!(range(&solids[1], 0).0 > range(&solids[2], 0).0);
    }
}
