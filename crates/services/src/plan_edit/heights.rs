//! Heights of walls, openings and objects (LH-90): set, or taken off with `None` so that the
//! default of `klappe_stellprobe` applies again.

use klappe_stellprobe::{opening_span, Source};

use super::openings::opening_mut;
use super::{PlanEdit, Scope};
use crate::error::ServiceError;
use crate::field::Field;

fn refused(field: Field, value: String) -> ServiceError {
    ServiceError::BadValue { field, value }
}

/// Applies a height edit; any other edit is none of its business.
pub(super) fn edit_height(scope: &mut Scope<'_>, edit: &PlanEdit) -> Result<(), ServiceError> {
    match edit {
        PlanEdit::SetWallHeight { source, height_m } => set_wall(scope, source, *height_m),
        PlanEdit::SetObjectHeight { source, height_m } => set_height(scope, source, *height_m),
        PlanEdit::SetObjectBase { source, base_m } => set_base(scope, source, *base_m),
        PlanEdit::SetOpeningSill { source, sill_m } => set_sill(scope, source, *sill_m),
        PlanEdit::SetOpeningHead { source, head_m } => set_head(scope, source, *head_m),
        _ => Ok(()),
    }
}

fn set_wall(
    scope: &mut Scope<'_>,
    source: &Source,
    height_m: Option<f64>,
) -> Result<(), ServiceError> {
    let Source::Wall(index) = source else {
        return Err(refused(Field::Source, String::new()));
    };
    let wall = scope
        .set
        .as_deref_mut()
        .and_then(|s| s.walls.get_mut(*index))
        .ok_or(ServiceError::Gone)?;

    wall.height_m = height_m;

    Ok(())
}

fn set_height(
    scope: &mut Scope<'_>,
    source: &Source,
    height_m: Option<f64>,
) -> Result<(), ServiceError> {
    let Source::Object(id) = source else {
        return Err(refused(Field::Source, String::new()));
    };
    let object = scope
        .set
        .as_deref_mut()
        .and_then(|s| s.objects.iter_mut().find(|o| o.id == *id))
        .ok_or(ServiceError::Gone)?;

    object.height_m = height_m;

    Ok(())
}

fn set_base(
    scope: &mut Scope<'_>,
    source: &Source,
    base_m: Option<f64>,
) -> Result<(), ServiceError> {
    let Source::Object(id) = source else {
        return Err(refused(Field::Source, String::new()));
    };
    let object = scope
        .set
        .as_deref_mut()
        .and_then(|s| s.objects.iter_mut().find(|o| o.id == *id))
        .ok_or(ServiceError::Gone)?;

    object.base_m = base_m;

    Ok(())
}

/// Sets the sill; with the default sill or head counted, it must stay below the head.
fn set_sill(
    scope: &mut Scope<'_>,
    source: &Source,
    sill_m: Option<f64>,
) -> Result<(), ServiceError> {
    let (wall, index) = opening_mut(scope, source)?;
    let opening = &mut wall.openings[index];

    opening.sill_m = sill_m;
    let (sill_m, head_m) = opening_span(opening);
    if sill_m >= head_m {
        return Err(refused(Field::Sill, sill_m.to_string()));
    }

    Ok(())
}

/// Sets the head; with the default sill or head counted, it must stay above the sill.
fn set_head(
    scope: &mut Scope<'_>,
    source: &Source,
    head_m: Option<f64>,
) -> Result<(), ServiceError> {
    let (wall, index) = opening_mut(scope, source)?;
    let opening = &mut wall.openings[index];

    opening.head_m = head_m;
    let (sill_m, head_m) = opening_span(opening);
    if sill_m >= head_m {
        return Err(refused(Field::Head, head_m.to_string()));
    }

    Ok(())
}
