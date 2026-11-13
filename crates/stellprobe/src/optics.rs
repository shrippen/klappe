use klappe_model::{Point, Sensor};

/// Horizontal angle of view in degrees (LH-31): `2·atan(w / 2f)` with the sensor's active width.
///
/// Example: Super 35 (24.89 mm) with a 35 mm lens sees about 39.1°.
pub fn horizontal_fov_deg(sensor: Sensor, lens_mm: f64) -> f64 {
    if lens_mm <= 0.0 {
        return 0.0;
    }

    (sensor.width_mm() / (2.0 * lens_mm)).atan().to_degrees() * 2.0
}

/// The camera's view on the plan as a triangle: the camera and the two edges of the frame
/// `reach_m` metres ahead.
///
/// ```text
///   left ─────── right
///      \       /
///       \     /      fov
///        \   /
///        camera ──► facing
/// ```
pub fn frustum(at: Point, facing_deg: f64, fov_deg: f64, reach_m: f64) -> [Point; 3] {
    let half = fov_deg / 2.0;
    let edge = |angle_deg: f64| {
        let r = angle_deg.to_radians();
        Point::new(at.x_m + reach_m * r.cos(), at.y_m + reach_m * r.sin())
    };

    [at, edge(facing_deg - half), edge(facing_deg + half)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn super35_35mm_sees_39_degrees() {
        let fov = horizontal_fov_deg(Sensor::Super35, 35.0);

        assert!((fov - 39.15).abs() < 0.01, "{fov}");
    }

    #[test]
    fn longer_lens_narrower_view() {
        let wide = horizontal_fov_deg(Sensor::FullFrame, 24.0);
        let tele = horizontal_fov_deg(Sensor::FullFrame, 85.0);

        assert!(wide > tele);
    }

    #[test]
    fn no_lens_no_view() {
        assert_eq!(horizontal_fov_deg(Sensor::Super35, 0.0), 0.0);
    }

    /// 0° faces right, 90° down (y grows downwards); the first edge is the one counter-
    /// clockwise of the facing on screen.
    #[test]
    fn frustum_turns_clockwise_on_screen() {
        let [_, left, right] = frustum(Point::default(), 90.0, 90.0, 2.0_f64.sqrt());

        assert!(
            (left.x_m - 1.0).abs() < 1e-9 && (left.y_m - 1.0).abs() < 1e-9,
            "{left:?}"
        );
        assert!(
            (right.x_m + 1.0).abs() < 1e-9 && (right.y_m - 1.0).abs() < 1e-9,
            "{right:?}"
        );
    }

    #[test]
    fn frustum_is_symmetric_around_facing() {
        let [apex, left, right] = frustum(Point::default(), 0.0, 90.0, 1.0);

        assert_eq!(apex, Point::default());
        assert!((left.x_m - right.x_m).abs() < 1e-9);
        assert!((left.y_m + right.y_m).abs() < 1e-9);
    }
}
