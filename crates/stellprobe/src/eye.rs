//! Where a view looks from (3D-KONZEPT.md, section 4.3): the camera of a shot or the free 3D
//! view around a point of the set. A pinhole turns points of the set into points of the
//! picture, and a pixel back into a point on the floor.
//!
//! ```text
//!   set (x, y on the plan, z up) ──local──► camera (right, up, ahead) ──image──► picture
//!                                                                    x 0..1, y 0..1/aspect
//! ```

use klappe_model::{Camera, FloorPlan, Point};

use crate::motion::{aim_at, camera_at};
use crate::optics::{horizontal_fov_deg, lens_at};

/// Picture width; the height is `1 / aspect`.
pub(crate) const FRAME_WIDTH: f64 = 1.0;
/// Nothing nearer than this is drawn.
pub(crate) const NEAR_M: f64 = 0.1;
/// Keeps `tan` finite when the camera looks straight up or down.
const MAX_TILT_DEG: f64 = 89.0;

pub(crate) type Vec3 = [f64; 3];

/// What a view looks through.
#[derive(Debug, Clone, PartialEq)]
pub enum Eye {
    /// Camera `camera` of the plan at `t_s` (LH-36), its picture `aspect` wide to high: the
    /// camera's own when `None`; the shape of the 3D view when looking through it (LH-99),
    /// which keeps the camera's width of view.
    Shot {
        camera: String,
        t_s: f64,
        aspect: Option<f64>,
    },
    /// The free 3D view (LH-94) on the set at `t_s`, `aspect` wide to high.
    Orbit { orbit: Orbit, t_s: f64, aspect: f64 },
}

impl Eye {
    /// The moment the figures stand at.
    pub(crate) fn t_s(&self) -> f64 {
        match self {
            Eye::Shot { t_s, .. } | Eye::Orbit { t_s, .. } => *t_s,
        }
    }
}

/// The free 3D view: it looks at `target` from `distance_m` away, turned `yaw_deg` about it
/// (0 looks along +x of the plan, 90 along +y) and `pitch_deg` above the floor.
///
/// ```text
///          eye
///           ╲  pitch
///   ─────────●─────── floor        seen from above:  eye ──yaw──► target
///         target
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Orbit {
    target: [f64; 3],
    yaw_deg: f64,
    pitch_deg: f64,
    distance_m: f64,
    /// How wide it sees, left to right.
    fov_deg: f64,
}

/// Pitch of a new view: steep enough to see into a room over its walls.
const START_PITCH_DEG: f64 = 55.0;
/// Yaw of a new view: from the plan's bottom left (the audience) towards its top right.
const START_YAW_DEG: f64 = -45.0;
const START_FOV_DEG: f64 = 50.0;
/// Pitch stays between grazing and straight down, so up stays up.
const MIN_PITCH_DEG: f64 = 2.0;
const MAX_PITCH_DEG: f64 = 89.0;
/// Turn per pixel dragged.
const TURN_DEG_PER_PX: f64 = 0.3;
/// One wheel step brings the eye this much closer.
const ZOOM_STEP: f64 = 0.85;
const MIN_DISTANCE_M: f64 = 0.5;
const MAX_DISTANCE_M: f64 = 2000.0;

impl Orbit {
    /// The view a set of `width_m` by `depth_m` opens with: its middle, all of it in sight.
    pub fn fitting(width_m: f64, depth_m: f64) -> Self {
        let size_m = width_m.hypot(depth_m).max(1.0);
        let half_fov = (START_FOV_DEG / 2.0).to_radians();

        Self {
            target: [width_m / 2.0, depth_m / 2.0, 0.0],
            yaw_deg: START_YAW_DEG,
            pitch_deg: START_PITCH_DEG,
            distance_m: size_m / 2.0 / half_fov.tan() * 1.1,
            fov_deg: START_FOV_DEG,
        }
    }

    /// Looks at `target` from where it looked before: only the middle moves; turn and
    /// distance stay.
    pub fn aim(&mut self, target: [f64; 3]) {
        self.target = target;
    }

