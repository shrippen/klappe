//! The locked versions of the screenplay (DB-40): the list `drehbuch/fassungen.toml` and the
//! text of each as it was locked, `drehbuch/fassungen/<id>.fountain`. Both are session-owned,
//! like `nummern.toml`: no save of the project writes them.

use std::fs;
use std::path::{Path, PathBuf};

use klappe_model::{is_version_date, RevisionColor, Version};
use serde::{Deserialize, Serialize};

use crate::files;
use crate::{check_ids, StoreError, VERSIONS_DIR, VERSIONS_FILE, VERSION_EXT};

/// `drehbuch/fassungen.toml`.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct List {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    fassung: Vec<Entry>,
}

/// One `[[fassung]]`, in the order of locking.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    id: String,
    name: String,
    datum: String,
    farbe: RevisionColor,
}

/// The versions of `dir` in the order they were locked; none when the file is missing. A file
/// with Git conflict marks, a wrong type, an unsafe or doubled id or a date that does not
/// exist is refused.
pub fn load_versions(dir: &Path) -> Result<Vec<Version>, StoreError> {
    let path = dir.join(VERSIONS_FILE);
    let Some(list): Option<List> = files::read_toml_merged(&path)? else {
        return Ok(Vec::new());
    };

    check_ids(list.fassung.iter().map(|e| e.id.as_str()))?;
    if let Some(entry) = list.fassung.iter().find(|e| !is_version_date(&e.datum)) {
        return Err(StoreError::Parse {
            path,
            message: format!("{:?} is no date (YYYY-MM-DD)", entry.datum),
        });
    }

    let versions = list.fassung.into_iter().map(|e| Version {
        id: e.id,
        name: e.name,
        date: e.datum,
        color: e.farbe,
    });

    Ok(versions.collect())
}

/// Writes the list of `versions` in `dir` and nothing else.
pub fn save_versions(dir: &Path, versions: &[Version]) -> Result<(), StoreError> {
    check_ids(versions.iter().map(|v| v.id.as_str()))?;

    let path = dir.join(VERSIONS_FILE);
    let list = List {
        fassung: versions
            .iter()
            .map(|v| Entry {
                id: v.id.clone(),
                name: v.name.clone(),
                datum: v.date.clone(),
                farbe: v.color,
            })
            .collect(),
    };
    let bytes = files::encode_toml(&path, &list)?;

    files::write_atomic(&path, &bytes)
}

/// The text of version `id`, exactly as it was written.
pub fn load_version_text(dir: &Path, id: &str) -> Result<String, StoreError> {
    let path = text_path(dir, id)?;

    fs::read_to_string(&path).map_err(|e| files::io_err(&path, e))
}

/// Writes the text of version `id` in `dir` atomically and nothing else.
pub fn save_version_text(dir: &Path, id: &str, text: &str) -> Result<(), StoreError> {
    files::write_atomic(&text_path(dir, id)?, text.as_bytes())
}

/// Deletes the text of version `id`; one that is not there is fine.
pub fn remove_version_text(dir: &Path, id: &str) -> Result<(), StoreError> {
    files::remove_file(&text_path(dir, id)?)
}

fn text_path(dir: &Path, id: &str) -> Result<PathBuf, StoreError> {
    if !files::safe_name(id) {
        return Err(StoreError::BadId { id: id.to_owned() });
    }

    Ok(dir.join(VERSIONS_DIR).join(format!("{id}.{VERSION_EXT}")))
}
