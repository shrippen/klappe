use klappe_model::{FloorPlan, Project, Scene};

use super::Session;
use crate::error::ServiceError;
use crate::plan_edit::{self, PlanEdit};
use crate::project_edit::{self, ProjectEdit, ShotsGo};

impl Session {
    pub fn scene(&self) -> Option<&Scene> {
        self.project.scenes.get(self.scene)
    }

    /// The plan on view in the current scene.
    pub fn plan(&self) -> Option<&FloorPlan> {
        self.scene()?.plans.get(self.plan)
    }

    pub fn select_scene(&mut self, id: &str) -> Result<(), ServiceError> {
        let index = self
            .project
            .scenes
            .iter()
            .position(|s| s.id == id)
            .ok_or_else(|| ServiceError::NoScene(id.to_owned()))?;

        self.scene = index;
        self.plan = 0;
        self.time_s = 0.0;
        self.selected_shot = None;
        Ok(())
    }

    /// Shows the plan with this id of the current scene.
    pub fn select_plan(&mut self, id: &str) -> Result<(), ServiceError> {
        let index = self
            .scene()
            .and_then(|s| s.plans.iter().position(|p| p.id == id))
            .ok_or_else(|| ServiceError::NoPlan(id.to_owned()))?;

        self.plan = index;
        self.time_s = 0.0;
        self.selected_shot = None;
        Ok(())
    }

    pub(super) fn scene_number(&self) -> String {
        self.scene().map(|s| s.number.clone()).unwrap_or_default()
    }

    /// The plan on view in `project`, a changed copy of this session's.
    pub(super) fn plan_in<'a>(
        &self,
        project: &'a mut Project,
    ) -> Result<&'a mut FloorPlan, ServiceError> {
        project
            .scenes
            .get_mut(self.scene)
            .and_then(|s| s.plans.get_mut(self.plan))
            .ok_or(ServiceError::Gone)
    }

    /// Changes the current plan; undoable. Points are taken as given: surfaces call `snap`.
    pub fn edit_plan(&mut self, edit: PlanEdit) -> Result<(), ServiceError> {
        self.check_lock(&edit)?;

        let mut next = self.project.clone();
        let at = (self.scene, self.plan);
        plan_edit::apply(
            &self.dir,
            &mut next,
            at.0,
            at.1,
            self.time_s,
            self.snapping,
            &edit,
        )?;
        self.check_cameras(&next)?;
        if next == self.project {
            return Ok(());
        }

        self.commit(next);
        Ok(())
    }

    /// Changes scenes, locations or characters; undoable. The scene on view stays, also when
    /// others come or go before it.
    pub fn edit_project(&mut self, edit: ProjectEdit) -> Result<(), ServiceError> {
        let mut next = self.project.clone();
        project_edit::apply(&mut next, &edit)?;
        if next == self.project {
            return Ok(());
        }

        let on_view = self.scene().map(|s| s.id.clone());
        let plan_on_view = self.plan().map(|p| p.id.clone());
        self.commit(next);
        self.follow_scene(on_view, plan_on_view);
        Ok(())
    }

    /// Adds an empty plan to the current scene and shows it; `name` empty: `Plan B` …
    /// Undoable. Returns the new plan's id.
    pub fn add_plan(&mut self, name: &str) -> Result<String, ServiceError> {
        let mut next = self.project.clone();
        let at = project_edit::add_plan(&mut next, self.scene, name)?;

        Ok(self.commit_new_plan(next, at))
    }

    /// Snapshot (LH-04): copies the plan on view under a new name and shows the copy; `name`
    /// empty: `Plan A (2)` …. Going back is choosing the earlier plan again. Undoable.
    /// Returns the copy's id.
    pub fn copy_plan(&mut self, name: &str) -> Result<String, ServiceError> {
        let mut next = self.project.clone();
        let at = project_edit::copy_plan(&mut next, self.scene, self.plan, name)?;

        Ok(self.commit_new_plan(next, at))
    }

    /// Changes the name of the plan `id` of the current scene; undoable.
    pub fn rename_plan(&mut self, id: &str, name: &str) -> Result<(), ServiceError> {
        let mut next = self.project.clone();
        project_edit::rename_plan(&mut next, self.scene, id, name)?;
        if next == self.project {
            return Ok(());
        }

        self.commit(next);
        Ok(())
    }

    /// Deletes the plan `id` of the current scene; one undo step. Refused for the last plan of
    /// a scene, and for one with shots unless `shots_go` is `Delete`, which takes them along.
    /// When it was on view, the plan before it is.
    pub fn remove_plan(&mut self, id: &str, shots_go: ShotsGo) -> Result<(), ServiceError> {
        let mut next = self.project.clone();
        project_edit::remove_plan(&mut next, self.scene, id, shots_go)?;

        let on_view = self.scene().map(|s| s.id.clone());
        let plan_on_view = self.plan().map(|p| p.id.clone());
        let was_shown = plan_on_view.as_deref() == Some(id);
        let before = self.plan.saturating_sub(1);
        self.commit(next);
        self.follow_scene(on_view, plan_on_view);
        if was_shown {
            self.plan = before;
        }
        Ok(())
    }

    /// Commits `next`, which has a new plan at `at` in the current scene, and shows it.
    fn commit_new_plan(&mut self, next: Project, at: usize) -> String {
        self.commit(next);
        self.plan = at;
        self.time_s = 0.0;
        self.selected_shot = None;

        self.plan().map(|p| p.id.clone()).unwrap_or_default()
    }

    /// "Szene 12 · INT. SÄNGERSAAL – TAG · Plan A" without the word "Szene": the surface adds it.
    pub fn plan_title(&self) -> String {
        let (Some(scene), Some(plan)) = (self.scene(), self.plan()) else {
            return String::new();
        };
        format!("{} · {} · {}", scene.number, scene.heading, plan.name)
    }
}
