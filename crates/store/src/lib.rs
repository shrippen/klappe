//! Reads and writes a Klappe project folder (FORMAT.md).
//!
//! ```text
//! save(dir, project, script) ──► store ──► projekt.toml, figuren.toml, drehorte/, szenen/
//! load(dir)                  ◄──         drehbuch.fountain
//! add_image(dir, file)       ───►         bilder/ (the user's images; `save` never deletes them)
//! save_backup(dir, ..)       ──► store ──► .sicherung/ (the same files; load_backup, drop_backup)
//! load_library, save_library ◄──►          bibliothek.toml, or any library file to pass on
//!                            builtin_templates, vorlagen.toml (the project's own templates)
//! load_template_file, save_template_file ◄──► vorlagen.toml, or any template file to pass on
//! write_atomic, write_tree, project_mtime, ..  whole-folder file operations (tree.rs)
//! read_files, folder_size, free_sibling, move_to_trash      a folder as bytes, its size, a free name beside it, the system trash (folder.rs)
//! ```
//!
//! Depends on the model only; knows nothing of Stellprobe or the script parser.

mod camera;
mod error;
mod files;
mod folder;
mod images;
mod library;
mod light;
mod template;
mod tree;

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use klappe_model::{
    is_figure_size_m, is_floor_offset_m, is_image_opacity, is_object_height_m, is_object_size_m,
    is_opening_width_m, is_outline, Character, Location, ObjectShape, Opening, PaperSetup, Project,
    SavedView, Scene, FORMAT_VERSION, MAX_FIGURE_M, MAX_GROUP, MAX_OBJECT_M, MIN_FIGURE_M,
    MIN_GROUP, MIN_IMAGE_OPACITY, MIN_OBJECT_M, NOTE_MIN_WIDTH_M,
};
use serde::{Deserialize, Serialize};

pub use camera::builtin_camera_presets;
pub use error::StoreError;
use files::{Dir, Kind};
pub use folder::{folder_size, free_sibling, move_to_trash, read_files};
pub use images::{add_image, image_size, image_stamp, read_image, ImageStamp};
pub use library::{builtin_forms, builtin_library, load_library, save_library};
pub use light::builtin_lights;
pub use template::{builtin_templates, check_template, load_template_file, save_template_file};
pub use tree::{is_empty_dir, project_mtime, remove_prefixed, write_atomic, write_tree};

/// Whether `id` may name a location or scene file on every system (FORMAT.md, Ids).
pub fn is_safe_id(id: &str) -> bool {
    files::safe_name(id)
}

const PROJECT_FILE: &str = "projekt.toml";
const CHARACTERS_FILE: &str = "figuren.toml";
const LIBRARY_FILE: &str = "bibliothek.toml";
const TEMPLATES_FILE: &str = "vorlagen.toml";
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
    /// Left out while it is the default, so older projects save unchanged.
    #[serde(default, skip_serializing_if = "is_default_paper")]
    paper: PaperSetup,
    /// The saved views (LH-119), each an `[[ansicht]]`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    ansicht: Vec<SavedView>,
}