    /// Turns the view by a drag of `dx_px`, `dy_px`: sideways about the target, up and down
    /// between grazing and straight down.
    pub fn turn(&mut self, dx_px: f64, dy_px: f64) {
        self.yaw_deg = (self.yaw_deg - dx_px * TURN_DEG_PER_PX).rem_euclid(360.0);
        self.pitch_deg =
            (self.pitch_deg + dy_px * TURN_DEG_PER_PX).clamp(MIN_PITCH_DEG, MAX_PITCH_DEG);
    }

    /// Shifts the view by a drag of `dx_px`, `dy_px` in a picture `width_px` wide: the floor
    /// under the pointer goes along with it, roughly.
    pub fn pan(&mut self, dx_px: f64, dy_px: f64, width_px: f64) {
        let metres_per_px =
            2.0 * self.distance_m * (self.fov_deg / 2.0).to_radians().tan() / width_px.max(1.0);
        let (sin, cos) = self.yaw_deg.to_radians().sin_cos();
        // Dragging right moves the set right: the target moves left.
        let (right, ahead) = (-dx_px * metres_per_px, dy_px * metres_per_px);

        self.target[0] += right * -sin + ahead * cos;
        self.target[1] += right * cos + ahead * sin;
    }

    /// Wheel `steps` (positive comes closer) towards `towards`, the floor under the pointer,
    /// which stays under it.
    pub fn zoom(&mut self, steps: f64, towards: Option<Point>) {
        let factor = ZOOM_STEP.powf(steps);
        let distance_m = (self.distance_m * factor).clamp(MIN_DISTANCE_M, MAX_DISTANCE_M);
        let moved = 1.0 - distance_m / self.distance_m;
        self.distance_m = distance_m;

        let Some(point) = towards else {
            return;
        };
        self.target[0] += (point.x_m - self.target[0]) * moved;
        self.target[1] += (point.y_m - self.target[1]) * moved;
    }
}

/// A camera at one moment: where it stands, where it looks, how wide it sees.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Pinhole {
    at: Vec3,
    forward: [f64; 2],
    pub(crate) right: [f64; 2],
    /// Sine and cosine of the tilt; positive looks up.
    tilt: (f64, f64),
    focal: f64,
    pub(crate) frame_height: f64,
}

impl Pinhole {
    /// The pinhole of `eye`; `None` for an unknown camera, one without marks, or one that sees
    /// nothing (no angle, no aspect).
    pub(crate) fn of(eye: &Eye, plan: &FloorPlan) -> Option<Self> {
        match eye {
            Eye::Shot {
                camera,
                t_s,
                aspect,
            } => Self::of_camera(plan.camera(camera)?, plan, *t_s, *aspect),
            Eye::Orbit { orbit, aspect, .. } => Self::of_orbit(orbit, *aspect),
        }
    }

    fn of_orbit(orbit: &Orbit, aspect: f64) -> Option<Self> {
        if orbit.fov_deg <= 0.0 || aspect <= 0.0 {
            return None;
        }

        let (sin, cos) = orbit.yaw_deg.to_radians().sin_cos();
        let pitch = orbit
            .pitch_deg
            .clamp(MIN_PITCH_DEG, MAX_PITCH_DEG)
            .to_radians();
        let (back, up) = (
            orbit.distance_m * pitch.cos(),
            orbit.distance_m * pitch.sin(),
        );
        let [x, y, z] = orbit.target;

        Some(Self {
            at: [x - back * cos, y - back * sin, z + up],
            forward: [cos, sin],
            right: [-sin, cos],
            tilt: (-pitch).sin_cos(),
            focal: FRAME_WIDTH / 2.0 / (orbit.fov_deg.to_radians() / 2.0).tan(),
            frame_height: FRAME_WIDTH / aspect,
        })
    }

