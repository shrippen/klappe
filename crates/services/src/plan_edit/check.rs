//! Checks of the values an edit brings before it applies: heights, tilts, holds, sizes, points.

use klappe_model::{
    is_camera_height_m, is_floor_offset_m, is_hold_s, is_object_height_m, is_object_size_m,
    is_opening_width_m, is_tilt_deg, DrawingShape, Point,
};

use super::PlanEdit;
use crate::camera_edit;
use crate::error::ServiceError;
use crate::field::Field;

/// Every point and angle is a finite number: NaN or infinity would be saved as `null`.
pub(super) fn check(edit: &PlanEdit) -> Result<(), ServiceError> {
    match edit {
        PlanEdit::Move { to: at, .. }
        | PlanEdit::AddMark { at, .. }
        | PlanEdit::AddMarkBefore { at, .. }
        | PlanEdit::AddCameraMark { at, .. }
        | PlanEdit::AddObjectMark { at, .. }
        | PlanEdit::AddLight { at, .. }
        | PlanEdit::AddObject { at, .. }
        | PlanEdit::AddOpening { at, .. }
        | PlanEdit::AddNote { at, .. }
        | PlanEdit::AddImage { at, .. } => check_point(*at),
        PlanEdit::Shift { dx_m, dy_m, .. }
        | PlanEdit::ShiftWay { dx_m, dy_m, .. }
        | PlanEdit::ShiftGroup { dx_m, dy_m, .. }
        | PlanEdit::Duplicate { dx_m, dy_m, .. } => check_point(Point::new(*dx_m, *dy_m)),
        PlanEdit::Spin {
            by_deg: facing_deg, ..
        }
        | PlanEdit::SpinGroup {
            by_deg: facing_deg, ..
        }
        | PlanEdit::Turn { facing_deg, .. } => check_angle(*facing_deg),
        PlanEdit::AddCamera { at, facing_deg } => {
            check_point(*at)?;
            check_angle(*facing_deg)
        }
        PlanEdit::SetOpeningWidth { width_m, .. } => check_opening_width(*width_m),
        PlanEdit::AddWall { from, to } | PlanEdit::AddRoom { from, to } => {
            check_point(*from)?;
            check_point(*to)
        }
        PlanEdit::Calibrate {
            from,
            to,
            distance_m,
        } => check_calibration((*from, *to), *distance_m),
        PlanEdit::MoveBackground { dx_m, dy_m } => check_point(Point::new(*dx_m, *dy_m)),
        PlanEdit::Resize { .. } | PlanEdit::SetObjectHeight { .. } => check_object(edit),
        PlanEdit::AddDrawing { shape, .. } => check_shape(shape),
        PlanEdit::SetSensor {
            width_mm, aspect, ..
        } => camera_edit::check_sensor(*width_mm, *aspect),
        PlanEdit::SetLens { lens_mm, .. } => camera_edit::check_lens(*lens_mm),
        PlanEdit::SetAspect { aspect, .. } => camera_edit::check_aspect(*aspect),
        PlanEdit::SetHold { hold_s, .. } => check_hold(*hold_s),
        PlanEdit::SetCameraAim {
            height_m, tilt_deg, ..
        } => check_camera_aim(*height_m, *tilt_deg),
        PlanEdit::SetMarkLens { lens_mm, .. } => lens_mm.map_or(Ok(()), camera_edit::check_lens),
        PlanEdit::SetWallHeight { .. }
        | PlanEdit::SetOpeningHead { .. }
        | PlanEdit::SetOpeningSill { .. }
        | PlanEdit::SetObjectBase { .. } => check_heights(edit),
        // The kind is checked against the library when it applies.
        PlanEdit::SetObject { .. } => Ok(()),
        PlanEdit::Remove { .. }
        | PlanEdit::RemoveGroup { .. }
        | PlanEdit::Straighten { .. }
        | PlanEdit::SetOpeningSwing { .. }
        | PlanEdit::EditNote { .. }
        | PlanEdit::SetBackground { .. }
        | PlanEdit::RemoveBackground
        | PlanEdit::Follow { .. }
        | PlanEdit::SetPose { .. }
        | PlanEdit::SetArrival { .. }
        | PlanEdit::SetDeparture { .. }
        | PlanEdit::SetCarrier { .. }
        | PlanEdit::SetRig { .. }
        | PlanEdit::SetPace { .. }
        | PlanEdit::SetCurve { .. }
        | PlanEdit::SetDrawingStyle { .. }
        | PlanEdit::SetImageOpacity { .. }
        | PlanEdit::ReplaceImage { .. }
        // `set_light` checks its change itself.
        | PlanEdit::SetLight { .. }
        // `add_shape` checks its outline itself.
        | PlanEdit::AddShape { .. } => Ok(()),
    }
}

fn check_calibration((from, to): (Point, Point), distance_m: f64) -> Result<(), ServiceError> {
    check_point(from)?;
    check_point(to)?;
    check_distance(distance_m)
}

