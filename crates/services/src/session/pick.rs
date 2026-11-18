use klappe_model::{Curve, DrawingShape, Point, Project};
use klappe_stellprobe::{lens_at, DisplayList, Item, Source};

use super::layers::layer_of;
use super::{Lock, Session};
use crate::error::ServiceError;
use crate::plan_edit::{self, Grid, MarkInfo, PlanEdit, Snapping};

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
        let at = (self.scene, self.plan);

        self.plan_view
            .list(&self.dir, project, at, self.time_s, selected)
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

    /// The shape of the way `source` names (LH-108): a mark, figure or camera (its mark at the
    /// playhead) or a grip on the way; `None` for anything else and for a first mark.
    pub fn way_curve(&self, source: &Source) -> Option<Curve> {
        plan_edit::way_curve(self.plan()?, self.time_s, source)
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
        if matches!(
            edit,
            PlanEdit::SetBackground { .. } | PlanEdit::AddImage { .. }
        ) {
            return Ok(self.display_list_for(selected));
        }
        plan_edit::apply(
            &self.dir,
            &mut next,
            at.0,
            at.1,
            self.time_s,
            (self.snapping, self.grid),
            edit,
        )?;
        self.check_cameras(&next)?;
        self.check_tripods(&next)?;

        Ok(self.list_of(&next, selected))
    }

    /// Topmost shown item at `at` (front of the display list first), within the pick radius.
    /// Items of locked layers are not taken.
    pub fn pick(&self, at: Point) -> Option<Source> {
        self.pick_for(at, None)
    }

    /// `pick` with the grips of the way `selected` belongs to in reach: they lie in front.
    pub fn pick_for(&self, at: Point, selected: Option<&Source>) -> Option<Source> {
        let locked =
            |item: &Item| self.lock(layer_of(item.role, item.source.as_ref())) == Lock::Locked;

        plan_edit::pick(&self.display_list_for(selected).items, at, locked)
    }

    /// `pick_for` for a click: only an area's point is a point of its own (Delete takes it
    /// out); on any other drawing a grip stands for the drawing.
    pub fn pick_click(&self, at: Point, selected: Option<&Source>) -> Option<Source> {
        let picked = self.pick_for(at, selected)?;
        let Source::DrawingPoint { drawing, .. } = &picked else {
            return Some(picked);
        };
        let area = self
            .plan()
            .and_then(|p| p.drawings.iter().find(|d| d.id == *drawing))
            .is_some_and(|d| matches!(d.shape, DrawingShape::Area { .. }));
        if area {
            return Some(picked);
        }

        Some(Source::Drawing(drawing.clone()))
    }

    /// Every element whose shown items lie wholly in the box with corners `a` and `b` (LH-08).
    /// Items of locked layers are not taken; no grips (they show for one selected way only).
    pub fn pick_in(&self, a: Point, b: Point) -> Vec<Source> {
        let locked =
            |item: &Item| self.lock(layer_of(item.role, item.source.as_ref())) == Lock::Locked;

        plan_edit::pick_in(&self.display_list().items, (a, b), locked)
    }

    /// Every element whose reference point lies in the freehand shape `lasso` (LH-08). Items of
    /// locked layers are not taken; no grips, as with `pick_in`.
    pub fn pick_lasso(&self, lasso: &[Point]) -> Vec<Source> {
        let locked =
            |item: &Item| self.lock(layer_of(item.role, item.source.as_ref())) == Lock::Locked;

        plan_edit::pick_lasso(&self.display_list().items, lasso, locked)
    }

    /// Every grip on a point (a bend, a point of a drawing or a free form) of the element
    /// `selected` names that lies wholly in the box with corners `a` and `b` (LH-08). Only the
    /// grips that show are taken, and no others than that element's.
    pub fn pick_grips_in(&self, selected: &Source, a: Point, b: Point) -> Vec<Source> {
        let locked =
            |item: &Item| self.lock(layer_of(item.role, item.source.as_ref())) == Lock::Locked;
        let grips = plan_edit::point_grips(self.display_list_for(Some(selected)).items);

        plan_edit::pick_in(&grips, (a, b), locked)
    }

    /// `pick_grips_in` for a freehand shape: the grips whose middle lies in `lasso`.
    pub fn pick_grips_lasso(&self, selected: &Source, lasso: &[Point]) -> Vec<Source> {
        let locked =
            |item: &Item| self.lock(layer_of(item.role, item.source.as_ref())) == Lock::Locked;
        let grips = plan_edit::point_grips(self.display_list_for(Some(selected)).items);

        plan_edit::pick_lasso(&grips, lasso, locked)
    }

    pub fn set_snapping(&mut self, snapping: Snapping) {
        self.snapping = snapping;
    }

    pub fn snapping(&self) -> Snapping {
        self.snapping
    }

    pub fn set_grid(&mut self, grid: Grid) {
        self.grid = grid;
    }

    pub fn grid(&self) -> Grid {
        self.grid
    }

    /// `at` on the grid when snapping is on.
    pub fn snap(&self, at: Point) -> Point {
        plan_edit::snap(at, self.snapping, self.grid)
    }
}
