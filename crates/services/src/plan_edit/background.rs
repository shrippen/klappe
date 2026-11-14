//! The plan's background image: set, calibrate to scale, move, remove.

use klappe_model::{Background, Point};
use std::path::Path;

use super::Scope;
use crate::error::ServiceError;
use crate::field::Field;

/// A new background lets the plan underneath show through.
const BACKGROUND_OPACITY: f64 = 0.5;

pub(super) fn set_background(
    scope: &mut Scope<'_>,
    dir: &Path,
    source: &Path,
) -> Result<(), ServiceError> {
    let set = scope
        .set
        .as_deref_mut()
        .ok_or_else(|| ServiceError::NoLocation(scope.location.to_owned()))?;
    if !(set.width_m.is_finite() && set.width_m > 0.0) {
        return Err(ServiceError::BadValue {
            field: Field::SetWidth,
            value: set.width_m.to_string(),
        });
    }

    set.background = Some(Background {
        file: klappe_store::add_image(dir, source)?,
        at: Point::default(),
        width_m: set.width_m,
        rotation_deg: 0.0,
        opacity: BACKGROUND_OPACITY,
    });

    Ok(())
}

/// The set's background, or `Gone` when it has none (or no set).
fn background<'a>(scope: &'a mut Scope<'_>) -> Result<&'a mut Background, ServiceError> {
    scope
        .set
        .as_deref_mut()
        .and_then(|s| s.background.as_mut())
        .ok_or(ServiceError::Gone)
}

/// Scaling the image by k about `from` keeps `from` where it is and turns every length into
/// k times as much: width, and the distance of its corner `at` from `from`. The rotation is
/// untouched, a scaling does not turn anything.
pub(super) fn calibrate(
    scope: &mut Scope<'_>,
    (from, to): (Point, Point),
    distance_m: f64,
) -> Result<(), ServiceError> {
    let shown_m = from.distance_m(to);
    if shown_m == 0.0 {
        return Err(ServiceError::BadValue {
            field: Field::Points,
            value: "same".to_owned(),
        });
    }

    let k = distance_m / shown_m;
    let bg = background(scope)?;
    bg.at = from.lerp(bg.at, k);
    bg.width_m *= k;

    Ok(())
}

pub(super) fn move_background(scope: &mut Scope<'_>, by_m: (f64, f64)) -> Result<(), ServiceError> {
    let bg = background(scope)?;
    bg.at = Point::new(bg.at.x_m + by_m.0, bg.at.y_m + by_m.1);

    Ok(())
}

pub(super) fn remove_background(scope: &mut Scope<'_>) -> Result<(), ServiceError> {
    background(scope)?;
    if let Some(set) = scope.set.as_deref_mut() {
        set.background = None;
    }

    Ok(())
}
