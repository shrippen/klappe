//! The grips of the selected element: on the bends of a way, on the points of a drawing.

use klappe_model::{
    Drawing, DrawingShape, FloorPlan, ObjectShape, Point, Project, SetObject, SetPlan,
};

use super::objects::object_corners;
use super::{rect_corners, Item, Role, Shape, Source};
use crate::card::note_card;
use crate::light::light_grip_place;
use crate::motion::{bend_grips, walked_ways};

/// A grip on a way (LH-21): a bend the way passes, or a spot to pull a new bend out of.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bend {
    /// Bend `n` of the way: drag to move it, remove it to take it out.
    At(usize),
    /// Halfway along piece `n` of the way, which has bends `n - 1` and `n` at its ends (the
    /// way's start and end for the first and last): drag to make a bend there, as number `n`.
    New(usize),
}

/// The end of an opening a grip sits at, seen from its wall's `from` towards `to`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpeningEnd {
    /// The end nearer `from`.
    Start,
    /// The end nearer `to`.
    End,
}

/// A grip on a lamp's beam (LH-50), both at the beam's reach.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightGrip {
    /// On the lamp's facing: drag to turn the lamp towards the pointer.
    Direction,
    /// On the beam's edge clockwise of the facing: drag to set the beam's angle.
    Beam,
}

/// A grip on a set object, in the object's own turned frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectGrip {
    /// Just outside the middle of the right edge: sets the width.
    Width,
    /// Just outside the middle of the bottom edge: sets the depth.
    Depth,
    /// Point `n` of a free form's outline (LH-92): drag to move it, remove it to take it out.
    Point(usize),
    /// Halfway along side `n` of a free form, from point `n` to the next (the last one closes
    /// to point 0): drag to make a point there (LH-92).
    Side(usize),
}

/// The grip on a way's bend.
const HANDLE_RADIUS_M: f64 = 0.15;
/// The grip that pulls a new bend out of a way.
const NEW_BEND_RADIUS_M: f64 = 0.1;
/// The grip that pulls a new point out of an area's edge.
const NEW_POINT_RADIUS_M: f64 = 0.1;

/// A side or edge shows a grip that makes a point only when it is longer than this: three
/// widths of that grip. A shorter one would cover the shape.
const SIDE_GRIP_MIN_M: f64 = 6.0 * NEW_POINT_RADIUS_M;

/// Whether the side from `a` to `b` is long enough for a grip that makes a point.
fn side_has_grip(a: Point, b: Point) -> bool {
    a.distance_m(b) > SIDE_GRIP_MIN_M
}

/// The grips on the way `selected` belongs to: a figure, camera or mark of it, or one of its
/// grips (LH-21). For each mark after the first, one on each bend of the way to it and a
/// smaller one halfway along each piece between bends, to pull out a new bend.
/// A selected drawing has one on each of its points, an area also a smaller one halfway along
/// each edge longer than `SIDE_GRIP_MIN_M` (LH-19).
/// A selected note has one in the middle of its card's right edge, to set its width (LH-16).
/// A selected door or window has one on each end, on the wall's middle line, to set its
/// width (LH-10).
/// A selected lamp has two at the beam's reach: one on its facing, one on the beam's edge
/// (LH-50).
/// A selected object drawn by its width and depth has one just outside the middle of its right
/// edge and one outside its bottom edge, to set them (LH-25); a free form has one on each point
/// of its outline instead and a small one halfway along each side longer than
/// `SIDE_GRIP_MIN_M` (LH-92).
/// Only the selected way shows them, so the plan is not full of dots; `render` leaves them
/// out, and with it the export.
pub fn handles(project: &Project, set: &SetPlan, plan: &FloorPlan, selected: &Source) -> Vec<Item> {
    match selected {
        Source::Figure(c)
        | Source::Mark { character: c, .. }
        | Source::FigureVia { character: c, .. } => figure_handles(project, plan, c),
        Source::Camera(id) | Source::CameraVia { camera: id, .. } => camera_handles(plan, id),
        Source::Drawing(id)
        | Source::DrawingPoint { drawing: id, .. }
        | Source::DrawingEdge { drawing: id, .. } => drawing_handles(plan, id),
        Source::Note(id) | Source::NoteWidth(id) => note_handles(set, id),
        Source::Opening { wall, index } | Source::OpeningEdge { wall, index, .. } => {
            opening_handles(set, *wall, *index)
        }
        Source::Light(id) | Source::LightGrip { light: id, .. } => light_handles(plan, id),
        Source::Object(id) | Source::ObjectGrip { object: id, .. } => object_handles(set, id),
        _ => Vec::new(),
    }
}

/// The grips on the beam of lamp `id`.
fn light_handles(plan: &FloorPlan, id: &str) -> Vec<Item> {
    let Some(light) = plan.lights.iter().find(|l| l.id == id) else {
        return Vec::new();
    };

    [LightGrip::Beam, LightGrip::Direction]
        .into_iter()
        .map(|grip| Item {
            role: Role::Handle,
            shape: Shape::Circle {
                center: light_grip_place(light, grip),
                radius_m: HANDLE_RADIUS_M,
            },
            label: None,
            source: Some(Source::LightGrip {
                light: id.to_owned(),
                grip,
            }),
        })
        .collect()
}

