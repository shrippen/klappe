//! A photo or sketch placed on the plan (LH-18): the file is copied in, the rest is moving,
//! turning and sizing as for any element.

use klappe_model::{is_image_opacity, PlanImage, Point, IMAGE_WIDTH_M};
use std::path::Path;

use klappe_stellprobe::Source;

use super::{free_id, PlanEdit, Scope};
use crate::error::ServiceError;
use crate::field::Field;

/// Applies an image edit of `edit`: adding one, its opacity, its file.
pub(super) fn edit_image(
    scope: &mut Scope<'_>,
    dir: &Path,
    edit: &PlanEdit,
) -> Result<(), ServiceError> {
    match edit {
        PlanEdit::AddImage { source, at } => add_image(scope, dir, source, *at),
        PlanEdit::SetImageOpacity { source, opacity } => set_image_opacity(scope, source, *opacity),
        PlanEdit::ReplaceImage { source, with } => replace_image(scope, dir, source, with),
        _ => Ok(()),
    }
}

fn add_image(
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
        opacity: 1.0,
    });

    Ok(())
}

/// The image `source` names; any other source is refused.
fn image_of<'a>(
    scope: &'a mut Scope<'_>,
    source: &Source,
) -> Result<&'a mut PlanImage, ServiceError> {
    let Source::Image(id) = source else {
        return Err(ServiceError::BadValue {
            field: Field::Source,
            value: String::new(),
        });
    };
    let set = scope.set.as_deref_mut().ok_or(ServiceError::Gone)?;

    set.images
        .iter_mut()
        .find(|i| i.id == *id)
        .ok_or(ServiceError::Gone)
}

/// Gives the image `source` names another opacity; checks it here, not in `check`.
fn set_image_opacity(
    scope: &mut Scope<'_>,
    source: &Source,
    opacity: f64,
) -> Result<(), ServiceError> {
    if !is_image_opacity(opacity) {
        return Err(ServiceError::BadValue {
            field: Field::Opacity,
            value: opacity.to_string(),
        });
    }
    image_of(scope, source)?.opacity = opacity;

    Ok(())
}

/// Swaps the file of the image `source` names for a copy of `with` in `bilder/`; place, width,
/// turn and opacity stay. The width stays, so the height follows the new file's aspect.
fn replace_image(
    scope: &mut Scope<'_>,
    dir: &Path,
    source: &Source,
    with: &Path,
) -> Result<(), ServiceError> {
    let image = image_of(scope, source)?;

    image.file = klappe_store::add_image(dir, with)?;

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
