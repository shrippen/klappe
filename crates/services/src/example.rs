//! The example project: scene 14 of Nachtwache, one copy per language.
//!
//! The files are part of the library (`build.rs` embeds `demo/beispiel-<lang>.klappe`), so a
//! packaged app finds them without a data folder. Only the two project folders are embedded,
//! nothing else of `demo/`.

use std::path::Path;

use crate::catalog::Language;
use crate::error::ServiceError;

include!(concat!(env!("OUT_DIR"), "/examples.rs"));

/// Writes the example of `language` into the project folder `dir` (missing or empty), so the
/// user can open and change a copy of their own.
pub fn copy_example(language: Language, dir: &Path) -> Result<(), ServiceError> {
    let files = match language {
        Language::German => EXAMPLE_DE,
        Language::English => EXAMPLE_EN,
    };
    check_empty(dir)?;

    for (name, bytes) in files {
        let path = dir.join(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|source| write_error(parent, source))?;
        }
        std::fs::write(&path, bytes).map_err(|source| write_error(&path, source))?;
    }

    Ok(())
}

/// `dir` may be missing or an empty folder; anything else would mix the example with it.
fn check_empty(dir: &Path) -> Result<(), ServiceError> {
    let entries = match std::fs::read_dir(dir) {
        Ok(mut entries) => entries.next().is_some(),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => false,
        Err(source) => return Err(write_error(dir, source)),
    };

    if entries {
        return Err(ServiceError::NotEmpty(dir.to_owned()));
    }

    Ok(())
}

fn write_error(path: &Path, source: std::io::Error) -> ServiceError {
    ServiceError::Write {
        path: path.to_owned(),
        source,
    }
}