fn check_shape(shape: &DrawingShape) -> Result<(), ServiceError> {
    match shape {
        DrawingShape::Line { from, to }
        | DrawingShape::Arrow { from, to }
        | DrawingShape::Rect { from, to }
        | DrawingShape::Ellipse { from, to } => {
            check_point(*from)?;
            check_point(*to)
        }
        DrawingShape::Area { points } => points.iter().try_for_each(|p| check_point(*p)),
    }
}

/// A pause is a finite time, not negative.
fn check_hold(hold_s: f64) -> Result<(), ServiceError> {
    if is_hold_s(hold_s) {
        return Ok(());
    }

    Err(ServiceError::BadValue {
        field: Field::HoldS,
        value: hold_s.to_string(),
    })
}

/// A lens height from the floor up to a crane's, and a tilt within straight down and up.
fn check_camera_aim(height_m: f64, tilt_deg: f64) -> Result<(), ServiceError> {
    if !is_camera_height_m(height_m) {
        return Err(ServiceError::BadValue {
            field: Field::Height,
            value: height_m.to_string(),
        });
    }
    if !is_tilt_deg(tilt_deg) {
        return Err(ServiceError::BadValue {
            field: Field::Tilt,
            value: tilt_deg.to_string(),
        });
    }

    Ok(())
}

/// The size and height an edit gives an object.
fn check_object(edit: &PlanEdit) -> Result<(), ServiceError> {
    match edit {
        PlanEdit::Resize {
            width_m, depth_m, ..
        } => check_object_size(*width_m, *depth_m),
        PlanEdit::SetObjectHeight { height_m, .. } => check_object_height(*height_m),
        _ => Ok(()),
    }
}

/// An object's height, when given: a number, not under the least height, not absurd.
fn check_object_height(height_m: Option<f64>) -> Result<(), ServiceError> {
    let Some(height_m) = height_m else {
        return Ok(());
    };
    if is_object_height_m(height_m) {
        return Ok(());
    }

    Err(ServiceError::BadValue {
        field: Field::Height,
        value: height_m.to_string(),
    })
}

/// The heights of LH-90; any other edit is none of its business.
fn check_heights(edit: &PlanEdit) -> Result<(), ServiceError> {
    match edit {
        PlanEdit::SetWallHeight { height_m, .. } => check_top(Field::Height, *height_m),
        PlanEdit::SetOpeningHead { head_m, .. } => check_top(Field::Head, *head_m),
        PlanEdit::SetOpeningSill { sill_m, .. } => check_offset(Field::Sill, *sill_m),
        PlanEdit::SetObjectBase { base_m, .. } => check_offset(Field::Base, *base_m),
        _ => Ok(()),
    }
}

/// A height above the floor that something ends at, when given: a number, not under the least
/// height, not absurd.
fn check_top(field: Field, height_m: Option<f64>) -> Result<(), ServiceError> {
    match height_m {
        Some(height_m) if !is_object_height_m(height_m) => Err(bad_height(field, height_m)),
        _ => Ok(()),
    }
}

/// A height above the floor that something begins at, when given: a number from the floor
/// up, not absurd.
fn check_offset(field: Field, height_m: Option<f64>) -> Result<(), ServiceError> {
    match height_m {
        Some(height_m) if !is_floor_offset_m(height_m) => Err(bad_height(field, height_m)),
        _ => Ok(()),
    }
}

fn bad_height(field: Field, height_m: f64) -> ServiceError {
    ServiceError::BadValue {
        field,
        value: height_m.to_string(),
    }
}

/// A length in truth: more than nothing, and a number.
fn check_distance(distance_m: f64) -> Result<(), ServiceError> {
    if distance_m.is_finite() && distance_m > 0.0 {
        return Ok(());
    }

    Err(ServiceError::BadValue {
        field: Field::Distance,
        value: distance_m.to_string(),
    })
}

/// An opening's width: a number, not under the least, not absurd.
fn check_opening_width(width_m: f64) -> Result<(), ServiceError> {
    if is_opening_width_m(width_m) {
        return Ok(());
    }

    Err(ServiceError::BadValue {
        field: Field::Width,
        value: width_m.to_string(),
    })
}

/// An object's sides: numbers, not under the least, not absurd.
fn check_object_size(width_m: f64, depth_m: f64) -> Result<(), ServiceError> {
    if is_object_size_m(width_m) && is_object_size_m(depth_m) {
        return Ok(());
    }

    Err(ServiceError::BadValue {
        field: Field::Size,
        value: format!("{width_m} x {depth_m}"),
    })
}

fn check_point(p: Point) -> Result<(), ServiceError> {
    if p.x_m.is_finite() && p.y_m.is_finite() {
        return Ok(());
    }

    Err(ServiceError::BadValue {
        field: Field::Point,
        value: format!("{}, {}", p.x_m, p.y_m),
    })
}

fn check_angle(deg: f64) -> Result<(), ServiceError> {
    if deg.is_finite() {
        return Ok(());
    }

    Err(ServiceError::BadValue {
        field: Field::Angle,
        value: deg.to_string(),
    })
}
