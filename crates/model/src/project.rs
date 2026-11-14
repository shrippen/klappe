use serde::{Deserialize, Serialize};

use crate::geometry::Point;
use crate::library::{LibraryItem, ObjectShape};
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
    /// The project's own objects (LH-12), next to the built-in library.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub library: Vec<LibraryItem>,
}

impl Project {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_owned(),
            characters: Vec::new(),
            locations: Vec::new(),
            scenes: Vec::new(),
            library: Vec::new(),
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
    /// How many people the character stands for (LH-23): a group such as the guests moves
    /// as one figure. `None` is a single person.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<u32>,
}

/// A group has two to 999 people; one person is a figure, more would not fit the label.
pub const MIN_GROUP: u32 = 2;
pub const MAX_GROUP: u32 = 999;

impl Character {
    /// The name with the head count of a group: `Gäste (12)`.
    pub fn title(&self) -> String {
        match self.group {
            Some(count) => format!("{} ({count})", self.name),
            None => self.name.clone(),
        }
    }
}

/// The colour a figure is drawn in. A role, not a hex value: one of Kante's label colours
/// (`KanteStyle.labels`), so both themes stay readable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FigureColor {
    Purple,
    Blue,
    Green,
    Yellow,
    Orange,
    Red,
    Cyan,
}

impl FigureColor {
    /// Every colour, in the order the form offers them.
    pub const ALL: [FigureColor; 7] = [
        FigureColor::Purple,
        FigureColor::Blue,
        FigureColor::Green,
        FigureColor::Yellow,
        FigureColor::Orange,
        FigureColor::Red,
        FigureColor::Cyan,
    ];

    /// The name the file format uses, e.g. `purple`; also Kante's label name.
    pub fn name(self) -> &'static str {
        match self {
            FigureColor::Purple => "purple",
            FigureColor::Blue => "blue",
            FigureColor::Green => "green",
            FigureColor::Yellow => "yellow",
            FigureColor::Orange => "orange",
            FigureColor::Red => "red",
            FigureColor::Cyan => "cyan",
        }
    }

    /// The colour `name` names; `None` for any other text.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|c| c.name() == name)
    }
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
    /// An image under the set, e.g. a floor plan or an aerial photo (LH-13).
    #[serde(default)]
    pub background: Option<Background>,
    /// Free text on the plan, e.g. `Vorhang schließen` (LH-16).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<Note>,
}

/// A text pinned to a place on the set.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Note {
    /// Stable id within the set, e.g. `n1`.
    pub id: String,
    pub at: Point,
    pub text: String,
}

/// An image behind the plan, scaled so that its pixels measure what they do on site. The
/// height follows from the image's aspect ratio, which the model does not know: whoever
/// needs it reads the pixel size of `file`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Background {
    /// Relative to the project folder, e.g. `bilder/saengersaal.png`.
    pub file: String,
    /// The image's top left corner on the plan.
    pub at: Point,
    pub width_m: f64,
    /// Clockwise on screen about `at`.
    pub rotation_deg: f64,
    /// 0 = invisible, 1 = opaque.
    pub opacity: f64,
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
    /// Drawn as a rectangle unless it says otherwise.
    #[serde(default, skip_serializing_if = "is_rect")]
    pub shape: ObjectShape,
    /// How tall the object is (LH-17); `None` when not given.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height_m: Option<f64>,
}

fn is_rect(shape: &ObjectShape) -> bool {
    *shape == ObjectShape::Rect
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

#[cfg(test)]
mod tests {
    use super::*;

    /// `name` is what the file holds, `from_name` reads it back, and `ALL` has no colour twice.
    #[test]
    fn names_are_the_file_names() {
        for color in FigureColor::ALL {
            let json = serde_json::to_string(&color).unwrap();
            assert_eq!(json, format!("\"{}\"", color.name()));
            assert_eq!(FigureColor::from_name(color.name()), Some(color));
        }
        assert_eq!(FigureColor::from_name("magenta"), None);

        let mut names: Vec<_> = FigureColor::ALL.iter().map(|c| c.name()).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), FigureColor::ALL.len());
    }
}
