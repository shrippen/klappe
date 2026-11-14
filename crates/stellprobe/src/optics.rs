use klappe_model::{Camera, Point, Sensor, Shot};

/// Horizontal angle of view in degrees (LH-31): `2·atan(w / 2f)` with the sensor's active width.
///
/// Example: Super 35 (24.89 mm) with a 35 mm lens sees about 39.1°.
pub fn horizontal_fov_deg(sensor: Sensor, lens_mm: f64) -> f64 {
    if lens_mm <= 0.0 {
        return 0.0;
    }

    (sensor.width_mm() / (2.0 * lens_mm)).atan().to_degrees() * 2.0
}

/// The lens in use at `t_s`: that of the first shot of this camera covering `t_s`, if it has
/// its own; else the camera's.
///
/// Example: camera 35 mm, shot 12.2 from 0 s to 4 s with 40 mm: at 2 s this gives 40.
pub fn lens_at(camera: &Camera, shots: &[Shot], t_s: f64) -> f64 {
    shots
        .iter()
        .find(|s| s.camera == camera.id && s.from_s <= t_s && t_s < s.to_s)
        .and_then(|s| s.lens_mm)
        .unwrap_or(camera.lens_mm)
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
    use klappe_model::{Movement, ShotSize};

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

    fn camera(lens_mm: f64) -> Camera {
        Camera {
            id: "a".into(),
            label: "A".into(),
            sensor: Sensor::Super35,
            lens_mm,
            aspect: 2.39,
            marks: vec![],
            follows: None,
        }
    }

    fn shot(camera: &str, lens_mm: Option<f64>, from_s: f64, to_s: f64) -> Shot {
        Shot {
            id: "s".into(),
            number: "1.1".into(),
            camera: camera.into(),
            size: ShotSize::Wide,
            movement: Movement::Static,
            lens_mm,
            from_s,
            to_s,
            description: String::new(),
            props: String::new(),
            gear: String::new(),
            crew: String::new(),
            note: String::new(),
        }
    }

    #[test]
    fn lens_comes_from_the_covering_shot() {
        let shots = [
            shot("b", Some(85.0), 0.0, 10.0),
            shot("a", None, 0.0, 4.0),
            shot("a", Some(40.0), 0.0, 4.0),
            shot("a", Some(50.0), 4.0, 8.0),
        ];
        let cam = camera(35.0);

        // First shot in order wins, even without its own lens: falls back to the camera.
        assert_eq!(lens_at(&cam, &shots, 2.0), 35.0);
        assert_eq!(lens_at(&cam, &shots[2..], 2.0), 40.0);
        assert_eq!(lens_at(&cam, &shots[2..], 4.0), 50.0);
        assert_eq!(lens_at(&cam, &shots, 9.0), 35.0);
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
