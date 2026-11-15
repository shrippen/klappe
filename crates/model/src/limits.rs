//! Bounds of the numbers a user can enter. An edit, a template file and a filed template all
//! stay within them, so that no value is saved that cannot be read back or drawn.

/// Largest side of a set, and furthest a place lies from the origin: a stadium fits; 1e300 m
/// would not draw.
pub const MAX_SET_M: f64 = 1000.0;

/// Highest a lens can be: a crane or a drone, not a satellite.
pub const MAX_CAMERA_HEIGHT_M: f64 = 50.0;

/// A camera looks straight down at -90° and straight up at 90°.
pub const MAX_TILT_DEG: f64 = 90.0;

/// Widest sensor and the range of lenses that make sense; beyond, a number is a typing error
/// (1e-300 mm is ~180 degrees of view).
pub const MAX_SENSOR_WIDTH_MM: f64 = 500.0;
pub const MIN_LENS_MM: f64 = 1.0;
pub const MAX_LENS_MM: f64 = 5000.0;

/// Widest frame, width to height; real ones stay below 4.
pub const MAX_ASPECT: f64 = 20.0;
