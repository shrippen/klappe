//! A photo or sketch placed on the plan (LH-18): the file is copied in, the rest is moving,
//! turning and sizing as for any element.

use klappe_model::{PlanImage, Point, IMAGE_WIDTH_M};
use std::path::Path;

use super::{free_id, Scope};
use crate::error::ServiceError;

pub(super) fn add_image(
    scope: &mut Scope<'_>,
    dir: &Path,
    source: &Path,
    at: Point,
) -> Result<(), ServiceError> {
    let set = scope
        .set
        .as_deref_mut()
        .ok_or_else(|| ServiceError::NoLocation(scope.location.to_owned()))?;
    let file = klappe_store::add_image(dir, source)?;
    let id = free_id("i", set.images.iter().map(|i| i.id.as_str()));

    set.images.push(PlanImage {
        id,
        file,
        at,
        width_m: IMAGE_WIDTH_M,
        rotation_deg: 0.0,
    });

    Ok(())
}

pub(super) fn remove_image(scope: &mut Scope<'_>, id: &str) -> Result<(), ServiceError> {
    let set = scope.set.as_deref_mut().ok_or(ServiceError::Gone)?;
    let index = set.images.iter().position(|i| i.id == id);

    set.images.remove(index.ok_or(ServiceError::Gone)?);

    Ok(())
}

/// A copy of image `id` under a new id, `by_m` (dx, dy) away; it shares the file.
pub(super) fn copy_image(
    scope: &mut Scope<'_>,
    id: &str,
    by_m: (f64, f64),
) -> Result<(), ServiceError> {
    let set = scope.set.as_deref_mut().ok_or(ServiceError::Gone)?;
    let mut image = set
        .images
        .iter()
        .find(|i| i.id == id)
        .cloned()
        .ok_or(ServiceError::Gone)?;

    image.id = free_id("i", set.images.iter().map(|i| i.id.as_str()));
    image.at = Point::new(image.at.x_m + by_m.0, image.at.y_m + by_m.1);
    set.images.push(image);

    Ok(())
}