    fn of_camera(camera: &Camera, plan: &FloorPlan, t_s: f64, aspect: Option<f64>) -> Option<Self> {
        let place = camera_at(camera, t_s)?;
        let (height_m, tilt_deg) = aim_at(camera, t_s)?;
        let fov = horizontal_fov_deg(camera.sensor, lens_at(camera, &plan.shots, t_s));
        let aspect = aspect.unwrap_or(camera.aspect);
        if fov <= 0.0 || aspect <= 0.0 {
            return None;
        }

        let (sin, cos) = place.facing_deg.to_radians().sin_cos();
        let tilt = tilt_deg.clamp(-MAX_TILT_DEG, MAX_TILT_DEG).to_radians();

        Some(Self {
            at: [place.at.x_m, place.at.y_m, height_m],
            forward: [cos, sin],
            // On the plan y grows downwards, so the right hand of one facing +x is +y.
            right: [-sin, cos],
            tilt: tilt.sin_cos(),
            focal: FRAME_WIDTH / 2.0 / (fov.to_radians() / 2.0).tan(),
            frame_height: FRAME_WIDTH / aspect,
        })
    }

    /// `p` in the camera's frame: right, up, ahead.
    pub(crate) fn local(&self, p: Vec3) -> Vec3 {
        let (dx, dy, up) = (p[0] - self.at[0], p[1] - self.at[1], p[2] - self.at[2]);
        let ahead = dx * self.forward[0] + dy * self.forward[1];
        let (sin, cos) = self.tilt;

        [
            dx * self.right[0] + dy * self.right[1],
            up * cos - ahead * sin,
            ahead * cos + up * sin,
        ]
    }

    /// Where the ray through picture point `image` meets the floor; `None` when it does not
    /// (above the horizon).
    pub(crate) fn floor_at(&self, image: [f64; 2]) -> Option<Point> {
        self.level_at(image, 0.0)
    }

    /// Where the ray through picture point `image` meets the level `height_m` above the floor,
    /// as a plan point; `None` when it runs away from that level.
    pub(crate) fn level_at(&self, image: [f64; 2], height_m: f64) -> Option<Point> {
        let ray = self.ray(image);
        let rise_m = height_m - self.at[2];
        if ray[2] == 0.0 || rise_m / ray[2] <= 0.0 {
            return None;
        }

        let t = rise_m / ray[2];
        Some(Point::new(self.at[0] + t * ray[0], self.at[1] + t * ray[1]))
    }

    /// How far above the level `held_m` the ray through `to` meets the vertical plane through
    /// the point of that level shown at `from`; the plane stands across the view direction
    /// (as seen from above), so a pointer that moves up lifts the point and sideways changes
    /// nothing. Negative: lower. `None` when `from` shows nothing of that level, or the ray
    /// runs away from the plane.
    ///
    /// ```text
    ///   eye ·········· ray(to)         plane across the view
    ///       ╲         ╲ ·             │
    ///        ╲ ray(from)●── z ────────┤  lift = z - held
    ///         ╲        │            │
    ///          ●───────●  held_m   │
    /// ```
    pub(crate) fn lift(&self, from: [f64; 2], to: [f64; 2], held_m: f64) -> Option<f64> {
        let grabbed = self.level_at(from, held_m)?;
        let ahead = [grabbed.x_m - self.at[0], grabbed.y_m - self.at[1]];
        let distance = ahead[0] * self.forward[0] + ahead[1] * self.forward[1];
        let ray = self.ray(to);
        let pace = ray[0] * self.forward[0] + ray[1] * self.forward[1];
        if pace <= 0.0 {
            return None;
        }

        Some(self.at[2] + distance / pace * ray[2] - held_m)
    }

    /// The point of the set picture point `image` shows at `depth_m` ahead (as the frame stores
    /// depth).
    pub(crate) fn point_at(&self, image: [f64; 2], depth_m: f64) -> Vec3 {
        let ray = self.ray(image);

        [
            self.at[0] + depth_m * ray[0],
            self.at[1] + depth_m * ray[1],
            self.at[2] + depth_m * ray[2],
        ]
    }

