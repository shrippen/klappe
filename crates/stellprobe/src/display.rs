use klappe_model::{
    Arrival, Background, Dash, Departure, DrawingShape, FigureColor, Fill, FloorPlan, Mark, Point,
    Project, SetPlan,
};

use crate::axis::axis_at;
use crate::card::note_card;
use crate::figure::{body_radius_m, group_dots, GROUP_DOT_RADIUS_M, GROUP_RADIUS_M};
use crate::light::{lamps, light_items, Lamp};
use crate::motion::{camera_at, figure_at, path, walked_ways};
use crate::optics::{frustum, horizontal_fov_deg, lens_at};
use crate::rig::{rig_dash, rig_runs};

mod handles;
mod objects;

pub use handles::{
    drawing_edge_places, drawing_handle_places, handles, object_grip_place, Bend, LightGrip,
    ObjectGrip, OpeningEnd,
};
pub use objects::label_point;
pub(crate) use objects::object_corners;
use objects::set_items;

/// What the GUI draws and the export writes, back to front, in metres (planung/KONZEPT.md,
/// section 4). Colours are roles; the GUI maps them to Kante.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DisplayList {
    pub items: Vec<Item>,
    /// What each lamp of the plan is, for a printed legend (LH-51).
    pub lamps: Vec<Lamp>,
    pub width_m: f64,
    pub depth_m: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub role: Role,
    pub shape: Shape,
    /// Text drawn next to the shape, e.g. a figure's name or a mark's number.
    pub label: Option<String>,
    /// What the item belongs to, for selection and tooltips.
    pub source: Option<Source>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    Line {
        from: Point,
        to: Point,
        width_m: f64,
    },
    Polyline {
        points: Vec<Point>,
    },
    Polygon {
        points: Vec<Point>,
    },
    Circle {
        center: Point,
        radius_m: f64,
    },
    /// A picture file of the project folder, `width_m` by `depth_m` from its top left corner
    /// `at`, turned clockwise about `at`. The list names the file; reading it is the caller's.
    Image {
        file: String,
        at: Point,
        width_m: f64,
        depth_m: f64,
        rotation_deg: f64,
        opacity: f64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// The set's background image, behind everything else.
    Background,
    Wall,
    /// A door in a wall (LH-10): the leaf and the quarter circle it swings through.
    Door,
    /// A window in a wall (LH-10): two thin lines in the gap.
    Window,
    Object,
    LightBeam,
    Light,
    Axis,
    /// One run of a camera's way, in the line its rig has (LH-33).
    CameraPath {
        dash: Dash,
    },
    Frustum,
    Camera,
    /// A text pinned to the plan (LH-16): a card, top left at the note's place, the wrapped text
    /// as its label.
    Note,
    /// A line, outline or area drawn by hand (LH-19).
    Drawing {
        color: FigureColor,
        dash: Dash,
        fill: Fill,
    },
    /// The filled head of an arrow, in the arrow's colour.
    ArrowHead(FigureColor),
    FigurePath(FigureColor),
    /// A grip on a point or corner of the selected drawing (LH-19); only drawn while it is
    /// selected.
    DrawingHandle,
    /// The grip that sets the width of the selected note's card (LH-16); only drawn while it
    /// is selected.
    NoteHandle,
    /// A grip on an end of the selected door or window (LH-10); only drawn while selected.
    OpeningHandle,
    /// A grip on the beam of the selected lamp (LH-50); only drawn while it is selected.
    LightHandle,
    /// A grip on the right or bottom edge of the selected object (LH-25); only drawn while
    /// selected.
    ObjectHandle,
    /// The grip that bends the way to a mark (LH-21); only drawn for a selected way.
    FigureHandle(FigureColor),
    CameraHandle,
    Mark(FigureColor),
    Figure(FigureColor),
    /// A dot inside a group's body; in the body's colour it would vanish.
    GroupDot,
    /// The height of the set object beside it (LH-93), in whole centimetres; `None` while
    /// the object has none. A caption at the item's point, no shape.
    Height {
        cm: Option<u32>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    Object(String),
    Light(String),
    Camera(String),
    Figure(String),
    Mark {
        character: String,
        index: usize,
    },
    /// A grip on the way to mark `index` of a character's track (LH-21).
    FigureVia {
        character: String,
        index: usize,
        bend: Bend,
    },
    /// A grip on the way to mark `index` of a camera (LH-21).
    CameraVia {
        camera: String,
        index: usize,
        bend: Bend,
    },
    /// Index into the set's walls.
    Wall(usize),
    /// Opening `index` of the set's wall `wall` (LH-10).
    Opening {
        wall: usize,
        index: usize,
    },
    /// The grip on one end of opening `index` of wall `wall`, on the wall's middle line
    /// (LH-10): drag to set that end, the other stays.
    OpeningEdge {
        wall: usize,
        index: usize,
        end: OpeningEnd,
    },
    /// A grip on the far edge of a lamp's beam (LH-50): drag to turn the lamp or set its beam.
    LightGrip {
        light: String,
        grip: LightGrip,
    },
    /// The grip in the middle of the right or bottom edge of a set object's box (LH-25): drag
    /// to set its width or depth, the left or top edge stays.
    ObjectGrip {
        object: String,
        grip: ObjectGrip,
    },
    Note(String),
    /// The grip in the middle of the right edge of a note's card (LH-16): drag to set its width.
    NoteWidth(String),
    Drawing(String),
    /// Point or corner `index` of a drawing (LH-19): a grip. A rectangle or ellipse has its
    /// four corners, clockwise from the top left before the turn.
    DrawingPoint {
        drawing: String,
        index: usize,
    },
    /// The grip halfway along the edge of an area from point `index` to the next (and from the
    /// last to the first): drag to make a point there (LH-19).
    DrawingEdge {
        drawing: String,
        index: usize,
    },
}

/// Figure body radius on the plan.
pub const FIGURE_RADIUS_M: f64 = 0.35;
/// Mark circles are smaller than the figure.
const MARK_RADIUS_M: f64 = 0.22;
/// Camera and lamp bodies.
pub(crate) const BODY_RADIUS_M: f64 = 0.25;
/// How long an arrow's head is, and how far its wings stand off the shaft.
const ARROW_HEAD_M: f64 = 0.45;
const ARROW_HEAD_HALF_DEG: f64 = 25.0;
/// Corners of the polygon an ellipse is drawn with.
const ELLIPSE_CORNERS: usize = 48;
/// How far a camera's view is drawn when the set gives no size.
const VIEW_REACH_FALLBACK_M: f64 = 8.0;
/// Shown before a mark's number where the figure appears, after it where it leaves (LH-26).
const APPEAR_SIGN: &str = "+";
const LEAVE_SIGN: &str = "-";
/// The facing wedge: how far it reaches beyond the body and how wide it is.
const NOSE_M: f64 = 0.25;
const NOSE_HALF_DEG: f64 = 35.0;

/// The display list of a floor plan on its location's set at time `t_s` (LH-60).
/// `background_px` is the pixel size of the set's background image; Stellprobe reads no
/// files, so the caller looks it up. Without it the background is left out.
pub fn render(
    project: &Project,
    set: &SetPlan,
    plan: &FloorPlan,
    t_s: f64,
    background_px: Option<(u32, u32)>,
) -> DisplayList {
    let mut items = Vec::new();
    let reach_m = set_diagonal_m(set).unwrap_or(VIEW_REACH_FALLBACK_M);

    if let (Some(background), Some(px)) = (&set.background, background_px) {
        background_item(background, px, &mut items);
    }
    set_items(set, &mut items);
    drawing_items(plan, &mut items);
    light_items(plan, &mut items);
    axis_items(plan, t_s, reach_m, &mut items);
    camera_items(plan, t_s, reach_m, &mut items);
    figure_items(project, plan, t_s, &mut items);
    note_items(set, &mut items);

    DisplayList {
        items,
        lamps: lamps(plan),
        width_m: set.width_m,
        depth_m: set.depth_m,
    }
}

fn set_diagonal_m(set: &SetPlan) -> Option<f64> {
    let d = set.width_m.hypot(set.depth_m);
    (d > 0.0).then_some(d)
}

/// The image keeps its aspect ratio: depth = width * pixel height / pixel width.
fn background_item(background: &Background, px: (u32, u32), items: &mut Vec<Item>) {
    let (width_px, height_px) = px;
    if width_px == 0 || height_px == 0 {
        return;
    }

    items.push(Item {
        role: Role::Background,
        shape: Shape::Image {
            file: background.file.clone(),
            at: background.at,
            width_m: background.width_m,
            depth_m: background.width_m * f64::from(height_px) / f64::from(width_px),
            rotation_deg: background.rotation_deg,
            opacity: background.opacity,
        },
        label: None,
        source: None,
    });
}

/// Drawings lie on the set but under everything that moves, so a figure stays pickable on
/// an area.
fn drawing_items(plan: &FloorPlan, items: &mut Vec<Item>) {
    for drawing in &plan.drawings {
        let role = Role::Drawing {
            color: drawing.color,
            dash: drawing.dash,
            fill: drawing.fill,
        };
        let source = Some(Source::Drawing(drawing.id.clone()));
        let at = |p: Point| drawing.placed(p);
        let all = |points: Vec<Point>| points.into_iter().map(at).collect::<Vec<_>>();
        let mut push = |role, shape| {
            items.push(Item {
                role,
                shape,
                label: None,
                source: source.clone(),
            });
        };

        match &drawing.shape {
            DrawingShape::Line { from, to } => push(role, line(at(*from), at(*to))),
            DrawingShape::Arrow { from, to } => {
                let (from, to) = (at(*from), at(*to));
                push(role, line(from, to));
                if let Some(head) = arrow_head(from, to) {
                    push(Role::ArrowHead(drawing.color), head);
                }
            }
            DrawingShape::Rect { from, to } => {
                push(role, closed(all(rect_corners(*from, *to)), drawing.fill));
            }
            DrawingShape::Ellipse { from, to } => {
                push(role, closed(all(ellipse_corners(*from, *to)), drawing.fill));
            }
            DrawingShape::Area { points } => push(role, closed(all(points.clone()), drawing.fill)),
        }
    }
}

/// A hairline: the stroke width comes from the role.
fn line(from: Point, to: Point) -> Shape {
    Shape::Line {
        from,
        to,
        width_m: 0.0,
    }
}

/// A filled shape is a polygon, taken by a click anywhere inside. An outline is a path back
/// to its first point, taken only on its stroke: a click inside it reaches what lies below.
fn closed(mut points: Vec<Point>, fill: Fill) -> Shape {
    if fill == Fill::Filled {
        return Shape::Polygon { points };
    }

    if let Some(first) = points.first().copied() {
        points.push(first);
    }
    Shape::Polyline { points }
}

/// The two wings and the tip at `to`; none for an arrow of no length. A short arrow gets a
/// head of half its length, so the head never outgrows the shaft.
fn arrow_head(from: Point, to: Point) -> Option<Shape> {
    let length_m = from.distance_m(to);
    if length_m == 0.0 {
        return None;
    }

    let head_m = ARROW_HEAD_M.min(length_m / 2.0);
    let back_deg = from.bearing_deg(to) + 180.0;
    let wing = |deg: f64| {
        let a = deg.to_radians();
        Point::new(to.x_m + head_m * a.cos(), to.y_m + head_m * a.sin())
    };

    Some(Shape::Polygon {
        points: vec![
            wing(back_deg - ARROW_HEAD_HALF_DEG),
            to,
            wing(back_deg + ARROW_HEAD_HALF_DEG),
        ],
    })
}

/// Clockwise from the top left, whichever way the corners were dragged.
fn rect_corners(a: Point, b: Point) -> Vec<Point> {
    let (x0, x1) = (a.x_m.min(b.x_m), a.x_m.max(b.x_m));
    let (y0, y1) = (a.y_m.min(b.y_m), a.y_m.max(b.y_m));

    vec![
        Point::new(x0, y0),
        Point::new(x1, y0),
        Point::new(x1, y1),
        Point::new(x0, y1),
    ]
}

/// The ellipse inscribed in the rectangle between `a` and `b`, from its rightmost point on.
fn ellipse_corners(a: Point, b: Point) -> Vec<Point> {
    let middle = a.lerp(b, 0.5);
    let (rx_m, ry_m) = ((b.x_m - a.x_m).abs() / 2.0, (b.y_m - a.y_m).abs() / 2.0);

    (0..ELLIPSE_CORNERS)
        .map(|i| {
            let turn = std::f64::consts::TAU * i as f64 / ELLIPSE_CORNERS as f64;
            Point::new(
                middle.x_m + rx_m * turn.cos(),
                middle.y_m + ry_m * turn.sin(),
            )
        })
        .collect()
}

/// Notes lie on top of everything, so a figure never hides one.
fn note_items(set: &SetPlan, items: &mut Vec<Item>) {
    for note in &set.notes {
        let (points, wrapped) = note_card(note.at, note.width_m, &note.text);
        items.push(Item {
            role: Role::Note,
            shape: Shape::Polygon { points },
            label: Some(wrapped),
            source: Some(Source::Note(note.id.clone())),
        });
    }
}

/// The axis drawn across the whole set, not only between the two figures.
fn axis_items(plan: &FloorPlan, t_s: f64, reach_m: f64, items: &mut Vec<Item>) {
    let Some(axis) = axis_at(plan, t_s) else {
        return;
    };
    let length = axis.a.distance_m(axis.b);
    if length == 0.0 {
        return;
    }

    let k = reach_m / length;
    let from = axis.a.lerp(axis.b, -k);
    let to = axis.a.lerp(axis.b, 1.0 + k);
    items.push(Item {
        role: Role::Axis,
        shape: Shape::Line {
            from,
            to,
            width_m: 0.0,
        },
        label: None,
        source: None,
    });
}

fn camera_items(plan: &FloorPlan, t_s: f64, reach_m: f64, items: &mut Vec<Item>) {
    for camera in &plan.cameras {
        let source = Some(Source::Camera(camera.id.clone()));

        for run in rig_runs(&camera.marks) {
            items.push(Item {
                role: Role::CameraPath {
                    dash: rig_dash(run[1].rig),
                },
                shape: Shape::Polyline {
                    points: path(run.iter().map(|m| (m.at, m.via.as_slice()))),
                },
                label: None,
                source: source.clone(),
            });
        }

        let Some(place) = camera_at(camera, t_s) else {
            continue;
        };
        let fov = horizontal_fov_deg(camera.sensor, lens_at(camera, &plan.shots, t_s));
        let view = frustum(place.at, place.facing_deg, fov, reach_m);

        items.push(Item {
            role: Role::Frustum,
            shape: Shape::Polygon {
                points: view.to_vec(),
            },
            label: None,
            source: source.clone(),
        });
        items.push(Item {
            role: Role::Camera,
            shape: Shape::Polygon {
                points: nose(place.at, place.facing_deg, BODY_RADIUS_M),
            },
            label: Some(camera.label.clone()),
            source,
        });
    }
}

fn figure_items(project: &Project, plan: &FloorPlan, t_s: f64, items: &mut Vec<Item>) {
    for track in &plan.figures {
        let Some(character) = project.character(&track.character) else {
            continue;
        };
        let color = character.color;

        // A way the figure does not walk (to a mark it appears at, or after it left) is not
        // drawn: the path breaks there (LH-26).
        let walked = walked_ways(track);
        let marks: Vec<_> = track.marks.iter().zip(&walked).collect();
        for way in marks.chunk_by(|_, (_, walked)| **walked) {
            if way.len() < 2 {
                continue;
            }

            items.push(Item {
                role: Role::FigurePath(color),
                shape: Shape::Polyline {
                    points: path(way.iter().map(|(m, _)| (m.at, m.via.as_slice()))),
                },
                label: None,
                source: Some(Source::Figure(track.character.clone())),
            });
        }

        // Numbered marks 1, 2, 3 …, `+` before an appearing one and `-` after a leaving one.
        for (index, mark) in track.marks.iter().enumerate() {
            items.push(Item {
                role: Role::Mark(color),
                shape: Shape::Circle {
                    center: mark.at,
                    radius_m: MARK_RADIUS_M,
                },
                label: Some(mark_label(index, mark)),
                source: Some(Source::Mark {
                    character: track.character.clone(),
                    index,
                }),
            });
        }

        let Some(place) = figure_at(track, t_s) else {
            continue;
        };
        let source = Some(Source::Figure(track.character.clone()));
        let radius_m = body_radius_m(character);
        items.push(Item {
            role: Role::Figure(color),
            shape: Shape::Polygon {
                points: nose(place.at, place.facing_deg, radius_m),
            },
            label: None,
            source: source.clone(),
        });
        items.push(Item {
            role: Role::Figure(color),
            shape: Shape::Circle {
                center: place.at,
                radius_m,
            },
            label: Some(character.title()),
            source: source.clone(),
        });

        // Dots inside the body mark a group; a click on one still takes the whole group.
        if character.group.is_none() {
            continue;
        }
        let scale = radius_m / GROUP_RADIUS_M;
        for center in group_dots(place.at, scale) {
            items.push(Item {
                role: Role::GroupDot,
                shape: Shape::Circle {
                    center,
                    radius_m: GROUP_DOT_RADIUS_M * scale,
                },
                label: None,
                source: source.clone(),
            });
        }
    }
}

fn mark_label(index: usize, mark: &Mark) -> String {
    let appears = if mark.arrival == Arrival::Appear {
        APPEAR_SIGN
    } else {
        ""
    };
    let leaves = if mark.departure == Departure::Leave {
        LEAVE_SIGN
    } else {
        ""
    };

    format!("{appears}{}{leaves}", index + 1)
}

/// A wedge in front of a body of `radius_m`, pointing where it faces.
fn nose(at: Point, facing_deg: f64, radius_m: f64) -> Vec<Point> {
    let point = |deg: f64, r: f64| {
        let a = deg.to_radians();
        Point::new(at.x_m + r * a.cos(), at.y_m + r * a.sin())
    };

    vec![
        point(facing_deg - NOSE_HALF_DEG, radius_m),
        point(facing_deg, radius_m + NOSE_M),
        point(facing_deg + NOSE_HALF_DEG, radius_m),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::figure::GROUP_DOTS;
    use klappe_model::Note;
    use klappe_model::{
        Arrival, Camera, CameraMark, Character, Departure, Drawing, FigureTrack, Light, LightKind,
        Mark, Movement, ObjectShape, Pose, Sensor, SetObject, Shot, ShotSize, Wall,
    };

    fn project() -> Project {
        let mut p = Project::new("Test");
        p.characters = vec![
            Character {
                id: "e".into(),
                name: "Elisabeth".into(),
                short: "E".into(),
                color: FigureColor::Purple,
                group: None,
                size_m: None,
            },
            Character {
                id: "v".into(),
                name: "Burgvogt".into(),
                short: "B".into(),
                color: FigureColor::Blue,
                group: None,
                size_m: None,
            },
        ];
        p
    }

    fn set() -> SetPlan {
        SetPlan {
            width_m: 20.0,
            depth_m: 13.0,
            walls: vec![Wall::new(Point::new(0.0, 0.0), Point::new(20.0, 0.0), 0.2)],
            ..Default::default()
        }
    }

    fn mark(at_s: f64, x: f64, y: f64) -> Mark {
        Mark {
            at_s,
            at: Point::new(x, y),
            facing_deg: 0.0,
            pose: Pose::Standing,
            hold_s: 0.0,
            via: Vec::new(),
            arrival: Arrival::Walk,
            departure: Departure::Stay,
        }
    }

    /// The grips of `selected` on a plan whose set has no notes.
    fn grips_of(plan: &FloorPlan, selected: &Source) -> Vec<Item> {
        handles(&project(), &SetPlan::default(), plan, selected)
    }

    fn plan() -> FloorPlan {
        let mut p = FloorPlan::new("a", "Plan A");
        p.figures = vec![
            FigureTrack {
                character: "e".into(),
                marks: vec![mark(0.0, 2.0, 6.0), mark(8.0, 7.0, 6.0)],
            },
            FigureTrack {
                character: "v".into(),
                marks: vec![mark(0.0, 12.0, 6.0)],
            },
        ];
        p.cameras = vec![Camera {
            id: "a".into(),
            label: "A".into(),
            sensor: Sensor::Super35,
            lens_mm: 35.0,
            aspect: 2.39,
            marks: vec![CameraMark {
                at_s: 0.0,
                at: Point::new(8.0, 12.0),
                facing_deg: -90.0,
                height_m: 1.6,
                tilt_deg: 0.0,
                hold_s: 0.0,
                via: Vec::new(),
                rig: Default::default(),
                lens_mm: None,
            }],
            follows: None,
        }];
        p.lights = vec![Light {
            id: "l1".into(),
            label: "L1".into(),
            kind: LightKind::LedPanel,
            at: Point::new(18.0, 2.0),
            facing_deg: 135.0,
            beam_deg: 60.0,
            color_temp_k: 5600.0,
            power_w: 200.0,
        }];
        p
    }

    fn count(list: &DisplayList, role: Role) -> usize {
        list.items.iter().filter(|i| i.role == role).count()
    }

    #[test]
    fn draws_every_part_of_the_plan() {
        let list = render(&project(), &set(), &plan(), 0.0, None);

        assert_eq!(count(&list, Role::Wall), 1);
        assert_eq!(count(&list, Role::Light), 1);
        assert_eq!(count(&list, Role::Axis), 1);
        assert_eq!(count(&list, Role::Frustum), 1);
        assert_eq!(count(&list, Role::Mark(FigureColor::Purple)), 2);
        assert_eq!(count(&list, Role::Figure(FigureColor::Blue)), 2);
    }

    fn group_list() -> DisplayList {
        let mut p = project();
        p.characters[0].group = Some(12);
        render(&p, &set(), &plan(), 0.0, None)
    }

    /// A group is one larger body with three dots, however many people it counts, and its
    /// label names the count (LH-23).
    #[test]
    fn a_group_is_drawn_as_a_group() {
        let list = group_list();
        let purple = |i: &&Item| i.role == Role::Figure(FigureColor::Purple);
        let radii: Vec<f64> = list
            .items
            .iter()
            .filter(purple)
            .filter_map(|i| match i.shape {
                Shape::Circle { radius_m, .. } => Some(radius_m),
                _ => None,
            })
            .collect();
        let label = list
            .items
            .iter()
            .filter(purple)
            .find_map(|i| i.label.clone());

        assert_eq!(radii.len(), 1);
        assert!(radii[0] > FIGURE_RADIUS_M);
        assert_eq!(label.as_deref(), Some("Elisabeth (12)"));
    }

    /// The dots in a group's body are a role of their own: in the body's colour they vanish.
    #[test]
    fn group_dots_differ_from_the_body() {
        let list = group_list();
        let dots: Vec<&Item> = list
            .items
            .iter()
            .filter(|i| matches!(i.shape, Shape::Circle { radius_m, .. } if radius_m == GROUP_DOT_RADIUS_M))
            .collect();

        assert_eq!(dots.len(), GROUP_DOTS);
        assert!(dots
            .iter()
            .all(|i| i.role != Role::Figure(FigureColor::Purple)));
    }

    /// A single person keeps the one small circle and the plain name.
    #[test]
    fn a_single_figure_has_no_dots() {
        let list = render(&project(), &set(), &plan(), 0.0, None);
        let circles = list
            .items
            .iter()
            .filter(|i| i.role == Role::Figure(FigureColor::Purple))
            .filter(|i| matches!(i.shape, Shape::Circle { .. }))
            .count();
        let label = list.items.iter().find_map(|i| match i.role {
            Role::Figure(FigureColor::Purple) => i.label.clone(),
            _ => None,
        });

        assert_eq!(circles, 1);
        assert_eq!(label.as_deref(), Some("Elisabeth"));
    }

    /// Every part of the group, dots and rim included, selects the group's one track.
    #[test]
    fn the_whole_group_is_hit() {
        let list = group_list();
        let centre = Point::new(2.0, 6.0);
        // Beyond a single figure's radius, inside the group's.
        let rim = Point::new(2.0 + 0.5, 6.0);
        let hits = |at: Point| -> Vec<Option<Source>> {
            list.items
                .iter()
                .filter(|i| i.role == Role::Figure(FigureColor::Purple))
                .filter(|i| crate::hit::hit(i, at, 0.0))
                .map(|i| i.source.clone())
                .collect()
        };

        assert!(hits(rim).contains(&Some(Source::Figure("e".into()))));
        assert!(hits(centre)
            .iter()
            .all(|s| *s == Some(Source::Figure("e".into()))));
        assert!(!hits(Point::new(2.0 + 1.5, 6.0)).contains(&Some(Source::Figure("e".into()))));
    }

    #[test]
    fn walls_are_selectable_by_index() {
        let list = render(&project(), &set(), &plan(), 0.0, None);

        assert_eq!(list.items[0].source, Some(Source::Wall(0)));
    }

    #[test]
    fn figure_moves_with_time() {
        let at = |t: f64| {
            render(&project(), &set(), &plan(), t, None)
                .items
                .into_iter()
                .find_map(|i| match (i.role, i.shape) {
                    (Role::Figure(FigureColor::Purple), Shape::Circle { center, .. }) => {
                        Some(center)
                    }
                    _ => None,
                })
        };

        assert_eq!(at(0.0), Some(Point::new(2.0, 6.0)));
        assert_eq!(at(4.0), Some(Point::new(4.5, 6.0)));
    }

    /// A note is a card with its wrapped text as label, on top of the figures.
    #[test]
    fn notes_come_last_with_their_text() {
        let mut room = set();
        room.notes = vec![Note {
            id: "n1".into(),
            at: Point::new(3.0, 4.0),
            text: "Vorhang schließen".into(),
            width_m: 2.4,
        }];

        let list = render(&project(), &room, &plan(), 0.0, None);

        let last = list.items.last();
        assert_eq!(last.map(|i| i.role), Some(Role::Note));
        assert_eq!(
            last.and_then(|i| i.label.as_deref()),
            Some("Vorhang schließen")
        );
        assert_eq!(
            last.and_then(|i| i.source.clone()),
            Some(Source::Note("n1".into()))
        );
        let corners = match last.map(|i| i.shape.clone()) {
            Some(Shape::Polygon { points }) => points,
            other => panic!("not a card: {other:?}"),
        };
        assert_eq!(corners[0], Point::new(3.0, 4.0));
        assert!(corners[2].x_m > 3.0 && corners[2].y_m > 4.0);
    }

    fn drawn(shape: DrawingShape, fill: Fill) -> Drawing {
        Drawing {
            id: "d1".into(),
            shape,
            color: FigureColor::Red,
            dash: Dash::Dashed,
            fill,
            rotation_deg: 0.0,
        }
    }

    /// The drawing's items, in list order.
    fn items_of_drawing(drawing: Drawing) -> Vec<Item> {
        let mut p = plan();
        p.drawings = vec![drawing];

        render(&project(), &set(), &p, 0.0, None)
            .items
            .into_iter()
            .filter(|i| i.source == Some(Source::Drawing("d1".into())))
            .collect()
    }

    /// An arrow from (0, 0) to (4, 0): the shaft, and a head whose tip is (4, 0) and whose
    /// wings lie 0.45 m back at 25° either side of the shaft: x = 4 + 0.45 cos 155°.
    #[test]
    fn an_arrow_is_a_shaft_and_a_filled_head() {
        let arrow = drawn(
            DrawingShape::Arrow {
                from: Point::new(0.0, 0.0),
                to: Point::new(4.0, 0.0),
            },
            Fill::Outline,
        );

        let items = items_of_drawing(arrow);

        assert_eq!(items.len(), 2);
        assert_eq!(
            items[0].role,
            Role::Drawing {
                color: FigureColor::Red,
                dash: Dash::Dashed,
                fill: Fill::Outline
            }
        );
        assert_eq!(items[1].role, Role::ArrowHead(FigureColor::Red));
        let Shape::Polygon { points } = &items[1].shape else {
            panic!("head is no polygon: {:?}", items[1].shape);
        };
        let (wing_x, wing_y) = (
            4.0 + 0.45 * 155.0_f64.to_radians().cos(),
            0.45 * 25.0_f64.to_radians().sin(),
        );
        assert_eq!(points[1], Point::new(4.0, 0.0));
        assert!((points[0].x_m - wing_x).abs() < 1e-6 && (points[0].y_m - wing_y).abs() < 1e-6);
        assert!((points[2].x_m - wing_x).abs() < 1e-6 && (points[2].y_m + wing_y).abs() < 1e-6);
    }

    /// A 0.4 m arrow gets a head of 0.2 m: it never outgrows the shaft.
    #[test]
    fn a_short_arrow_has_a_short_head() {
        let head = arrow_head(Point::new(0.0, 0.0), Point::new(0.4, 0.0));

        let Some(Shape::Polygon { points }) = head else {
            panic!("no head");
        };
        assert!(points[0].distance_m(points[1]) <= 0.2 + 1e-9);
        assert_eq!(arrow_head(Point::new(1.0, 1.0), Point::new(1.0, 1.0)), None);
    }

    /// Corners dragged from bottom right to top left still come out clockwise from the top
    /// left; filled it is a polygon, as an outline a closed path.
    #[test]
    fn a_rectangle_has_four_corners_whichever_way_it_was_dragged() {
        let shape = |fill| {
            let rect = drawn(
                DrawingShape::Rect {
                    from: Point::new(3.0, 4.0),
                    to: Point::new(1.0, 2.0),
                },
                fill,
            );
            items_of_drawing(rect)[0].shape.clone()
        };
        let corners = vec![
            Point::new(1.0, 2.0),
            Point::new(3.0, 2.0),
            Point::new(3.0, 4.0),
            Point::new(1.0, 4.0),
        ];

        let mut closed = corners.clone();
        closed.push(corners[0]);
        assert_eq!(shape(Fill::Filled), Shape::Polygon { points: corners });
        assert_eq!(shape(Fill::Outline), Shape::Polyline { points: closed });
    }

    /// Ellipse in the box (0, 0)-(6, 2): centre (3, 1), radii 3 and 1; starts at its right
    /// end (6, 1); every corner satisfies ((x-3)/3)^2 + ((y-1)/1)^2 = 1.
    #[test]
    fn an_ellipse_is_inscribed_in_its_box() {
        let ellipse = drawn(
            DrawingShape::Ellipse {
                from: Point::new(6.0, 2.0),
                to: Point::new(0.0, 0.0),
            },
            Fill::Filled,
        );

        let items = items_of_drawing(ellipse);

        let Shape::Polygon { points } = &items[0].shape else {
            panic!("not a polygon: {:?}", items[0].shape);
        };
        assert_eq!(points.len(), ELLIPSE_CORNERS);
        assert!((points[0].x_m - 6.0).abs() < 1e-9 && (points[0].y_m - 1.0).abs() < 1e-9);
        for p in points {
            let r = ((p.x_m - 3.0) / 3.0).powi(2) + (p.y_m - 1.0).powi(2);
            assert!((r - 1.0).abs() < 1e-9, "{p:?}");
        }
    }

    /// A click inside a hollow shape reaches what lies below it; inside a filled one it
    /// takes the shape.
    #[test]
    fn only_a_filled_shape_is_taken_from_inside() {
        let at = |fill| {
            let rect = drawn(
                DrawingShape::Rect {
                    from: Point::new(0.0, 0.0),
                    to: Point::new(4.0, 4.0),
                },
                fill,
            );
            let item = items_of_drawing(rect).remove(0);
            (
                crate::hit::hit(&item, Point::new(2.0, 2.0), 0.15),
                crate::hit::hit(&item, Point::new(4.05, 2.0), 0.15),
            )
        };

        assert_eq!(at(Fill::Outline), (false, true));
        assert_eq!(at(Fill::Filled), (true, true));
    }

    /// An area is as drawn: its points, as a polygon when filled.
    #[test]
    fn an_area_keeps_its_points() {
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(2.0, 0.0),
            Point::new(1.0, 2.0),
        ];

        let items = items_of_drawing(drawn(
            DrawingShape::Area {
                points: points.clone(),
            },
            Fill::Filled,
        ));

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].shape, Shape::Polygon { points });
    }

    /// Drawings lie on the set (after the wall) and under the lamps, cameras and figures.
    #[test]
    fn drawings_lie_between_set_and_figures() {
        let mut p = plan();
        p.drawings = vec![drawn(
            DrawingShape::Line {
                from: Point::new(1.0, 1.0),
                to: Point::new(2.0, 2.0),
            },
            Fill::Outline,
        )];

        let list = render(&project(), &set(), &p, 0.0, None);

        let roles: Vec<_> = list.items.iter().map(|i| i.role).collect();
        let at = |f: &dyn Fn(&Role) -> bool| roles.iter().position(f);
        let drawing = at(&|r| matches!(r, Role::Drawing { .. }));
        assert!(at(&|r| *r == Role::Wall) < drawing);
        assert!(drawing < at(&|r| *r == Role::Light));
        assert!(drawing < at(&|r| matches!(r, Role::Figure(_))));
    }

    #[test]
    fn walls_come_first_figures_last() {
        let list = render(&project(), &set(), &plan(), 0.0, None);

        assert_eq!(list.items.first().map(|i| i.role), Some(Role::Wall));
        assert!(matches!(
            list.items.last().map(|i| i.role),
            Some(Role::Figure(_))
        ));
    }

    /// 1000 x 500 px set 12.5 m wide is 6.25 m deep; it is the first, hindmost item.
    #[test]
    fn background_is_first_and_keeps_its_aspect() {
        let mut room = set();
        room.background = Some(Background {
            file: "bilder/saal.png".into(),
            at: Point::new(1.0, 2.0),
            width_m: 12.5,
            rotation_deg: 30.0,
            opacity: 0.5,
        });

        let list = render(&project(), &room, &plan(), 0.0, Some((1000, 500)));

        assert_eq!(
            list.items.first(),
            Some(&Item {
                role: Role::Background,
                shape: Shape::Image {
                    file: "bilder/saal.png".into(),
                    at: Point::new(1.0, 2.0),
                    width_m: 12.5,
                    depth_m: 6.25,
                    rotation_deg: 30.0,
                    opacity: 0.5,
                },
                label: None,
                source: None,
            })
        );
        assert_eq!(list.items.get(1).map(|i| i.role), Some(Role::Wall));
    }

    #[test]
    fn background_needs_a_set_entry_and_a_size() {
        let mut room = set();
        let none_set = render(&project(), &room, &plan(), 0.0, Some((10, 10)));
        room.background = Some(Background {
            file: "bilder/saal.png".into(),
            at: Point::default(),
            width_m: 5.0,
            rotation_deg: 0.0,
            opacity: 1.0,
        });
        let unknown_size = render(&project(), &room, &plan(), 0.0, None);
        let zero_size = render(&project(), &room, &plan(), 0.0, Some((0, 10)));

        for list in [none_set, unknown_size, zero_size] {
            assert_eq!(count(&list, Role::Background), 0);
        }
    }

    /// A 2 m wide, flat object turned by 90° stands upright: corners at y = -1 and y = 1.
    #[test]
    fn objects_turn_clockwise() {
        let o = SetObject {
            id: "t".into(),
            kind: "table".into(),
            label: String::new(),
            at: Point::new(0.0, 0.0),
            width_m: 2.0,
            depth_m: 0.0,
            rotation_deg: 90.0,
            shape: ObjectShape::Rect,
            height_m: None,
            base_m: None,
        };
        let corners = object_corners(&o);

        assert!((corners[0].x_m).abs() < 1e-9 && (corners[0].y_m + 1.0).abs() < 1e-9);
        assert!((corners[1].x_m).abs() < 1e-9 && (corners[1].y_m - 1.0).abs() < 1e-9);
    }

    fn object(kind: &str, shape: ObjectShape, rotation_deg: f64) -> SetObject {
        SetObject {
            id: "o1".into(),
            kind: kind.into(),
            label: String::new(),
            at: Point::new(10.0, 5.0),
            width_m: 4.0,
            depth_m: 2.0,
            rotation_deg,
            shape,
            height_m: None,
            base_m: None,
        }
    }

    /// A 4 by 2 m ellipse at (10, 5): rightmost point (12, 5), lowest (10, 6), every corner
    /// on the curve; turned by 90° the long axis stands upright.
    #[test]
    fn an_ellipse_object_is_inscribed_in_its_box() {
        let corners = object_corners(&object("pillar", ObjectShape::Ellipse, 0.0));

        assert_eq!(corners.len(), ELLIPSE_CORNERS);
        assert!((corners[0].x_m - 12.0).abs() < 1e-9 && (corners[0].y_m - 5.0).abs() < 1e-9);
        let quarter = corners[ELLIPSE_CORNERS / 4];
        assert!((quarter.x_m - 10.0).abs() < 1e-9 && (quarter.y_m - 6.0).abs() < 1e-9);
        for c in &corners {
            let (dx, dy) = ((c.x_m - 10.0) / 2.0, (c.y_m - 5.0) / 1.0);
            assert!((dx * dx + dy * dy - 1.0).abs() < 1e-9);
        }

        let upright = object_corners(&object("pillar", ObjectShape::Ellipse, 90.0));
        assert!((upright[0].x_m - 10.0).abs() < 1e-9 && (upright[0].y_m - 7.0).abs() < 1e-9);
    }

    /// An object whose kind no library knows (an old project) is a rectangle of its own size.
    #[test]
    fn an_unknown_kind_is_drawn_as_a_rectangle() {
        let corners = object_corners(&object("zeppelin", ObjectShape::default(), 0.0));

        assert_eq!(corners.len(), 4);
        assert_eq!((corners[0].x_m, corners[0].y_m), (8.0, 4.0));
        assert_eq!((corners[2].x_m, corners[2].y_m), (12.0, 6.0));
    }

    /// The nose points where the body faces: 90° is down.
    #[test]
    fn nose_points_where_it_faces() {
        let tip = nose(Point::new(0.0, 0.0), 90.0, 1.0)[1];

        assert!(tip.x_m.abs() < 1e-9);
        assert!((tip.y_m - (1.0 + NOSE_M)).abs() < 1e-9);
    }

    /// The axis runs through both figures and on by the set's diagonal on each side.
    #[test]
    fn axis_reaches_across_the_set() {
        let list = render(&project(), &set(), &plan(), 0.0, None);
        let axis = list.items.iter().find_map(|i| match (i.role, &i.shape) {
            (Role::Axis, Shape::Line { from, to, .. }) => Some((*from, *to)),
            _ => None,
        });
        let diagonal = 20.0_f64.hypot(13.0);

        let (from, to) = axis.unwrap();
        assert!((from.x_m - (2.0 - diagonal)).abs() < 1e-9 && (from.y_m - 6.0).abs() < 1e-9);
        assert!((to.x_m - (12.0 + diagonal)).abs() < 1e-9);
    }

    fn shot(lens_mm: Option<f64>, from_s: f64, to_s: f64) -> Shot {
        Shot {
            id: "s".into(),
            number: "1.1".into(),
            camera: "a".into(),
            size: ShotSize::Wide,
            angle: Default::default(),
            movement: Movement::Static,
            lens_mm,
            from_s,
            to_s,
            description: String::new(),
            props: String::new(),
            gear: String::new(),
            crew: String::new(),
            note: String::new(),
        }
    }

    fn frustum_of(plan: &FloorPlan, t_s: f64) -> Vec<Point> {
        render(&project(), &set(), plan, t_s, None)
            .items
            .into_iter()
            .find_map(|i| match (i.role, i.shape) {
                (Role::Frustum, Shape::Polygon { points }) => Some(points),
                _ => None,
            })
            .unwrap_or_default()
    }

    /// The shot's own lens wins while it runs; outside it the camera's lens applies.
    #[test]
    fn frustum_follows_the_shots_lens() {
        let mut p = plan();
        p.shots = vec![shot(Some(40.0), 0.0, 4.0)];
        let fov = |lens_mm: f64| horizontal_fov_deg(Sensor::Super35, lens_mm);
        let expect = |lens_mm: f64| {
            frustum(
                Point::new(8.0, 12.0),
                -90.0,
                fov(lens_mm),
                20.0_f64.hypot(13.0),
            )
        };

        let during = frustum_of(&p, 2.0);
        let after = frustum_of(&p, 5.0);

        assert!(
            (during[1].x_m - expect(40.0)[1].x_m).abs() < 1e-6,
            "{during:?}"
        );
        assert!(
            (after[1].x_m - expect(35.0)[1].x_m).abs() < 1e-6,
            "{after:?}"
        );
    }

    /// A lens a mark states reaches the frustum; a shot's lens does not count then (LH-33).
    #[test]
    fn frustum_follows_the_lens_of_the_marks() {
        let mut by_mark = plan();
        by_mark.cameras[0].marks[0].lens_mm = Some(70.0);
        by_mark.shots = vec![shot(Some(40.0), 0.0, 4.0)];
        let mut by_shot = plan();
        by_shot.shots = vec![shot(Some(70.0), 0.0, 4.0)];

        assert_eq!(frustum_of(&by_mark, 2.0), frustum_of(&by_shot, 2.0));
    }

    #[test]
    fn unknown_character_is_skipped() {
        let mut p = plan();
        p.figures[0].character = "nobody".into();

        let list = render(&project(), &set(), &p, 0.0, None);

        assert_eq!(count(&list, Role::Mark(FigureColor::Purple)), 0);
    }

    fn path_of(list: &DisplayList, role: Role) -> Vec<Point> {
        let item = list.items.iter().find(|i| i.role == role).unwrap();
        match &item.shape {
            Shape::Polyline { points } => points.clone(),
            other => panic!("{other:?}"),
        }
    }

    /// A way with a `via` is drawn along the curve; without, it stays two points.
    #[test]
    fn a_curved_way_is_a_polyline_along_the_curve() {
        let mut p = plan();
        let straight = render(&project(), &set(), &p, 0.0, None);
        p.figures[0].marks[1].via = vec![Point::new(4.5, 3.0)];
        let curved = render(&project(), &set(), &p, 0.0, None);

        let role = Role::FigurePath(FigureColor::Purple);
        assert_eq!(path_of(&straight, role).len(), 2);

        // From (2, 6) over (4.5, 3) to (7, 6): the way passes the bend.
        let points = path_of(&curved, role);
        assert!(points.len() > 2);
        assert!(points.contains(&Point::new(4.5, 3.0)));
        assert_eq!(points.first(), Some(&Point::new(2.0, 6.0)));
        assert_eq!(points.last(), Some(&Point::new(7.0, 6.0)));
    }

    /// The camera's path bends the same way.
    #[test]
    fn a_curved_camera_path_is_a_polyline_too() {
        let mut p = plan();
        let mut second = p.cameras[0].marks[0].clone();
        second.at = Point::new(12.0, 12.0);
        second.via = vec![Point::new(10.0, 6.0)];
        p.cameras[0].marks.push(second);

        let list = render(&project(), &set(), &p, 0.0, None);

        assert!(path_of(&list, Role::CameraPath { dash: Dash::Dashed }).len() > 2);
    }

    /// Grips: one per bend of the selected way and one to pull a new bend out of each piece
    /// between them, none in `render`.
    #[test]
    fn only_the_selected_way_has_handles() {
        let mut p = plan();
        p.figures[0].marks[1].via = vec![Point::new(4.5, 3.0)];

        assert!(render(&project(), &set(), &p, 0.0, None)
            .items
            .iter()
            .all(|i| !matches!(i.source, Some(Source::FigureVia { .. }))));

        let via = |bend| Source::FigureVia {
            character: "e".into(),
            index: 1,
            bend,
        };
        let grips = grips_of(&p, &Source::Figure("e".into()));
        let sources: Vec<_> = grips.iter().map(|g| g.source.clone()).collect();
        assert_eq!(
            sources,
            [
                Some(via(Bend::New(0))),
                Some(via(Bend::New(1))),
                Some(via(Bend::At(0)))
            ]
        );
        assert!(grips
            .iter()
            .all(|g| g.role == Role::FigureHandle(FigureColor::Purple)));
        assert!(matches!(
            grips[2].shape,
            Shape::Circle { center, .. } if center == Point::new(4.5, 3.0)
        ));

        // The grip of a selected grip, and of a mark, belong to the same way.
        assert_eq!(grips_of(&p, &via(Bend::At(0))), grips);
        let mark = Source::Mark {
            character: "e".into(),
            index: 0,
        };
        assert_eq!(grips_of(&p, &mark), grips);

        // Another way's, a one-mark way's, and a wall's: none.
        assert!(grips_of(&p, &Source::Figure("v".into())).is_empty());
        assert!(grips_of(&p, &Source::Wall(0)).is_empty());
    }

    #[test]
    fn a_camera_has_handles_when_selected() {
        let mut p = plan();
        let mut second = p.cameras[0].marks[0].clone();
        second.at = Point::new(12.0, 12.0);
        p.cameras[0].marks.push(second);

        let grips = grips_of(&p, &Source::Camera("a".into()));

        assert_eq!(grips.len(), 1);
        assert_eq!(grips[0].role, Role::CameraHandle);
        assert_eq!(
            grips[0].source,
            Some(Source::CameraVia {
                camera: "a".into(),
                index: 1,
                bend: Bend::New(0)
            })
        );
    }

    /// Elisabeth: mark 1 at 0 s, mark 2 (leaves) at 10 s, mark 3 (appears) at 20 s.
    fn coming_and_going() -> FloorPlan {
        let mut p = plan();
        let mut track = FigureTrack {
            character: "e".into(),
            marks: vec![
                mark(0.0, 1.0, 1.0),
                mark(10.0, 5.0, 1.0),
                mark(20.0, 9.0, 1.0),
            ],
        };
        track.marks[1].departure = Departure::Leave;
        track.marks[2].arrival = Arrival::Appear;
        p.figures = vec![track];

        p
    }

    fn bodies(list: &DisplayList) -> usize {
        count(list, Role::Figure(FigureColor::Purple))
    }

    /// A figure that is not in the scene is not drawn; its marks and way stay (LH-26).
    #[test]
    fn a_figure_out_of_the_scene_is_not_drawn() {
        let p = coming_and_going();
        let at = |t_s| render(&project(), &set(), &p, t_s, None);

        assert_eq!(bodies(&at(5.0)), 2);
        assert_eq!(bodies(&at(15.0)), 0);
        assert_eq!(bodies(&at(20.0)), 2);
        assert_eq!(count(&at(15.0), Role::Mark(FigureColor::Purple)), 3);
        assert_eq!(count(&at(15.0), Role::FigurePath(FigureColor::Purple)), 1);
    }

    /// The way to a mark that is appeared at is not drawn: two marks and no way after it.
    #[test]
    fn no_way_leads_to_an_appearing_mark() {
        let mut p = coming_and_going();
        let way = |p: &FloorPlan| {
            let list = render(&project(), &set(), p, 0.0, None);
            list.items
                .iter()
                .filter_map(|i| match (&i.role, &i.shape) {
                    (Role::FigurePath(_), Shape::Polyline { points }) => Some(points.len()),
                    _ => None,
                })
                .collect::<Vec<_>>()
        };

        assert_eq!(way(&p), [2]);

        // Review of PR 35: after a leave the figure is gone at a walked-to mark as well, so
        // no way leads there either.
        p.figures[0].marks[2].arrival = Arrival::Walk;
        assert_eq!(way(&p), [2]);

        p.figures[0].marks[1].departure = Departure::Stay;
        assert_eq!(way(&p), [3]);
    }

    /// Review of PR 35: no grip on a way after a leave, which the figure does not walk.
    #[test]
    fn a_way_after_a_leave_has_no_grip() {
        let mut p = coming_and_going();
        p.figures[0].marks[2].arrival = Arrival::Walk;
        let grips = grips_of(&p, &Source::Figure("e".into()));

        assert_eq!(grips.len(), 1);
    }

    /// An appearing mark has no way, so no grip to bend it (LH-26).
    #[test]
    fn an_appearing_mark_has_no_grip() {
        let p = coming_and_going();
        let grips = grips_of(&p, &Source::Figure("e".into()));

        assert_eq!(grips.len(), 1);
        assert_eq!(
            grips[0].source,
            Some(Source::FigureVia {
                character: "e".into(),
                index: 1,
                bend: Bend::New(0)
            })
        );
    }

    /// The mark's number says it: `+` where the figure appears, `-` where it leaves.
    #[test]
    fn marks_show_appearing_and_leaving() {
        let list = render(&project(), &set(), &coming_and_going(), 0.0, None);
        let labels: Vec<_> = list
            .items
            .iter()
            .filter(|i| matches!(i.role, Role::Mark(FigureColor::Purple)))
            .filter_map(|i| i.label.as_deref())
            .collect();

        assert_eq!(labels, ["1", "2-", "+3"]);
    }
}
