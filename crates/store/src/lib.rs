//! Reads and writes a Klappe project folder (FORMAT.md).
//!
//! ```text
//! save(dir, project, script) ──► store ──► projekt.toml, figuren.toml, drehorte/, szenen/
//! load(dir)                  ◄──         drehbuch.fountain
//! ```
//!
//! Depends on the model only; knows nothing of Stellprobe or the script parser.

mod error;
mod files;

use std::collections::HashSet;
use std::path::Path;

use klappe_model::{Character, Location, Project, Scene, FORMAT_VERSION};
use serde::{Deserialize, Serialize};

pub use error::StoreError;
use files::{Dir, Kind};

const PROJECT_FILE: &str = "projekt.toml";
const CHARACTERS_FILE: &str = "figuren.toml";
const SCRIPT_FILE: &str = "drehbuch.fountain";
const LOCATIONS_DIR: &str = "drehorte";
const SCENES_DIR: &str = "szenen";
const TOML_EXT: &str = "toml";
const JSON_EXT: &str = "json";

/// What a project folder holds.
#[derive(Debug, Clone, PartialEq)]
pub struct Loaded {
    pub project: Project,
    /// The screenplay as Fountain text; `None` if the folder has no `drehbuch.fountain`.
    pub script: Option<String>,
}

/// `projekt.toml`.
#[derive(Serialize, Deserialize)]
struct Header {
    format_version: u32,
    title: String,
}

/// `figuren.toml`.
#[derive(Serialize, Deserialize)]
struct Characters {
    #[serde(default)]
    figur: Vec<Character>,
}

/// Reads the project folder `dir`. Locations and scenes come back sorted by id.
pub fn load(dir: &Path) -> Result<Loaded, StoreError> {
    let header: Header = files::read_toml(&dir.join(PROJECT_FILE))?;
    if header.format_version > FORMAT_VERSION {
        return Err(StoreError::NewerFormat {
            path: dir.join(PROJECT_FILE),
            found: header.format_version,
            supported: FORMAT_VERSION,
        });
    }

    let characters_path = dir.join(CHARACTERS_FILE);
    let characters: Characters = match files::exists(&characters_path)? {
        true => files::read_toml(&characters_path)?,
        false => Characters { figur: Vec::new() },
    };

    let locations = load_all::<Location>(&dir.join(LOCATIONS_DIR), Kind::Toml, |l| &l.id)?;
    let scenes = load_all::<Scene>(&dir.join(SCENES_DIR), Kind::Json, |s| &s.id)?;
    let script = files::read_optional(&dir.join(SCRIPT_FILE))?;

    Ok(Loaded {
        project: Project {
            title: header.title,
            characters: characters.figur,
            locations,
            scenes,
        },
        script,
    })
}

/// Writes `project` into `dir`, creating it if needed. Each file is replaced atomically;
/// location and scene files of other ids are removed. `script: None` leaves an existing
/// `drehbuch.fountain` untouched.
pub fn save(dir: &Path, project: &Project, script: Option<&str>) -> Result<(), StoreError> {
    // Ids become file names: check all before the first write.
    check_ids(project.locations.iter().map(|l| l.id.as_str()))?;
    check_ids(project.scenes.iter().map(|s| s.id.as_str()))?;

    let header = Header {
        format_version: FORMAT_VERSION,
        title: project.title.clone(),
    };
    files::write_toml(&dir.join(PROJECT_FILE), &header)?;

    let characters = Characters {
        figur: project.characters.clone(),
    };
    files::write_toml(&dir.join(CHARACTERS_FILE), &characters)?;

    let locations = Dir::new(dir.join(LOCATIONS_DIR), Kind::Toml);
    let mut keep = HashSet::new();
    for location in &project.locations {
        locations.write(&location.id, location)?;
        keep.insert(location.id.as_str());
    }
    locations.remove_except(&keep)?;

    let scenes = Dir::new(dir.join(SCENES_DIR), Kind::Json);
    let mut keep = HashSet::new();
    for scene in &project.scenes {
        scenes.write(&scene.id, scene)?;
        keep.insert(scene.id.as_str());
    }
    scenes.remove_except(&keep)?;

    if let Some(text) = script {
        files::write_atomic(&dir.join(SCRIPT_FILE), text.as_bytes())?;
    }

    Ok(())
}

/// Reads every file of a kind in `dir`, sorted by file name; the id inside must match it.
fn load_all<T>(dir: &Path, kind: Kind, id_of: impl Fn(&T) -> &String) -> Result<Vec<T>, StoreError>
where
    T: for<'de> Deserialize<'de>,
{
    let dir = Dir::new(dir.to_path_buf(), kind);
    let mut items = Vec::new();

    for (stem, path) in dir.list()? {
        let item: T = dir.read(&path)?;
        let found = id_of(&item);
        if *found != stem {
            return Err(StoreError::IdMismatch {
                path,
                found: found.clone(),
            });
        }
        items.push(item);
    }

    Ok(items)
}

/// Rejects ids that are unsafe as file names (path traversal) or used twice.
fn check_ids<'a>(ids: impl Iterator<Item = &'a str>) -> Result<(), StoreError> {
    let mut seen = HashSet::new();

    for id in ids {
        if !files::safe_name(id) {
            return Err(StoreError::BadId { id: id.to_owned() });
        }
        if !seen.insert(id) {
            return Err(StoreError::DuplicateId { id: id.to_owned() });
        }
    }

    Ok(())
}
