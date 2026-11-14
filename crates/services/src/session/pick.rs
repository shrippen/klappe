use klappe_model::{Point, Project};
use klappe_stellprobe::{handles, lens_at, render, DisplayList, Source};

use super::layers::layer_of;
use super::{Lock, Session};
use crate::error::ServiceError;
use crate::plan_edit::{self, MarkInfo, PlanEdit, Snapping};

impl Session {
    /// The plan at the playhead, without hidden layers and tracks.
    pub fn display_list(&self) -> DisplayList {
        self.list_of(&self.project, None)
    }

    /// `display_list` with the grips on the bends of the way `selected` belongs to (LH-21).
    pub fn display_list_for(&self, selected: Option<&Source>) -> DisplayList {
        self.list_of(&self.project, selected)
    }

    /// `display_list` of `project`, a changed copy of this session's.
    fn list_of(&self, project: &Project, selected: Option<&Source>) -> DisplayList {
        let scene = project.scenes.get(self.scene);
        let Some((scene, plan)) = scene.and_then(|s| Some((s, s.plans.get(self.plan)?))) else {
            return DisplayList::default();
        };
        let Some(location) = project.location(&scene.location) else {
            return DisplayList::default();
        };

        let background_px = location
            .set
            .background
            .as_ref()
            .and_then(|b| self.image_size(&b.file));
        let mut list = render(project, &location.set, plan, self.time_s, background_px);
        if let Some(selected) = selected {
            list.items.extend(handles(project, plan, selected));
        }
        list.items
            .retain(|item| self.shows(item.role, item.source.as_ref()));
        list
    }

    /// Pixel size of an image of the project folder; remembered per file.
    fn image_size(&self, file: &str) -> Option<(u32, u32)> {
        let stamp = klappe_store::image_stamp(&self.dir, file).ok()?;
        let mut known = self.image_px.borrow_mut();
        if let Some((_, px)) = known.get(file).filter(|(seen, _)| *seen == stamp) {
            return Some(*px);
        }

        let px = klappe_store::image_size(&self.dir, file).ok()?;
        known.insert(file.to_owned(), (stamp, px));

        Some(px)
    }

    /// The lens a camera has at the playhead: its shot's own, else its own.
    pub fn lens_now_mm(&self, camera: &str) -> Option<f64> {
        let plan = self.plan()?;
        let camera = plan.camera(camera)?;
        Some(lens_at(camera, &plan.shots, self.time_s))
    }

    /// What the plan says about the mark `source` names; `None` for anything but a figure's
    /// mark, a figure or a camera (those at the playhead).
    pub fn mark_info(&self, source: &Source) -> Option<MarkInfo> {
        plan_edit::mark_info(self.plan()?, self.time_s, source)
    }

    /// The display list as it would be after `edit`, without committing it or taking an undo
    /// step: what a drag shows while the button is down.
    pub fn preview(&self, edit: &PlanEdit) -> Result<DisplayList, ServiceError> {
        self.preview_for(edit, None)
    }

    /// `preview` with the grips of the way `selected` belongs to, as after the edit.
    pub fn preview_for(
        &self,
        edit: &PlanEdit,
        selected: Option<&Source>,
    ) -> Result<DisplayList, ServiceError> {
        self.check_lock(edit)?;

        let mut next = self.project.clone();
        let at = (self.scene, self.plan);
        // Copying a file is no preview: the list stays as it is until the edit is made.
        if matches!(edit, PlanEdit::SetBackground { .. }) {
            return Ok(self.display_list_for(selected));
        }
        plan_edit::apply(
            &self.dir,
            &mut next,
            at.0,
            at.1,
            self.time_s,
            self.snapping,
            edit,
        )?;
        self.check_cameras(&next)?;

        Ok(self.list_of(&next, selected))
    }

    /// Topmost shown item at `at` (front of the display list first), within the pick radius.
    /// Items of locked layers are not taken.
    pub fn pick(&self, at: Point) -> Option<Source> {
        self.pick_for(at, None)
    }

    /// `pick` with the grips of the way `selected` belongs to in reach: they lie in front.
    pub fn pick_for(&self, at: Point, selected: Option<&Source>) -> Option<Source> {
        let locked = |role| self.lock(layer_of(role)) == Lock::Locked;

        plan_edit::pick(&self.display_list_for(selected).items, at, locked)
    }

    /// Every element whose shown items lie wholly in the box with corners `a` and `b` (LH-08).
    /// Items of locked layers are not taken; no grips (they show for one selected way only).
    pub fn pick_in(&self, a: Point, b: Point) -> Vec<Source> {
        let locked = |role| self.lock(layer_of(role)) == Lock::Locked;

        plan_edit::pick_in(&self.display_list().items, (a, b), locked)
    }

    pub fn set_snapping(&mut self, snapping: Snapping) {
        self.snapping = snapping;
    }

    pub fn snapping(&self) -> Snapping {
        self.snapping
    }

    /// `at` on the grid when snapping is on.
    pub fn snap(&self, at: Point) -> Point {
        plan_edit::snap(at, self.snapping)
    }
}
