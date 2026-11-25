//! The breakdown (DB-60, D-22): the elements of the film and what each scene needs of them.

use std::collections::BTreeMap;

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

/// A named look of a character, e.g. `Ölzeug, nass`: the costume and make-up elements it
/// is made of. In a scene's sheet a character in the scene gets one of its looks
/// (`Sheet::looks`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Look {
    /// Random and stable: the name may change, the id does not.
    pub id: String,
    /// The id of the character it belongs to.
    pub character: String,
    pub name: String,
    /// Ids of elements of the categories costume and make-up.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub elements: Vec<String>,
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
    /// Where elements were marked in the scene's text (D-23).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub marks: Vec<TextMark>,
    /// The look each character in the scene wears: character id to look id. Only characters
    /// in the scene have one.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub looks: BTreeMap<String, String>,
}

/// A place in a scene's text marked as an element (D-23), held as a quote: the words and
/// which of their occurrences in the scene it is. The Fountain text stays clean (F-02).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextMark {
    /// The id of the element.
    pub element: String,
    /// The marked words, e.g. `Stapel Seiten`.
    pub text: String,
    /// Which occurrence of `text` in the scene's text, from 0.
    #[serde(default)]
    pub occurrence: usize,
}

impl Sheet {
    /// Whether the sheet holds nothing: it then has no file.
    pub fn is_empty(&self) -> bool {
        *self == Sheet::default()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    /// The place of a category in `ALL`, for its order. That `ALL` has every variant at all
    /// is checked against the source in `services/tests/step_texts.rs`.
    fn category_place(category: ElementCategory) -> usize {
        match category {
            ElementCategory::Costume => 0,
            ElementCategory::Makeup => 1,
            ElementCategory::Construction => 2,
            ElementCategory::SetDressing => 3,
            ElementCategory::Prop => 4,
            ElementCategory::Graphics => 5,
            ElementCategory::Vehicle => 6,
            ElementCategory::Animal => 7,
            ElementCategory::Vfx => 8,
            ElementCategory::Sfx => 9,
            ElementCategory::Sound => 10,
            ElementCategory::Music => 11,
            ElementCategory::CameraLight => 12,
            ElementCategory::Stunt => 13,
            ElementCategory::Other => 14,
        }
    }

    /// Like `category_place`, for the status.
    fn status_place(status: ElementStatus) -> usize {
        match status {
            ElementStatus::Open => 0,
            ElementStatus::Working => 1,
            ElementStatus::Ready => 2,
        }
    }

    /// `ALL` has every category once, in the order of the scene sheet; `name` is what the file
    /// holds and `from_name` reads it back.
    #[test]
    fn category_names_are_the_file_names() {
        for (place, category) in ElementCategory::ALL.into_iter().enumerate() {
            assert_eq!(category_place(category), place);

            let json = serde_json::to_string(&category).unwrap();
            assert_eq!(json, format!("\"{}\"", category.name()));
            assert_eq!(ElementCategory::from_name(category.name()), Some(category));
        }
        assert_eq!(ElementCategory::from_name("nothing"), None);
    }

    /// The same for the status.
    #[test]
    fn status_names_are_the_file_names() {
        for (place, status) in ElementStatus::ALL.into_iter().enumerate() {
            assert_eq!(status_place(status), place);

            let json = serde_json::to_string(&status).unwrap();
            assert_eq!(json, format!("\"{}\"", status.name()));
            assert_eq!(ElementStatus::from_name(status.name()), Some(status));
        }
        assert_eq!(ElementStatus::from_name("nothing"), None);
    }
}
