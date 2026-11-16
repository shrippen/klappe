use klappe_model::{Camera, CameraMark, Point, Sensor, Shot};

use crate::motion::progress;

/// Horizontal angle of view in degrees (LH-31): `2·atan(w / 2f)` with the sensor's active width.
///
/// Example: Super 35 (24.89 mm) with a 35 mm lens sees about 39.1°.
pub fn horizontal_fov_deg(sensor: Sensor, lens_mm: f64) -> f64 {
    if lens_mm <= 0.0 {
        return 0.0;
    }

    (sensor.width_mm() / (2.0 * lens_mm)).atan().to_degrees() * 2.0
}

/// The lens in use at `t_s`: that of the camera's marks when any states one (LH-33), else that
/// of the first shot of this camera covering `t_s`, if it has its own, else the camera's. At
/// the end of a shot nothing else of this camera starts at, that shot still holds (the scene's
/// end is `to_s`).
///
/// Example: camera 35 mm, shot 12.2 from 0 s to 4 s with 40 mm: at 2 s this gives 40.
pub fn lens_at(camera: &Camera, shots: &[Shot], t_s: f64) -> f64 {
    if let Some(lens_mm) = mark_lens_at(camera, t_s) {
        return lens_mm;
    }

    let of_camera = || shots.iter().filter(|s| s.camera == camera.id);
    let covering = of_camera().find(|s| s.from_s <= t_s && t_s < s.to_s);

    // A shot ending exactly at `t_s` holds only when no shot of the camera starts there.
    covering
        .or_else(|| of_camera().find(|s| s.to_s == t_s))
        .and_then(|s| s.lens_mm)
        .unwrap_or(camera.lens_mm)
}

/// The lens of a shot (LH-33): what `lens_at` gives at its start, and the smallest and
/// largest lens within it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShotLens {
    pub from_mm: f64,
    pub min_mm: f64,
    pub max_mm: f64,
    /// The camera's marks state the lens, so the shot's own and the camera's do not count.
    pub from_marks: bool,
}

impl ShotLens {
    /// Smallest and largest lens when they differ: a zoom, also one that ends where it began.
    pub fn span_mm(&self) -> Option<(f64, f64)> {
        (self.max_mm != self.min_mm).then_some((self.min_mm, self.max_mm))
    }
}

/// The lens of `shot` on `camera`: with marks that state a lens, what `lens_at` gives at the
/// shot's start and end and at the marks within it; else the shot's own, else the camera's.
/// The one source for the shot list, the export and the setups.
///
/// Example: marks 35 mm at 0 s and 85 mm at 6 s, shot from 0 s to 6 s: 35 to 85 mm.
pub fn shot_lens(camera: &Camera, shot: &Shot) -> ShotLens {
    let marks = mark_lens_at(camera, shot.from_s).zip(mark_lens_at(camera, shot.to_s));
    if let Some((from_mm, to_mm)) = marks {
        let within = camera
            .marks
            .iter()
            .filter(|m| m.at_s > shot.from_s && m.at_s < shot.to_s)
            .filter_map(|m| m.lens_mm);
        let (min_mm, max_mm) = within.fold(
            (from_mm.min(to_mm), from_mm.max(to_mm)),
            |(min, max), mm| (min.min(mm), max.max(mm)),
        );

        return ShotLens {
            from_mm,
            min_mm,
            max_mm,
            from_marks: true,
        };
    }

    let lens_mm = shot.lens_mm.unwrap_or(camera.lens_mm);

    ShotLens {
        from_mm: lens_mm,
        min_mm: lens_mm,
        max_mm: lens_mm,
        from_marks: false,
    }
}

