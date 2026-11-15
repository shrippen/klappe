//! Where a view looks from (3D-KONZEPT.md, section 4.3): today the camera of a shot, later the
//! free 3D view. A pinhole turns points of the set into points of the picture.
//!
//! ```text
//!   set (x, y on the plan, z up) ──local──► camera (right, up, ahead) ──image──► picture
//!                                                                    x 0..1, y 0..1/aspect
//! ```

use klappe_model::{Camera, FloorPlan};

use crate::motion::{aim_at, camera_at};
use crate::optics::{horizontal_fov_deg, lens_at};

/// Picture width; the height is `1 / aspect`.
pub const FRAME_WIDTH: f64 = 1.0;
/// Nothing nearer than this is drawn.
pub(crate) const NEAR_M: f64 = 0.1;
/// Keeps `tan` finite when the camera looks straight up or down.
const MAX_TILT_DEG: f64 = 89.0;

pub(crate) type Vec3 = [f64; 3];

/// What a view looks through.
#[derive(Debug, Clone, PartialEq)]
pub enum Eye {
    /// Camera `camera` of the plan at `t_s` (LH-36).
    Shot { camera: String, t_s: f64 },
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
            Eye::Shot { camera, t_s } => Self::of_camera(plan.camera(camera)?, plan, *t_s),
        }
    }

    fn of_camera(camera: &Camera, plan: &FloorPlan, t_s: f64) -> Option<Self> {
        let place = camera_at(camera, t_s)?;
        let (height_m, tilt_deg) = aim_at(camera, t_s)?;
        let fov = horizontal_fov_deg(camera.sensor, lens_at(camera, &plan.shots, t_s));
        if fov <= 0.0 || camera.aspect <= 0.0 {
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
            frame_height: FRAME_WIDTH / camera.aspect,
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
                rig: Default::default(),
                lens_mm: None,
            }],
            follows: None,
        }];
        plan
    }

    fn pinhole(height_m: f64, tilt_deg: f64, aspect: f64) -> Pinhole {
        let eye = Eye::Shot {
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
            camera: "x".into(),
            t_s: 0.0,
        };

        assert!(Pinhole::of(&eye, &plan_with(1.6, 0.0, 1.0)).is_none());
    }
}
