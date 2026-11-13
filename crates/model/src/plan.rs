use serde::{Deserialize, Serialize};

use crate::geometry::Point;
use crate::shot::Shot;

/// The plan of one scene: who stands and moves where, cameras, lights, shots (LH-04: a scene
/// may have variants, `Plan A` and `Plan B`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FloorPlan {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub figures: Vec<FigureTrack>,
    #[serde(default)]
    pub cameras: Vec<Camera>,
    #[serde(default)]
    pub lights: Vec<Light>,
    /// Shots in cut order (LH-72).
    #[serde(default)]
    pub shots: Vec<Shot>,
    #[serde(default)]
    pub links: Vec<ScriptLink>,
    /// The two characters the 180° line runs between (LH-40); `None` takes the first two
    /// figures.
    #[serde(default)]
    pub axis: Option<AxisPair>,
}

/// Ids of the two characters that span the axis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AxisPair {
    pub a: String,
    pub b: String,
}

impl FloorPlan {
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_owned(),
            name: name.to_owned(),
            figures: Vec::new(),
            cameras: Vec::new(),
            lights: Vec::new(),
            shots: Vec::new(),
            links: Vec::new(),
            axis: None,
        }
    }

    pub fn figure(&self, character: &str) -> Option<&FigureTrack> {
        self.figures.iter().find(|f| f.character == character)
    }

    pub fn camera(&self, id: &str) -> Option<&Camera> {
        self.cameras.iter().find(|c| c.id == id)
    }

    /// End of the last mark of any figure or camera: the length of the scene's blocking.
    pub fn duration_s(&self) -> f64 {
        let figures = self
            .figures
            .iter()
            .flat_map(|f| f.marks.iter().map(|m| m.at_s));
        let cameras = self
            .cameras
            .iter()
            .flat_map(|c| c.marks.iter().map(|m| m.at_s));
        let shots = self.shots.iter().map(|s| s.to_s);

        figures.chain(cameras).chain(shots).fold(0.0, f64::max)
    }
}

/// A character's way through the scene: marks 1, 2, 3 … in time order (LH-21).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FigureTrack {
    /// Id of the project's character.
    pub character: String,
    pub marks: Vec<Mark>,
}

/// Where a figure is at a moment: arrives at `at_s`, stays until it leaves for the next mark.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Mark {
    pub at_s: f64,
    pub at: Point,
    pub facing_deg: f64,
    #[serde(default)]
    pub pose: Pose,
    /// Seconds the figure stays before it walks on (LH-63: pause at a mark).
    #[serde(default)]
    pub hold_s: f64,
}

/// LH-22.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Pose {
    #[default]
    Standing,
    Sitting,
    Lying,
}

/// A camera body with its lens (LH-30 – LH-34).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Camera {
    pub id: String,
    /// `A`, `B`, `C`.
    pub label: String,
    pub sensor: Sensor,
    pub lens_mm: f64,
    /// Width to height of the recorded frame, e.g. 2.39 (LH-38).
    pub aspect: f64,
    pub marks: Vec<CameraMark>,
    /// A figure the camera keeps its distance and angle to when that figure is moved (LH-37).
    #[serde(default)]
    pub follows: Option<String>,
}

/// Camera position at a moment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CameraMark {
    pub at_s: f64,
    pub at: Point,
    pub facing_deg: f64,
    pub height_m: f64,
    #[serde(default)]
    pub tilt_deg: f64,
    #[serde(default)]
    pub hold_s: f64,
}

/// Sensor formats with their active width; the frame's aspect crops the height (LH-32).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Sensor {
    /// Super 35, 24.89 mm wide.
    Super35,
    /// Full frame, 36 mm wide.
    FullFrame,
    /// Micro Four Thirds, 17.3 mm wide.
    MicroFourThirds,
    /// Any other sensor, by its width.
    Custom { width_mm: f64 },
}

impl Sensor {
    pub fn width_mm(self) -> f64 {
        match self {
            Sensor::Super35 => 24.89,
            Sensor::FullFrame => 36.0,
            Sensor::MicroFourThirds => 17.3,
            Sensor::Custom { width_mm } => width_mm,
        }
    }
}

/// A lamp (LH-50, LH-51).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Light {
    pub id: String,
    pub label: String,
    pub kind: LightKind,
    pub at: Point,
    pub facing_deg: f64,
    pub beam_deg: f64,
    pub color_temp_k: f64,
    pub power_w: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LightKind {
    Fresnel,
    LedPanel,
    Tube,
    Balloon,
    Reflector,
    Flag,
}

/// Ties an element of the scene's script to the plan (design B: "when what happens").
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScriptLink {
    /// Index of the element (action, dialogue …) within the scene in the script.
    pub element: usize,
    pub target: LinkTarget,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "kind")]
pub enum LinkTarget {
    /// Mark `index` (0-based) of a character's track.
    Mark {
        character: String,
        index: usize,
    },
    Shot {
        id: String,
    },
}