/// How far outside its edge an object's grip sits: farther than a grip catches a click
/// (radius and pick tolerance, 0.3 m), so a click on a small object takes the object, and so
/// far that the two grips of a 10 cm box (at least 0.45 m · √2 apart) do not cover each other.
const OBJECT_GRIP_GAP_M: f64 = 0.45;

/// Where the grip `grip` of `object` sits on the plan: just outside the middle of its right or
/// bottom edge, turned with the object; a free form's point sits on the point, where it lies
/// on the plan (the object's centre for a point the form does not have).
pub fn object_grip_place(object: &SetObject, grip: ObjectGrip) -> Point {
    let (x_m, y_m) = match grip {
        ObjectGrip::Width => (object.width_m / 2.0 + OBJECT_GRIP_GAP_M, 0.0),
        ObjectGrip::Depth => (0.0, object.depth_m / 2.0 + OBJECT_GRIP_GAP_M),
        ObjectGrip::Point(n) => return form_point_place(object, n),
        ObjectGrip::Side(n) => return form_side_place(object, n),
    };

    object.placed(x_m, y_m)
}

/// Where point `index` of a free form lies on the plan, stretched to the object's size and
/// turned; the object's centre when `object` has no such point.
fn form_point_place(object: &SetObject, index: usize) -> Point {
    if !matches!(object.shape, ObjectShape::Polygon { .. }) {
        return object.at;
    }

    object_corners(object)
        .get(index)
        .copied()
        .unwrap_or(object.at)
}

/// Where the middle of side `index` of a free form lies on the plan, from point `index` to the
/// next; the object's centre when `object` has no such side.
fn form_side_place(object: &SetObject, index: usize) -> Point {
    if !matches!(object.shape, ObjectShape::Polygon { .. }) {
        return object.at;
    }

    let corners = object_corners(object);
    let (Some(from), Some(to)) = (
        corners.get(index),
        corners.get((index + 1) % corners.len().max(1)),
    ) else {
        return object.at;
    };

    from.lerp(*to, 0.5)
}

/// The grips of object `id`: width and depth, or for a free form one on each point and one
/// halfway along each side longer than `SIDE_GRIP_MIN_M`; the sides come first, so the points
/// lie on top of them.
fn object_handles(set: &SetPlan, id: &str) -> Vec<Item> {
    let Some(object) = set.objects.iter().find(|o| o.id == id) else {
        return Vec::new();
    };
    let grips = match &object.shape {
        ObjectShape::Polygon { points } => (0..points.len())
            .filter(|n| {
                let from = form_point_place(object, *n);
                side_has_grip(from, form_point_place(object, (n + 1) % points.len()))
            })
            .map(ObjectGrip::Side)
            .chain((0..points.len()).map(ObjectGrip::Point))
            .collect(),
        _ => vec![ObjectGrip::Width, ObjectGrip::Depth],
    };

    grips
        .into_iter()
        .map(|grip| Item {
            role: Role::Handle,
            shape: Shape::Circle {
                center: object_grip_place(object, grip),
                radius_m: match grip {
                    ObjectGrip::Side(_) => NEW_POINT_RADIUS_M,
                    _ => HANDLE_RADIUS_M,
                },
            },
            label: None,
            source: Some(Source::ObjectGrip {
                object: id.to_owned(),
                grip,
            }),
        })
        .collect()
}

/// The grips on the two ends of opening `index` of wall `wall`.
fn opening_handles(set: &SetPlan, wall: usize, index: usize) -> Vec<Item> {
    let Some(opening) = set
        .walls
        .get(wall)
        .and_then(|w| w.openings.get(index).map(|o| (w, o)))
    else {
        return Vec::new();
    };
    let (wall_ref, opening) = opening;
    let (start_m, end_m) = opening.span_m();

    [(OpeningEnd::Start, start_m), (OpeningEnd::End, end_m)]
        .into_iter()
        .map(|(end, along_m)| Item {
            role: Role::Handle,
            shape: Shape::Circle {
                center: wall_ref.point_at(along_m),
                radius_m: HANDLE_RADIUS_M,
            },
            label: None,
            source: Some(Source::OpeningEdge { wall, index, end }),
        })
        .collect()
}

/// The grip on the card of note `id`: where its right edge is half as high as the card.
fn note_handles(set: &SetPlan, id: &str) -> Vec<Item> {
    let Some(note) = set.notes.iter().find(|n| n.id == id) else {
        return Vec::new();
    };
    let (corners, _) = note_card(note.at, note.width_m, &note.text);
    let center = corners[1].lerp(corners[2], 0.5);

    vec![Item {
        role: Role::Handle,
        shape: Shape::Circle {
            center,
            radius_m: HANDLE_RADIUS_M,
        },
        label: None,
        source: Some(Source::NoteWidth(id.to_owned())),
    }]
}

