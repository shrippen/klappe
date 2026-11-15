//! Plain data for the surfaces: what a row, a track or a script line shows.

use klappe_model::{Dash, FigureColor, Fill, LibraryItem, ObjectShape, OpeningKind, Template};
use serde::Serialize;

/// The frame a shot shows when its camera is gone (16:9).
pub const FALLBACK_ASPECT: f64 = 16.0 / 9.0;

/// One row of the shot list (LH-71, LH-72).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ShotRow {
    pub id: String,
    pub number: String,
    pub camera: String,
    pub camera_label: String,
    /// Width to height of the camera's frame (LH-38); `FALLBACK_ASPECT` if the camera is gone.
    pub aspect: f64,
    /// Character id of the figure the shot's camera follows (LH-37), else empty.
    pub follows: String,
    /// Model name, e.g. `close-up`; the surface translates it.
    pub size: String,
    /// Model name, e.g. `birds-eye`; the surface translates it.
    pub angle: String,
    pub movement: String,
    /// What the camera's marks do during a shot without a movement of its own (LH-33): a
    /// model name like `crane`; empty when the shot has a movement or the camera is static.
    pub movement_hint: String,
    /// The shot's lens, the smallest of a zoom (`shot_lens`).
    pub lens_mm: f64,
    /// The largest lens when it differs: a zoom; `None` for one lens.
    pub lens_to_mm: Option<f64>,
    /// The camera's marks give the lens: the shot's own field is not in use.
    pub lens_from_marks: bool,
    /// Only the shot's own lens; `None` follows the camera.
    pub own_lens_mm: Option<f64>,
    pub from_s: f64,
    pub to_s: f64,
    pub description: String,
    pub props: String,
    pub gear: String,
    pub crew: String,
    pub note: String,
    /// The camera is on the other side of the axis than that of another shot (LH-41).
    pub axis_jump: bool,
    /// Numbers of those shots, comma separated; empty without a jump.
    pub axis_against: String,
    /// The camera setup (1-based): same camera, place and lens (LH-72).
    pub setup: usize,
}

/// The two characters the 180° axis runs between (LH-40).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AxisChoice {
    /// Character ids.
    pub a: String,
    pub b: String,
    /// Chosen for the plan; otherwise the first two figures.
    pub own: bool,
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
    /// Id of the character a cue names (D-08); empty for any other line.
    pub character: String,
}

/// A scene a character plays in (D-08): how often it speaks there and in how many plans it
/// has a figure.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SceneUse {
    pub id: String,
    pub number: String,
    pub heading: String,
    pub cues: usize,
    pub figures: usize,
}

/// A character that plays in a scene (D-08).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CastUse {
    pub id: String,
    pub name: String,
    pub color: &'static str,
    pub cues: usize,
}

/// What a scene's card shows beyond its row (D-08).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SceneCard {
    /// The time of day at the end of the heading (`TAG`); empty when it has none.
    pub time: String,
    pub plans: usize,
    pub shots: usize,
    pub cast: Vec<CastUse>,
}

/// One entry of the object library, as the object tool lists it (LH-11, LH-12).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LibraryEntry {
    /// What a placed object's `kind` is, e.g. `table`.
    pub kind: String,
    /// The name of an own entry; empty for a built-in one, which the surface translates
    /// (`objectKind.<kind>`).
    pub label: String,
    /// The group of the list: a key of the catalogue (`objectGroup.<group>`) for a built-in
    /// entry, free text for an own one; empty for none.
    pub group: String,
    pub width_m: f64,
    pub depth_m: f64,
    pub shape: ObjectShape,
    /// Height of a new object of this entry (LH-17); `None` when it has none.
    pub height_m: Option<f64>,
    /// An entry of the project (`bibliothek.toml`) rather than a built-in one.
    pub custom: bool,
    /// How many objects of the project's sets have this kind (set by `Session::library`).
    pub uses: usize,
}

impl LibraryEntry {
    pub(crate) fn of(item: &LibraryItem, custom: bool) -> Self {
        Self {
            kind: item.kind.clone(),
            label: item.label.clone(),
            group: item.group.clone(),
            width_m: item.width_m,
            depth_m: item.depth_m,
            shape: item.shape,
            height_m: item.height_m,
            custom,
            uses: 0,
        }
    }
}

/// One template, as the template list offers it (LH-07).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TemplateEntry {
    /// What `Session::apply_template` takes, e.g. `shot-reverse`.
    pub id: String,
    /// The name of an own template; empty for a built-in one, which the surface translates
    /// (`template.<id>`).
    pub label: String,
    pub figures: usize,
    pub cameras: usize,
    /// A template of the project (`vorlagen.toml`) rather than a built-in one.
    pub custom: bool,
}

impl TemplateEntry {
    pub(crate) fn of(template: &Template, custom: bool) -> Self {
        Self {
            id: template.id.clone(),
            label: template.label.clone(),
            figures: template.figures.len(),
            cameras: template.cameras.len(),
            custom,
        }
    }
}

