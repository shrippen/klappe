//! The object library: the project's own objects in `bibliothek.toml`, library files to pass
//! on, and the built-in library (LH-11, LH-12).

use std::path::Path;
use std::sync::OnceLock;

use klappe_model::{is_object_size_m, LibraryItem};
use serde::{Deserialize, Serialize};

use crate::files;
use crate::StoreError;

const BUILTIN: &str = include_str!("../library/standard.toml");

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

/// Every item has a kind, a size an object can have, and a kind of its own.
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
        if items[..index].iter().any(|other| other.kind == item.kind) {
            return Err(format!("object {:?} is there twice", item.kind));
        }
    }

    Ok(())
}