/// The lens the camera's marks give at `t_s`; `None` when no mark states one. Between two
/// marks with a lens it waits out the pause of the first, then runs linearly to the second,
/// like the place does; before the first and after the last it holds.
///
/// ```text
///  lens   35 ───────┐            ┌────── 85
///                    ╲          ╱
///                     ╲________╱   (linear between the marks)
///  marks  A (35)    hold  ...   B (85)
/// ```
pub(crate) fn mark_lens_at(camera: &Camera, t_s: f64) -> Option<f64> {
    let stated: Vec<(&CameraMark, f64)> = camera
        .marks
        .iter()
        .filter_map(|m| m.lens_mm.map(|lens_mm| (m, lens_mm)))
        .collect();
    let (first, first_mm) = *stated.first()?;
    if t_s < first.at_s {
        return Some(first_mm);
    }

    for pair in stated.windows(2) {
        let ((from, from_mm), (to, to_mm)) = (pair[0], pair[1]);
        if t_s >= to.at_s {
            continue;
        }

        return Some(from_mm + (to_mm - from_mm) * progress(from, to, t_s));
    }

    stated.last().map(|(_, lens_mm)| *lens_mm)
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
            color: None,
        }
    }

    fn shot(camera: &str, lens_mm: Option<f64>, from_s: f64, to_s: f64) -> Shot {
        Shot {
            id: "s".into(),
            number: "1.1".into(),
            camera: camera.into(),
            size: ShotSize::Wide,
            angle: Default::default(),
            movement: Movement::Static,
            lens_mm,
            from_s,
            to_s,
            description: String::new(),
            props: String::new(),
            gear: String::new(),
            crew: String::new(),
            note: String::new(),
            setup_of: None,
            short_name: String::new(),
            image: None,
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
    fn lens_holds_at_the_end_of_the_last_shot() {
        let shots = [
            shot("a", Some(85.0), 14.0, 20.0),
            shot("a", Some(50.0), 20.0, 24.0),
            shot("a", Some(70.0), 30.0, 40.0),
        ];
        let cam = camera(35.0);

        // Scene ends at 40 s: the last shot still holds.
        assert_eq!(lens_at(&cam, &shots[2..], 40.0), 70.0);
        // End of a shot with a successor: the successor starts there.
        assert_eq!(lens_at(&cam, &shots[..2], 20.0), 50.0);
        // End of a shot, nothing follows.
        assert_eq!(lens_at(&cam, &shots[..1], 20.0), 85.0);
        // Past the end.
        assert_eq!(lens_at(&cam, &shots[..1], 21.0), 35.0);
    }

    fn lens_mark(at_s: f64, hold_s: f64, lens_mm: Option<f64>) -> CameraMark {
        CameraMark {
            at_s,
            at: Point::default(),
            facing_deg: 0.0,
            height_m: 1.6,
            tilt_deg: 0.0,
            hold_s,
            via: Vec::new(),
            curve: Default::default(),
            rig: Default::default(),
            lens_mm,
        }
    }

    fn zoom_camera() -> Camera {
        let mut cam = camera(35.0);
        cam.marks = vec![
            lens_mark(0.0, 0.0, None),
            lens_mark(2.0, 2.0, Some(30.0)),
            lens_mark(6.0, 0.0, None),
            lens_mark(10.0, 0.0, Some(70.0)),
            lens_mark(12.0, 0.0, None),
        ];

        cam
    }

    /// The lens runs between two marks that state one, through marks that do not; it holds
    /// before the first and after the last, and during the pause of the first.
    #[test]
    fn lens_runs_linearly_between_marks_with_a_lens() {
        let cam = zoom_camera();

        assert_eq!(lens_at(&cam, &[], 0.0), 30.0);
        assert_eq!(lens_at(&cam, &[], 3.0), 30.0);
        assert_eq!(lens_at(&cam, &[], 4.0), 30.0);
        assert_eq!(lens_at(&cam, &[], 7.0), 50.0);
        assert_eq!(lens_at(&cam, &[], 10.0), 70.0);
        assert_eq!(lens_at(&cam, &[], 20.0), 70.0);
    }

    /// With a lens on a mark, shots and the camera's own lens no longer count.
    #[test]
    fn marks_come_before_shots_and_the_camera() {
        let cam = zoom_camera();
        let shots = [shot("a", Some(85.0), 0.0, 20.0)];

        assert_eq!(lens_at(&cam, &shots, 0.0), 30.0);

        let mut plain = camera(35.0);
        plain.marks = vec![lens_mark(0.0, 0.0, None)];

        assert_eq!(lens_at(&plain, &shots, 1.0), 85.0);
        assert_eq!(lens_at(&plain, &[], 1.0), 35.0);
    }

    /// One value when start and end agree, else the span; the marks decide, else the shot's own
    /// lens, else the camera's.
    #[test]
    fn a_shot_has_one_lens_or_a_span() {
        let cam = zoom_camera();
        let marked = |from_s, to_s| shot_lens(&cam, &shot("a", Some(85.0), from_s, to_s));

        // 30 mm until 4 s, 70 mm from 10 s: the shot's own 85 mm does not count.
        assert_eq!(
            marked(0.0, 4.0),
            ShotLens {
                from_mm: 30.0,
                min_mm: 30.0,
                max_mm: 30.0,
                from_marks: true
            }
        );
        assert_eq!(
            marked(0.0, 10.0),
            ShotLens {
                from_mm: 30.0,
                min_mm: 30.0,
                max_mm: 70.0,
                from_marks: true
            }
        );

        let plain = camera(35.0);
        let shots = [
            shot("a", Some(40.0), 0.0, 4.0),
            shot("a", Some(50.0), 4.0, 8.0),
        ];

        assert_eq!(
            shot_lens(&plain, &shots[0]),
            ShotLens {
                from_mm: 40.0,
                min_mm: 40.0,
                max_mm: 40.0,
                from_marks: false
            }
        );
        assert_eq!(shot_lens(&plain, &shot("a", None, 8.0, 9.0)).from_mm, 35.0);
    }

    /// 35, 85, 35 mm at 0, 5, 10 s: the ends agree, the shot still spans 35 to 85 mm.
    #[test]
    fn a_zoom_in_and_out_spans_the_extremes() {
        let mut cam = camera(50.0);
        cam.marks = vec![
            lens_mark(0.0, 0.0, Some(35.0)),
            lens_mark(5.0, 0.0, Some(85.0)),
            lens_mark(10.0, 0.0, Some(35.0)),
        ];

        let lens = shot_lens(&cam, &shot("a", None, 0.0, 10.0));

        assert_eq!(lens.span_mm(), Some((35.0, 85.0)));
        assert_eq!(lens.from_mm, 35.0);
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
