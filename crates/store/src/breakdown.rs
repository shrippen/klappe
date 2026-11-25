//! The breakdown (DB-60, D-22): `breakdown/elemente.toml` (the elements), `breakdown/looks.toml`
//! (the looks) and, per scene whose sheet holds something, `breakdown/szenen/<scene-id>.toml`
//! (the scene sheet). All are optional.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use klappe_model::{Element, Look, Project, Scene, Sheet};
use serde::{Deserialize, Serialize};

use crate::files::{self, Dir, Kind};
use crate::{OptionalFile, StoreError, ELEMENTS_FILE, LOOKS_FILE, SHEETS_DIR};

/// `breakdown/elemente.toml`.
#[derive(Serialize, Deserialize)]
struct Elements {
    #[serde(default)]
    element: Vec<Element>,
}

/// `breakdown/looks.toml`.
#[derive(Serialize, Deserialize)]
struct Looks {
    #[serde(default)]
    look: Vec<Look>,
}

fn dir_of(dir: &Path) -> Dir {
    Dir::new(dir.join(SHEETS_DIR), Kind::Toml)
}

/// The elements, the looks, and the sheets onto `scenes`. A sheet without a scene is left out;
/// ids it names that no element, character or look has are left in, so is a look of a
/// character that is gone: the services ignore them.
pub(crate) fn load(
    dir: &Path,
    scenes: &mut [Scene],
) -> Result<(Vec<Element>, Vec<Look>), StoreError> {
    let folder = dir_of(dir);
    let path = dir.join(ELEMENTS_FILE);
    let elements = match files::exists(&path)? {
        true => files::read_toml::<Elements>(&path)?.element,
        false => Vec::new(),
    };
    check(&elements).map_err(|message| StoreError::Parse { path, message })?;

    let path = dir.join(LOOKS_FILE);
    let looks = match files::exists(&path)? {
        true => files::read_toml::<Looks>(&path)?.look,
        false => Vec::new(),
    };
    check_looks(&looks).map_err(|message| StoreError::Parse { path, message })?;

    let mut sheets: HashMap<String, Sheet> = HashMap::new();
    for (stem, path) in folder.list()? {
        sheets.insert(stem, folder.read(&path)?);
    }

    for scene in scenes {
        if let Some(sheet) = sheets.remove(&scene.id) {
            scene.sheet = sheet;
        }
    }

    Ok((elements, looks))
}

/// Ids are not empty and used once.
fn check(elements: &[Element]) -> Result<(), String> {
    let mut seen = HashSet::new();

    for element in elements {
        if element.id.trim().is_empty() {
            return Err("an element has no id".into());
        }
        if !seen.insert(element.id.as_str()) {
            return Err(format!("{}: the id is used twice", element.id));
        }
    }

    Ok(())
}

/// Ids are not empty and used once.
fn check_looks(looks: &[Look]) -> Result<(), String> {
    let mut seen = HashSet::new();

    for look in looks {
        if look.id.trim().is_empty() {
            return Err("a look has no id".into());
        }
        if !seen.insert(look.id.as_str()) {
            return Err(format!("{}: the id is used twice", look.id));
        }
    }

    Ok(())
}

/// The sheet files a save keeps; it removes the others.
pub(crate) fn kept(project: &Project) -> HashSet<&str> {
    let sheets = project.scenes.iter().filter(|s| !s.sheet.is_empty());

    sheets.map(|s| s.id.as_str()).collect()
}

/// The files of the breakdown: `elemente.toml` (`None` when there are no elements),
/// `looks.toml` (`None` when there are no looks) and a sheet file for each scene whose sheet is not empty.
pub(crate) fn encode(dir: &Path, project: &Project) -> Result<Vec<OptionalFile>, StoreError> {
    let folder = dir_of(dir);
    let mut files = Vec::new();

    let path = dir.join(ELEMENTS_FILE);
    let elements = match project.elements.is_empty() {
        true => None,
        false => {
            check(&project.elements).map_err(|message| StoreError::Parse {
                path: path.clone(),
                message,
            })?;
            let file = Elements {
                element: project.elements.clone(),
            };
            Some(files::encode_toml(&path, &file)?)
        }
    };
    files.push((path, elements));

    let path = dir.join(LOOKS_FILE);
    let looks = match project.looks.is_empty() {
        true => None,
        false => {
            check_looks(&project.looks).map_err(|message| StoreError::Parse {
                path: path.clone(),
                message,
            })?;
            let file = Looks {
                look: project.looks.clone(),
            };
            Some(files::encode_toml(&path, &file)?)
        }
    };
    files.push((path, looks));

    for scene in project.scenes.iter().filter(|s| !s.sheet.is_empty()) {
        let (path, bytes): (PathBuf, Vec<u8>) = folder.encode(&scene.id, &scene.sheet)?;
        files.push((path, Some(bytes)));
    }

    Ok(files)
}