    /// The ray through picture point `image` in the set's frame, 1 ahead of the camera.
    fn ray(&self, image: [f64; 2]) -> Vec3 {
        let right = (image[0] - FRAME_WIDTH / 2.0) / self.focal;
        let rise = (self.frame_height / 2.0 - image[1]) / self.focal;
        let (sin, cos) = self.tilt;
        let ahead = cos - rise * sin;

        [
            right * self.right[0] + ahead * self.forward[0],
            right * self.right[1] + ahead * self.forward[1],
            rise * cos + sin,
        ]
    }

    /// The picture point of a point in the camera's frame; its depth is above `NEAR_M`.
    pub(crate) fn image(&self, p: Vec3) -> [f64; 2] {
        [
            FRAME_WIDTH / 2.0 + p[0] / p[2] * self.focal,
            self.frame_height / 2.0 - p[1] / p[2] * self.focal,
        ]
    }
}

/// The part of `polygon` (in the camera's frame) at or beyond the near plane.
pub(crate) fn clip_near(polygon: &[Vec3]) -> Vec<Vec3> {
    clip(
        polygon,
        |p| p[2] - NEAR_M,
        |a, b| {
            let t = (NEAR_M - a[2]) / (b[2] - a[2]);
            [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, NEAR_M]
        },
    )
}

/// Sutherland-Hodgman: keeps the part of `polygon` where `side` is not negative; `cut` gives
/// the point where an edge crosses the line.
pub(crate) fn clip<const N: usize>(
    polygon: &[[f64; N]],
    side: impl Fn(&[f64; N]) -> f64,
    cut: impl Fn(&[f64; N], &[f64; N]) -> [f64; N],
) -> Vec<[f64; N]> {
    let mut kept = Vec::new();
    for (i, a) in polygon.iter().enumerate() {
        let b = &polygon[(i + 1) % polygon.len()];
        let (a_in, b_in) = (side(a) >= 0.0, side(b) >= 0.0);
        if a_in {
            kept.push(*a);
        }
        if a_in != b_in {
            kept.push(cut(a, b));
        }
    }

    kept
}

#[cfg(test)]
mod tests {
    use klappe_model::{CameraMark, Point, Sensor};

    use super::*;

    /// Image numbers worked out by hand. Camera (10, 10) facing -90 degrees, 1.6 m high unless
    /// said, Super 35, 35 mm, aspect 1: f = 35 / 24.89 = 1.40619 of the frame width. A standing
    /// figure 5 m ahead is 0.5 m wide and 1.75 m tall:
    ///   feet = 0.5 + 1.6 / 5 * f                  = 0.94998
    ///   head = 0.5 - (1.75 - 1.6) / 5 * f         = 0.45781
    ///   horizon with tilt t: 0.5 + tan(t) * f      = 0.74795 (+10), 0.25205 (-10)
    ///   feet with tilt -10: depth 5 cos10 - 1.6 sin10 = 5.20188, up -1.6 cos10 + 5 sin10 = -0.70746
    ///                       y = 0.5 + 0.70746 / 5.20188 * f = 0.69124
    const TOLERANCE: f64 = 1e-5;
    /// Far enough that a point at the camera's height lies on the horizon.
    const FAR_M: f64 = 1.0e7;

    /// Aiming moves the target and nothing else.
    #[test]
    fn aim_keeps_turn_and_distance() {
        let mut orbit = Orbit::fitting(10.0, 6.0);
        orbit.turn(30.0, 10.0);
        let kept = orbit;
        orbit.aim([2.0, 3.0, 0.5]);

        assert_eq!(orbit.target, [2.0, 3.0, 0.5]);
        assert_eq!(
            (
                orbit.yaw_deg,
                orbit.pitch_deg,
                orbit.distance_m,
                orbit.fov_deg
            ),
            (kept.yaw_deg, kept.pitch_deg, kept.distance_m, kept.fov_deg)
        );
    }

    fn plan_with(height_m: f64, tilt_deg: f64, aspect: f64) -> FloorPlan {
        let mut plan = FloorPlan::new("p", "Plan");
        plan.cameras = vec![Camera {
            id: "a".into(),
            label: "A".into(),
            sensor: Sensor::Super35,
            lens_mm: 35.0,
            aspect,
            marks: vec![CameraMark {
                at_s: 0.0,
                at: Point::new(10.0, 10.0),
                facing_deg: -90.0,
                height_m,
                tilt_deg,
                hold_s: 0.0,
                via: Vec::new(),
                curve: Default::default(),
                rig: Default::default(),
                lens_mm: None,
            }],
            follows: None,
            color: None,
        }];
        plan
    }

