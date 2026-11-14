use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use klappe_model::Project;
use klappe_script::Script;
use klappe_store::ImageStamp;

use crate::error::{write_failed, ServiceError};
use crate::plan_edit::Snapping;

mod export;
mod file;
mod layers;
mod objects;
mod pick;
mod playback;
mod scene;
mod shots;
mod timeline;
mod undo;
mod views;

/// Groups of the plan the timeline's layer switches show or hide.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Layer {
    Set,
    Figures,
    Cameras,
    Lights,
    Notes,
    Drawings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Shown,
    Hidden,
}

/// A locked layer can be seen but not picked or changed (LH-15).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lock {
    Unlocked,
    Locked,
}

/// Which way a jump between marks goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Back,
    On,
}

/// What playback does at the end of the shot (LH-60).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Looping {
    Off,
    /// Repeats the selected shot, or the whole scene when no shot is selected.
    Shot,
}

/// Whether playback has more to play after a tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Playing {
    Running,
    Ended,
}

/// What `Session::autosave` did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Autosave {
    /// The backup now holds the current state.
    Written,
    /// Nothing changed since the last save or autosave.
    Clean,
    /// An unanswered backup from an earlier run is on disk; writing would destroy it.
    /// `restore_backup` or `discard_backup` first.
    Held,
}

/// The backup of unsaved changes that `Session::open` found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackupState {
    None,
    /// Differs from the saved project: offer `restore_backup` and `discard_backup`.
    Found,
    /// A backup exists but cannot be read (the message says why): offer `discard_backup`.
    /// `open` must not fail on it, or the project could not be opened at all.
    Unreadable(String),
}

/// The pixel size of an image file and the stamp it was read at.
type StampedSize = (ImageStamp, (u32, u32));

/// One open project and what the user looks at.
#[derive(Debug)]
pub struct Session {
    dir: PathBuf,
    project: Project,
    script_text: Option<String>,
    script: Script,
    scene: usize,
    plan: usize,
    time_s: f64,
    selected_shot: Option<String>,
    hidden_layers: HashSet<Layer>,
    locked_layers: HashSet<Layer>,
    hidden_tracks: HashSet<String>,
    snapping: Snapping,
    undo: Vec<Project>,
    redo: Vec<Project>,
    dirty: bool,
    /// Pixel size of the image files read so far: every redraw asks, the file header is read
    /// once. `bilder/` is the user's, so a file may change under its name: an entry counts
    /// only while the file's stamp (length, modification time) is the one it was read at.
    /// Unreadable files are not kept and are tried again.
    image_px: RefCell<HashMap<String, StampedSize>>,
    /// Changed since the last save or autosave: what `autosave` writes.
    unbacked: bool,
    /// The project (not only the screenplay) changed since the last save.
    project_changed: bool,
    backup: BackupState,
}

/// `dir` may be missing or an empty folder; anything else would mix the project with it.
/// Also for the example (`copy_example`).
pub(crate) fn check_empty(dir: &Path) -> Result<(), ServiceError> {
    if !klappe_store::is_empty_dir(dir).map_err(write_failed)? {
        return Err(ServiceError::NotEmpty(dir.to_owned()));
    }
    Ok(())
}
