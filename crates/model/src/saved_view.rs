use serde::{Deserialize, Serialize};

/// A named choice of what the plan shows (LH-119): which layers, cameras and figures are
/// visible. Whatever a list does not name is hidden when the view is chosen.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SavedView {
    /// Unique in the project, e.g. `nur-kameras`.
    pub id: String,
    pub name: String,
    /// Names of the visible layers (`set`, `figures` …).
    #[serde(default)]
    pub layers: Vec<String>,
    /// Ids of the visible cameras.
    #[serde(default)]
    pub cameras: Vec<String>,
    /// Ids of the characters whose figures are visible.
    #[serde(default)]
    pub figures: Vec<String>,
}
