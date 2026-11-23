use std::collections::HashSet;
use std::path::{Path, PathBuf};

use klappe_model::Project;
use klappe_script::Script;
use klappe_stellprobe::{Cut, Orbit};

use crate::error::{write_failed, ServiceError};
use crate::field::Field;
use crate::plan_edit::{Grid, Snapping};
use plan_view::PlanView;
use undo::Snapshot;

mod blank;
mod book;
mod breakdown;
mod export;
mod fade;
mod figures;
mod file;
mod free_view;
mod layers;
mod numbering;
mod objects;
mod outside;
mod pages;
mod pick;
mod places;
mod plan_view;
mod playback;
mod questions;
mod retime;
mod saved;
mod scene;
mod scene_book;
mod script_pages;
mod shooting;
mod shots;
mod sketch;
mod speech;
mod templates;
mod timeline;
mod undo;
mod uses;
mod video;
mod views;

pub use book::Answer;
pub use export::ExportOptions;
pub use fade::Fade;
pub use figures::FigureAnswer;
pub use free_view::{Size, View, ViewStep};
pub use outside::{OnOutside, Outside, OutsideFile, Saved};
pub use pages::Pages;
pub use places::PlaceAnswer;
pub use questions::Answered;
pub use questions::{AnswerLine, BookQuestion, QuestionKey};
pub use retime::{Grab, TimeSnap};
pub use script_pages::Laid;
pub use speech::{Speech, SpokenLine};
pub use video::{frame_count as video_frame_count, VideoJob};

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

/// What a tick of playback did.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tick {
    pub playing: Playing,
    /// Set when a loop jumped back in this tick: the second the loop starts at (the start of
    /// the selected shot, else of the scene). The playhead is that plus the rest of the tick.
    pub looped_to_s: Option<f64>,
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
    plan_view: PlanView,
    locked_layers: HashSet<Layer>,
    snapping: Snapping,
    grid: Grid,
    undo: Vec<Snapshot>,
    redo: Vec<Snapshot>,
    dirty: bool,
    /// Changed since the last save or autosave: what `autosave` writes.
    unbacked: bool,
    /// The project (not only the screenplay) changed since the last save.
    project_changed: bool,
    /// Which state of the plan on view this is (`edits`).
    edits: u64,
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
    /// Each scene's length in the last `paginate`, by scene id: it stays the scene's while
    /// typing moves its heading to another line (DN-02: the last calculation until the next).
    lengths: std::collections::HashMap<String, String>,
    /// Entries the book could not place (`pending_scenes`).
    pending: Vec<crate::book::PendingScene>,
    /// Cues of the book no character answers to (`pending_figures`).
    pending_figures: Vec<crate::figures::PendingFigure>,
    /// The figures as last read from or written to `figuren.toml`: a character of the file
    /// not among them is new there, one the session lacks was deleted here.
    file_figures: crate::figure_file::Figures,
    /// The figure or place answer `undo` or `redo` just went over, for `save_answer`.
    disk_step: Option<(undo::AnswerDelta, crate::figure_file::Way)>,
    /// For each entry of the register its scene in the book and that scene's heading line
    /// (`refresh_scene_map`), kept so a refresh of the window does not align the book again.
    book_scene: Vec<Option<usize>>,
    heading_line: Vec<Option<usize>>,
    /// The scripted locations as `orte.toml` held them when last read or written (DB-51). Not
    /// part of the project: no backup, no save; `orte.toml` is their only source. An answer is
    /// an undo step that carries its entry, not the places.
    places: Vec<klappe_model::Place>,
    /// Place names of the headings no scripted location is called by (`pending_places`).
    pending_places: Vec<crate::places::PendingPlace>,
    /// The Sperrstand of the scene numbers (DB-33) as saved last; `None` without the file.
    /// Only a save changes it, so undo and the backup never carry it.
    lock: Option<Vec<klappe_model::LockedNumber>>,
    /// How many numbers two scenes carry that the Sperrstand cannot settle (`book_report`).
    twin_numbers: usize,
    /// The files of the folder as last read or written by this session (`outside.rs`).
    disk: klappe_store::Tree,
    /// What the session encodes of the same files at that moment, after load normalisation.
    own: klappe_store::Tree,
    /// Files changed on disk and in the session, which no save or check has settled yet.
    conflicts: Vec<OutsideFile>,
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
