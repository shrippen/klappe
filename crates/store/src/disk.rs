//! A project folder as bytes, for telling what changed outside the program and for writing only
//! what an edit changed.
//!
//! ```text
//! read_tree(dir)          folder ──► Tree   (name ──► bytes, the files a project is made of)
//! encode_tree(project)    project ─► Tree   (what `save` would write)
//! load_tree(dir, tree)    Tree ────► Loaded (through `.abgleich/`, a copy `load` can read)
//! write_changes(dir, ..)  Tree diff ► folder (changed files only, removals last)
//! ```

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;

use klappe_model::Project;

use crate::files::{self, io_err, Dir, Kind};
use crate::{
    encode_project, load, write_tree, Loaded, StoreError, CHARACTERS_FILE, ELEMENTS_FILE,
    LIBRARY_FILE, LOCATIONS_DIR, PROJECT_FILE, SCENES_DIR, SCENES_FILE, SCRIPT_FILE, SHEETS_DIR,
    TEMPLATES_FILE,
};

/// Where `load_tree` builds the folder it reads; hidden, so listings and `project_mtime`
/// skip it. A crash leaves it behind: the next call removes it first.
const OVERLAY_DIR: &str = ".abgleich";

/// Files by name (`/` below the project folder), with their bytes. A file that is not there
/// is not in the tree.
pub type Tree = BTreeMap<String, Vec<u8>>;

/// A file to write (`Some`) or to remove (`None`), by name.
type Change = (String, Option<Vec<u8>>);

/// What a file of the project is, for the texts that name it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileKind {
    Project,
    Figures,
    Library,
    Templates,
    Location,
    Register,
    Scene,
    Breakdown,
    Script,
}

/// What the file `name` of a `Tree` is; `None` for a name that is no file of the project.
pub fn kind_of(name: &str) -> Option<FileKind> {
    let kind = match name {
        PROJECT_FILE => FileKind::Project,
        CHARACTERS_FILE => FileKind::Figures,
        LIBRARY_FILE => FileKind::Library,
        TEMPLATES_FILE => FileKind::Templates,
        SCENES_FILE => FileKind::Register,
        SCRIPT_FILE => FileKind::Script,
        _ if is_in(name, LOCATIONS_DIR) => FileKind::Location,
        _ if is_in(name, SCENES_DIR) => FileKind::Scene,
        ELEMENTS_FILE => FileKind::Breakdown,
        _ if is_in(name, SHEETS_DIR) => FileKind::Breakdown,
        _ => return None,
    };

    Some(kind)
}

/// The name of the one file of the kind `kind`; `None` for a kind with many files (a folder).
pub const fn file_name(kind: FileKind) -> Option<&'static str> {
    match kind {
        FileKind::Project => Some(PROJECT_FILE),
        FileKind::Figures => Some(CHARACTERS_FILE),
        FileKind::Library => Some(LIBRARY_FILE),
        FileKind::Templates => Some(TEMPLATES_FILE),
        FileKind::Register => Some(SCENES_FILE),
        FileKind::Script => Some(SCRIPT_FILE),
        FileKind::Location | FileKind::Scene | FileKind::Breakdown => None,
    }
}

/// Whether `name` is `<folder>/<file>`.
fn is_in(name: &str, folder: &str) -> bool {
    name.strip_prefix(folder)
        .and_then(|rest| rest.strip_prefix('/'))
        .is_some_and(|file| !file.is_empty() && !file.contains('/'))
}

/// The files of the project folder `dir` as they are on disk: the single files, the location
/// files, the scene files and the breakdown files. Images, hidden folders (backups) and `orte.toml`, `nummern.toml`
/// are not part of it.
pub fn read_tree(dir: &Path) -> Result<Tree, StoreError> {
    let mut tree = Tree::new();

    for name in [
        PROJECT_FILE,
        CHARACTERS_FILE,
        LIBRARY_FILE,
        TEMPLATES_FILE,
        SCENES_FILE,
        SCRIPT_FILE,
        ELEMENTS_FILE,
    ] {
        read_into(&mut tree, dir, name)?;
    }
    for (folder, kind) in [
        (LOCATIONS_DIR, Kind::Toml),
        (SCENES_DIR, Kind::Json),
        (SHEETS_DIR, Kind::Toml),
    ] {
        for (_, path) in Dir::new(dir.join(folder), kind).list()? {
            let Some(file) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            read_into(&mut tree, dir, &format!("{folder}/{file}"))?;
        }
    }

    Ok(tree)
}

