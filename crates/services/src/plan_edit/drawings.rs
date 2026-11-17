//! Changes to a drawing that is on the plan already (LH-19): its style and its points.

use klappe_model::{Dash, Drawing, DrawingShape, FigureColor, Fill, Point};
use klappe_stellprobe::{drawing_edge_places, drawing_handle_places, Source};

use super::add::check_drawable;
use super::{PlanEdit, Scope};
use crate::error::ServiceError;
use crate::field::Field;

/// Applies `SetDrawingStyle`; any other edit is none of its business.
pub(super) fn set_drawing_style(
    scope: &mut Scope<'_>,
    edit: &PlanEdit,
) -> Result<(), ServiceError> {
    match edit {
        PlanEdit::SetDrawingStyle {
            source,
            color,
            dash,
            fill,
        } => restyle(scope, source, (*color, *dash, *fill)),
        _ => Ok(()),
    }
}

/// Gives the drawing `source` names a new look; any other source is refused.
fn restyle(
    scope: &mut Scope<'_>,
    source: &Source,
    (color, dash, fill): (FigureColor, Dash, Fill),
) -> Result<(), ServiceError> {
    let Source::Drawing(id) = source else {
        return Err(ServiceError::BadValue {
            field: Field::Source,
            value: String::new(),
        });
    };
    let drawing = scope
        .plan
        .drawings
        .iter_mut()
        .find(|d| d.id == *id)
        .ok_or(ServiceError::Gone)?;

    drawing.color = color;
    drawing.dash = dash;
    drawing.fill = fill;

    Ok(())
}

/// Moves the grip `index` of `drawing` to `to`, where it lies on the plan; the other grips
/// stay where they are, turn included. A line, arrow, rectangle or ellipse has two fixed ends
/// (a rectangle's opposite corner), so its middle, the centre of the turn, is the middle of
/// those two. An area's middle shifts with its box: the points move by what keeps the others
/// in place. A drag that leaves nothing to draw is refused.
pub(super) fn drag_handle(
    drawing: &mut Drawing,
    index: usize,
    to: Point,
) -> Result<(), ServiceError> {
    let places = drawing_handle_places(drawing);
    let turn_deg = drawing.rotation_deg;
    let middle = drawing.middle().ok_or(ServiceError::Gone)?;
    if index >= places.len() {
        return Err(ServiceError::Gone);
    }

    let mut shape = drawing.shape.clone();
    match &mut shape {
        DrawingShape::Line { from, to: end } | DrawingShape::Arrow { from, to: end } => {
            let other = places[1 - index];
            let (a, b) = if index == 0 { (to, other) } else { (other, to) };
            (*from, *end) = ends(a, b, turn_deg);
        }
        DrawingShape::Rect { from, to: end } | DrawingShape::Ellipse { from, to: end } => {
            let opposite = places[(index + 2) % places.len()];
            (*from, *end) = ends(to, opposite, turn_deg);
        }
        DrawingShape::Area { points } => {
            points[index] = to.turned(middle, -turn_deg);
            keep_others(points, middle, turn_deg);
        }
    }
    check_drawable(&shape)?;

    drawing.shape = shape;

    Ok(())
}

/// Where the grip on edge `index` of an area sits on the plan.
pub(super) fn edge_place(drawing: &Drawing, index: usize) -> Option<Point> {
    drawing_edge_places(drawing).get(index).copied()
}

/// Makes a point on edge `index` of an area, after point `index`, where `to` lies on the plan;
/// the other points stay where they are, turn included.
pub(super) fn insert_point(
    drawing: &mut Drawing,
    index: usize,
    to: Point,
) -> Result<(), ServiceError> {
    let turn_deg = drawing.rotation_deg;
    let middle = drawing.middle().ok_or(ServiceError::Gone)?;
    let DrawingShape::Area { points } = &mut drawing.shape else {
        return Err(ServiceError::Gone);
    };
    if index >= points.len() {
        return Err(ServiceError::Gone);
    }

    points.insert(index + 1, to.turned(middle, -turn_deg));
    keep_others(points, middle, turn_deg);

    Ok(())
}

/// Takes out the point `source` names, if it is one of an area; an edge grip is no element.
pub(super) fn remove_grip(scope: &mut Scope<'_>, source: &Source) -> Result<(), ServiceError> {
    let Source::DrawingPoint { drawing, index } = source else {
        return Err(ServiceError::BadValue {
            field: Field::Source,
            value: String::new(),
        });
    };
    let drawing = scope.plan.drawings.iter_mut().find(|d| d.id == *drawing);

    remove_point(drawing.ok_or(ServiceError::Gone)?, *index)
}

/// Takes point `index` out of an area, which keeps at least three; the other points stay
/// where they are, turn included. The reason of a refusal reaches the status line.
fn remove_point(drawing: &mut Drawing, index: usize) -> Result<(), ServiceError> {
    let turn_deg = drawing.rotation_deg;
    let middle = drawing.middle().ok_or(ServiceError::Gone)?;
    let mut shape = drawing.shape.clone();
    let DrawingShape::Area { points } = &mut shape else {
        return Err(ServiceError::BadValue {
            field: Field::Source,
            value: String::new(),
        });
    };
    if index >= points.len() {
        return Err(ServiceError::Gone);
    }

    points.remove(index);
    keep_others(points, middle, turn_deg);
    check_drawable(&shape).map_err(|_| ServiceError::FewPoints)?;

    drawing.shape = shape;

    Ok(())
}

/// The two points as they lie before a turn of `turn_deg` about their middle.
fn ends(a: Point, b: Point, turn_deg: f64) -> (Point, Point) {
    let middle = a.lerp(b, 0.5);

    (a.turned(middle, -turn_deg), b.turned(middle, -turn_deg))
}

/// `points` had one point set; the box, and with it the centre of the turn, moved from
/// `old_middle`. Shifts all of them so the others lie on the plan where they did.
fn keep_others(points: &mut [Point], old_middle: Point, turn_deg: f64) {
    let Some(first) = points.first() else {
        return;
    };
    let (mut min, mut max) = (*first, *first);
    for p in points.iter() {
        min = Point::new(min.x_m.min(p.x_m), min.y_m.min(p.y_m));
        max = Point::new(max.x_m.max(p.x_m), max.y_m.max(p.y_m));
    }
    let middle = min.lerp(max, 0.5);

    let drift = middle.turned(old_middle, turn_deg);
    let (dx_m, dy_m) = (middle.x_m - drift.x_m, middle.y_m - drift.y_m);
    for p in points.iter_mut() {
        *p = Point::new(p.x_m - dx_m, p.y_m - dy_m);
    }
}
