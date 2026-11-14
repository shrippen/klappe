//! The example project: scene 14 of Nachtwache, one copy per language.
//!
//! The files are part of the library (`build.rs` embeds `demo/beispiel-<lang>.klappe`), so a
//! packaged app finds them without a data folder. Only the two project folders are embedded,
//! nothing else of `demo/`.

use std::path::Path;

use crate::catalog::Language;
use crate::error::ServiceError;
use crate::session::check_empty;

include!(concat!(env!("OUT_DIR"), "/examples.rs"));

/// Beside the project folder while it is written: `beispiel-de.klappe.part`.
const PART_SUFFIX: &str = ".part";

/// Writes the example of `language` into the project folder `dir` (missing or empty), so the
/// user can open and change a copy of their own. The copy is written beside `dir` and moved in
/// whole: a copy cut off midway (disk full) leaves no half project behind.
pub fn copy_example(language: Language, dir: &Path) -> Result<(), ServiceError> {
    let files = match language {
        Language::German => EXAMPLE_DE,
        Language::English => EXAMPLE_EN,
    };
    check_empty(dir)?;

    let mut name = dir.file_name().unwrap_or_default().to_owned();
    name.push(PART_SUFFIX);
    let part = dir.with_file_name(name);
    match std::fs::remove_dir_all(&part) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(source) => return Err(write_error(&part, source)),
    }

    for (name, bytes) in files {
        let path = part.join(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|source| write_error(parent, source))?;
        }
        std::fs::write(&path, bytes).map_err(|source| write_error(&path, source))?;
    }

    // An empty `dir` would stop the rename; check_empty made sure it is empty or missing.
    if dir.exists() {
        std::fs::remove_dir(dir).map_err(|source| write_error(dir, source))?;
    }
    std::fs::rename(&part, dir).map_err(|source| write_error(dir, source))
}

/// The user's copy of the example in `dir`: written when `dir` is missing or empty, else left
/// as it is, so what the user changed in it is not lost by another click.
pub fn ensure_example(language: Language, dir: &Path) -> Result<(), ServiceError> {
    match check_empty(dir) {
        Err(ServiceError::NotEmpty(_)) => Ok(()),
        Err(e) => Err(e),
        Ok(()) => copy_example(language, dir),
    }
}

fn write_error(path: &Path, source: std::io::Error) -> ServiceError {
    ServiceError::Write {
        path: path.to_owned(),
        source,
    }
}
