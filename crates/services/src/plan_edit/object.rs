//! Edits of a placed object that take the library into account: its size (LH-25), its kind
//! (LH-114), mirroring (LH-115) and the size of its entry again (LH-116); also the labels bound
//! to elements (LH-118), which `bound` holds.

use klappe_model::{LibraryItem, ObjectShape, SetObject, FREE_FORM_KIND};
use klappe_stellprobe::Source;

use super::{bound, PlanEdit, Scope};
use crate::error::ServiceError;
use crate::field::Field;
use crate::library;

/// What `PlanEdit::SetObject` does to the element `source` names.
#[derive(Debug, Clone, PartialEq)]
pub enum ObjectChange {
    /// Makes the object an object of library entry `kind` (LH-114); place, turn, layer and
    /// base stay. Size, height and name follow the new entry unless the user changed them (they
    /// differ from the old entry's); the shape follows it, a free form's does not.
    Kind(String),
    /// Flips the object left to right (LH-115); again flips it back. A free form's points
    /// are flipped; any other object takes `mirrored`, which its 3D body form shows (a
    /// rectangle and an ellipse look the same on the plan).
    Mirror,
    /// Sets width, depth and height of the object to those of its library entry (LH-116).
    /// Refused for a kind no library knows (a free form too).
    ResetSize,
    /// Pins a note with this text beside the element (an object, a figure or a mark),
    /// bound to it (LH-118): it follows when the element moves.
    Label(String),
    /// Lets the note `source` names go from the element it is bound to (LH-118).
    Unbind,
}

/// Applies an object edit; any other edit is none of its business.
pub(super) fn edit(scope: &mut Scope<'_>, edit: &PlanEdit) -> Result<(), ServiceError> {
    match edit {
        PlanEdit::Resize {
            source,
            width_m,
            depth_m,
        } => resize(scope, source, (*width_m, *depth_m)),
        PlanEdit::SetObject { source, change } => match change {
            ObjectChange::Kind(kind) => set_kind(scope, source, kind),
            ObjectChange::Mirror => mirror(scope, source),
            ObjectChange::ResetSize => reset_size(scope, source),
            ObjectChange::Label(text) => bound::add_label(scope, source, text),
            ObjectChange::Unbind => bound::unbind(scope, source),
        },
        _ => Ok(()),
    }
}

fn bad(field: Field, value: &str) -> ServiceError {
    ServiceError::BadValue {
        field,
        value: value.to_owned(),
    }
}

/// The object `source` names, on the scene's set.
fn object_mut<'a>(
    scope: &'a mut Scope<'_>,
    source: &Source,
) -> Result<&'a mut SetObject, ServiceError> {
    let Source::Object(id) = source else {
        return Err(bad(Field::Source, ""));
    };

    scope
        .set
        .as_deref_mut()
        .and_then(|s| s.objects.iter_mut().find(|o| o.id == *id))
        .ok_or(ServiceError::Gone)
}

fn resize(scope: &mut Scope<'_>, source: &Source, size_m: (f64, f64)) -> Result<(), ServiceError> {
    let object = object_mut(scope, source)?;

    (object.width_m, object.depth_m) = size_m;

    Ok(())
}

/// Whether the object still has what its library entry gave it: size, height and shape. A
/// resized object does not, nor does one of a kind no library knows.
fn as_in_library(object: &SetObject, entry: Option<&LibraryItem>) -> bool {
    entry.is_some_and(|e| {
        e.width_m == object.width_m
            && e.depth_m == object.depth_m
            && e.height_m == object.height_m
            && e.shape == object.shape
    })
}

/// Changes the kind. What the user did to the object stays: a size, height or name that is not
/// the old entry's is kept, so a table made longer stays long as a bench. A free form keeps its
/// outline, whatever its new kind.
fn set_kind(scope: &mut Scope<'_>, source: &Source, kind: &str) -> Result<(), ServiceError> {
    let library = scope.library;
    let entry = match kind != FREE_FORM_KIND {
        true => library::find(library, kind),
        false => None,
    };
    let entry = entry.ok_or_else(|| bad(Field::Kind, kind))?;
    let object = object_mut(scope, source)?;
    let old = library::find(library, &object.kind);
    let untouched = as_in_library(object, old);
    let unnamed = old.map_or(object.label.is_empty(), |o| o.label == object.label);

    if untouched {
        object.width_m = entry.width_m;
        object.depth_m = entry.depth_m;
        object.height_m = entry.height_m;
    }
    if unnamed {
        object.label = entry.label.clone();
    }
    if !matches!(object.shape, ObjectShape::Polygon { .. }) {
        object.shape = entry.shape.clone();
    }
    object.kind = kind.to_owned();

    Ok(())
}

/// A free form's outline is turned round in its points, so every grip stays where the plan
/// shows it; anything else takes the flag, which its body form in 3D reads.
fn mirror(scope: &mut Scope<'_>, source: &Source) -> Result<(), ServiceError> {
    let object = object_mut(scope, source)?;

    if let ObjectShape::Polygon { points } = &mut object.shape {
        points.iter_mut().for_each(|p| p.x_m = -p.x_m);
        points.reverse();
        return Ok(());
    }

    object.mirrored = !object.mirrored;

    Ok(())
}

/// Width, depth and height of the object's library entry; the base and the turn stay.
fn reset_size(scope: &mut Scope<'_>, source: &Source) -> Result<(), ServiceError> {
    let library = scope.library;
    let object = object_mut(scope, source)?;
    let entry =
        library::find(library, &object.kind).ok_or_else(|| bad(Field::Kind, &object.kind))?;

    object.width_m = entry.width_m;
    object.depth_m = entry.depth_m;
    object.height_m = entry.height_m;

    Ok(())
}
