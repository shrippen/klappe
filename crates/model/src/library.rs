use serde::{Deserialize, Serialize};

/// Largest side of an object: a stadium fits; 1e300 m would not draw.
pub const MAX_OBJECT_M: f64 = 1000.0;

/// Whether `size_m` is a side an object can have: a number, more than nothing, not absurd.
pub fn is_object_size_m(size_m: f64) -> bool {
    size_m.is_finite() && size_m > 0.0 && size_m <= MAX_OBJECT_M
}

/// How an object is drawn within its width and depth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ObjectShape {
    #[default]
    Rect,
    Ellipse,
}

/// An entry of the object library (LH-11, LH-12): what a new object of this `kind` looks like.
/// Built-in entries are named by the catalogue (`objectKind.<kind>`); an own entry carries
/// its `label`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LibraryItem {
    /// What `SetObject::kind` names, e.g. `table`.
    pub kind: String,
    /// The name of an own entry; empty for a built-in one.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub label: String,
    pub width_m: f64,
    pub depth_m: f64,
    #[serde(default)]
    pub shape: ObjectShape,
}
