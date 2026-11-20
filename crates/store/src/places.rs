//! The scripted locations: `orte.toml` (DB-51).

use std::path::{Path, PathBuf};

use klappe_model::Place;
use serde::{Deserialize, Serialize};

use crate::files;
use crate::{StoreError, PLACES_FILE};

/// Whether a project save writes `orte.toml`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum PlacesFile {
    Write,
    Keep,
}

/// `orte.toml`.
#[derive(Serialize, Deserialize)]
struct Places {
    #[serde(default)]
    ort: Vec<Place>,
}

/// The scripted locations of `dir`; none when the file is missing.
pub fn load_places(dir: &Path) -> Result<Vec<Place>, StoreError> {
    let path = dir.join(PLACES_FILE);
    if !files::exists(&path)? {
        return Ok(Vec::new());
    }

    let places: Places = files::read_toml(&path)?;
    check(&places.ort).map_err(|message| StoreError::Parse { path, message })?;

    Ok(places.ort)
}

/// `orte.toml` of `places`, encoded.
pub(crate) fn encode(dir: &Path, places: &[Place]) -> Result<(PathBuf, Vec<u8>), StoreError> {
    validate(dir, places)?;
    let path = dir.join(PLACES_FILE);
    let bytes = files::encode_toml(
        &path,
        &Places {
            ort: places.to_vec(),
        },
    )?;

    Ok((path, bytes))
}

/// Writes `orte.toml` of `places` in `dir` and nothing else (the file goes when there are
/// none): the answer about a heading is kept at once, from the places as the caller read
/// them, so a program running beside this one keeps its other changes (TUI beside the
/// window).
pub fn save_places(dir: &Path, places: &[Place]) -> Result<(), StoreError> {
    let (path, bytes) = encode(dir, places)?;
    if places.is_empty() {
        return files::remove_file(&path);
    }

    files::write_atomic(&path, &bytes)
}

/// Refuses places that a load would refuse, before anything is written.
pub(crate) fn validate(dir: &Path, places: &[Place]) -> Result<(), StoreError> {
    check(places).map_err(|message| StoreError::Parse {
        path: dir.join(PLACES_FILE),
        message,
    })
}

/// Ids are used once and no name or variant is empty or belongs to two places: a heading
/// must name one place (compared as headings are, in capitals).
pub(crate) fn check(places: &[Place]) -> Result<(), String> {
    for (index, place) in places.iter().enumerate() {
        if place.id.trim().is_empty() {
            return Err("a place has no id".into());
        }

        let rest = &places[index + 1..];
        if rest.iter().any(|o| o.id == place.id) {
            return Err(format!("{}: the id is used twice", place.id));
        }

        let names = std::iter::once(&place.name).chain(&place.variants);
        for name in names {
            if name.trim().is_empty() {
                return Err(format!("{}: a name is empty", place.id));
            }
            if let Some(other) = rest.iter().find(|o| o.is_called(name)) {
                return Err(format!(
                    "{}: {name} is also a name of {}",
                    place.id, other.id
                ));
            }
        }
    }

    Ok(())
}
