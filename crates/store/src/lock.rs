//! The Sperrstand of the scene numbers: `nummern.toml` (DB-33).

use std::path::Path;

use klappe_model::LockedNumber;
use serde::{Deserialize, Serialize};

use crate::files;
use crate::register::CONFLICT_MARK;
use crate::{StoreError, LOCK_FILE};

/// `nummern.toml`.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Lock {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    nummer: Vec<Entry>,
}

/// One `[[nummer]]`: a numbered heading of the saved book, in the order of the text.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    number: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    id: String,
}

/// The Sperrstand of `dir`; `None` when the file is missing. Git conflict markers refuse it.
pub fn load_lock(dir: &Path) -> Result<Option<Vec<LockedNumber>>, StoreError> {
    let path = dir.join(LOCK_FILE);
    let Some(text) = files::read_optional(&path)? else {
        return Ok(None);
    };
    let conflict = text.lines().position(|l| l.starts_with(CONFLICT_MARK));
    if let Some(index) = conflict {
        return Err(StoreError::Conflict {
            path,
            line: index + 1,
        });
    }

    let lock: Lock = toml::from_str(&text).map_err(|e| StoreError::Parse {
        path,
        message: e.to_string(),
    })?;
    let numbers = lock.nummer.into_iter().map(|e| LockedNumber {
        number: e.number,
        id: e.id,
    });

    Ok(Some(numbers.collect()))
}

/// Writes `nummern.toml` of `numbers` in `dir` and nothing else.
pub fn save_lock(dir: &Path, numbers: &[LockedNumber]) -> Result<(), StoreError> {
    let path = dir.join(LOCK_FILE);
    let lock = Lock {
        nummer: numbers
            .iter()
            .map(|n| Entry {
                number: n.number.clone(),
                id: n.id.clone(),
            })
            .collect(),
    };
    let bytes = files::encode_toml(&path, &lock)?;

    files::write_atomic(&path, &bytes)
}
