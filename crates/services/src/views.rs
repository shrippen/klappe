//! Plain data for the surfaces: what a row, a track or a script line shows.

use serde::Serialize;

/// One row of the shot list (LH-71, LH-72), in cut order.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ShotRow {
    pub id: String,
    pub number: String,
    pub camera: String,
    pub camera_label: String,
    /// Model name, e.g. `close-up`; the surface translates it.
    pub size: String,
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
