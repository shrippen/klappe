//! Lower level: files, atomic writes, directory listings.

use std::collections::HashSet;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{StoreError, JSON_EXT, TOML_EXT};

const TEMP_PREFIX: &str = ".";
const TEMP_SUFFIX: &str = ".tmp";
/// Characters Windows does not allow in file names (`/` and `\` are checked as separators).
const WINDOWS_FORBIDDEN: &str = "<>:\"|?*/\\";
const WINDOWS_DEVICES: [&str; 4] = ["con", "prn", "aux", "nul"];

#[derive(Debug, Clone, Copy)]
pub(crate) enum Kind {
    Toml,
    Json,
}

impl Kind {
    fn ext(self) -> &'static str {
        match self {
            Kind::Toml => TOML_EXT,
            Kind::Json => JSON_EXT,
        }
    }
}

/// A folder of one-file-per-id documents of one kind.
pub(crate) struct Dir {
    path: PathBuf,
    kind: Kind,
}

impl Dir {
    pub(crate) fn new(path: PathBuf, kind: Kind) -> Self {
        Self { path, kind }
    }

    /// `(id, path)` of every document, sorted by id; hidden files are ignored.
    pub(crate) fn list(&self) -> Result<Vec<(String, PathBuf)>, StoreError> {
        let entries = match fs::read_dir(&self.path) {
            Ok(entries) => entries,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(io_err(&self.path, e)),
        };

        let mut found = Vec::new();
        for entry in entries {
            let path = entry.map_err(|e| io_err(&self.path, e))?.path();
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            let right_ext = path.extension().is_some_and(|e| e == self.kind.ext());
            if !right_ext || stem.starts_with('.') || !path.is_file() {
                continue;
            }
            found.push((stem.to_owned(), path));
        }
        found.sort();

        Ok(found)
    }

    pub(crate) fn read<T: for<'de> Deserialize<'de>>(&self, path: &Path) -> Result<T, StoreError> {
        match self.kind {
            Kind::Toml => read_toml(path),
            Kind::Json => read_json(path),
        }
    }

    /// The target path and file content of one document.
    pub(crate) fn encode<T: Serialize + for<'de> Deserialize<'de>>(
        &self,
        id: &str,
        value: &T,
    ) -> Result<(PathBuf, Vec<u8>), StoreError> {
        let path = self.path.join(format!("{id}.{}", self.kind.ext()));
        let bytes = match self.kind {
            Kind::Toml => encode_toml(&path, value)?,
            Kind::Json => encode_json(&path, value)?,
        };

        Ok((path, bytes))
    }

    /// Deletes the documents whose id is not in `keep`.
    pub(crate) fn remove_except(&self, keep: &HashSet<&str>) -> Result<(), StoreError> {
        for (id, path) in self.list()? {
            if keep.contains(id.as_str()) {
                continue;
            }
            fs::remove_file(&path).map_err(|e| io_err(&path, e))?;
        }

        Ok(())
    }
}

/// Ids become file names, so they must be valid on every system: no separators, no `..`,
/// not hidden, not empty, and nothing Windows rejects.
pub(crate) fn safe_name(id: &str) -> bool {
    let unsafe_id = id.is_empty()
        || id.starts_with('.')
        || id.contains("..")
        || id.ends_with([' ', '.'])
        || id.contains(|c: char| c.is_control() || WINDOWS_FORBIDDEN.contains(c))
        || is_device_name(id);

    !unsafe_id
}

/// Windows reserves `con`, `nul`, `com1` … with any extension (`con.txt` too).
fn is_device_name(id: &str) -> bool {
    let stem = id.split('.').next().unwrap_or(id).to_ascii_lowercase();
    let numbered = |prefix: &str| {
        stem.strip_prefix(prefix)
            .is_some_and(|n| matches!(n.as_bytes(), [b'1'..=b'9']))
    };

    WINDOWS_DEVICES.contains(&stem.as_str()) || numbered("com") || numbered("lpt")
}

pub(crate) fn exists(path: &Path) -> Result<bool, StoreError> {
    path.try_exists().map_err(|e| io_err(path, e))
}

pub(crate) fn read_optional(path: &Path) -> Result<Option<String>, StoreError> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(io_err(path, e)),
    }
}

pub(crate) fn read_toml<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, StoreError> {
    let text = fs::read_to_string(path).map_err(|e| io_err(path, e))?;

    toml::from_str(&text).map_err(|e| StoreError::Parse {
        path: path.to_path_buf(),
        message: e.to_string(),
    })
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, StoreError> {
    let text = fs::read_to_string(path).map_err(|e| io_err(path, e))?;

    serde_json::from_str(&text).map_err(|e| StoreError::Parse {
        path: path.to_path_buf(),
        message: e.to_string(),
    })
}

/// Serializes `value` to TOML text. Refuses NaN and infinity.
pub(crate) fn encode_toml<T: Serialize + for<'de> Deserialize<'de>>(
    path: &Path,
    value: &T,
) -> Result<Vec<u8>, StoreError> {
    check_finite(path, value)?;
    let mut text = toml::to_string_pretty(value).map_err(|e| StoreError::Encode {
        path: path.to_path_buf(),
        message: e.to_string(),
    })?;
    if !text.ends_with('\n') {
        text.push('\n');
    }

    Ok(text.into_bytes())
}

/// Serializes `value` to pretty JSON. Refuses NaN and infinity.
fn encode_json<T: Serialize + for<'de> Deserialize<'de>>(
    path: &Path,
    value: &T,
) -> Result<Vec<u8>, StoreError> {
    check_finite(path, value)?;
    let mut text = serde_json::to_string_pretty(value).map_err(|e| StoreError::Encode {
        path: path.to_path_buf(),
        message: e.to_string(),
    })?;
    text.push('\n');

    Ok(text.into_bytes())
}

/// serde_json turns NaN and infinity into `null`, which does not read back as a number.
/// Converting to a JSON value and back fails exactly then, and names the field.
fn check_finite<T: Serialize + for<'de> Deserialize<'de>>(
    path: &Path,
    value: &T,
) -> Result<(), StoreError> {
    let encode_err = |message: String| StoreError::Encode {
        path: path.to_path_buf(),
        message,
    };
    let json = serde_json::to_value(value).map_err(|e| encode_err(e.to_string()))?;

    serde_json::from_value::<T>(json)
        .map(drop)
        .map_err(|e| encode_err(format!("non-finite number: {e}")))
}

/// Writes to a temp file next to `path`, then renames it over `path`: a crash leaves the old
/// file or the new one, never half of one (LH-05).
pub(crate) fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), StoreError> {
    let (Some(parent), Some(name)) = (path.parent(), path.file_name()) else {
        return Err(io_err(path, io::ErrorKind::InvalidInput.into()));
    };
    fs::create_dir_all(parent).map_err(|e| io_err(parent, e))?;

    let temp = parent.join(format!(
        "{TEMP_PREFIX}{}{TEMP_SUFFIX}",
        name.to_string_lossy()
    ));
    let written = fs::File::create(&temp).and_then(|mut file| {
        file.write_all(bytes)?;
        file.sync_all()
    });
    if let Err(e) = written {
        let _ = fs::remove_file(&temp);
        return Err(io_err(&temp, e));
    }

    fs::rename(&temp, path).map_err(|e| {
        let _ = fs::remove_file(&temp);
        io_err(path, e)
    })
}

pub(crate) fn io_err(path: &Path, source: io::Error) -> StoreError {
    StoreError::Io {
        path: path.to_path_buf(),
        source,
    }
}
