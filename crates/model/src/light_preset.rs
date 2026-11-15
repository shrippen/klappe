use serde::{Deserialize, Serialize};

use crate::plan::LightKind;

/// Widest beam is the full circle.
pub const MAX_BEAM_DEG: f64 = 360.0;
/// Strongest lamp: more is a typing error.
pub const MAX_POWER_W: f64 = 100_000.0;
/// Colour temperatures of candle to blue sky.
pub const MIN_TEMP_K: f64 = 1000.0;
pub const MAX_TEMP_K: f64 = 20_000.0;

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
