//! The grips of the selected element: on the bends of a way, on the points of a drawing.

use klappe_model::{Drawing, DrawingShape, FloorPlan, Point, Project, SetPlan};

use super::{rect_corners, Item, Role, Shape, Source};
use crate::card::note_card;
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

/// The grip on a way's bend.
const HANDLE_RADIUS_M: f64 = 0.15;
/// The grip that pulls a new bend out of a way.
const NEW_BEND_RADIUS_M: f64 = 0.1;
/// The grip that pulls a new point out of an area's edge.
const NEW_POINT_RADIUS_M: f64 = 0.1;

/// The grips on the way `selected` belongs to: a figure, camera or mark of it, or one of its
/// grips (LH-21). For each mark after the first, one on each bend of the way to it and a
/// smaller one halfway along each piece between bends, to pull out a new bend.
/// A selected drawing has one on each of its points, an area also a smaller one halfway along
/// each edge (LH-19).
/// A selected note has one in the middle of its card's right edge, to set its width (LH-16).
/// A selected door or window has one on each end, on the wall's middle line, to set its
/// width (LH-10).
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
        _ => Vec::new(),
    }
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
            role: Role::OpeningHandle,
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
        role: Role::NoteHandle,
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
        role: Role::DrawingHandle,
        shape: Shape::Circle { center, radius_m },
        label: None,
        source: Some(source),
    };

    // The edges come first, so the points lie on top of them.
    let edges = drawing_edge_places(drawing).into_iter().enumerate();
    let points = drawing_handle_places(drawing).into_iter().enumerate();
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
