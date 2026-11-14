//! Reads and writes a Klappe project folder (FORMAT.md).
//!
//! ```text
//! save(dir, project, script) ──► store ──► projekt.toml, figuren.toml, drehorte/, szenen/
//! load(dir)                  ◄──         drehbuch.fountain
//! add_image(dir, file)       ───►         bilder/ (the user's images; `save` never deletes them)
//! save_backup(dir, ..)       ──► store ──► .sicherung/ (the same files; load_backup, drop_backup)
//! load_library, save_library ◄──►          bibliothek.toml, or any library file to pass on
//! ```
//!
//! Depends on the model only; knows nothing of Stellprobe or the script parser.

mod camera;
mod error;
mod files;
mod images;
mod library;

use std::collections::HashSet;
use std::path::Path;

use klappe_model::{Character, Location, Project, Scene, FORMAT_VERSION, MAX_GROUP, MIN_GROUP};
use serde::{Deserialize, Serialize};

pub use camera::builtin_camera_presets;
pub use error::StoreError;
use files::{Dir, Kind};
pub use images::{add_image, image_size, image_stamp, read_image, ImageStamp};
pub use library::{builtin_library, load_library, save_library};

/// Whether `id` may name a location or scene file on every system (FORMAT.md, Ids).
pub fn is_safe_id(id: &str) -> bool {
    files::safe_name(id)
}

const PROJECT_FILE: &str = "projekt.toml";
const CHARACTERS_FILE: &str = "figuren.toml";
const LIBRARY_FILE: &str = "bibliothek.toml";
const SCRIPT_FILE: &str = "drehbuch.fountain";
const LOCATIONS_DIR: &str = "drehorte";
const SCENES_DIR: &str = "szenen";
const IMAGES_DIR: &str = "bilder";
/// Folder of the unsaved-changes backup inside a project folder (FORMAT.md).
const BACKUP_DIR: &str = ".sicherung";
/// Where a new backup is built; it replaces `BACKUP_DIR` as a whole.
const BACKUP_NEW_DIR: &str = ".sicherung.neu";
/// Written last into `BACKUP_NEW_DIR`: only then the folder is complete.
const BACKUP_DONE_FILE: &str = "fertig";
/// The old backup while a finished `BACKUP_NEW_DIR` takes its place.
const BACKUP_OLD_DIR: &str = ".sicherung.alt";
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

/// Only the version of `projekt.toml`: read first, so a newer file reports its version even
/// if its other fields changed.
#[derive(Deserialize)]
struct Version {
    format_version: u32,
}

/// `figuren.toml`.
#[derive(Serialize, Deserialize)]
struct Characters {
    #[serde(default)]
    figur: Vec<Character>,
}

/// Reads the project folder `dir`. Locations and scenes come back sorted by id.
pub fn load(dir: &Path) -> Result<Loaded, StoreError> {
    let version: Version = files::read_toml(&dir.join(PROJECT_FILE))?;
    if version.format_version > FORMAT_VERSION {
        return Err(StoreError::NewerFormat {
            path: dir.join(PROJECT_FILE),
            found: version.format_version,
            supported: FORMAT_VERSION,
        });
    }
    let header: Header = files::read_toml(&dir.join(PROJECT_FILE))?;

    let characters_path = dir.join(CHARACTERS_FILE);
    let characters: Characters = match files::exists(&characters_path)? {
        true => files::read_toml(&characters_path)?,
        false => Characters { figur: Vec::new() },
    };
    check_groups(&characters.figur).map_err(|message| StoreError::Parse {
        path: characters_path.clone(),
        message,
    })?;

    let library_path = dir.join(LIBRARY_FILE);
    let library = match files::exists(&library_path)? {
        true => load_library(&library_path)?,
        false => Vec::new(),
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
            library,
        },
        script,
    })
}

/// Writes `project` into `dir`, creating it if needed. Each file is replaced atomically;
/// location and scene files of other ids are removed, and `bibliothek.toml` when the project
/// has no own objects. `script: None` leaves an existing `drehbuch.fountain` untouched.
pub fn save(dir: &Path, project: &Project, script: Option<&str>) -> Result<(), StoreError> {
    // Ids become file names: check all before the first write.
    check_ids(project.locations.iter().map(|l| l.id.as_str()))?;
    check_ids(project.scenes.iter().map(|s| s.id.as_str()))?;

    // Encode everything first: a document that cannot be encoded fails before any file changes.
    let mut writes = Vec::new();

    let header = Header {
        format_version: FORMAT_VERSION,
        title: project.title.clone(),
    };
    let path = dir.join(PROJECT_FILE);
    writes.push((path.clone(), files::encode_toml(&path, &header)?));

    let characters = Characters {
        figur: project.characters.clone(),
    };
    let path = dir.join(CHARACTERS_FILE);
    writes.push((path.clone(), files::encode_toml(&path, &characters)?));

    let library_path = dir.join(LIBRARY_FILE);
    if !project.library.is_empty() {
        writes.push((
            library_path.clone(),
            library::encode(&library_path, &project.library)?,
        ));
    }

    let locations = Dir::new(dir.join(LOCATIONS_DIR), Kind::Toml);
    for location in &project.locations {
        writes.push(locations.encode(&location.id, location)?);
    }

    let scenes = Dir::new(dir.join(SCENES_DIR), Kind::Json);
    for scene in &project.scenes {
        writes.push(scenes.encode(&scene.id, scene)?);
    }

    for (path, bytes) in &writes {
        files::write_atomic(path, bytes)?;
    }

    // No own objects left: the file goes, so the folder shows what the project holds.
    if project.library.is_empty() {
        files::remove_file(&library_path)?;
    }

    let keep: HashSet<&str> = project.locations.iter().map(|l| l.id.as_str()).collect();
    locations.remove_except(&keep)?;
    let keep: HashSet<&str> = project.scenes.iter().map(|s| s.id.as_str()).collect();
    scenes.remove_except(&keep)?;

    if let Some(text) = script {
        files::write_atomic(&dir.join(SCRIPT_FILE), text.as_bytes())?;
    }

    Ok(())
}

