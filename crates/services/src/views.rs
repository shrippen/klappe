//! Plain data for the surfaces: what a row, a track or a script line shows.

use klappe_model::{LibraryItem, ObjectShape};
use serde::Serialize;

/// One row of the shot list (LH-71, LH-72).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ShotRow {
    pub id: String,
    pub number: String,
    pub camera: String,
    pub camera_label: String,
    /// Character id of the figure the shot's camera follows (LH-37), else empty.
    pub follows: String,
    /// Model name, e.g. `close-up`; the surface translates it.
    pub size: String,
    /// Model name, e.g. `birds-eye`; the surface translates it.
    pub angle: String,
    pub movement: String,
    /// The shot's lens, else its camera's.
    pub lens_mm: f64,
    /// Only the shot's own lens; `None` follows the camera.
    pub own_lens_mm: Option<f64>,
    pub from_s: f64,
    pub to_s: f64,
    pub description: String,
    pub props: String,
    pub gear: String,
    pub crew: String,
    pub note: String,
    /// The camera crosses the axis against the shot before (LH-41).
    pub axis_jump: bool,
    /// The camera setup (1-based): same camera, place and lens (LH-72).
    pub setup: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TrackKind {
    Cut,
    Figure,
    Camera,
    Light,
}

/// One lane of the timeline (design B: cut order, figures, cameras, lights).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Track {
    pub kind: TrackKind,
    /// Character, camera or light id; `cut` for the cut order.
    pub id: String,
    pub name: String,
    /// Kante label colour (`KanteStyle.labels`), `""` for text colour.
    pub label: String,
    pub shown: bool,
    pub segments: Vec<Segment>,
    pub markers: Vec<Marker>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Segment {
    pub from_s: f64,
    pub to_s: f64,
    pub text: String,
    /// Shot id for cut segments, else empty.
    pub key: String,
    pub selected: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Marker {
    pub at_s: f64,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LinkKind {
    Mark,
    Shot,
}

/// A link of a script line, shown as a chip beside it ("E · M2", "12.3").
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Chip {
    pub kind: LinkKind,
    pub text: String,
    /// Character id (marks) or shot id (shots).
    pub target: String,
    pub label: String,
    /// When it happens: the mark's arrival or the shot's start.
    pub at_s: f64,
}

/// One element of the current scene's script with its links.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScriptLine {
    /// Index within the scene, as `ScriptLink::element`.
    pub index: usize,
    /// `action`, `character`, `parenthetical`, `dialogue`, `transition` or `other`.
    pub kind: &'static str,
    pub text: String,
    pub chips: Vec<Chip>,
    /// The line the playback is at: the last linked line whose time has come.
    pub current: bool,
}

/// One entry of the object library, as the object tool lists it (LH-11, LH-12).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LibraryEntry {
    /// What a placed object's `kind` is, e.g. `table`.
    pub kind: String,
    /// The name of an own entry; empty for a built-in one, which the surface translates
    /// (`objectKind.<kind>`).
    pub label: String,
    pub width_m: f64,
    pub depth_m: f64,
    pub shape: ObjectShape,
    /// Height of a new object of this entry (LH-17); `None` when it has none.
    pub height_m: Option<f64>,
    /// An entry of the project (`bibliothek.toml`) rather than a built-in one.
    pub custom: bool,
}

impl LibraryEntry {
    pub(crate) fn of(item: &LibraryItem, custom: bool) -> Self {
        Self {
            kind: item.kind.clone(),
            label: item.label.clone(),
            width_m: item.width_m,
            depth_m: item.depth_m,
            shape: item.shape,
            height_m: item.height_m,
            custom,
        }
    }
}
