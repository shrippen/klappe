use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use klappe_model::Project;
use klappe_script::Script;
use klappe_stellprobe::{Cut, Orbit};
use klappe_store::ImageStamp;

use crate::error::{write_failed, ServiceError};
use crate::field::Field;
use crate::plan_edit::{Grid, Snapping};
use undo::Snapshot;

mod export;
mod fade;
mod file;
mod free_view;
mod layers;
mod objects;
mod pages;
mod pick;
mod playback;
mod retime;
mod saved;
mod scene;
mod script_pages;
mod shots;
mod sketch;
mod templates;
mod timeline;
mod undo;
mod uses;
mod views;

pub use export::ExportOptions;
pub use fade::Fade;
pub use free_view::{Size, View};
pub use pages::Pages;
pub use retime::{Grab, TimeSnap};

/// Groups of the plan the timeline's layer switches show or hide.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Layer {
    Set,
    Figures,
    Cameras,
    Lights,
    Notes,
    Drawings,
    Sketches,
}

impl Layer {
    /// Every layer, in the order the layer list shows.
    pub const ALL: [Layer; 7] = [
        Layer::Set,
        Layer::Figures,
        Layer::Cameras,
        Layer::Lights,
        Layer::Notes,
        Layer::Drawings,
        Layer::Sketches,
    ];

    /// How QML and the catalogue (`layer.<name>`) call the layer.
    pub fn name(self) -> &'static str {
        match self {
            Layer::Set => "set",
            Layer::Figures => "figures",
            Layer::Cameras => "cameras",
            Layer::Lights => "lights",
            Layer::Notes => "notes",
            Layer::Drawings => "drawings",
            Layer::Sketches => "sketches",
        }
    }

    /// The layer `name` names; `None` for any other text.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|l| l.name() == name)
    }
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
    faded_layers: HashMap<Layer, Fade>,
    hidden_tracks: HashSet<String>,
    snapping: Snapping,
    grid: Grid,
    undo: Vec<Snapshot>,
    redo: Vec<Snapshot>,
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
    /// What `open` found odd in the project (an orphan scene file …).
    load_notes: Vec<klappe_store::LoadNote>,
    /// Which view of the set shows, and where the 3D view looks once it has been opened.
    view: View,
    orbit: Option<Orbit>,
    /// The 3D view's wall cut (LH-98) and the camera it looks through (LH-99), if any.
    cut: Cut,
    through: Option<Through>,
    /// The screenplay as last laid out on pages (DN-02); only `paginate` changes it.
    layout: Option<crate::script_pages::Layout>,
}

/// A camera the 3D view looks through, with the scene and plan it was chosen on: it ends
/// for good (`end_stale_look`) when another plan shows or the camera is gone.
#[derive(Debug)]
struct Through {
    scene: String,
    plan: String,
    camera: String,
}

/// A project title without the blanks at its ends; empty is refused.
pub(crate) fn check_title(title: &str) -> Result<&str, ServiceError> {
    let title = title.trim();
    if title.is_empty() {
        return Err(ServiceError::BadValue {
            field: Field::Title,
            value: String::new(),
        });
    }
    Ok(title)
}

/// `dir` may be missing or an empty folder; anything else would mix the project with it.
/// Also for the example (`copy_example`).
pub(crate) fn check_empty(dir: &Path) -> Result<(), ServiceError> {
    if !klappe_store::is_empty_dir(dir).map_err(write_failed)? {
        return Err(ServiceError::NotEmpty(dir.to_owned()));
    }
    Ok(())
}