    fn pinhole(height_m: f64, tilt_deg: f64, aspect: f64) -> Pinhole {
        let eye = Eye::Shot {
            aspect: None,
            camera: "a".into(),
            t_s: 0.0,
        };

        Pinhole::of(&eye, &plan_with(height_m, tilt_deg, aspect)).expect("pinhole")
    }

    /// Where the picture shows `p` (plan metres, height up).
    fn at(pinhole: &Pinhole, p: Vec3) -> [f64; 2] {
        pinhole.image(pinhole.local(p))
    }

    /// The row of the horizon: a point of the camera's height, far ahead.
    fn horizon(pinhole: &Pinhole, height_m: f64) -> f64 {
        at(pinhole, [10.0, 10.0 - FAR_M, height_m])[1]
    }

    #[test]
    fn level_camera_has_its_horizon_in_the_middle() {
        let level = pinhole(1.6, 0.0, 1.0);

        assert!((horizon(&level, 1.6) - 0.5).abs() < TOLERANCE);
    }

    #[test]
    fn tilting_up_puts_the_horizon_at_0_74795() {
        let up = pinhole(1.6, 10.0, 1.0);

        assert!((horizon(&up, 1.6) - 0.74795).abs() < TOLERANCE);
    }

    #[test]
    fn tilting_down_puts_the_horizon_at_0_25205_and_the_feet_at_0_69124() {
        let down = pinhole(1.6, -10.0, 1.0);

        assert!((horizon(&down, 1.6) - 0.25205).abs() < TOLERANCE);
        assert!((at(&down, [10.0, 5.0, 0.0])[1] - 0.69124).abs() < TOLERANCE);
    }

    /// Camera 1.6 m high, level, f = 1.40619. The floor point 5 m ahead shows at y 0.94998; the
    /// point 1 m above it at 0.5 + 0.6 / 5 * f = 0.668742, 2 m above at 0.5 - 0.4 / 5 * f =
    /// 0.387505, and the camera's own height on the horizon, 0.5.
    #[test]
    fn a_pointer_up_lifts_the_grabbed_point_by_what_the_plane_shows() {
        let level = pinhole(1.6, 0.0, 1.0);
        let from = [0.5, 0.949980];

        let one = level.lift(from, [0.5, 0.668742], 0.0).expect("lift");
        let two = level.lift(from, [0.5, 0.387505], 0.0).expect("lift");
        let none = level.lift(from, from, 0.0).expect("lift");
        let level_with_eye = level.lift(from, [0.5, 0.5], 0.0).expect("lift");

        assert!((one - 1.0).abs() < TOLERANCE, "one {one}");
        assert!((two - 2.0).abs() < TOLERANCE, "two {two}");
        assert!(none.abs() < TOLERANCE, "none {none}");
        assert!((level_with_eye - 1.6).abs() < TOLERANCE, "{level_with_eye}");
    }

    /// Held 1 m up, the pointer on the point itself lifts nothing; sideways changes nothing.
    #[test]
    fn a_pointer_sideways_lifts_nothing_and_none_above_the_horizon() {
        let level = pinhole(1.6, 0.0, 1.0);
        let held = [0.5, 0.668742];

        let sideways = level.lift(held, [0.7, 0.668742], 1.0).expect("lift");

        assert!(sideways.abs() < TOLERANCE, "sideways {sideways}");
        // The grabbed level lies below the eye: a start above the horizon meets none.
        assert!(level.lift([0.5, 0.3], [0.5, 0.2], 1.0).is_none());
    }

