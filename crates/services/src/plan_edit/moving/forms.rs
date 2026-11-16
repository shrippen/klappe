//! The points of a free form (LH-92): moving one, taking one out.

use klappe_model::{is_outline, outline_box, ObjectShape, Point, SetObject};
use klappe_stellprobe::{object_grip_place, ObjectGrip, Source};

use super::super::{drawings, Scope};
use crate::error::ServiceError;
use crate::field::Field;

/// Where the points of the free form `object` lie on the plan; empty for any other shape.
fn form_places(object: &SetObject) -> Vec<Point> {
    let ObjectShape::Polygon { points } = &object.shape else {
        return Vec::new();
    };

    (0..points.len())
        .map(|n| object_grip_place(object, ObjectGrip::Point(n)))
        .collect()
}

/// Gives the free form `object` the outline through `places` (plan positions, turn included):
/// the points relative to the box's centre, which becomes `at`, and the box's size. The box
/// is measured in the object's own frame, so a turned form stays turned and every point
/// keeps its place on the plan. `false` and nothing changed if `places` are no outline.
fn refit(object: &mut SetObject, places: &[Point]) -> bool {
    let (old_at, turn_deg) = (object.at, object.rotation_deg);
    let level: Vec<Point> = places.iter().map(|p| p.turned(old_at, -turn_deg)).collect();
    if !is_outline(&level) {
        return false;
    }

    let (middle, width_m, depth_m) = outline_box(&level);
    let points = level
        .iter()
        .map(|p| Point::new(p.x_m - middle.x_m, p.y_m - middle.y_m))
        .collect();

    object.shape = ObjectShape::Polygon { points };
    object.width_m = width_m;
    object.depth_m = depth_m;
    object.at = middle.turned(old_at, turn_deg);

    true
}

/// `refit`, or the refusal of a drag that leaves no outline.
fn refit_or_refuse(object: &mut SetObject, places: &[Point]) -> Result<(), ServiceError> {
    if refit(object, places) {
        return Ok(());
    }

    Err(ServiceError::BadValue {
        field: Field::Points,
        value: places.len().to_string(),
    })
}

/// Moves point `index` of a free form to `to` on the plan; the others stay where they are on
/// the plan. A move that leaves no outline is refused.
pub(in super::super) fn drag_form_point(
    object: &mut SetObject,
    index: usize,
    to: Point,
) -> Result<(), ServiceError> {
    let mut places = form_places(object);
    *places.get_mut(index).ok_or(ServiceError::Gone)? = to;

    refit_or_refuse(object, &places)
}

/// Makes a point on side `index` of a free form, after point `index`, where `to` lies on the
/// plan; the others stay where they are on the plan. A point that leaves no outline is refused.
pub(in super::super) fn insert_form_point(
    object: &mut SetObject,
    index: usize,
    to: Point,
) -> Result<(), ServiceError> {
    let mut places = form_places(object);
    if index >= places.len() {
        return Err(ServiceError::Gone);
    }

    places.insert(index + 1, to);

    refit_or_refuse(object, &places)
}

/// Takes point `index` of a free form out; the others stay where they are on the plan. No
/// outline left (fewer than three points) is refused, the reason reaches the status line.
fn remove_form_point(scope: &mut Scope<'_>, source: &Source) -> Result<(), ServiceError> {
    let Source::ObjectGrip {
        object,
        grip: ObjectGrip::Point(index),
    } = source
    else {
        return Err(ServiceError::Gone);
    };
    let object = scope
        .set
        .as_deref_mut()
        .and_then(|s| s.objects.iter_mut().find(|o| o.id == *object))
        .ok_or(ServiceError::Gone)?;
    let mut places = form_places(object);
    if *index >= places.len() {
        return Err(ServiceError::Gone);
    }

    places.remove(*index);
    if refit(object, &places) {
        return Ok(());
    }

    Err(ServiceError::FewPoints)
}

/// Takes out the point `source` names if it is a form's, else what `drawings::remove_grip`
/// does.
pub(in super::super) fn remove_grip(
    scope: &mut Scope<'_>,
    source: &Source,
) -> Result<(), ServiceError> {
    match source {
        Source::ObjectGrip {
            grip: ObjectGrip::Point(_),
            ..
        } => remove_form_point(scope, source),
        _ => drawings::remove_grip(scope, source),
    }
}
