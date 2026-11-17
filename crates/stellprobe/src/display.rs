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
use crate::prop::set_at;
use crate::rig::{rig_dash, rig_runs};

mod handles;
mod images;
mod objects;

pub use handles::{
    drawing_edge_places, drawing_handle_places, handles, image_grip_place, object_grip_place, Bend,
    LightGrip, ObjectGrip, OpeningEnd,
};
use images::image_items;
pub use objects::label_point;
pub(crate) use objects::object_corners;
use objects::{set_items, track_items};

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
    /// A photo or sketch placed on the plan (LH-18): its picture, when the file can be read,
    /// and the frame that is the element, with the file's name while there is no picture.
    Picture,
    Wall,
    /// A door in a wall (LH-10): the leaf and the quarter circle it swings through.
    Door,
    /// A window in a wall (LH-10): two thin lines in the gap.
    Window,
    Object,
    LightBeam,
    Light,
    /// Sunlight through a window on the floor (LH-52): a patch under the set's walls.
    SunPatch,
    /// The arrow of the sun's direction of travel (LH-52), its azimuth and elevation as label.
    SunRay,
    /// The arrow of north (LH-52).
    North,
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
    /// A stroke of the sketch pen (LH-120), a polyline over the whole plan.
    Sketch,
    /// The filled head of an arrow, in the arrow's colour.
    ArrowHead(FigureColor),
    FigurePath(FigureColor),
    /// A grip on the selected element (drawing, note, image, door or window, lamp, object); only
    /// drawn while it is selected. The element it belongs to is `Source::owner` of the
    /// item's source.
    Handle,
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

impl Source {
    /// The element a grip of this source stands for: a click on the grip selects it, its layer
    /// is the grip's. `None` for an element, and for a grip on a way's bend, which is selected
    /// itself (it is the bend to straighten).
    pub fn owner(&self) -> Option<Source> {
        match self {
            Source::DrawingPoint { drawing, .. } | Source::DrawingEdge { drawing, .. } => {
                Some(Source::Drawing(drawing.clone()))
            }
            Source::NoteWidth(note) => Some(Source::Note(note.clone())),
            Source::ImageSize(image) => Some(Source::Image(image.clone())),
            Source::ObjectGrip { object, .. } => Some(Source::Object(object.clone())),
            Source::OpeningEdge { wall, index, .. } => Some(Source::Opening {
                wall: *wall,
                index: *index,
            }),
            Source::LightGrip { light, .. } => Some(Source::Light(light.clone())),
            // Elements, and the bend grips of a way, which are selected themselves.
            Source::Object(_)
            | Source::Light(_)
            | Source::Camera(_)
            | Source::Figure(_)
            | Source::Mark { .. }
            | Source::ObjectMark { .. }
            | Source::FigureVia { .. }
            | Source::CameraVia { .. }
            | Source::Wall(_)
            | Source::Opening { .. }
            | Source::Note(_)
            | Source::Image(_)
            | Source::Drawing(_) => None,
        }
    }

    /// Whether a click on this grip selects the grip itself, not its element: a drawing's point,
    /// a free form's point (both so Delete takes the point out) and the bend of a way (the bend to
    /// straighten).
    pub fn selected_itself(&self) -> bool {
        matches!(
            self,
            Source::DrawingPoint { .. }
                | Source::ObjectGrip {
                    grip: ObjectGrip::Point(_),
                    ..
                }
                | Source::FigureVia { .. }
                | Source::CameraVia { .. }
        )
    }
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
    /// Mark `index` of a set object's track (LH-24).
    ObjectMark {
        object: String,
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
    /// The grip just outside the right or bottom edge of a set object's box (LH-25): drag to
    /// set its width or depth, the left or top edge stays.
    ObjectGrip {
        object: String,
        grip: ObjectGrip,
    },
    Note(String),
    /// The grip in the middle of the right edge of a note's card (LH-16): drag to set its width.
    NoteWidth(String),
    /// A photo or sketch (LH-18).
    Image(String),
    /// The grip just outside the middle of the right edge of an image (LH-18): drag to set its
    /// width, the left edge stays.
    ImageSize(String),
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
/// `image_px` is the pixel size of an image file of the set, the background's included;
/// Stellprobe reads no files, so the caller looks them up. Without a size the background is
/// left out and an image is drawn as an empty frame with its file name.
pub fn render(
    project: &Project,
    set: &SetPlan,
    plan: &FloorPlan,
    t_s: f64,
    image_px: &dyn Fn(&str) -> Option<(u32, u32)>,
) -> DisplayList {
    let mut items = Vec::new();
    let reach_m = set_diagonal_m(set).unwrap_or(VIEW_REACH_FALLBACK_M);
    let set = &*set_at(set, plan, t_s);

    if let Some(background) = &set.background {
        if let Some(px) = image_px(&background.file) {
            background_item(background, px, &mut items);
        }
    }
    image_items(set, image_px, &mut items);
    set_items(set, &mut items);
    track_items(set, plan, &mut items);
    drawing_items(plan, &mut items);
    light_items(plan, &mut items);
    axis_items(plan, t_s, reach_m, &mut items);
    camera_items(plan, t_s, reach_m, &mut items);
    figure_items(project, plan, t_s, &mut items);
    note_items(set, plan, &mut items);
    sketch_items(plan, &mut items);

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

/// The pen's strokes lie over everything and belong to no element, so a click goes through
/// them: only the clear command takes them away.
fn sketch_items(plan: &FloorPlan, items: &mut Vec<Item>) {
    for sketch in &plan.sketches {
        items.push(Item {
            role: Role::Sketch,
            shape: Shape::Polyline {
                points: sketch.points.clone(),
            },
            label: None,
            source: None,
        });
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

/// Notes lie on top of everything, so a figure never hides one. The plan's labels of figures
/// and marks (LH-118) are drawn as notes are.
fn note_items(set: &SetPlan, plan: &FloorPlan, items: &mut Vec<Item>) {
    for note in set.notes.iter().chain(&plan.labels) {
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
                    points: path(run.iter().map(|m| (m.at, m.via.as_slice(), m.curve))),
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
                    points: path(way.iter().map(|(m, _)| (m.at, m.via.as_slice(), m.curve))),
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
mod tests;
