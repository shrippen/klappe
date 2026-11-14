//! Whole-folder file operations the layers above need without touching the file system
//! themselves: writing exports and trees, listing and timing a project folder.

use std::fs;
use std::io;
use std::path::Path;
use std::time::SystemTime;

use crate::error::StoreError;
use crate::files::{self, io_err};

/// Beside the target folder while a tree is written: `beispiel-de.klappe.part`.
const PART_SUFFIX: &str = ".part";
/// What the store hides from the user: backups and temp files start with a dot.
const HIDDEN_PREFIX: char = '.';

/// Writes `bytes` to `path` as a whole: a crash leaves the old file or the new one, never
/// half of one. Missing folders above `path` are made.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), StoreError> {
    files::write_atomic(path, bytes)
}

/// Whether `dir` holds nothing; a missing `dir` counts as empty.
pub fn is_empty_dir(dir: &Path) -> Result<bool, StoreError> {
    match fs::read_dir(dir) {
        Ok(mut entries) => Ok(entries.next().is_none()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(true),
        Err(e) => Err(io_err(dir, e)),
    }
}

/// Writes `files` (path inside the folder with `/`, bytes) as the new folder `dir`, which may
/// be missing or empty. The tree is written beside `dir` and moved in whole: a copy cut off
/// midway (disk full) leaves no half folder behind, and what an earlier cut-off copy left
/// beside it is no obstacle.
pub fn write_tree(dir: &Path, files: &[(&str, &[u8])]) -> Result<(), StoreError> {
    let mut name = dir.file_name().unwrap_or_default().to_owned();
    name.push(PART_SUFFIX);
    let part = dir.with_file_name(name);
    match fs::remove_dir_all(&part) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
        Err(e) => return Err(io_err(&part, e)),
    }

    for (name, bytes) in files {
        files::write_atomic(&part.join(name), bytes)?;
    }

    // An existing `dir` would stop the rename; the caller made sure it is empty.
    if dir.exists() {
        fs::remove_dir(dir).map_err(|e| io_err(dir, e))?;
    }
    fs::rename(&part, dir).map_err(|e| io_err(dir, e))
}

/// Removes the files in `folder` whose name starts with `prefix`; a missing folder is fine.
pub fn remove_prefixed(folder: &Path, prefix: &str) {
    let Ok(entries) = fs::read_dir(folder) else {
        return;
    };

    for entry in entries.flatten() {
        if entry.file_name().to_string_lossy().starts_with(prefix) {
            let _ = fs::remove_file(entry.path());
        }
    }
}

/// Newest modification time of the project's own files below `dir`. The backup and other
/// hidden entries are left out: saving a backup is no change of the project.
pub fn project_mtime(dir: &Path) -> Option<SystemTime> {
    let mut newest = None;
    let Ok(entries) = fs::read_dir(dir) else {
        return None;
    };

    for entry in entries.flatten() {
        if entry
            .file_name()
            .to_string_lossy()
            .starts_with(HIDDEN_PREFIX)
        {
            continue;
        }
        let Ok(meta) = entry.metadata() else {
            continue;
        };
        let time = if meta.is_dir() {
            project_mtime(&entry.path())
        } else {
            meta.modified().ok()
        };
        newest = newest.max(time);
    }

    newest
}