fn read_into(tree: &mut Tree, dir: &Path, name: &str) -> Result<(), StoreError> {
    let path = dir.join(name);
    match fs::read(&path) {
        Ok(bytes) => {
            tree.insert(name.to_owned(), bytes);
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
        Err(e) => return Err(io_err(&path, e)),
    }

    Ok(())
}

/// What `save` would write for `project`, as a tree. `script: None` leaves the screenplay out.
/// The same value always gives the same bytes.
pub fn encode_tree(project: &Project, script: Option<&str>) -> Result<Tree, StoreError> {
    let (writes, _) = encode_project(Path::new(""), project)?;

    let mut tree = Tree::new();
    for (path, bytes) in writes {
        let parts: Vec<_> = path
            .components()
            .map(|c| c.as_os_str().to_string_lossy())
            .collect();
        tree.insert(parts.join("/"), bytes);
    }
    if let Some(text) = script {
        tree.insert(SCRIPT_FILE.to_owned(), text.as_bytes().to_vec());
    }

    Ok(tree)
}

/// Reads `tree` as a project: it is written to `dir/.abgleich/` (beside the real files, which
/// stay as they are), read by `load`, and the folder removed again, also after an error.
pub fn load_tree(dir: &Path, tree: &Tree) -> Result<Loaded, StoreError> {
    let overlay = dir.join(OVERLAY_DIR);
    files::remove_dir(&overlay)?;

    let named: Vec<(&str, &[u8])> = tree
        .iter()
        .map(|(name, bytes)| (name.as_str(), bytes.as_slice()))
        .collect();
    let loaded = write_tree(&overlay, &named).and_then(|()| load(&overlay));
    files::remove_dir(&overlay)?;

    loaded
}

/// Writes and removes what `changes` names and nothing else: the scene files first and the
/// register last of the writes (a crash leaves a file the register lacks, which loads, never an
/// entry whose plans are gone), the removals after.
pub fn write_changes(dir: &Path, changes: &[Change]) -> Result<(), StoreError> {
    for (name, change) in ordered(changes) {
        let path = dir.join(name);
        match change {
            Some(bytes) => files::write_atomic(&path, bytes)?,
            None => files::remove_file(&path)?,
        }
    }

    Ok(())
}

/// `changes` in the order they are applied.
fn ordered(changes: &[Change]) -> Vec<(&str, Option<&[u8]>)> {
    let mut order: Vec<_> = changes
        .iter()
        .map(|(name, bytes)| (name.as_str(), bytes.as_deref()))
        .collect();
    let rank = |(name, bytes): &(&str, Option<&[u8]>)| match (bytes, kind_of(name)) {
        (None, _) => 3,
        (_, Some(FileKind::Scene)) => 0,
        (_, Some(FileKind::Register)) => 2,
        _ => 1,
    };
    order.sort_by_key(rank);

    order
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_names_round_trip() {
        let kinds = [
            FileKind::Project,
            FileKind::Figures,
            FileKind::Library,
            FileKind::Templates,
            FileKind::Register,
            FileKind::Script,
        ];

        for kind in kinds {
            let name = file_name(kind).expect("single file");
            assert_eq!(kind_of(name), Some(kind));
        }
    }

    fn change(name: &str, write: bool) -> Change {
        (name.to_owned(), write.then(|| b"x".to_vec()))
    }

    #[test]
    fn register_is_the_last_write_and_removals_follow() {
        let changes = [
            change("szenen.toml", true),
            change("szenen/old.json", false),
            change("drehbuch.fountain", true),
            change("szenen/a.json", true),
            change("projekt.toml", true),
        ];
        let names: Vec<_> = ordered(&changes).into_iter().map(|(n, _)| n).collect();

        assert_eq!(
            names,
            [
                "szenen/a.json",
                "drehbuch.fountain",
                "projekt.toml",
                "szenen.toml",
                "szenen/old.json"
            ]
        );
    }

    #[test]
    fn kinds_follow_the_names() {
        assert_eq!(kind_of("szenen/a.json"), Some(FileKind::Scene));
        assert_eq!(kind_of("drehorte/saal.toml"), Some(FileKind::Location));
        assert_eq!(kind_of("szenen.toml"), Some(FileKind::Register));
        assert_eq!(kind_of("orte.toml"), None);
        assert_eq!(kind_of("szenen/a/b.json"), None);
    }
}
