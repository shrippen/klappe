//! What the start page does to a project's folder without opening it: rename, archive, trash.
//!
//! ```text
//! archive: store::folder_size ──► export::zip_fits (refuses a folder too big)
//!          store::read_files ──► export::zip_files ──► store::write_atomic (<folder>.zip)
//!                                                  └──► store::move_to_trash (the folder)
//! ```

use std::path::{Path, PathBuf};

use crate::session::check_title;
use crate::ServiceError;

const ARCHIVE_EXT: &str = "zip";

/// Gives the project in `dir` the title `title`; the folder keeps its name.
pub fn rename_project(dir: &Path, title: &str) -> Result<(), ServiceError> {
    let title = check_title(title)?;

    let mut loaded = crate::load::load_project(dir)?;
    loaded.project.title = title.to_owned();
    klappe_store::save(dir, &loaded.project, None)?;

    Ok(())
}

/// Packs the folder `dir` into `<dir>.zip` beside it (`<dir> (2).zip` if that is taken), then
/// moves the folder to the trash. The archive holds the folder's name as its one top entry.
/// Returns the archive; a folder that cannot go to the trash stays beside its archive. A folder
/// of 4 GiB or more, or of more than 65535 files, is refused (`ZipTooBig`) and left alone.
pub fn archive_project(dir: &Path) -> Result<PathBuf, ServiceError> {
    // Asked before anything is read: a huge folder must not fill the memory first.
    let (count, bytes) = klappe_store::folder_size(dir)?;
    klappe_export::zip_fits(count, bytes)?;

    let root = dir.file_name().unwrap_or_default().to_string_lossy();
    let files: Vec<(String, Vec<u8>)> = klappe_store::read_files(dir)?
        .into_iter()
        .map(|(name, bytes)| (format!("{root}/{name}"), bytes))
        .collect();
    let entries: Vec<(&str, &[u8])> = files
        .iter()
        .map(|(name, bytes)| (name.as_str(), bytes.as_slice()))
        .collect();

    let target = klappe_store::free_sibling(dir, ARCHIVE_EXT)?;
    klappe_store::write_atomic(&target, &klappe_export::zip_files(&entries)?)?;
    klappe_store::move_to_trash(dir)?;

    Ok(target)
}

/// Moves the folder `dir` to the system trash.
pub fn trash_project(dir: &Path) -> Result<(), ServiceError> {
    klappe_store::move_to_trash(dir)?;

    Ok(())
}