/// Where the grips of `drawing` sit on the plan, turned like the drawing: its points, or for
/// a rectangle and an ellipse its four corners.
pub fn drawing_handle_places(drawing: &Drawing) -> Vec<Point> {
    let points = match &drawing.shape {
        DrawingShape::Rect { from, to } | DrawingShape::Ellipse { from, to } => {
            rect_corners(*from, *to)
        }
        _ => drawing.points(),
    };

    points.into_iter().map(|p| drawing.placed(p)).collect()
}

/// Where the grips that make a point sit, turned like the drawing: halfway along each edge of
/// an area, edge `n` running from point `n` to the next. None for the other shapes.
pub fn drawing_edge_places(drawing: &Drawing) -> Vec<Point> {
    if !matches!(drawing.shape, DrawingShape::Area { .. }) {
        return Vec::new();
    }
    let places = drawing_handle_places(drawing);

    places
        .iter()
        .enumerate()
        .map(|(n, a)| a.lerp(places[(n + 1) % places.len()], 0.5))
        .collect()
}

fn drawing_handles(plan: &FloorPlan, id: &str) -> Vec<Item> {
    let Some(drawing) = plan.drawings.iter().find(|d| d.id == id) else {
        return Vec::new();
    };
    let grip = |source, center, radius_m| Item {
        role: Role::Handle,
        shape: Shape::Circle { center, radius_m },
        label: None,
        source: Some(source),
    };

    // The edges come first, so the points lie on top of them.
    let corners = drawing_handle_places(drawing);
    let edges = drawing_edge_places(drawing)
        .into_iter()
        .enumerate()
        .filter(|(n, _)| side_has_grip(corners[*n], corners[(n + 1) % corners.len()]));
    let points = corners.iter().copied().enumerate();
    let mut grips: Vec<_> = edges
        .map(|(index, at)| {
            let source = Source::DrawingEdge {
                drawing: id.to_owned(),
                index,
            };

            grip(source, at, NEW_POINT_RADIUS_M)
        })
        .collect();
    grips.extend(points.map(|(index, at)| {
        let source = Source::DrawingPoint {
            drawing: id.to_owned(),
            index,
        };

        grip(source, at, HANDLE_RADIUS_M)
    }));

    grips
}

fn figure_handles(project: &Project, plan: &FloorPlan, character: &str) -> Vec<Item> {
    let (Some(track), Some(who)) = (plan.figure(character), project.character(character)) else {
        return Vec::new();
    };
    let stops: Vec<_> = track
        .marks
        .iter()
        .map(|m| (m.at, m.via.as_slice()))
        .collect();

    // A way the figure does not walk has nothing to bend (LH-26).
    let walked = walked_ways(track);
    handle_places(&stops)
        .into_iter()
        .filter(|(index, _, _)| walked.get(*index).copied().unwrap_or(false))
        .map(|(index, bend, center)| Item {
            role: Role::FigureHandle(who.color),
            shape: bend_shape(bend, center),
            label: None,
            source: Some(Source::FigureVia {
                character: character.to_owned(),
                index,
                bend,
            }),
        })
        .collect()
}

fn camera_handles(plan: &FloorPlan, id: &str) -> Vec<Item> {
    let Some(camera) = plan.camera(id) else {
        return Vec::new();
    };
    let stops: Vec<_> = camera
        .marks
        .iter()
        .map(|m| (m.at, m.via.as_slice()))
        .collect();

    handle_places(&stops)
        .into_iter()
        .map(|(index, bend, center)| Item {
            role: Role::CameraHandle,
            shape: bend_shape(bend, center),
            label: None,
            source: Some(Source::CameraVia {
                camera: id.to_owned(),
                index,
                bend,
            }),
        })
        .collect()
}

/// A new bend's grip is smaller than a bend's, which it would hide.
fn bend_shape(bend: Bend, center: Point) -> Shape {
    let radius_m = match bend {
        Bend::At(_) => HANDLE_RADIUS_M,
        Bend::New(_) => NEW_BEND_RADIUS_M,
    };

    Shape::Circle { center, radius_m }
}

/// Mark index, kind and place of each grip of the ways: the bends, and halfway along each
/// piece between them (a straight leg: its middle). The bends come last, so they are on top.
fn handle_places(stops: &[(Point, &[Point])]) -> Vec<(usize, Bend, Point)> {
    let mut grips = Vec::new();

    for (i, pair) in stops.windows(2).enumerate() {
        let (a, (b, via)) = (pair[0].0, pair[1]);

        grips.extend(
            bend_grips(a, via, b)
                .into_iter()
                .enumerate()
                .map(|(n, at)| (i + 1, Bend::New(n), at)),
        );
        grips.extend(
            via.iter()
                .enumerate()
                .map(|(n, at)| (i + 1, Bend::At(n), *at)),
        );
    }

    grips
}
