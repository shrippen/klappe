//! Checks of the sun's data on load (LH-52): a place off the globe or a date that does not exist
//! is refused, as an edit would refuse it.

use std::path::Path;

use klappe_model::{Location, Scene};

use crate::error::StoreError;

/// A location's place is on the globe.
pub(crate) fn check_sites(path: &Path, locations: &[Location]) -> Result<(), StoreError> {
    let bad = locations
        .iter()
        .find(|l| l.site.is_some_and(|site| !site.is_valid()));
    let Some(location) = bad else {
        return Ok(());
    };

    Err(StoreError::Parse {
        path: path.to_path_buf(),
        message: format!(
            "{}: the place of the location is not on the globe",
            location.id
        ),
    })
}

/// A scene's date and time exist.
pub(crate) fn check_suns(path: &Path, scenes: &[Scene]) -> Result<(), StoreError> {
    let bad = scenes
        .iter()
        .find(|s| s.sun.is_some_and(|time| !time.is_valid()));
    let Some(scene) = bad else {
        return Ok(());
    };

    Err(StoreError::Parse {
        path: path.to_path_buf(),
        message: format!("{}: the date or time of the scene does not exist", scene.id),
    })
}