/// Writes `drehbuch.fountain` in `dir` atomically and nothing else.
pub fn save_script(dir: &Path, script: &str) -> Result<(), StoreError> {
    files::write_atomic(&dir.join(SCRIPT_FILE), script.as_bytes())
}

/// Writes `project` as a backup into `dir/.sicherung`, in the project format. The project
/// files stay as they are, so the user alone decides what a save changes (LH-05).
///
/// A backup folder counts only with its marker `fertig`: it is written last and removed
/// first, so a crash while writing or deleting never leaves a readable half.
///
/// ```text
/// 1. promote a finished .sicherung.neu left by a crash (see 3.)
/// 2. build .sicherung.neu/, last file `fertig`   (crash: old .sicherung stays valid)
/// 3. promote: .sicherung -> .sicherung.alt       (crash: .sicherung.neu is finished)
///             .sicherung.neu -> .sicherung
///             remove .sicherung.alt
/// ```
pub fn save_backup(dir: &Path, project: &Project, script: Option<&str>) -> Result<(), StoreError> {
    let new = dir.join(BACKUP_NEW_DIR);

    // A finished leftover may be the only copy: keep it before anything else is deleted.
    promote(dir)?;
    // What is left has no marker: a crashed write.
    remove_backup(&new)?;
    save(&new, project, script)?;
    files::write_atomic(&new.join(BACKUP_DONE_FILE), b"")?;

    promote(dir)
}

/// Reads the backup of `dir`; `None` if there is none. Only a folder with the marker `fertig`
/// counts. A finished `.sicherung.neu` is newer than `.sicherung` and wins.
pub fn load_backup(dir: &Path) -> Result<Option<Loaded>, StoreError> {
    for name in [BACKUP_NEW_DIR, BACKUP_DIR] {
        let folder = dir.join(name);
        if files::exists(&folder.join(BACKUP_DONE_FILE))? {
            return load(&folder).map(Some);
        }
    }

    Ok(None)
}

/// Removes the backup of `dir` and any leftover of a write; no backup is not an error.
pub fn drop_backup(dir: &Path) -> Result<(), StoreError> {
    for name in [BACKUP_NEW_DIR, BACKUP_DIR, BACKUP_OLD_DIR] {
        remove_backup(&dir.join(name))?;
    }

    Ok(())
}

/// Makes a finished `.sicherung.neu` the backup `.sicherung`; nothing if there is none.
fn promote(dir: &Path) -> Result<(), StoreError> {
    let (new, current, old) = (
        dir.join(BACKUP_NEW_DIR),
        dir.join(BACKUP_DIR),
        dir.join(BACKUP_OLD_DIR),
    );

    remove_backup(&old)?;
    if !files::exists(&new.join(BACKUP_DONE_FILE))? {
        return Ok(());
    }

    if files::exists(&current)? {
        files::rename(&current, &old)?;
    }
    files::rename(&new, &current)?;

    remove_backup(&old)
}

/// Deletes a backup folder: the marker first (atomic, the folder is invalid from then on),
/// then the rest, so a crash in between leaves nothing that reads as a backup.
fn remove_backup(folder: &Path) -> Result<(), StoreError> {
    files::remove_file(&folder.join(BACKUP_DONE_FILE))?;
    files::remove_dir(folder)
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

/// A group has between `MIN_GROUP` and `MAX_GROUP` people.
fn check_groups(characters: &[Character]) -> Result<(), String> {
    for character in characters {
        let Some(count) = character.group else {
            continue;
        };
        if !(MIN_GROUP..=MAX_GROUP).contains(&count) {
            return Err(format!(
                "{}: group of {count} is not usable ({MIN_GROUP} to {MAX_GROUP})",
                character.id
            ));
        }
    }

    Ok(())
}

/// Rejects ids that are unsafe as file names (path traversal) or used twice.
fn check_ids<'a>(ids: impl Iterator<Item = &'a str>) -> Result<(), StoreError> {
    let mut seen: HashSet<String> = HashSet::new();

    for id in ids {
        if !files::safe_name(id) {
            return Err(StoreError::BadId { id: id.to_owned() });
        }
        // Case-insensitive file systems (macOS, Windows) would merge `A` and `a`.
        if !seen.insert(id.to_lowercase()) {
            return Err(StoreError::DuplicateId { id: id.to_owned() });
        }
    }

    Ok(())
}
