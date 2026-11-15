use serde::{Deserialize, Serialize};

use crate::plan::LightKind;

/// A lamp to choose from (LH-51): the kind and what a new lamp of it starts with. Named by the
/// catalogue (`lightPreset.<id>`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LightPreset {
    pub id: String,
    pub kind: LightKind,
    pub power_w: f64,
    pub color_temp_k: f64,
    pub beam_deg: f64,
}
