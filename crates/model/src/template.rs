use serde::{Deserialize, Serialize};

use crate::plan::{Camera, Mark};

/// A set of figures and cameras for a typical set-up (LH-07), e.g. shot and reverse shot.
/// Places are in metres from the insertion point, which is where the template is put down.
/// Built-in entries are named by the catalogue (`template.<id>`); an own entry carries its
/// `label`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Template {
    /// Unique among the built-in and the project's own templates, e.g. `shot-reverse`.
    pub id: String,
    /// The name of an own entry; empty for a built-in one.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub label: String,
    #[serde(default)]
    pub figures: Vec<TemplateFigure>,
    /// Their `id` and `label` are placeholders: a placed camera gets free ones. `follows`
    /// names a figure's `slot`.
    #[serde(default)]
    pub cameras: Vec<Camera>,
}

/// A placeholder for a figure: whoever places the template binds it to a character.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemplateFigure {
    /// Unique within the template; also the name of a character made for it.
    pub slot: String,
    pub marks: Vec<Mark>,
}
