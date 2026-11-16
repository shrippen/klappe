use serde::{Deserialize, Serialize};

use crate::plan::LightKind;

/// Widest beam is the full circle.
pub const MAX_BEAM_DEG: f64 = 360.0;

/// Narrowest beam: typing and dragging leave at least this.
pub const MIN_BEAM_DEG: f64 = 1.0;

/// Whether `beam_deg` is a beam: from the narrowest up to the full circle. Excludes NaN.
pub fn is_beam_deg(beam_deg: f64) -> bool {
    (MIN_BEAM_DEG..=MAX_BEAM_DEG).contains(&beam_deg)
}

/// Strongest lamp: more is a typing error.
pub const MAX_POWER_W: f64 = 100_000.0;

/// Whether `power_w` is a lamp power: from none up to the strongest. Excludes NaN.
pub fn is_power_w(power_w: f64) -> bool {
    (0.0..=MAX_POWER_W).contains(&power_w)
}

/// Colour temperatures of candle to blue sky.
pub const MIN_TEMP_K: f64 = 1000.0;
pub const MAX_TEMP_K: f64 = 20_000.0;

/// Whether `temp_k` is a colour temperature a lamp can have. Excludes NaN.
pub fn is_temp_k(temp_k: f64) -> bool {
    (MIN_TEMP_K..=MAX_TEMP_K).contains(&temp_k)
}

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
