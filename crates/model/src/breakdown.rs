//! The breakdown (DB-60, D-22): the elements of the film and what each scene needs of them.

use serde::{Deserialize, Serialize};

/// What kind of thing an element is; the order is the one of the scene sheet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ElementCategory {
    Costume,
    Makeup,
    Construction,
    SetDressing,
    Prop,
    Graphics,
    Vehicle,
    Animal,
    Vfx,
    Sfx,
    Sound,
    Music,
    CameraLight,
    Stunt,
    Other,
}

impl ElementCategory {
    pub const ALL: [ElementCategory; 15] = [
        ElementCategory::Costume,
        ElementCategory::Makeup,
        ElementCategory::Construction,
        ElementCategory::SetDressing,
        ElementCategory::Prop,
        ElementCategory::Graphics,
        ElementCategory::Vehicle,
        ElementCategory::Animal,
        ElementCategory::Vfx,
        ElementCategory::Sfx,
        ElementCategory::Sound,
        ElementCategory::Music,
        ElementCategory::CameraLight,
        ElementCategory::Stunt,
        ElementCategory::Other,
    ];

    /// The name the file format uses, e.g. `set-dressing`; also the end of the text key.
    pub fn name(self) -> &'static str {
        match self {
            ElementCategory::Costume => "costume",
            ElementCategory::Makeup => "makeup",
            ElementCategory::Construction => "construction",
            ElementCategory::SetDressing => "set-dressing",
            ElementCategory::Prop => "prop",
            ElementCategory::Graphics => "graphics",
            ElementCategory::Vehicle => "vehicle",
            ElementCategory::Animal => "animal",
            ElementCategory::Vfx => "vfx",
            ElementCategory::Sfx => "sfx",
            ElementCategory::Sound => "sound",
            ElementCategory::Music => "music",
            ElementCategory::CameraLight => "camera-light",
            ElementCategory::Stunt => "stunt",
            ElementCategory::Other => "other",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|c| c.name() == name)
    }
}

/// How far the department is with an element.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ElementStatus {
    #[default]
    Open,
    Working,
    Ready,
}

impl ElementStatus {
    pub const ALL: [ElementStatus; 3] = [
        ElementStatus::Open,
        ElementStatus::Working,
        ElementStatus::Ready,
    ];

    /// The name the file format uses, e.g. `working`; also the end of the text key.
    pub fn name(self) -> &'static str {
        match self {
            ElementStatus::Open => "open",
            ElementStatus::Working => "working",
            ElementStatus::Ready => "ready",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|s| s.name() == name)
    }
}

/// A thing a department provides: a costume, a prop, a vehicle. One element may stand in
/// several scenes (`Sheet::elements`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Element {
    /// Random and stable: the name may change, the id does not.
    pub id: String,
    pub category: ElementCategory,
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(default)]
    pub status: ElementStatus,
}

/// The scene sheet of a scene (DB-60): what the production notes down for it.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Sheet {
    /// The day of the story the scene plays on, e.g. `Tag 3`.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub story_day: String,
    /// The pre-stop (Vorstopp): how long the scene runs, in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prestop_s: Option<u32>,
    /// The time planned for shooting it, in minutes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub planned_min: Option<u32>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub continuity: String,
    /// A note for the production.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub note: String,
    /// Ids of the elements the scene needs, in the order the user took them.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub elements: Vec<String>,
    /// Ids of the elements the set suggested and the user turned down.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub excluded: Vec<String>,
    /// Ids of the characters in the scene that do not speak: silent ones and extras (a
    /// character with a head count, LH-23). The speaking ones follow from the book.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub characters: Vec<String>,
}

impl Sheet {
    /// Whether the sheet holds nothing: it then has no file.
    pub fn is_empty(&self) -> bool {
        *self == Sheet::default()
    }
}
