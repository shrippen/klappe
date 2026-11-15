//! The object library: the project's own objects in `bibliothek.toml`, library files to pass
//! on, and the built-in library (LH-11, LH-12).

use std::path::Path;
use std::sync::OnceLock;

use klappe_model::{
    is_object_height_m, is_object_size_m, is_outline, BodyForm, LibraryItem, ObjectShape,
};
use serde::{Deserialize, Serialize};

use crate::files;
use crate::StoreError;

const BUILTIN: &str = include_str!("../library/standard.toml");
const BUILTIN_FORMS: &str = include_str!("../library/forms.toml");

/// A library file: one `[[objekt]]` per item. Other keys or no `objekt` at all mean the file
/// is no library (an import of `projekt.toml` would read as an empty one otherwise).
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LibraryFile {
    objekt: Vec<LibraryItem>,
}

/// The library that ships with Klappe. Embedded and tested to parse; an unreadable one would
/// only leave the library empty.
pub fn builtin_library() -> &'static [LibraryItem] {
    static ITEMS: OnceLock<Vec<LibraryItem>> = OnceLock::new();

    ITEMS.get_or_init(|| {
        toml::from_str::<LibraryFile>(BUILTIN)
            .map(|file| file.objekt)
            .unwrap_or_default()
    })
}

/// The body forms the built-in library names (LH-97): one `[[form]]` per form, each with a
/// `[[form.box]]` per box.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FormFile {
    form: Vec<BodyForm>,
}

/// The body forms that ship with Klappe. Embedded and tested to load; an unreadable file would
/// only leave the objects as plain prisms.
pub fn builtin_forms() -> &'static [BodyForm] {
    static FORMS: OnceLock<Vec<BodyForm>> = OnceLock::new();

    FORMS.get_or_init(|| {
        toml::from_str::<FormFile>(BUILTIN_FORMS)
            .ok()
            .map(|file| file.form)
            .filter(|forms| check_forms(forms).is_ok())
            .unwrap_or_default()
    })
}

/// Every form has a name of its own and boxes that lie within the object.
fn check_forms(forms: &[BodyForm]) -> Result<(), String> {
    for (index, form) in forms.iter().enumerate() {
        if form.boxes.is_empty() || !form.boxes.iter().all(|b| b.is_usable()) {
            return Err(format!("form {:?}: boxes are not usable", form.name));
        }
        if forms[..index].iter().any(|other| other.name == form.name) {
            return Err(format!("form {:?} is there twice", form.name));
        }
    }

    Ok(())
}

/// Reads a library file (the project's `bibliothek.toml` or one passed on). A file with an
/// item that cannot be used is refused as a whole.
pub fn load_library(path: &Path) -> Result<Vec<LibraryItem>, StoreError> {
    let file: LibraryFile = files::read_toml(path)?;
    check_items(&file.objekt).map_err(|message| StoreError::Parse {
        path: path.to_path_buf(),
        message,
    })?;

    Ok(file.objekt)
}

/// Writes `items` as a library file, replacing the file atomically.
pub fn save_library(path: &Path, items: &[LibraryItem]) -> Result<(), StoreError> {
    let bytes = encode(path, items)?;

    files::write_atomic(path, &bytes)
}

/// The file content of `items`; refuses items that could not be read back.
pub(crate) fn encode(path: &Path, items: &[LibraryItem]) -> Result<Vec<u8>, StoreError> {
    check_items(items).map_err(|message| StoreError::Encode {
        path: path.to_path_buf(),
        message,
    })?;
    let file = LibraryFile {
        objekt: items.to_vec(),
    };

    files::encode_toml(path, &file)
}

/// Every item has a kind, a size an object can have, an outline if it is a free form, a height
/// if it has one, a built-in form if it names one, and a kind of its own.
fn check_items(items: &[LibraryItem]) -> Result<(), String> {
    for (index, item) in items.iter().enumerate() {
        if item.kind.trim().is_empty() {
            return Err(format!("object {}: kind is empty", index + 1));
        }
        if !is_object_size_m(item.width_m) || !is_object_size_m(item.depth_m) {
            return Err(format!(
                "object {:?}: size {} x {} m is not usable",
                item.kind, item.width_m, item.depth_m
            ));
        }
        if matches!(&item.shape, ObjectShape::Polygon { points } if !is_outline(points)) {
            return Err(format!("object {:?}: outline is not usable", item.kind));
        }
        if item.height_m.is_some_and(|h| !is_object_height_m(h)) {
            return Err(format!(
                "object {:?}: height {:?} m is not usable",
                item.kind, item.height_m
            ));
        }
        if item.form.as_ref().is_some_and(|name| !is_form(name)) {
            return Err(format!("object {:?}: form is unknown", item.kind));
        }
        if items[..index].iter().any(|other| other.kind == item.kind) {
            return Err(format!("object {:?} is there twice", item.kind));
        }
    }

    Ok(())
}

/// Whether `name` is a built-in body form.
fn is_form(name: &str) -> bool {
    builtin_forms().iter().any(|form| form.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The embedded files load and pass their checks: an unreadable forms file would only
    /// show as plain prisms.
    #[test]
    fn the_built_in_files_load_and_pass_their_checks() {
        let library: LibraryFile = toml::from_str(BUILTIN).expect("library");
        let forms: FormFile = toml::from_str(BUILTIN_FORMS).expect("forms");

        assert!(check_items(&library.objekt).is_ok());
        assert_eq!(check_forms(&forms.form), Ok(()));
        assert!(!builtin_forms().is_empty());
    }

    /// Every form a built-in entry names exists; a form no entry names is dead data.
    #[test]
    fn built_in_entries_and_forms_belong_together() {
        for item in builtin_library() {
            assert!(item.form.as_deref().is_none_or(is_form), "{}", item.kind);
        }
        for form in builtin_forms() {
            let used = builtin_library()
                .iter()
                .any(|i| i.form.as_deref() == Some(form.name.as_str()));

            assert!(used, "{} is named by no entry", form.name);
        }
    }

    #[test]
    fn a_form_needs_usable_boxes_and_a_name_of_its_own() {
        let parse = |text: &str| toml::from_str::<FormFile>(text).expect("parse").form;
        let one = "[[form]]\nname = \"a\"\n[[form.box]]\nfrom = [0, 0, 0]\nto = [1, 1, 1]\n";

        assert_eq!(check_forms(&parse(one)), Ok(()));
        assert!(check_forms(&parse(&one.repeat(2))).is_err());
        assert!(check_forms(&parse(&one.replace("[1, 1, 1]", "[1, 1, 2]"))).is_err());
        assert!(check_forms(&parse(&one.replace("[1, 1, 1]", "[0, 1, 1]"))).is_err());
        assert!(check_forms(&parse("[[form]]\nname = \"a\"\nbox = []\n")).is_err());
    }
}
