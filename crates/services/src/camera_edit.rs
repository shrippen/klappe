//! Edits of a camera's sensor, lens and frame (LH-32, LH-38).

use klappe_model::{
    is_aspect, is_camera_lens_mm, is_sensor_width_mm, Camera, FigureColor, FloorPlan, Sensor,
};

use crate::error::ServiceError;
use crate::field::Field;
use crate::plan_edit::PlanEdit;

/// Refuses a sensor, lens or frame edit whose value no camera can have.
pub(crate) fn check(edit: &PlanEdit) -> Result<(), ServiceError> {
    match edit {
        PlanEdit::SetSensor {
            width_mm, aspect, ..
        } => check_sensor(*width_mm, *aspect),
        PlanEdit::SetLens { lens_mm, .. } => check_lens(*lens_mm),
        PlanEdit::SetAspect { aspect, .. } => check_aspect(*aspect),
        _ => Ok(()),
    }
}

/// Applies an edit of a camera's sensor, lens, frame or colour; any other edit changes nothing.
pub(crate) fn apply(plan: &mut FloorPlan, edit: &PlanEdit) -> Result<(), ServiceError> {
    match edit {
        PlanEdit::SetSensor {
            camera,
            width_mm,
            aspect,
        } => set_sensor(plan, camera, *width_mm, *aspect),
        PlanEdit::SetLens { camera, lens_mm } => set_lens(plan, camera, *lens_mm),
        PlanEdit::SetAspect { camera, aspect } => set_aspect(plan, camera, *aspect),
        PlanEdit::SetCameraColor { camera, color } => set_color(plan, camera, *color),
        _ => Ok(()),
    }
}

fn check_sensor(width_mm: f64, aspect: Option<f64>) -> Result<(), ServiceError> {
    if !is_sensor_width_mm(width_mm) {
        return Err(ServiceError::BadValue {
            field: Field::Sensor,
            value: width_mm.to_string(),
        });
    }

    match aspect {
        Some(aspect) => check_aspect(aspect),
        None => Ok(()),
    }
}

pub(crate) fn check_lens(lens_mm: f64) -> Result<(), ServiceError> {
    if is_camera_lens_mm(lens_mm) {
        return Ok(());
    }

    Err(ServiceError::BadValue {
        field: Field::Lens,
        value: lens_mm.to_string(),
    })
}

pub(crate) fn check_aspect(aspect: f64) -> Result<(), ServiceError> {
    if is_aspect(aspect) {
        return Ok(());
    }

    Err(ServiceError::BadValue {
        field: Field::Aspect,
        value: aspect.to_string(),
    })
}

fn camera<'a>(plan: &'a mut FloorPlan, id: &str) -> Result<&'a mut Camera, ServiceError> {
    plan.cameras
        .iter_mut()
        .find(|c| c.id == id)
        .ok_or(ServiceError::Gone)
}

/// Gives the camera the sensor of `width_mm`, and the frame `aspect` when there is one: a
/// body brings both.
fn set_sensor(
    plan: &mut FloorPlan,
    id: &str,
    width_mm: f64,
    aspect: Option<f64>,
) -> Result<(), ServiceError> {
    let camera = camera(plan, id)?;

    camera.sensor = Sensor::of_width_mm(width_mm);
    if let Some(aspect) = aspect {
        camera.aspect = aspect;
    }

    Ok(())
}

fn set_lens(plan: &mut FloorPlan, id: &str, lens_mm: f64) -> Result<(), ServiceError> {
    camera(plan, id)?.lens_mm = lens_mm;

    Ok(())
}

fn set_aspect(plan: &mut FloorPlan, id: &str, aspect: f64) -> Result<(), ServiceError> {
    camera(plan, id)?.aspect = aspect;

    Ok(())
}

/// Gives the camera its own colour, or the one its place gives with `None` (LH-111).
fn set_color(
    plan: &mut FloorPlan,
    id: &str,
    color: Option<FigureColor>,
) -> Result<(), ServiceError> {
    camera(plan, id)?.color = color;

    Ok(())
}
