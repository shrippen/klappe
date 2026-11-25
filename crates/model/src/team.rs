//! The team and the cast (Besetzung/Team, DATENMODELL §3): persons, their contacts and who plays
//! which character.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::breakdown::ElementCategory;

/// The department a person works in; the order is the one of the lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Department {
    Direction,
    Production,
    UnitManagement,
    Casting,
    Camera,
    Lighting,
    Sound,
    ProductionDesign,
    SetDecoration,
    Props,
    Costume,
    Makeup,
    Editing,
    Vfx,
    Sfx,
    Stunt,
    Cast,
}

impl Department {
    pub const ALL: [Department; 17] = [
        Department::Direction,
        Department::Production,
        Department::UnitManagement,
        Department::Casting,
        Department::Camera,
        Department::Lighting,
        Department::Sound,
        Department::ProductionDesign,
        Department::SetDecoration,
        Department::Props,
        Department::Costume,
        Department::Makeup,
        Department::Editing,
        Department::Vfx,
        Department::Sfx,
        Department::Stunt,
        Department::Cast,
    ];

    const NAMES: [(Department, &'static str); 17] = [
        (Department::Direction, "direction"),
        (Department::Production, "production"),
        (Department::UnitManagement, "unit-management"),
        (Department::Casting, "casting"),
        (Department::Camera, "camera"),
        (Department::Lighting, "lighting"),
        (Department::Sound, "sound"),
        (Department::ProductionDesign, "production-design"),
        (Department::SetDecoration, "set-decoration"),
        (Department::Props, "props"),
        (Department::Costume, "costume"),
        (Department::Makeup, "makeup"),
        (Department::Editing, "editing"),
        (Department::Vfx, "vfx"),
        (Department::Sfx, "sfx"),
        (Department::Stunt, "stunt"),
        (Department::Cast, "cast"),
    ];

    /// The name the file format uses, e.g. `unit-management`; also the end of the text key
    /// `department.<name>`.
    pub fn name(self) -> &'static str {
        let named = Self::NAMES.iter().find(|(d, _)| *d == self);

        named.map_or("", |(_, name)| name)
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|d| d.name() == name)
    }

    /// The department that usually answers for elements of `category`.
    pub fn of_category(category: ElementCategory) -> Self {
        match category {
            ElementCategory::Costume => Department::Costume,
            ElementCategory::Makeup => Department::Makeup,
            ElementCategory::Construction => Department::ProductionDesign,
            ElementCategory::SetDressing | ElementCategory::Graphics => Department::SetDecoration,
            ElementCategory::Prop => Department::Props,
            ElementCategory::Vehicle | ElementCategory::Animal => Department::UnitManagement,
            ElementCategory::Vfx => Department::Vfx,
            ElementCategory::Sfx => Department::Sfx,
            ElementCategory::Sound | ElementCategory::Music => Department::Sound,
            ElementCategory::CameraLight => Department::Camera,
            ElementCategory::Stunt => Department::Stunt,
            ElementCategory::Other => Department::Production,
        }
    }
}

/// A person of the production: crew or actor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Person {
    /// Random and stable: the name may change, the id does not.
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub department: Option<Department>,
    /// What the person does, e.g. `Oberbeleuchter`.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub function: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub agency: String,
}

/// How to reach a person; kept apart from the rest (`kontakte.toml`, DATENMODELL §10).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Contact {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub phone: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub mail: String,
    /// May run over several lines.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub address: String,
}

impl Contact {
    pub fn is_empty(&self) -> bool {
        *self == Contact::default()
    }
}

/// Who plays a character: persons proposed for it, one of them fixed.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Casting {
    /// The character's id.
    pub character: String,
    /// The person who plays it; always one of `proposed`. Schedule and call sheet take only
    /// this one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fixed: Option<String>,
    /// Person ids, in the order they were proposed.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub proposed: Vec<String>,
    /// The cast number when it is stored: set by hand, or by locking the numbers. Without one
    /// the number follows from the book.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cast_number: Option<u32>,
}

impl Casting {
    /// Whether it says nothing: it then has no entry.
    pub fn is_empty(&self) -> bool {
        self.fixed.is_none() && self.proposed.is_empty() && self.cast_number.is_none()
    }
}

/// The persons, the casting of the characters and the contacts.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Team {
    /// In the order they were made.
    pub persons: Vec<Person>,
    /// At most one per character; ids of unknown characters or persons stay (DATENMODELL §2).
    pub castings: Vec<Casting>,
    /// The cast numbers no longer move with the book: a new character takes the next one.
    pub numbers_locked: bool,
    /// By person id.
    pub contacts: BTreeMap<String, Contact>,
}

impl Team {
    pub fn person(&self, id: &str) -> Option<&Person> {
        self.persons.iter().find(|p| p.id == id)
    }

    pub fn casting(&self, character: &str) -> Option<&Casting> {
        self.castings.iter().find(|c| c.character == character)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    /// `name` is what the file holds and `from_name` reads it back.
    #[test]
    fn department_names_are_the_file_names() {
        for department in Department::ALL {
            let json = serde_json::to_string(&department).unwrap();

            assert_eq!(json, format!("\"{}\"", department.name()));
            assert_eq!(Department::from_name(department.name()), Some(department));
        }
        assert_eq!(Department::from_name("nothing"), None);
    }

    /// Every category has a department, and the lists have no department twice.
    #[test]
    fn categories_map_to_a_department() {
        for category in ElementCategory::ALL {
            assert!(Department::ALL.contains(&Department::of_category(category)));
        }
        for (place, department) in Department::ALL.into_iter().enumerate() {
            assert!(!Department::ALL[..place].contains(&department));
        }
    }
}
