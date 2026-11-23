//! The project's library: taking objects and files in, and changing its own entries (LH-12).

use klappe_model::{is_object_height_m, is_object_size_m, LibraryItem, Project, FREE_FORM_KIND};

use super::{name_of, slug, unique_id};
use crate::error::ServiceError;
use crate::field::Field;
use crate::library;

/// Edits of the project's library (LH-12).
#[derive(Debug, Clone, PartialEq)]
pub enum LibraryEdit {
    /// Files the object `object` of the set at `location` in the project's library under
    /// `name` (LH-12): its size and shape, a kind of its own.
    LibraryFromObject {
        location: String,
        object: String,
        name: String,
    },
    /// Takes the entries of a library file over (LH-12). An entry the project has already
    /// (same kind and same content) is skipped; one whose kind is taken by something else
    /// gets a kind of its own.
    ImportLibrary { items: Vec<LibraryItem> },
    /// Renames and regroups the project's own entry `kind` (LH-12); no group takes it out of
    /// any. Placed objects keep their kind. A built-in entry is not the project's to change.
    EditLibraryItem {
        kind: String,
        name: String,
        group: String,
    },
    /// Takes the project's own entry `kind` out of the library (LH-12); objects placed from
    /// it stay as they are.
    RemoveLibraryItem { kind: String },
}

/// What the library edits do.
pub(super) fn apply(project: &mut Project, edit: &LibraryEdit) -> Result<(), ServiceError> {
    match edit {
        LibraryEdit::LibraryFromObject {
            location,
            object,
            name,
        } => from_object(project, location, object, name),
        LibraryEdit::ImportLibrary { items } => import(project, items),
        LibraryEdit::EditLibraryItem { kind, name, group } => edit_item(project, kind, name, group),
        LibraryEdit::RemoveLibraryItem { kind } => remove_item(project, kind),
    }
}

const LIBRARY_KIND_FALLBACK: &str = "objekt";

fn from_object(
    project: &mut Project,
    location: &str,
    object: &str,
    name: &str,
) -> Result<(), ServiceError> {
    let name = name_of(name)?;
    let object = project
        .location(location)
        .ok_or_else(|| ServiceError::NoLocation(location.to_owned()))?
        .set
        .objects
        .iter()
        .find(|o| o.id == object)
        .ok_or(ServiceError::Gone)?;
    let (width_m, depth_m, shape) = (object.width_m, object.depth_m, object.shape.clone());
    let height_m = object.height_m;

    // A hand-edited set can hold sizes no library takes; checked here, or every later save
    // fails on `bibliothek.toml`.
    let kind = free_kind(project, &slug(&name, LIBRARY_KIND_FALLBACK));
    let item = LibraryItem {
        kind,
        label: name,
        group: String::new(),
        width_m,
        depth_m,
        shape,
        height_m,
        form: None,
    };
    check_item(&item)?;

    project.library.push(item);
    Ok(())
}

fn import(project: &mut Project, items: &[LibraryItem]) -> Result<(), ServiceError> {
    for item in items {
        let mut item = item.clone();
        check_item(&item)?;
        if item.label.trim().is_empty() {
            item.label = item.kind.clone();
        }

        // The same entry again (a file passed on twice) adds nothing.
        let known = library::find(&project.library, &item.kind);
        if known.is_some_and(|k| same_entry(k, &item)) {
            continue;
        }

        if known.is_some() || item.kind == FREE_FORM_KIND {
            item.kind = free_kind(project, &slug(&item.kind, LIBRARY_KIND_FALLBACK));
        }
        project.library.push(item);
    }
    Ok(())
}

/// The entry's content, ignoring the name a built-in entry gets from the catalogue.
fn same_entry(a: &LibraryItem, b: &LibraryItem) -> bool {
    let named = |item: &LibraryItem| match item.label.is_empty() {
        true => item.kind.clone(),
        false => item.label.clone(),
    };

    let content = |item: &LibraryItem| {
        (
            item.width_m,
            item.depth_m,
            item.shape.clone(),
            item.height_m,
            item.group.clone(),
            item.form.clone(),
        )
    };

    named(a) == named(b) && content(a) == content(b)
}

fn check_item(item: &LibraryItem) -> Result<(), ServiceError> {
    if item.kind.trim().is_empty() {
        return Err(ServiceError::BadValue {
            field: Field::Kind,
            value: String::new(),
        });
    }
    if !is_object_size_m(item.width_m) || !is_object_size_m(item.depth_m) {
        return Err(ServiceError::BadValue {
            field: Field::Size,
            value: format!("{} x {}", item.width_m, item.depth_m),
        });
    }
    if item.height_m.is_some_and(|h| !is_object_height_m(h)) {
        return Err(ServiceError::BadValue {
            field: Field::Height,
            value: format!("{:?}", item.height_m),
        });
    }
    Ok(())
}

/// `base`, or `base-2` … until no entry of the library, built in or own, has the kind; the
/// free forms' kind is never free.
fn free_kind(project: &Project, base: &str) -> String {
    let taken = library::entries(&project.library);
    let kinds = taken.iter().map(|e| e.kind.as_str());

    unique_id(base, kinds.chain([FREE_FORM_KIND]))
}

fn edit_item(
    project: &mut Project,
    kind: &str,
    name: &str,
    group: &str,
) -> Result<(), ServiceError> {
    let name = name_of(name)?;
    let item = project
        .library
        .iter_mut()
        .find(|i| i.kind == kind)
        .ok_or(ServiceError::Gone)?;

    item.label = name;
    item.group = group.trim().to_owned();
    Ok(())
}

fn remove_item(project: &mut Project, kind: &str) -> Result<(), ServiceError> {
    let before = project.library.len();

    project.library.retain(|i| i.kind != kind);
    if project.library.len() == before {
        return Err(ServiceError::Gone);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_group_is_a_changed_entry() {
        let item = LibraryItem {
            kind: "thron".into(),
            label: "Thron".into(),
            group: String::new(),
            width_m: 0.9,
            depth_m: 0.8,
            shape: klappe_model::ObjectShape::Rect,
            height_m: None,
            form: None,
        };
        let regrouped = LibraryItem {
            group: "Hof".into(),
            ..item.clone()
        };

        assert!(same_entry(&item, &item.clone()));
        assert!(!same_entry(&item, &regrouped));
    }
}
