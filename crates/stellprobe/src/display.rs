use klappe_model::{
    Background, Dash, DrawingShape, FigureColor, Fill, FloorPlan, Note, ObjectShape, Point,
    Project, SetObject, SetPlan,
};

use crate::axis::axis_at;
use crate::motion::{bend_point, camera_at, figure_at, path};
use crate::optics::{frustum, horizontal_fov_deg, lens_at};

/// What the GUI draws and the export writes, back to front, in metres (planung/KONZEPT.md,
/// section 4). Colours are roles; the GUI maps them to Kante.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DisplayList {
    pub items: Vec<Item>,
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
    Object,
    LightBeam,
    Light,
    Axis,
    CameraPath,
    Frustum,
    Camera,
    /// A text pinned to the plan (LH-16): a small marker, the text as its label.
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
    /// The grip that bends the way to a mark (LH-21); only drawn for a selected way.
    FigureHandle(FigureColor),
    CameraHandle,
    Mark(FigureColor),
    Figure(FigureColor),
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
    /// The bend of the way to mark `index` of a character's track (LH-21).
    FigureVia {
        character: String,
        index: usize,
    },
    /// The bend of the way to mark `index` of a camera (LH-21).
    CameraVia {
        camera: String,
        index: usize,
    },
    /// Index into the set's walls.
    Wall(usize),
    Note(String),
    Drawing(String),
}

/// Figure body radius on the plan.
pub const FIGURE_RADIUS_M: f64 = 0.35;
/// Mark circles are smaller than the figure.
const MARK_RADIUS_M: f64 = 0.22;
/// Camera and lamp bodies.
const BODY_RADIUS_M: f64 = 0.25;
/// The grip on a way's bend.
const HANDLE_RADIUS_M: f64 = 0.15;
/// The marker a note is pinned with.
const NOTE_RADIUS_M: f64 = 0.15;
/// How long an arrow's head is, and how far its wings stand off the shaft.
const ARROW_HEAD_M: f64 = 0.45;
const ARROW_HEAD_HALF_DEG: f64 = 25.0;
/// Corners of the polygon an ellipse is drawn with.
const ELLIPSE_CORNERS: usize = 48;
/// How far a lamp's beam is drawn.
const BEAM_REACH_M: f64 = 4.0;
/// How far a camera's view is drawn when the set gives no size.
const VIEW_REACH_FALLBACK_M: f64 = 8.0;
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

fn set_items(set: &SetPlan, items: &mut Vec<Item>) {
    for (index, wall) in set.walls.iter().enumerate() {
        items.push(Item {
            role: Role::Wall,
            shape: Shape::Line {
                from: wall.from,
                to: wall.to,
                width_m: wall.thickness_m,
            },
            label: None,
            source: Some(Source::Wall(index)),
        });
    }

    for object in &set.objects {
        items.push(Item {
            role: Role::Object,
            shape: Shape::Polygon {
                points: object_corners(object),
            },
            label: Some(object.label.clone()).filter(|l| !l.is_empty()),
            source: Some(Source::Object(object.id.clone())),
        });
    }
}

