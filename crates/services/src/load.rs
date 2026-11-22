//! Opens a project folder through the store and refuses what the store cannot see: Git
//! conflict markers in the screenplay (DATENMODELL §4). The store checks `szenen.toml` itself.

use std::path::Path;

use klappe_store::{Loaded, StoreError, Tree};

use crate::error::ServiceError;

/// `klappe_store::load` with a conflicted `drehbuch.fountain` or `szenen.toml` refused as
/// a `StoreError::Conflict`: the markers would otherwise read as action and match against
/// rubbish.
pub(crate) fn load_project(dir: &Path) -> Result<Loaded, ServiceError> {
    refuse_conflict(dir, klappe_store::load(dir)?)
}

/// Like `load_project`, for the files of `tree` instead of the folder's own (an outside
/// change laid over the session's own).
pub(crate) fn load_overlay(dir: &Path, tree: &Tree) -> Result<Loaded, ServiceError> {
    refuse_conflict(dir, klappe_store::load_tree(dir, tree)?)
}

fn refuse_conflict(dir: &Path, loaded: Loaded) -> Result<Loaded, ServiceError> {
    let conflict = loaded
        .script
        .as_deref()
        .and_then(klappe_script::conflict_line);
    if let Some(line) = conflict {
        return Err(ServiceError::Store(StoreError::Conflict {
            path: klappe_store::script_path(dir),
            line,
        }));
    }

    Ok(loaded)
}
