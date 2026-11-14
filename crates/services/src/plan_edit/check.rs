//! Checks of the values an edit brings before it applies: heights, tilts, holds, sizes, points.

use klappe_model::{is_object_height_m, is_object_size_m, DrawingShape, Point};

use super::PlanEdit;
use crate::camera_edit;
use crate::error::ServiceError;
use crate::field::Field;

/// Highest a lens can be: a crane or a drone, not a satellite.
const MAX_CAMERA_HEIGHT_M: f64 = 50.0;

/// A camera looks straight down at -90° and straight up at 90°.
const MAX_TILT_DEG: f64 = 90.0;

/// Every point and angle is a finite number: NaN or infinity would be saved as `null`.
pub(super) fn check(edit: &PlanEdit) -> Result<(), ServiceError> {
    match edit {
        PlanEdit::Move { to: at, .. }
        | PlanEdit::AddMark { at, .. }
        | PlanEdit::AddLight { at, .. }
        | PlanEdit::AddObject { at, .. }
        | PlanEdit::AddNote { at, .. } => check_point(*at),
        PlanEdit::Shift { dx_m, dy_m, .. }
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
        PlanEdit::AddWall { from, to } => {
            check_point(*from)?;
            check_point(*to)
        }
        PlanEdit::Calibrate {
            from,
            to,
            distance_m,
        } => {
            check_point(*from)?;
            check_point(*to)?;
            check_distance(*distance_m)
        }
        PlanEdit::MoveBackground { dx_m, dy_m } => check_point(Point::new(*dx_m, *dy_m)),
        PlanEdit::Resize {
            width_m, depth_m, ..
        } => check_object_size(*width_m, *depth_m),
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
        PlanEdit::SetObjectHeight { height_m, .. } => check_object_height(*height_m),
        PlanEdit::Remove { .. }
        | PlanEdit::RemoveGroup { .. }
        | PlanEdit::Straighten { .. }
        | PlanEdit::EditNote { .. }
        | PlanEdit::SetBackground { .. }
        | PlanEdit::RemoveBackground
        | PlanEdit::Follow { .. }
        | PlanEdit::SetPose { .. }
        | PlanEdit::SetArrival { .. }
        | PlanEdit::SetDeparture { .. }
        | PlanEdit::SetPace { .. } => Ok(()),
    }
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
    if hold_s.is_finite() && hold_s >= 0.0 {
        return Ok(());
    }

    Err(ServiceError::BadValue {
        field: Field::HoldS,
        value: hold_s.to_string(),
    })
}

/// A lens height from the floor up to a crane's, and a tilt within straight down and up.
fn check_camera_aim(height_m: f64, tilt_deg: f64) -> Result<(), ServiceError> {
    if !height_m.is_finite() || !(0.0..=MAX_CAMERA_HEIGHT_M).contains(&height_m) {
        return Err(ServiceError::BadValue {
            field: Field::Height,
            value: height_m.to_string(),
        });
    }
    if !tilt_deg.is_finite() || !(-MAX_TILT_DEG..=MAX_TILT_DEG).contains(&tilt_deg) {
        return Err(ServiceError::BadValue {
            field: Field::Tilt,
            value: tilt_deg.to_string(),
        });
    }

    Ok(())
}

/// An object's height, when given: a number, more than nothing, not absurd.
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

/// An object's sides: numbers, more than nothing, not absurd.
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
