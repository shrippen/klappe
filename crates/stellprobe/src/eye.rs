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

    /// Where it stands on the plan.
    pub(crate) fn ground(&self) -> [f64; 2] {
        [self.at[0], self.at[1]]
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

    /// Where the horizon lies in the picture.
    pub(crate) fn horizon_y(&self) -> f64 {
        self.frame_height / 2.0 + self.tilt.0 / self.tilt.1 * self.focal
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
