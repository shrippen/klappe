//! Opens a project folder through the store and refuses what the store cannot see: Git
//! conflict markers in the screenplay (DATENMODELL §4). The store checks `szenen.toml` itself.

use std::path::Path;

use klappe_store::{Loaded, StoreError};

use crate::error::ServiceError;

/// `klappe_store::load` with a conflicted `drehbuch.fountain` or `szenen.toml` refused as
/// a `StoreError::Conflict`: the markers would otherwise read as action and match against
/// rubbish.
pub(crate) fn load_project(dir: &Path) -> Result<Loaded, ServiceError> {
    let loaded = klappe_store::load(dir)?;

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
