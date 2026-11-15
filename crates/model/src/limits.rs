//! Bounds of the numbers a user can enter. An edit, a template file and a filed template all
//! stay within them, so that no value is saved that cannot be read back or drawn.

/// Largest side of a set, and furthest a place lies from the origin: a stadium fits; 1e300 m
/// would not draw.
pub const MAX_SET_M: f64 = 1000.0;

/// Highest a lens can be: a crane or a drone, not a satellite.
pub const MAX_CAMERA_HEIGHT_M: f64 = 50.0;

/// Whether `height_m` is a lens height: from the floor up to a crane's. Excludes NaN.
pub fn is_camera_height_m(height_m: f64) -> bool {
    (0.0..=MAX_CAMERA_HEIGHT_M).contains(&height_m)
}

/// A camera looks straight down at -90° and straight up at 90°.
pub const MAX_TILT_DEG: f64 = 90.0;

/// Whether `tilt_deg` is a tilt a camera can have. Excludes NaN.
pub fn is_tilt_deg(tilt_deg: f64) -> bool {
    (-MAX_TILT_DEG..=MAX_TILT_DEG).contains(&tilt_deg)
}

/// Widest sensor; beyond, a number is a typing error.
pub const MAX_SENSOR_WIDTH_MM: f64 = 500.0;

/// Whether `width_mm` is a sensor width: more than nothing, not absurd. Excludes NaN.
pub fn is_sensor_width_mm(width_mm: f64) -> bool {
    width_mm > 0.0 && width_mm <= MAX_SENSOR_WIDTH_MM
}

/// Range of lenses a camera can have; beyond, a number is a typing error (1e-300 mm is ~180
/// degrees of view). A shot's own lens has a narrower range, kept in the session.
pub const MIN_CAMERA_LENS_MM: f64 = 1.0;
pub const MAX_CAMERA_LENS_MM: f64 = 5000.0;

/// Whether `lens_mm` is a focal length a camera can have. Excludes NaN.
pub fn is_camera_lens_mm(lens_mm: f64) -> bool {
    (MIN_CAMERA_LENS_MM..=MAX_CAMERA_LENS_MM).contains(&lens_mm)
}

/// Widest frame, width to height; real ones stay below 4.
pub const MAX_ASPECT: f64 = 20.0;

/// Whether `aspect` is a frame ratio: more than nothing, not absurd. Excludes NaN.
pub fn is_aspect(aspect: f64) -> bool {
    aspect > 0.0 && aspect <= MAX_ASPECT
}

/// Whether `hold_s` is a pause: a finite time, not negative.
pub fn is_hold_s(hold_s: f64) -> bool {
    hold_s.is_finite() && hold_s >= 0.0
}