    #[test]
    fn a_standing_figure_has_the_hand_computed_image() {
        let level = pinhole(1.6, 0.0, 1.0);

        let [left, feet] = at(&level, [9.75, 5.0, 0.0]);
        let [right, head] = at(&level, [10.25, 5.0, 1.75]);

        assert!((left - 0.42969).abs() < TOLERANCE, "left {left}");
        assert!((right - 0.57031).abs() < TOLERANCE, "right {right}");
        assert!((head - 0.45781).abs() < TOLERANCE, "head {head}");
        assert!((feet - 0.94998).abs() < TOLERANCE, "feet {feet}");
    }

    #[test]
    fn height_alone_keeps_the_horizon_but_lifts_the_feet() {
        let (low, high) = (pinhole(1.0, 0.0, 1.0), pinhole(2.0, 0.0, 1.0));

        assert!((horizon(&low, 1.0) - horizon(&high, 2.0)).abs() < TOLERANCE);
        assert!(at(&high, [10.0, 5.0, 0.0])[1] > at(&low, [10.0, 5.0, 0.0])[1]);
    }

    #[test]
    fn aspect_sets_the_height() {
        assert!((pinhole(1.6, 0.0, 2.0).frame_height - 0.5).abs() < 1e-9);
        assert!((pinhole(1.6, 0.0, 1.0).frame_height - 1.0).abs() < 1e-9);
    }

    #[test]
    fn unknown_camera_has_no_pinhole() {
        let eye = Eye::Shot {
            aspect: None,
            camera: "x".into(),
            t_s: 0.0,
        };

        assert!(Pinhole::of(&eye, &plan_with(1.6, 0.0, 1.0)).is_none());
    }

    /// An orbit eye worked out by hand: target (0, 0, 0), yaw 0 (looks along +x), pitch 30,
    /// 10 m away, 90 degrees wide, square. The eye stands at (-10 cos 30, 0, 10 sin 30) =
    /// (-8.66025, 0, 5); f = 0.5 / tan 45 = 0.5.
    fn orbit_pinhole() -> Pinhole {
        let orbit = Orbit {
            target: [0.0, 0.0, 0.0],
            yaw_deg: 0.0,
            pitch_deg: 30.0,
            distance_m: 10.0,
            fov_deg: 90.0,
        };

        Pinhole::of_orbit(&orbit, 1.0).expect("pinhole")
    }

    /// The centre ray runs down at 30 degrees: (cos 30, 0, -sin 30) per metre ahead. It meets
    /// the level 2.5 m after t = (2.5 - 5) / -0.5 = 5: x = -8.66025 + 5 cos 30 = -4.33013;
    /// and 5 m ahead it is that same point.
    #[test]
    fn the_centre_ray_meets_a_level_and_a_depth_where_worked_out() {
        let eye = orbit_pinhole();

        let level = eye.level_at([0.5, 0.5], 2.5).expect("level");
        assert!((level.x_m + 4.33013).abs() < TOLERANCE && level.y_m.abs() < TOLERANCE);
        let point = eye.point_at([0.5, 0.5], 5.0);
        assert!((point[0] + 4.33013).abs() < TOLERANCE, "{point:?}");
        assert!(point[1].abs() < TOLERANCE && (point[2] - 2.5).abs() < TOLERANCE);
        assert_eq!(
            eye.floor_at([0.5, 0.5]).map(|p| p.x_m.abs() < TOLERANCE),
            Some(true)
        );
    }

    /// The top edge of the picture: rise 1, so the ray is (cos 30 + sin 30, 0, cos 30 - sin 30)
    /// = (1.36603, 0, 0.36603): it climbs, meets no floor, but the level 8 m (3 m over the eye)
    /// after t = 3 / 0.36603 = 8.19615, at x = -8.66025 + 8.19615 * 1.36603 = 2.53590.
    #[test]
    fn a_ray_over_the_horizon_meets_a_higher_level() {
        let eye = orbit_pinhole();

        assert_eq!(eye.floor_at([0.5, 0.0]), None);
        let level = eye.level_at([0.5, 0.0], 8.0).expect("level");
        assert!((level.x_m - 2.53590).abs() < TOLERANCE, "{level:?}");
        assert_eq!(eye.level_at([0.5, 0.0], 4.0), None);
    }
}
