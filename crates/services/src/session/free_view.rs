//! The free 3D view (LH-94): the plan's set seen from any side. Plan and 3D are two views of
//! one set, not two modes: the session keeps which one shows and where the 3D view looks;
//! tools, selection and undo stay the plan's.
//!
//! ```text
//!   J ──► View::Plan ⇄ View::Free
//!   drag (turn, pan), wheel (zoom to the pointer) ──► Orbit
//!   Y ──► Cut (walls end at the chest, LH-98)
//!   look through a camera (LH-99) ──► Eye::Shot at the playhead; a turn goes back to Orbit
//!   pixel ──► floor_at (a plan point for the tools) │ source_at (what a click takes)
//! ```

use klappe_model::Point;
use klappe_stellprobe::{
    floor_at, grab_at, label_point, level_at, lift_at, object_extent, picture, source_at, Cut, Eye,
    Orbit, Picture, Source,
};

use super::{Session, Through};
use crate::library;

/// Which view of the set the window shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum View {
    #[default]
    Plan,
    Free,
}

/// A picture's size in pixels: width and height.
pub type Size = (f64, f64);

/// A key's turn of the 3D view, in degrees.
const KEY_TURN_DEG: f64 = 15.0;

/// A key's shift of the 3D view, as a share of the picture's width.
const KEY_PAN_SHARE: f64 = 0.1;

/// A key's zoom: one step of the wheel.
const KEY_ZOOM_STEPS: f64 = 1.0;

/// What a key does to the 3D view. Each is the drag of the same name: `Left` is a drag to the
/// left, so the turn brings the set's right side round and the shift moves the set left.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewStep {
    TurnLeft,
    TurnRight,
    TurnUp,
    TurnDown,
    PanLeft,
    PanRight,
    PanUp,
    PanDown,
    ZoomIn,
    ZoomOut,
}

