//! Edits of a camera's sensor, lens and frame (LH-32, LH-38).

use klappe_model::{Camera, FloorPlan, Sensor};

use crate::error::ServiceError;

/// Widest sensor and longest lens that make sense; beyond, a number is a typing error.
const MAX_SENSOR_WIDTH_MM: f64 = 500.0;
const MAX_LENS_MM: f64 = 5000.0;
/// Widest frame, width to height; real ones stay below 4.
const MAX_ASPECT: f64 = 20.0;

pub(crate) fn check_sensor(width_mm: f64, aspect: Option<f64>) -> Result<(), ServiceError> {
    check_value("sensor", width_mm, MAX_SENSOR_WIDTH_MM)?;
    match aspect {
        Some(aspect) => check_aspect(aspect),
        None => Ok(()),
    }
}

pub(crate) fn check_lens(lens_mm: f64) -> Result<(), ServiceError> {
    check_value("lens", lens_mm, MAX_LENS_MM)
}

pub(crate) fn check_aspect(aspect: f64) -> Result<(), ServiceError> {
    check_value("aspect", aspect, MAX_ASPECT)
}

/// A number more than nothing and not above `max`.
fn check_value(field: &'static str, value: f64, max: f64) -> Result<(), ServiceError> {
    if value.is_finite() && value > 0.0 && value <= max {
        return Ok(());
    }

    Err(ServiceError::BadValue {
        field,
        value: value.to_string(),
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
pub(crate) fn set_sensor(
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

pub(crate) fn set_lens(plan: &mut FloorPlan, id: &str, lens_mm: f64) -> Result<(), ServiceError> {
    camera(plan, id)?.lens_mm = lens_mm;

    Ok(())
}

pub(crate) fn set_aspect(plan: &mut FloorPlan, id: &str, aspect: f64) -> Result<(), ServiceError> {
    camera(plan, id)?.aspect = aspect;

    Ok(())
}
