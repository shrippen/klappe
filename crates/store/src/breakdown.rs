//! The breakdown (DB-60, D-22): `breakdown/elemente.toml` (the elements) and, per scene whose
//! sheet holds something, `breakdown/<scene-id>.toml` (the scene sheet). Both are optional.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use klappe_model::{Element, Project, Scene, Sheet};
use serde::{Deserialize, Serialize};

use crate::files::{self, Dir, Kind};
use crate::{OptionalFile, StoreError, BREAKDOWN_DIR, ELEMENTS_STEM};

/// `breakdown/elemente.toml`.
#[derive(Serialize, Deserialize)]
struct Elements {
    #[serde(default)]
    element: Vec<Element>,
}

fn dir_of(dir: &Path) -> Dir {
    Dir::new(dir.join(BREAKDOWN_DIR), Kind::Toml)
}

/// The elements, and the sheets onto `scenes`. A sheet without a scene is left out; ids it
/// names that no element or character has are left in: the services ignore them.
pub(crate) fn load(dir: &Path, scenes: &mut [Scene]) -> Result<Vec<Element>, StoreError> {
    let folder = dir_of(dir);
    let mut elements = Vec::new();
    let mut sheets: HashMap<String, Sheet> = HashMap::new();

    for (stem, path) in folder.list()? {
        if stem == ELEMENTS_STEM {
            let file: Elements = folder.read(&path)?;
            check(&file.element).map_err(|message| StoreError::Parse { path, message })?;
            elements = file.element;
            continue;
        }
        sheets.insert(stem, folder.read(&path)?);
    }

    for scene in scenes {
        if let Some(sheet) = sheets.remove(&scene.id) {
            scene.sheet = sheet;
        }
    }

    Ok(elements)
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

/// The names of the files in `breakdown/` a save keeps; it removes the others.
pub(crate) fn kept(project: &Project) -> HashSet<&str> {
    let sheets = project.scenes.iter().filter(|s| !s.sheet.is_empty());

    sheets
        .map(|s| s.id.as_str())
        .chain([ELEMENTS_STEM])
        .collect()
}

/// The files of the breakdown: `elemente.toml` (`None` when there are no elements) and a sheet
/// file for each scene whose sheet is not empty.
pub(crate) fn encode(dir: &Path, project: &Project) -> Result<Vec<OptionalFile>, StoreError> {
    let folder = dir_of(dir);
    let mut files = Vec::new();

    let path = dir
        .join(BREAKDOWN_DIR)
        .join(format!("{ELEMENTS_STEM}.toml"));
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

    for scene in project.scenes.iter().filter(|s| !s.sheet.is_empty()) {
        // The elements' file name is taken.
        if scene.id == ELEMENTS_STEM {
            return Err(StoreError::BadId {
                id: scene.id.clone(),
            });
        }
        let (path, bytes): (PathBuf, Vec<u8>) = folder.encode(&scene.id, &scene.sheet)?;
        files.push((path, Some(bytes)));
    }

    Ok(files)
}
