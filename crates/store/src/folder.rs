//! A project folder as a whole, for archiving and throwing away: its files as bytes, a free
//! name beside it, and the system trash.

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::StoreError;
use crate::files::{self, io_err};

/// Names tried beside a folder before `free_sibling` gives up: `a.zip`, `a (2).zip` ...
const MAX_COPIES: u32 = 1000;

/// Extension of a folder set aside.
const ASIDE_EXT: &str = "alt";

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

/// How many files lie below `dir` and their bytes in all, without reading them.
pub fn folder_size(dir: &Path) -> Result<(usize, u64), StoreError> {
    let mut count = 0;
    let mut bytes = 0;
    for entry in fs::read_dir(dir).map_err(|e| io_err(dir, e))? {
        let entry = entry.map_err(|e| io_err(dir, e))?;
        let path = entry.path();
        if path.is_dir() {
            let (inner_count, inner_bytes) = folder_size(&path)?;
            count += inner_count;
            bytes += inner_bytes;
            continue;
        }

        count += 1;
        bytes += entry.metadata().map_err(|e| io_err(&path, e))?.len();
    }

    Ok((count, bytes))
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

/// The first `N` from 1 for which `<dir>/<stem>-<N><ext>` is not there, with that path.
pub fn free_numbered(dir: &Path, stem: &str, ext: &str) -> Result<(u32, PathBuf), StoreError> {
    for n in 1..=MAX_COPIES {
        let path = dir.join(format!("{stem}-{n}{ext}"));
        if !path.exists() {
            return Ok((n, path));
        }
    }

    Err(io_err(
        dir,
        std::io::Error::new(std::io::ErrorKind::AlreadyExists, "no free name"),
    ))
}

/// Renames the folder `dir` to a free name beside it (`<dir>.alt`, `<dir> (2).alt` ...) and
/// returns that name; for a copy that must not be overwritten or deleted.
pub fn set_aside(dir: &Path) -> Result<PathBuf, StoreError> {
    let aside = free_sibling(dir, ASIDE_EXT)?;
    files::rename(dir, &aside)?;

    Ok(aside)
}

/// Moves `dir` to the system trash (freedesktop, Windows recycle bin, macOS trash).
pub fn move_to_trash(dir: &Path) -> Result<(), StoreError> {
    trash::delete(dir).map_err(|e| StoreError::Trash {
        path: dir.to_owned(),
        message: e.to_string(),
    })
}