fn is_default_paper(paper: &PaperSetup) -> bool {
    *paper == PaperSetup::default()
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

/// `projekt.toml` read and checked.
fn read_header(dir: &Path) -> Result<Header, StoreError> {
    let path = dir.join(PROJECT_FILE);
    let header: Header = files::read_toml(&path)?;
    check_views(&header.ansicht).map_err(|message| StoreError::Parse { path, message })?;

    Ok(header)
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
    let header = read_header(dir)?;

    let characters_path = dir.join(CHARACTERS_FILE);
    let characters: Characters = match files::exists(&characters_path)? {
        true => files::read_toml(&characters_path)?,
        false => Characters { figur: Vec::new() },
    };
    check_characters(&characters.figur).map_err(|message| StoreError::Parse {
        path: characters_path.clone(),
        message,
    })?;

    let library_path = dir.join(LIBRARY_FILE);
    let library = match files::exists(&library_path)? {
        true => load_library(&library_path)?,
        false => Vec::new(),
    };

    let templates = template::load_templates(&dir.join(TEMPLATES_FILE))?;

    let locations = load_all::<Location>(&dir.join(LOCATIONS_DIR), Kind::Toml, |l| &l.id)?;
    check_sets(&dir.join(LOCATIONS_DIR), &locations)?;
    let scenes = load_scenes(&dir.join(SCENES_DIR))?;
    let script = files::read_optional(&dir.join(SCRIPT_FILE))?;

    Ok(Loaded {
        project: Project {
            title: header.title,
            paper: header.paper,
            views: header.ansicht,
            characters: characters.figur,
            locations,
            scenes,
            library,
            templates,
        },
        script,
    })
}

/// Writes `project` into `dir`, creating it if needed. Each file is replaced atomically;
/// location and scene files of other ids are removed, and `bibliothek.toml` and `vorlagen.toml`
/// when the project has no own objects or templates. `script: None` leaves an existing
/// `drehbuch.fountain` untouched.
pub fn save(dir: &Path, project: &Project, script: Option<&str>) -> Result<(), StoreError> {
    // Ids become file names: check all before the first write.
    check_ids(project.locations.iter().map(|l| l.id.as_str()))?;
    check_ids(project.scenes.iter().map(|s| s.id.as_str()))?;

    // Encode everything first: a document that cannot be encoded fails before any file changes.
    let mut writes = Vec::new();

    let header = Header {
        format_version: FORMAT_VERSION,
        title: project.title.clone(),
        paper: project.paper,
        ansicht: project.views.clone(),
    };
    let path = dir.join(PROJECT_FILE);
    writes.push((path.clone(), files::encode_toml(&path, &header)?));

    let characters = Characters {
        figur: project.characters.clone(),
    };
    let path = dir.join(CHARACTERS_FILE);
    writes.push((path.clone(), files::encode_toml(&path, &characters)?));

    let optional = optional_files(dir, project)?;
    writes.extend(optional.iter().filter_map(|(path, bytes)| {
        let bytes = bytes.clone()?;
        Some((path.clone(), bytes))
    }));

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

    // None left: the file goes, so the folder shows what the project holds.
    for (path, _) in optional.iter().filter(|(_, bytes)| bytes.is_none()) {
        files::remove_file(path)?;
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

/// A file's path with its content, or `None` when there is to be no file.
type OptionalFile = (PathBuf, Option<Vec<u8>>);

/// The files that exist only while there are entries: own objects and own templates, each
/// with its content, or `None` when there are none.
fn optional_files(dir: &Path, project: &Project) -> Result<Vec<OptionalFile>, StoreError> {
    let library_path = dir.join(LIBRARY_FILE);
    let templates_path = dir.join(TEMPLATES_FILE);
    let library = match project.library.is_empty() {
        true => None,
        false => Some(library::encode(&library_path, &project.library)?),
    };
    let templates = match project.templates.is_empty() {
        true => None,
        false => Some(template::encode(&templates_path, &project.templates)?),
    };

    Ok(vec![(library_path, library), (templates_path, templates)])
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

/// Older files may give a reflector or flag the power of the lamp it was switched from.
fn clear_passive_power(scenes: &mut [Scene]) {
    let lights = scenes
        .iter_mut()
        .flat_map(|s| &mut s.plans)
        .flat_map(|p| &mut p.lights);
    for light in lights.filter(|l| !l.kind.gives_light()) {
        light.power_w = 0.0;
    }
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

/// A group has between `MIN_GROUP` and `MAX_GROUP` people (LH-23); a size is between
/// `MIN_FIGURE_M` and `MAX_FIGURE_M` (LH-25).
fn check_characters(characters: &[Character]) -> Result<(), String> {
    for character in characters {
        if let Some(count) = character.group {
            if !(MIN_GROUP..=MAX_GROUP).contains(&count) {
                return Err(format!(
                    "{}: group of {count} is not usable ({MIN_GROUP} to {MAX_GROUP})",
                    character.id
                ));
            }
        }
        if let Some(size_m) = character.size_m {
            if !is_figure_size_m(size_m) {
                return Err(format!(
                    "{}: size of {size_m} m is not usable ({MIN_FIGURE_M} to {MAX_FIGURE_M})",
                    character.id
                ));
            }
        }
    }

    Ok(())
}

/// Saved views (LH-119): an id and a name each, and no id twice.
fn check_views(views: &[SavedView]) -> Result<(), String> {
    for (index, view) in views.iter().enumerate() {
        if view.id.trim().is_empty() || view.name.trim().is_empty() {
            return Err("a saved view needs an id and a name".to_owned());
        }
        if views[..index]
            .iter()
            .any(|other| other.id.to_lowercase() == view.id.to_lowercase())
        {
            return Err(format!("{}: saved view twice", view.id));
        }
    }

    Ok(())
}

/// The sets of `locations`: their openings and heights.
fn check_sets(path: &Path, locations: &[Location]) -> Result<(), StoreError> {
    check_openings(path, locations)?;
    check_outlines(path, locations)?;
    check_notes(path, locations)?;
    check_images(path, locations)?;

    check_heights(path, locations)
}

/// A note's card is at least as wide as a drag of its grip allows (LH-16), and finite: an
/// endless width could never be saved again.
fn check_notes(path: &Path, locations: &[Location]) -> Result<(), StoreError> {
    for location in locations {
        for note in &location.set.notes {
            if !note.id.starts_with(NOTE_ID_PREFIX) {
                return Err(StoreError::Parse {
                    path: path.to_path_buf(),
                    message: format!(
                        "{}: note id {} does not start with {NOTE_ID_PREFIX}",
                        location.id, note.id
                    ),
                });
            }
            let bound = note.bound.as_deref();
            if !bound.is_none_or(|b| b.strip_prefix("object:").is_some_and(|id| !id.is_empty())) {
                return Err(StoreError::Parse {
                    path: path.to_path_buf(),
                    message: format!(
                        "{}: note {} is bound to {bound:?}, not an object",
                        location.id, note.id
                    ),
                });
            }
            if note.width_m.is_finite() && note.width_m >= NOTE_MIN_WIDTH_M {
                continue;
            }

            return Err(StoreError::Parse {
                path: path.to_path_buf(),
                message: format!(
                    "{}: note {} is {} m wide, not a finite width of at least {NOTE_MIN_WIDTH_M} m",
                    location.id, note.id, note.width_m
                ),
            });
        }
    }

    Ok(())
}

/// The scenes of `path`, their labels checked.
fn load_scenes(path: &Path) -> Result<Vec<Scene>, StoreError> {
    let mut scenes = load_all::<Scene>(path, Kind::Json, |s| &s.id)?;

    check_labels(path, &scenes)?;
    clear_passive_power(&mut scenes);

    Ok(scenes)
}

/// Notes of a location are `n<k>` and labels of a plan `t<k>`: `Source::Note` names either by id,
/// and a scene may move to another location, so the two must never be equal.
const NOTE_ID_PREFIX: char = 'n';
const LABEL_ID_PREFIX: char = 't';

/// A plan's labels (LH-118) are as wide as a note's card, have ids of their own in the plan, and
/// are bound to a figure or a mark (`figure:<id>`, `mark:<id>:<index>`); an object's label is a
/// note of the set.
fn check_labels(path: &Path, scenes: &[Scene]) -> Result<(), StoreError> {
    let parse = |message: String| StoreError::Parse {
        path: path.to_path_buf(),
        message,
    };

    for scene in scenes {
        for plan in &scene.plans {
            let at = format!("{}/{}", scene.id, plan.id);
            let mut seen = HashSet::new();

            for label in &plan.labels {
                if !label.width_m.is_finite() || label.width_m < NOTE_MIN_WIDTH_M {
                    return Err(parse(format!(
                        "{at}: label {} is {} m wide, not a finite width of at least {NOTE_MIN_WIDTH_M} m",
                        label.id, label.width_m
                    )));
                }
                if !seen.insert(label.id.as_str()) {
                    return Err(parse(format!("{at}: label id {} is used twice", label.id)));
                }
                if !label.id.starts_with(LABEL_ID_PREFIX) {
                    return Err(parse(format!(
                        "{at}: label id {} does not start with {LABEL_ID_PREFIX}",
                        label.id
                    )));
                }
                if !is_plan_binding(label.bound.as_deref()) {
                    return Err(parse(format!(
                        "{at}: label {} is bound to {:?}, not a figure or a mark",
                        label.id, label.bound
                    )));
                }
            }
        }
    }

    Ok(())
}

/// Whether `bound` is what a plan's label has: `figure:<id>` or `mark:<id>:<index>`; unbound is
/// fine too.
fn is_plan_binding(bound: Option<&str>) -> bool {
    let Some(bound) = bound else {
        return true;
    };

    match bound.split_once(':') {
        Some(("figure", id)) => !id.is_empty(),
        Some(("mark", rest)) => rest
            .rsplit_once(':')
            .is_some_and(|(id, index)| !id.is_empty() && index.parse::<usize>().is_ok()),
        _ => false,
    }
}

/// An image (LH-18) has a width a drag of its grip allows and the plan can draw: finite, within
/// the object limits; and an opacity the card allows.
fn check_images(path: &Path, locations: &[Location]) -> Result<(), StoreError> {
    for location in locations {
        for image in &location.set.images {
            let bad = |message: String| StoreError::Parse {
                path: path.to_path_buf(),
                message: format!("{}: image {} {message}", location.id, image.id),
            };

            if !is_object_size_m(image.width_m) {
                return Err(bad(format!(
                    "is {} m wide, not a width between {MIN_OBJECT_M} m and {MAX_OBJECT_M} m",
                    image.width_m
                )));
            }
            if !is_image_opacity(image.opacity) {
                return Err(bad(format!(
                    "has opacity {}, not one between {MIN_IMAGE_OPACITY} and 1",
                    image.opacity
                )));
            }
        }
    }

    Ok(())
}

/// A free form (LH-92) has an outline an edit would accept: three or more finite points
/// round an area.
fn check_outlines(path: &Path, locations: &[Location]) -> Result<(), StoreError> {
    for location in locations {
        for object in &location.set.objects {
            let ObjectShape::Polygon { points } = &object.shape else {
                continue;
            };
            if is_outline(points) {
                continue;
            }

            return Err(StoreError::Parse {
                path: path.to_path_buf(),
                message: format!(
                    "{}: free form {} has {} points that enclose no usable area",
                    location.id,
                    object.id,
                    points.len()
                ),
            });
        }
    }

    Ok(())
}

/// An opening is a gap with a width, somewhere along its wall; `path` names the folder in the
/// message.
fn check_openings(path: &Path, locations: &[Location]) -> Result<(), StoreError> {
    for location in locations {
        let openings = location.set.walls.iter().flat_map(|w| &w.openings);

        for opening in openings {
            if is_opening_width_m(opening.width_m) && opening.at_m.is_finite() {
                continue;
            }

            return Err(StoreError::Parse {
                path: path.to_path_buf(),
                message: format!(
                    "{}: opening of {} m at {} m is not usable",
                    location.id, opening.width_m, opening.at_m
                ),
            });
        }
    }

    Ok(())
}

/// Heights an edit would refuse (LH-90, LH-91): a room, wall or head is not under the least height; a sill
/// or underside starts at the floor or above; a sill lies below its head.
fn check_heights(path: &Path, locations: &[Location]) -> Result<(), StoreError> {
    for location in locations {
        let set = &location.set;
        let walls = set.walls.iter().map(|w| w.height_m);
        let tops = std::iter::once(Some(set.wall_height_m)).chain(walls);
        let tops_ok = tops.flatten().all(is_object_height_m);
        let bases_ok = set
            .objects
            .iter()
            .filter_map(|o| o.base_m)
            .all(is_floor_offset_m);
        let openings_ok = set
            .walls
            .iter()
            .flat_map(|w| &w.openings)
            .all(opening_heights_ok);
        if tops_ok && bases_ok && openings_ok {
            continue;
        }

        return Err(StoreError::Parse {
            path: path.to_path_buf(),
            message: format!("{}: a height of the set is not usable", location.id),
        });
    }

    Ok(())
}

fn opening_heights_ok(opening: &Opening) -> bool {
    let (sill_m, head_m) = opening.sill_head_m();

    is_floor_offset_m(sill_m) && is_object_height_m(head_m) && sill_m < head_m
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
