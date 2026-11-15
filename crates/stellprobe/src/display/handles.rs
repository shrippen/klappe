//! The grips of the selected element: on the bends of a way, on the points of a drawing.

use klappe_model::{Drawing, DrawingShape, FloorPlan, Point, Project};

use super::{rect_corners, Bend, Item, Role, Shape, Source};
use crate::motion::{bend_grips, walked_ways};

/// The grip on a way's bend.
const HANDLE_RADIUS_M: f64 = 0.15;
/// The grip that pulls a new bend out of a way.
const NEW_BEND_RADIUS_M: f64 = 0.1;

/// The grips on the way `selected` belongs to: a figure, camera or mark of it, or one of its
/// grips (LH-21). For each mark after the first, one on each bend of the way to it and a
/// smaller one halfway along each piece between bends, to pull out a new bend.
/// A selected drawing has one on each of its points (LH-19).
/// Only the selected way shows them, so the plan is not full of dots; `render` leaves them
/// out, and with it the export.
pub fn handles(project: &Project, plan: &FloorPlan, selected: &Source) -> Vec<Item> {
    match selected {
        Source::Figure(c)
        | Source::Mark { character: c, .. }
        | Source::FigureVia { character: c, .. } => figure_handles(project, plan, c),
        Source::Camera(id) | Source::CameraVia { camera: id, .. } => camera_handles(plan, id),
        Source::Drawing(id) | Source::DrawingPoint { drawing: id, .. } => drawing_handles(plan, id),
        _ => Vec::new(),
    }
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

fn drawing_handles(plan: &FloorPlan, id: &str) -> Vec<Item> {
    let Some(drawing) = plan.drawings.iter().find(|d| d.id == id) else {
        return Vec::new();
    };

    drawing_handle_places(drawing)
        .into_iter()
        .enumerate()
        .map(|(index, center)| Item {
            role: Role::DrawingHandle,
            shape: handle_shape(center),
            label: None,
            source: Some(Source::DrawingPoint {
                drawing: id.to_owned(),
                index,
            }),
        })
        .collect()
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

fn handle_shape(center: Point) -> Shape {
    Shape::Circle {
        center,
        radius_m: HANDLE_RADIUS_M,
    }
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
