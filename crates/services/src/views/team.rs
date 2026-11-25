//! The rows of the cast and the team (Besetzung/Team).

use serde::Serialize;

/// A person with their contact.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PersonRow {
    pub id: String,
    pub name: String,
    /// The department's name, as `lighting`; `None` for none.
    pub department: Option<&'static str>,
    pub function: String,
    pub agency: String,
    pub phone: String,
    pub mail: String,
    pub address: String,
}

/// Who plays a character and under which number.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CastingRow {
    pub character: String,
    /// The id of the person who plays it.
    pub fixed: Option<String>,
    /// The ids of the persons proposed, the fixed one among them.
    pub proposed: Vec<String>,
    /// The cast number; `None` for a group of extras.
    pub number: Option<u32>,
    /// The number is stored (set by hand or locked), not worked out from the book.
    pub stored: bool,
    /// In how many scenes of the book the character is.
    pub scenes: usize,
}