/// One scene in the project lists.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneRow {
    pub id: String,
    pub number: String,
    pub heading: String,
    pub location: String,
    pub location_name: String,
}

/// One location in the project lists.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LocationRow {
    pub id: String,
    pub name: String,
    #[serde(rename = "width")]
    pub width_m: f64,
    #[serde(rename = "depth")]
    pub depth_m: f64,
    /// The set's room height (LH-90).
    #[serde(rename = "height")]
    pub wall_height_m: f64,
    /// How many scenes play here.
    pub scenes: usize,
}

/// One character in the project lists.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CharacterRow {
    pub id: String,
    pub name: String,
    pub short: String,
    /// The colour's role name, e.g. `purple`.
    pub color: &'static str,
    /// Head count; 0 for a single person.
    pub group: u32,
    /// Diameter of the body in metres (LH-25); `None` is the standard.
    #[serde(rename = "size")]
    pub size_m: Option<f64>,
}

/// One plan variant of the scene on view (LH-04).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PlanRow {
    pub id: String,
    pub name: String,
    pub shots: usize,
}

/// The lists of the project view: scenes, locations, characters and the plans of the scene on
/// view.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Overview {
    pub scenes: Vec<SceneRow>,
    pub locations: Vec<LocationRow>,
    pub characters: Vec<CharacterRow>,
    pub plans: Vec<PlanRow>,
    /// Id of the plan shown; empty without one.
    pub plan: String,
}

/// What a tooltip over an element of the plan says; the surface puts it into words.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Hint {
    Camera {
        label: String,
        /// The lens at the playhead.
        #[serde(rename = "lens")]
        lens_mm: f64,
        /// The camera's own lens.
        #[serde(rename = "ownLens")]
        own_lens_mm: f64,
        #[serde(rename = "sensor")]
        sensor_mm: f64,
        aspect: f64,
        /// Id of the body with this sensor; empty for a custom sensor.
        body: String,
        /// Horizontal angle of view, to a tenth of a degree.
        #[serde(rename = "fov")]
        fov_deg: f64,
        /// How the camera came to its mark at the playhead, e.g. `crane`; empty at the first
        /// mark (LH-33).
        rig: String,
        /// Lens height at the playhead (it runs between marks, LH-33).
        #[serde(rename = "height")]
        height_m: f64,
        /// Numbers of the shots from this camera, comma separated.
        shots: String,
        /// Name of the character the camera follows, else empty.
        follows: String,
    },
    Figure {
        name: String,
        index: Option<usize>,
        #[serde(rename = "time")]
        at_s: Option<f64>,
    },
    Mark {
        name: String,
        /// 1-based.
        index: Option<usize>,
        #[serde(rename = "time")]
        at_s: Option<f64>,
    },
    /// A bend of a way.
    Bend {
        /// 1-based.
        index: Option<usize>,
    },
    Light {
        label: String,
        /// The lamp kind's file name, e.g. `led-panel`.
        #[serde(rename = "type")]
        light_kind: String,
        #[serde(rename = "facing")]
        facing_deg: f64,
        #[serde(rename = "beam")]
        beam_deg: f64,
        /// Whether the colour temperature can be set (an LED).
        tunable: bool,
        /// Whether the lamp makes light of its own: not a bounce or a flag.
        gives_light: bool,
        #[serde(rename = "temp")]
        temp_k: f64,
        #[serde(rename = "power")]
        power_w: f64,
    },
    Object {
        label: String,
        /// What the object's `kind` is, e.g. `table`.
        library: String,
        #[serde(rename = "width")]
        width_m: f64,
        #[serde(rename = "depth")]
        depth_m: f64,
        #[serde(rename = "height")]
        height_m: Option<f64>,
        /// Where it begins above the floor (LH-90): its own, else what it stands on.
        #[serde(rename = "base")]
        base_m: f64,
        #[serde(rename = "baseDefault")]
        base_default: bool,
    },
    /// A door or window in a wall (LH-10).
    Opening {
        #[serde(rename = "type")]
        opening: OpeningKind,
        #[serde(rename = "width")]
        width_m: f64,
        /// Sill and head above the floor (LH-90): its own, else its kind's.
        #[serde(rename = "sill")]
        sill_m: f64,
        #[serde(rename = "head")]
        head_m: f64,
        #[serde(rename = "sillDefault")]
        sill_default: bool,
        #[serde(rename = "headDefault")]
        head_default: bool,
    },
    /// A wall, for the card that sets its height (LH-90).
    Wall {
        /// Its own height, else the room's.
        #[serde(rename = "height")]
        height_m: f64,
        #[serde(rename = "heightDefault")]
        height_default: bool,
    },
    Note {
        text: String,
    },
    /// A drawing, for the card that changes its style (LH-19).
    Drawing {
        color: FigureColor,
        dash: Dash,
        fill: Fill,
        /// Whether the shape has an inside, so a fill means something.
        closed: bool,
    },
}
