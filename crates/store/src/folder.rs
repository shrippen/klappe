//! A project folder as a whole, for archiving and throwing away: its files as bytes, a free
//! name beside it, and the system trash.

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::StoreError;
use crate::files::io_err;

/// Names tried beside a folder before `free_sibling` gives up: `a.zip`, `a (2).zip` ...
const MAX_COPIES: u32 = 1000;

/// Every file below `dir` (the hidden backup too) as path inside the folder with `/` and
/// bytes, sorted by path.
pub fn read_files(dir: &Path) -> Result<Vec<(String, Vec<u8>)>, StoreError> {
    let mut found = Vec::new();
    collect(dir, "", &mut found)?;
    found.sort_by(|a, b| a.0.cmp(&b.0));

    Ok(found)
}

fn collect(dir: &Path, prefix: &str, out: &mut Vec<(String, Vec<u8>)>) -> Result<(), StoreError> {
    for entry in fs::read_dir(dir).map_err(|e| io_err(dir, e))? {
        let entry = entry.map_err(|e| io_err(dir, e))?;
        let path = entry.path();
        let name = format!("{prefix}{}", entry.file_name().to_string_lossy());
        if path.is_dir() {
            collect(&path, &format!("{name}/"), out)?;
            continue;
        }

        out.push((name, fs::read(&path).map_err(|e| io_err(&path, e))?));
    }

    Ok(())
}

/// `<dir>.<ext>` beside `dir`, or `<dir> (2).<ext>`, `(3)` ... when that is taken.
pub fn free_sibling(dir: &Path, ext: &str) -> Result<PathBuf, StoreError> {
    let name = dir.file_name().unwrap_or_default().to_string_lossy();
    let first = dir.with_file_name(format!("{name}.{ext}"));
    if !first.exists() {
        return Ok(first);
    }

    for n in 2..=MAX_COPIES {
        let next = dir.with_file_name(format!("{name} ({n}).{ext}"));
        if !next.exists() {
            return Ok(next);
        }
    }

    Err(io_err(
        &first,
        std::io::Error::new(std::io::ErrorKind::AlreadyExists, "no free name"),
    ))
}

/// Moves `dir` to the system trash (freedesktop, Windows recycle bin, macOS trash).
pub fn move_to_trash(dir: &Path) -> Result<(), StoreError> {
    trash::delete(dir).map_err(|e| StoreError::Trash {
        path: dir.to_owned(),
        message: e.to_string(),
    })
}
