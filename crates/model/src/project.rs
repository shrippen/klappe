use serde::{Deserialize, Serialize};

use crate::geometry::Point;
use crate::plan::FloorPlan;

/// One film: everything every module works on.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub title: String,
    #[serde(default)]
    pub characters: Vec<Character>,
    #[serde(default)]
    pub locations: Vec<Location>,
    #[serde(default)]
    pub scenes: Vec<Scene>,
}

impl Project {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_owned(),
            characters: Vec::new(),
            locations: Vec::new(),
            scenes: Vec::new(),
        }
    }

    pub fn character(&self, id: &str) -> Option<&Character> {
        self.characters.iter().find(|c| c.id == id)
    }

    pub fn location(&self, id: &str) -> Option<&Location> {
        self.locations.iter().find(|l| l.id == id)
    }

    pub fn scene(&self, id: &str) -> Option<&Scene> {
        self.scenes.iter().find(|s| s.id == id)
    }
}

/// A role in the script, project-wide (LH-03).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Character {
    /// Stable id, e.g. `elisabeth`; names may change, ids do not.
    pub id: String,
    pub name: String,
    /// Short label drawn in the figure, e.g. `E`.
    pub short: String,
    pub color: FigureColor,
}

/// The colour a figure is drawn in. A role, not a hex value: the UI maps it to Kante's
/// semantic colours, so both themes stay readable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FigureColor {
    Purple,
    Blue,
    Aqua,
    Green,
    Yellow,
    Orange,
    Red,
    Cyan,
}

/// A place with a set: the Sängersaal, the castle yard (LH-02).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Location {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub set: SetPlan,
}

/// The built set of a location, shared by every scene there.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct SetPlan {
    /// Width and depth of the drawing area in metres.
    pub width_m: f64,
    pub depth_m: f64,
    #[serde(default)]
    pub walls: Vec<Wall>,
    #[serde(default)]
    pub objects: Vec<SetObject>,
}

/// A wall segment (LH-10); doors and windows are set objects on it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Wall {
    pub from: Point,
    pub to: Point,
    pub thickness_m: f64,
}

/// Furniture, props, pillars: anything from the library (LH-11).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetObject {
    pub id: String,
    /// Library item, e.g. `table`, `pillar`, `door`, `window`, `fireplace`.
    pub kind: String,
    pub label: String,
    /// Centre of the object.
    pub at: Point,
    pub width_m: f64,
    pub depth_m: f64,
    pub rotation_deg: f64,
}

/// A scene from the script, at one location (LH-01).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Scene {
    pub id: String,
    /// Scene number as in the script, e.g. `12` or `12A`.
    pub number: String,
    /// Scene heading, e.g. `INT. SÄNGERSAAL – TAG`.
    pub heading: String,
    pub location: String,
    #[serde(default)]
    pub plans: Vec<FloorPlan>,
}