impl ViewStep {
    /// The step `name` (`turnLeft`, `panUp`, `zoomIn` …) names; `None` for any other text.
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "turnLeft" => ViewStep::TurnLeft,
            "turnRight" => ViewStep::TurnRight,
            "turnUp" => ViewStep::TurnUp,
            "turnDown" => ViewStep::TurnDown,
            "panLeft" => ViewStep::PanLeft,
            "panRight" => ViewStep::PanRight,
            "panUp" => ViewStep::PanUp,
            "panDown" => ViewStep::PanDown,
            "zoomIn" => ViewStep::ZoomIn,
            "zoomOut" => ViewStep::ZoomOut,
            _ => return None,
        })
    }
}

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

    /// The wall cut of the 3D view.
    pub fn cut(&self) -> Cut {
        self.cut
    }

    /// Cuts the walls of the 3D view down to the chest, or back (key Y).
    pub fn toggle_cut(&mut self) {
        self.cut = match self.cut {
            Cut::Off => Cut::Chest,
            Cut::Chest => Cut::Off,
        };
    }

    /// Whether the 3D view looks through a camera of the plan.
    pub fn looks_through(&self) -> bool {
        self.through_camera().is_some()
    }

    /// The camera looked through, while its plan is on view and it still has marks.
    fn through_camera(&self) -> Option<&str> {
        let through = self.through.as_ref()?;
        let on_view = self.scene()?.id == through.scene && self.plan()?.id == through.plan;
        let camera = self.plan()?.camera(&through.camera)?;

        (on_view && !camera.marks.is_empty()).then_some(through.camera.as_str())
    }

    /// Ends a look that no longer holds (other plan, camera gone) for good, so it cannot
    /// come back with the plan, an undo or a new camera of the same id.
    pub(super) fn end_stale_look(&mut self) {
        if self.through_camera().is_none() {
            self.through = None;
        }
    }

    /// In 3D, looks through the camera of the selected shot, else through `fallback` (the
    /// camera picked on the plan), or back to the orbit when it already does. The view follows
    /// the playhead. Returns whether it looks through a camera now; a plan, no camera or one
    /// without marks (it would see nothing) change nothing.
    pub fn look_through(&mut self, fallback: Option<&str>) -> bool {
        if self.view != View::Free {
            return false;
        }
        if self.through_camera().is_some() {
            self.through = None;
            return false;
        }

        let camera = self.shot_camera(fallback);
        let seen = camera
            .as_deref()
            .and_then(|id| self.plan()?.camera(id))
            .is_some_and(|c| !c.marks.is_empty());
        if !seen {
            return false;
        }

        self.through = camera
            .zip(self.scene().zip(self.plan()))
            .map(|(camera, (s, p))| Through {
                scene: s.id.clone(),
                plan: p.id.clone(),
                camera,
            });

        true
    }

    /// Turns the 3D view by a drag of `dx`, `dy` pixels.
    pub fn turn_view(&mut self, dx: f64, dy: f64) {
        self.through = None;
        if let Some(orbit) = self.orbit.as_mut() {
            orbit.turn(dx, dy);
        }
    }

    /// Shifts the 3D view by a drag of `dx`, `dy` pixels in a picture of `size`.
    pub fn pan_view(&mut self, dx: f64, dy: f64, size: Size) {
        self.through = None;
        if let Some(orbit) = self.orbit.as_mut() {
            orbit.pan(dx, dy, size.0);
        }
    }

    /// The keyboard's way to turn, shift and zoom the 3D view: a fixed step per key, zooming to
    /// the middle of the picture of `size`. The plan and a view before its first switch to 3D
    /// change nothing.
    pub fn step_view(&mut self, step: ViewStep, size: Size) {
        if self.view != View::Free {
            return;
        }

        let pan_px = size.0 * KEY_PAN_SHARE;
        let middle = (size.0 / 2.0, size.1 / 2.0);
        match step {
            ViewStep::TurnLeft => self.turn_deg(KEY_TURN_DEG, 0.0),
            ViewStep::TurnRight => self.turn_deg(-KEY_TURN_DEG, 0.0),
            ViewStep::TurnUp => self.turn_deg(0.0, -KEY_TURN_DEG),
            ViewStep::TurnDown => self.turn_deg(0.0, KEY_TURN_DEG),
            ViewStep::PanLeft => self.pan_view(-pan_px, 0.0, size),
            ViewStep::PanRight => self.pan_view(pan_px, 0.0, size),
            ViewStep::PanUp => self.pan_view(0.0, -pan_px, size),
            ViewStep::PanDown => self.pan_view(0.0, pan_px, size),
            ViewStep::ZoomIn => self.zoom_view(KEY_ZOOM_STEPS, middle, size),
            ViewStep::ZoomOut => self.zoom_view(-KEY_ZOOM_STEPS, middle, size),
        }
    }

    fn turn_deg(&mut self, yaw_deg: f64, pitch_deg: f64) {
        self.through = None;
        if let Some(orbit) = self.orbit.as_mut() {
            orbit.turn_deg(yaw_deg, pitch_deg);
        }
    }

    /// In 3D, turns the view's middle to object `id` at half its height, so the view follows
    /// it (LH-93); distance and angle stay. The plan and an unknown id change nothing.
    pub fn aim_at_object(&mut self, id: &str) {
        if self.view != View::Free {
            return;
        }
        let Some(set) = self.set_now() else {
            return;
        };
        let set = &*set;
        let Some(index) = set.objects.iter().position(|o| o.id == id) else {
            return;
        };

        let (base_m, top_m) = object_extent(set, index);
        let at = label_point(&set.objects[index]);
        let target = [at.x_m, at.y_m, (base_m + top_m) / 2.0];
        if let Some(orbit) = self.orbit.as_mut() {
            orbit.aim(target);
        }
    }

    /// Wheel `steps` (positive comes closer) towards the floor under `pointer`.
    pub fn zoom_view(&mut self, steps: f64, pointer: Size, size: Size) {
        // Leaving the camera: the zoom goes on from the last orbit.
        self.through = None;
        let towards = self.free_floor_at(pointer, size);
        if let Some(orbit) = self.orbit.as_mut() {
            orbit.zoom(steps, towards);
        }
    }

    /// The 3D view at the playhead, `size` in pixels; `None` before the first switch to 3D,
    /// without a plan, or at no size.
    pub fn free_picture(&self, size: Size) -> Option<Picture> {
        let (plan, set) = (self.plan()?, &self.scene_location()?.set);

        let forms = library::forms_of(&self.project.library, set);

        picture(
            &self.project,
            set,
            plan,
            &forms,
            &self.free_eye(size)?,
            size.0 as usize,
            self.cut,
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

    /// How far a drag from `from` to `to` lifts what it grabbed at `held_m` above the floor
    /// (height drag in 3D, LH-95); negative: lowers. `None` when `from` shows no point of
    /// that level.
    pub fn free_lift(&self, from: Size, to: Size, size: Size, held_m: f64) -> Option<f64> {
        lift_at(
            self.plan()?,
            &self.free_eye(size)?,
            size.0,
            from,
            to,
            held_m,
        )
    }

    fn free_eye(&self, (width, height): Size) -> Option<Eye> {
        (width > 0.0 && height > 0.0).then_some(())?;

        let aspect = width / height;
        if let Some(camera) = self.through_camera() {
            return Some(Eye::Shot {
                camera: camera.to_owned(),
                t_s: self.time_s,
                aspect: Some(aspect),
            });
        }

        Some(Eye::Orbit {
            orbit: self.orbit?,
            t_s: self.time_s,
            aspect,
        })
    }

    fn fitting_orbit(&self) -> Option<Orbit> {
        let set = &self.scene_location()?.set;

        Some(Orbit::fitting(set.width_m, set.depth_m))
    }
}
