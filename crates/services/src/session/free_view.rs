//! The free 3D view (LH-94): the plan's set seen from any side. Plan and 3D are two views of
//! one set, not two modes: the session keeps which one shows and where the 3D view looks;
//! tools, selection and undo stay the plan's.
//!
//! ```text
//!   J ──► View::Plan ⇄ View::Free
//!   drag (turn, pan), wheel (zoom to the pointer) ──► Orbit
//!   pixel ──► floor_at (a plan point for the tools) │ source_at (what a click takes)
//! ```

use klappe_model::Point;
use klappe_stellprobe::{
    floor_at, grab_at, level_at, picture, source_at, Eye, Orbit, Picture, Source,
};

use super::Session;

/// Which view of the set the window shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum View {
    #[default]
    Plan,
    Free,
}

/// A picture's size in pixels: width and height.
pub type Size = (f64, f64);

impl Session {
    pub fn view(&self) -> View {
        self.view
    }

    /// Switches between plan and 3D (key J). The first switch to 3D looks at the whole set.
    pub fn toggle_view(&mut self) {
        self.view = match self.view {
            View::Plan => View::Free,
            View::Free => View::Plan,
        };
        if self.view == View::Free && self.orbit.is_none() {
            self.orbit = self.fitting_orbit();
        }
    }

    /// Turns the 3D view by a drag of `dx`, `dy` pixels.
    pub fn turn_view(&mut self, dx: f64, dy: f64) {
        if let Some(orbit) = self.orbit.as_mut() {
            orbit.turn(dx, dy);
        }
    }

    /// Shifts the 3D view by a drag of `dx`, `dy` pixels in a picture of `size`.
    pub fn pan_view(&mut self, dx: f64, dy: f64, size: Size) {
        if let Some(orbit) = self.orbit.as_mut() {
            orbit.pan(dx, dy, size.0);
        }
    }

    /// Wheel `steps` (positive comes closer) towards the floor under `pointer`.
    pub fn zoom_view(&mut self, steps: f64, pointer: Size, size: Size) {
        let towards = self.free_floor_at(pointer, size);
        if let Some(orbit) = self.orbit.as_mut() {
            orbit.zoom(steps, towards);
        }
    }

    /// The 3D view at the playhead, `size` in pixels; `None` before the first switch to 3D,
    /// without a plan, or at no size.
    pub fn free_picture(&self, size: Size) -> Option<Picture> {
        let (plan, set) = (self.plan()?, &self.scene_location()?.set);

        picture(
            &self.project,
            set,
            plan,
            &self.free_eye(size)?,
            size.0 as usize,
        )
    }

    /// The plan point the floor shows under `pointer` in 3D: where a tool sets or drags to.
    pub fn free_floor_at(&self, pointer: Size, size: Size) -> Option<Point> {
        floor_at(self.plan()?, &self.free_eye(size)?, size.0, pointer)
    }

    /// What a click at `pointer` takes in 3D: the element seen there, as on the plan.
    pub fn free_pick(&self, pointer: Size, size: Size) -> Option<Source> {
        let picture = self.free_picture(size)?;

        source_at(&picture, (pointer.0 as usize, pointer.1 as usize)).cloned()
    }

    /// What a drag starting at `pointer` takes in 3D, and at which height above the floor it
    /// holds it: the drag then runs on that level (`free_level_at`).
    pub fn free_grab(&self, pointer: Size, size: Size) -> Option<(Source, f64)> {
        let picture = self.free_picture(size)?;
        let pixel = (pointer.0 as usize, pointer.1 as usize);

        grab_at(self.plan()?, &self.free_eye(size)?, &picture, pixel)
    }

    /// The plan point under `pointer` on the level `height_m` above the floor.
    pub fn free_level_at(&self, pointer: Size, size: Size, height_m: f64) -> Option<Point> {
        level_at(
            self.plan()?,
            &self.free_eye(size)?,
            size.0,
            pointer,
            height_m,
        )
    }

    fn free_eye(&self, (width, height): Size) -> Option<Eye> {
        (width > 0.0 && height > 0.0).then_some(())?;

        Some(Eye::Orbit {
            orbit: self.orbit?,
            t_s: self.time_s,
            aspect: width / height,
        })
    }

    fn fitting_orbit(&self) -> Option<Orbit> {
        let set = &self.scene_location()?.set;

        Some(Orbit::fitting(set.width_m, set.depth_m))
    }
}
