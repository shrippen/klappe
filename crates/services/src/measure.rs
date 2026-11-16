//! What a measurement shows beyond the plan's calculations.

use klappe_model::{normalize_deg, Point};

/// The direction from `from` to `to`, counted as a figure's `facing_deg`: 0 to the right, 90
/// down the plan (y grows downwards), within [0, 360), with `decimals` places. `None` if the
/// points coincide.
pub fn measure_angle_deg(from: Point, to: Point, decimals: usize) -> Option<f64> {
    if from == to {
        return None;
    }

    // Round first: 359.97 shows as 360, and -0.02 as -0. `+ 0.0` turns -0 into 0.
    let step = 10f64.powi(decimals as i32);
    let rounded = (from.bearing_deg(to) * step).round() / step;

    Some(normalize_deg(rounded) + 0.0)
}