/// The outline of a set object, rotated about its centre: four corners for a rectangle, the
/// ellipse inscribed in that rectangle otherwise.
fn object_corners(o: &SetObject) -> Vec<Point> {
    let (hw, hd) = (o.width_m / 2.0, o.depth_m / 2.0);
    let (sin, cos) = o.rotation_deg.to_radians().sin_cos();

    let local = match o.shape {
        ObjectShape::Rect => vec![(-hw, -hd), (hw, -hd), (hw, hd), (-hw, hd)],
        ObjectShape::Ellipse => (0..ELLIPSE_CORNERS)
            .map(|i| {
                let turn = std::f64::consts::TAU * i as f64 / ELLIPSE_CORNERS as f64;
                (hw * turn.cos(), hd * turn.sin())
            })
            .collect(),
    };

    local
        .iter()
        .map(|(x, y)| Point::new(o.at.x_m + x * cos - y * sin, o.at.y_m + x * sin + y * cos))
        .collect()
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
        let mut push = |role, shape| {
            items.push(Item {
                role,
                shape,
                label: None,
                source: source.clone(),
            });
        };

        match &drawing.shape {
            DrawingShape::Line { from, to } => push(role, line(*from, *to)),
            DrawingShape::Arrow { from, to } => {
                push(role, line(*from, *to));
                if let Some(head) = arrow_head(*from, *to) {
                    push(Role::ArrowHead(drawing.color), head);
                }
            }
            DrawingShape::Rect { from, to } => {
                push(role, closed(rect_corners(*from, *to), drawing.fill));
            }
            DrawingShape::Ellipse { from, to } => {
                push(role, closed(ellipse_corners(*from, *to), drawing.fill));
            }
            DrawingShape::Area { points } => push(role, closed(points.clone(), drawing.fill)),
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
    for Note { id, at, text } in &set.notes {
        items.push(Item {
            role: Role::Note,
            shape: Shape::Circle {
                center: *at,
                radius_m: NOTE_RADIUS_M,
            },
            label: Some(text.clone()),
            source: Some(Source::Note(id.clone())),
        });
    }
}

fn light_items(plan: &FloorPlan, items: &mut Vec<Item>) {
    for light in &plan.lights {
        let source = Some(Source::Light(light.id.clone()));
        let beam = frustum(light.at, light.facing_deg, light.beam_deg, BEAM_REACH_M);

        items.push(Item {
            role: Role::LightBeam,
            shape: Shape::Polygon {
                points: beam.to_vec(),
            },
            label: None,
            source: source.clone(),
        });
        items.push(Item {
            role: Role::Light,
            shape: Shape::Circle {
                center: light.at,
                radius_m: BODY_RADIUS_M,
            },
            label: Some(light.label.clone()),
            source,
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

        if camera.marks.len() > 1 {
            items.push(Item {
                role: Role::CameraPath,
                shape: Shape::Polyline {
                    points: path(camera.marks.iter().map(|m| (m.at, m.via))),
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

        if track.marks.len() > 1 {
            items.push(Item {
                role: Role::FigurePath(color),
                shape: Shape::Polyline {
                    points: path(track.marks.iter().map(|m| (m.at, m.via))),
                },
                label: None,
                source: Some(Source::Figure(track.character.clone())),
            });
        }

        // Numbered marks 1, 2, 3 …
        for (index, mark) in track.marks.iter().enumerate() {
            items.push(Item {
                role: Role::Mark(color),
                shape: Shape::Circle {
                    center: mark.at,
                    radius_m: MARK_RADIUS_M,
                },
                label: Some((index + 1).to_string()),
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
        items.push(Item {
            role: Role::Figure(color),
            shape: Shape::Polygon {
                points: nose(place.at, place.facing_deg, FIGURE_RADIUS_M),
            },
            label: None,
            source: source.clone(),
        });
        items.push(Item {
            role: Role::Figure(color),
            shape: Shape::Circle {
                center: place.at,
                radius_m: FIGURE_RADIUS_M,
            },
            label: Some(character.name.clone()),
            source,
        });
    }
}

/// The grips on the bends of the way `selected` belongs to: a figure, camera or mark of it,
/// or one of its grips (LH-21). One per mark after the first, halfway along the way to it.
/// Only the selected way shows them, so the plan is not full of dots; `render` leaves them
/// out, and with it the export.
pub fn handles(project: &Project, plan: &FloorPlan, selected: &Source) -> Vec<Item> {
    match selected {
        Source::Figure(c)
        | Source::Mark { character: c, .. }
        | Source::FigureVia { character: c, .. } => figure_handles(project, plan, c),
        Source::Camera(id) | Source::CameraVia { camera: id, .. } => camera_handles(plan, id),
        _ => Vec::new(),
    }
}

fn figure_handles(project: &Project, plan: &FloorPlan, character: &str) -> Vec<Item> {
    let (Some(track), Some(who)) = (plan.figure(character), project.character(character)) else {
        return Vec::new();
    };
    let stops: Vec<_> = track.marks.iter().map(|m| (m.at, m.via)).collect();

    handle_places(&stops)
        .into_iter()
        .map(|(index, center)| Item {
            role: Role::FigureHandle(who.color),
            shape: handle_shape(center),
            label: None,
            source: Some(Source::FigureVia {
                character: character.to_owned(),
                index,
            }),
        })
        .collect()
}

fn camera_handles(plan: &FloorPlan, id: &str) -> Vec<Item> {
    let Some(camera) = plan.camera(id) else {
        return Vec::new();
    };
    let stops: Vec<_> = camera.marks.iter().map(|m| (m.at, m.via)).collect();

    handle_places(&stops)
        .into_iter()
        .map(|(index, center)| Item {
            role: Role::CameraHandle,
            shape: handle_shape(center),
            label: None,
            source: Some(Source::CameraVia {
                camera: id.to_owned(),
                index,
            }),
        })
        .collect()
}

fn handle_shape(center: Point) -> Shape {
    Shape::Circle {
        center,
        radius_m: HANDLE_RADIUS_M,
    }
}

/// Mark index and place of each grip: halfway along the curve, or the leg.
fn handle_places(stops: &[(Point, Option<Point>)]) -> Vec<(usize, Point)> {
    stops
        .windows(2)
        .enumerate()
        .map(|(i, pair)| (i + 1, bend_point(pair[0].0, pair[1].1, pair[1].0)))
        .collect()
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
    use klappe_model::{
        Camera, CameraMark, Character, Drawing, FigureTrack, Light, LightKind, Mark, Movement,
        Pose, Sensor, Shot, ShotSize, Wall,
    };

    fn project() -> Project {
        let mut p = Project::new("Test");
        p.characters = vec![
            Character {
                id: "e".into(),
                name: "Elisabeth".into(),
                short: "E".into(),
                color: FigureColor::Purple,
            },
            Character {
                id: "v".into(),
                name: "Burgvogt".into(),
                short: "B".into(),
                color: FigureColor::Blue,
            },
        ];
        p
    }

    fn set() -> SetPlan {
        SetPlan {
            width_m: 20.0,
            depth_m: 13.0,
            walls: vec![Wall {
                from: Point::new(0.0, 0.0),
                to: Point::new(20.0, 0.0),
                thickness_m: 0.2,
            }],
            objects: vec![],
            background: None,
            notes: Vec::new(),
        }
    }

    fn mark(at_s: f64, x: f64, y: f64) -> Mark {
        Mark {
            at_s,
            at: Point::new(x, y),
            facing_deg: 0.0,
            pose: Pose::Standing,
            hold_s: 0.0,
            via: None,
        }
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
                via: None,
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

    /// A note is a marker with its text as label, on top of the figures.
    #[test]
    fn notes_come_last_with_their_text() {
        let mut room = set();
        room.notes = vec![Note {
            id: "n1".into(),
            at: Point::new(3.0, 4.0),
            text: "Vorhang schließen".into(),
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
        assert_eq!(
            last.map(|i| i.shape.clone()),
            Some(Shape::Circle {
                center: Point::new(3.0, 4.0),
                radius_m: NOTE_RADIUS_M,
            })
        );
    }

    fn drawn(shape: DrawingShape, fill: Fill) -> Drawing {
        Drawing {
            id: "d1".into(),
            shape,
            color: FigureColor::Red,
            dash: Dash::Dashed,
            fill,
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
        p.figures[0].marks[1].via = Some(Point::new(4.5, 0.0));
        let curved = render(&project(), &set(), &p, 0.0, None);

        let role = Role::FigurePath(FigureColor::Purple);
        assert_eq!(path_of(&straight, role).len(), 2);

        // From (2, 6) over (4.5, 0) to (7, 6): halfway along it is at (4.5, 3).
        let points = path_of(&curved, role);
        assert!(points.len() > 2);
        assert!(points
            .iter()
            .any(|q| q.distance_m(Point::new(4.5, 3.0)) < 1e-9));
        assert_eq!(points.first(), Some(&Point::new(2.0, 6.0)));
        assert_eq!(points.last(), Some(&Point::new(7.0, 6.0)));
    }

    /// The camera's path bends the same way.
    #[test]
    fn a_curved_camera_path_is_a_polyline_too() {
        let mut p = plan();
        let mut second = p.cameras[0].marks[0].clone();
        second.at = Point::new(12.0, 12.0);
        second.via = Some(Point::new(10.0, 6.0));
        p.cameras[0].marks.push(second);

        let list = render(&project(), &set(), &p, 0.0, None);

        assert!(path_of(&list, Role::CameraPath).len() > 2);
    }

    /// Grips: one per leg of the selected way, halfway along it, none in `render`.
    #[test]
    fn only_the_selected_way_has_handles() {
        let mut p = plan();
        p.figures[0].marks[1].via = Some(Point::new(4.5, 0.0));

        assert!(render(&project(), &set(), &p, 0.0, None)
            .items
            .iter()
            .all(|i| !matches!(i.source, Some(Source::FigureVia { .. }))));

        let via = |index| Source::FigureVia {
            character: "e".into(),
            index,
        };
        let grips = handles(&project(), &p, &Source::Figure("e".into()));
        assert_eq!(grips.len(), 1);
        assert_eq!(grips[0].source, Some(via(1)));
        assert_eq!(grips[0].role, Role::FigureHandle(FigureColor::Purple));
        assert!(matches!(
            grips[0].shape,
            Shape::Circle { center, .. } if center == Point::new(4.5, 3.0)
        ));

        // The grip of a selected grip, and of a mark, belong to the same way.
        assert_eq!(handles(&project(), &p, &via(1)), grips);
        let mark = Source::Mark {
            character: "e".into(),
            index: 0,
        };
        assert_eq!(handles(&project(), &p, &mark), grips);

        // Another way's, a one-mark way's, and a wall's: none.
        assert!(handles(&project(), &p, &Source::Figure("v".into())).is_empty());
        assert!(handles(&project(), &p, &Source::Wall(0)).is_empty());
    }

    #[test]
    fn a_camera_has_handles_when_selected() {
        let mut p = plan();
        let mut second = p.cameras[0].marks[0].clone();
        second.at = Point::new(12.0, 12.0);
        p.cameras[0].marks.push(second);

        let grips = handles(&project(), &p, &Source::Camera("a".into()));

        assert_eq!(grips.len(), 1);
        assert_eq!(grips[0].role, Role::CameraHandle);
        assert_eq!(
            grips[0].source,
            Some(Source::CameraVia {
                camera: "a".into(),
                index: 1
            })
        );
    }
}
