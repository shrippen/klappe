//! The example project: scene 14 of Nachtwache, one copy per language.
//!
//! The files are part of the library (`build.rs` embeds `demo/beispiel-<lang>.klappe`), so a
//! packaged app finds them without a data folder. Only the two project folders are embedded,
//! nothing else of `demo/`.

use std::path::{Path, PathBuf};

use crate::catalog::Language;
use crate::error::{write_failed, ServiceError};
use crate::session::{check_empty, Session};

include!(concat!(env!("OUT_DIR"), "/examples.rs"));

/// Writes the example of `language` into the project folder `dir` (missing or empty), so the
/// user can open and change a copy of their own. The copy is written beside `dir` and moved in
/// whole: a copy cut off midway (disk full) leaves no half project behind.
pub fn copy_example(language: Language, dir: &Path) -> Result<(), ServiceError> {
    let files = match language {
        Language::German => EXAMPLE_DE,
        Language::English => EXAMPLE_EN,
    };
    check_empty(dir)?;

    klappe_store::write_tree(dir, files).map_err(write_failed)
}

/// Suffix of a copy set aside; a further one gets `-2`, `-3` ... after it.
const ASIDE: &str = ".alt";

/// The user's copy of the example in `dir`: written when `dir` is missing or empty, else left
/// as it is, so what the user changed in it is not lost by another click. A copy that does not
/// load (an older format) is renamed beside `dir`, never deleted, and a fresh one is written;
/// the result is where the old one went.
pub fn ensure_example(language: Language, dir: &Path) -> Result<Option<PathBuf>, ServiceError> {
    match check_empty(dir) {
        Err(ServiceError::NotEmpty(_)) => {}
        Err(e) => return Err(e),
        Ok(()) => return copy_example(language, dir).map(|()| None),
    }
    if Session::open(dir).is_ok() {
        return Ok(None);
    }

    let aside = free_name(dir);
    std::fs::rename(dir, &aside).map_err(|source| ServiceError::Write {
        path: dir.to_owned(),
        source,
    })?;
    // From here the old copy is somewhere else; the error says where.
    if let Err(source) = copy_example(language, dir) {
        return Err(ServiceError::ExampleAside {
            aside,
            source: Box::new(source),
        });
    }

    Ok(Some(aside))
}

/// `dir` with `.alt`, or `.alt-2`, `.alt-3` ... after its name: the first that is free.
fn free_name(dir: &Path) -> PathBuf {
    let name = dir.file_name().unwrap_or_default().to_string_lossy();
    let mut taken = dir.with_file_name(format!("{name}{ASIDE}"));
    let mut n = 2;

    while taken.exists() {
        taken = dir.with_file_name(format!("{name}{ASIDE}-{n}"));
        n += 1;
    }

    taken
}
