use std::collections::HashSet;

use klappe_model::{Department, ElementCategory, FloorPlan, Project, Scene, Site};

use super::undo::{next_edit, step_name};
use super::Session;
use crate::error::ServiceError;
use crate::plan_edit::{self, PlanEdit};
use crate::project_edit::{self, BreakdownEdit, PlanShift, ProjectEdit, ShotsGo, TeamEdit};
use crate::scene_location;
use crate::script_refs;

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
        self.edits = next_edit();
        self.end_stale_look();
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
        self.edits = next_edit();
        self.end_stale_look();
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

    /// Refuses a result `next` that moves a camera mark off its tripod (LH-33).
    pub(super) fn check_tripods(&self, next: &Project) -> Result<(), ServiceError> {
        let at = (self.scene, self.plan);
        let (Some(before), Some(after)) = (plan_at(&self.project, at), plan_at(next, at)) else {
            return Ok(());
        };

        plan_edit::check_tripods(before, after)
    }

    /// The project after `edit`, with every check `edit_plan` makes; `None` when the edit
    /// changes nothing.
    fn plan_after(&self, edit: &PlanEdit) -> Result<Option<Project>, ServiceError> {
        self.check_lock(edit)?;

        let mut next = self.project.clone();
        let at = (self.scene, self.plan);
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
        self.check_notes(&next)?;
        self.check_tripods(&next)?;

        Ok((next != self.project).then_some(next))
    }

    /// Changes the current plan; undoable. Points are taken as given: surfaces call `snap`.
    pub fn edit_plan(&mut self, edit: PlanEdit) -> Result<(), ServiceError> {
        let Some(next) = self.plan_after(&edit)? else {
            return Ok(());
        };

        self.commit_step(next, &step_name(&edit));
        self.end_stale_look();
        Ok(())
    }

    /// Whether `edit_plan(edit)` would change the plan: the menu offers only such commands.
    pub fn changes(&self, edit: &PlanEdit) -> bool {
        matches!(self.plan_after(edit), Ok(Some(_)))
    }

    /// Changes scenes, locations or characters; undoable. The scene on view stays, also when
    /// others come or go before it.
    pub fn edit_project(&mut self, edit: ProjectEdit) -> Result<(), ServiceError> {
        let edit = self.plain_own(edit);
        self.check_script_number(&edit)?;
        self.check_location_use(&edit)?;
        self.check_sheet_character(&edit)?;
        self.check_cast_number(&edit)?;
        let step = project_step(&edit);
        let mut next = self.project.clone();
        project_edit::apply(&mut next, &edit)?;
        if let ProjectEdit::AddScene { .. } = edit {
            self.seat_new_scene(&mut next);
        }
        self.store_numbers(&edit, &mut next);
        self.check_figure_sizes(&next)?;
        if next == self.project {
            return Ok(());
        }

        let on_view = self.scene().map(|s| s.id.clone());
        let plan_on_view = self.plan().map(|p| p.id.clone());
        match self.renamed_script(&edit) {
            Some(text) => self.commit_script(next, &text, &step),
            None => self.commit_step(next, &step),
        }
        self.follow_scene(on_view, plan_on_view);
        self.end_stale_look();
        Ok(())
    }

    /// Adds the element and returns its id; undoable. Refused like `AddElement`.
    pub fn add_element(
        &mut self,
        category: ElementCategory,
        name: &str,
    ) -> Result<String, ServiceError> {
        let edit = ProjectEdit::Breakdown(BreakdownEdit::AddElement {
            category,
            name: name.to_owned(),
        });

        self.add_and_name(edit, |p| p.elements.iter().map(|e| e.id.as_str()).collect())
    }

    /// Adds the person and returns their id; undoable. Refused like `AddPerson`.
    pub fn add_person(
        &mut self,
        name: &str,
        department: Option<Department>,
    ) -> Result<String, ServiceError> {
        let edit = ProjectEdit::Team(TeamEdit::AddPerson {
            name: name.to_owned(),
            department,
        });

        self.add_and_name(edit, |p| {
            p.team.persons.iter().map(|p| p.id.as_str()).collect()
        })
    }

    /// Adds the location and returns its id; undoable. Refused like `AddLocation`.
    pub fn add_location(
        &mut self,
        name: &str,
        size_m: (f64, f64),
        wall_height_m: f64,
        site: Option<Site>,
    ) -> Result<String, ServiceError> {
        let edit = ProjectEdit::AddLocation {
            name: name.to_owned(),
            width_m: size_m.0,
            depth_m: size_m.1,
            wall_height_m,
            site,
        };

        self.add_and_name(edit, |p| {
            p.locations.iter().map(|l| l.id.as_str()).collect()
        })
    }

    /// Applies an edit that adds one entry and returns the id `ids` did not list before it.
    fn add_and_name(
        &mut self,
        edit: ProjectEdit,
        ids: impl Fn(&Project) -> Vec<&str>,
    ) -> Result<String, ServiceError> {
        let known: HashSet<String> = ids(&self.project).into_iter().map(str::to_owned).collect();
        self.edit_project(edit)?;

        let added = ids(&self.project)
            .into_iter()
            .find(|id| !known.contains(*id));
        Ok(added.unwrap_or_default().to_owned())
    }

    /// The scene `edit` added is the one the session did not have: where it is shot already
    /// (DB-51), it gets `Plan A` on that set, starting with the speakers of the screenplay
    /// (LH-09). Shot nowhere yet, it has no plan.
    fn seat_new_scene(&self, next: &mut Project) {
        let new = next
            .scenes
            .iter()
            .position(|s| self.project.scene(&s.id).is_none());
        let Some(scene) = new else {
            return;
        };
        let location = scene_location::location_of(next, &self.places, &next.scenes[scene])
            .map(|l| l.id.clone());
        let Some(location) = location else {
            return;
        };

        next.scenes[scene]
            .plans
            .push(project_edit::first_plan(&location));
        self.seat_speakers(next, scene, 0);
    }

    /// A new scene number must not be one that another heading of the script carries: both
    /// would read as one scene.
    fn check_script_number(&self, edit: &ProjectEdit) -> Result<(), ServiceError> {
        let ProjectEdit::EditScene { id, number, .. } = edit else {
            return Ok(());
        };
        let Some(scene) = self.project.scene(id) else {
            return Ok(());
        };
        let same = number.trim().to_lowercase() == scene.number.to_lowercase();
        if same || !script_refs::has_number(self.script_text(), number) {
            return Ok(());
        }

        Err(ServiceError::NumberTaken(number.trim().to_owned()))
    }

    /// The screenplay with the scene's heading, number or the character's name of `edit`
    /// carried over (D-08); `None` when the text stays.
    fn renamed_script(&self, edit: &ProjectEdit) -> Option<String> {
        let text = self.script_text();

        match edit {
            ProjectEdit::EditScene {
                id,
                number,
                heading,
                ..
            } => script_refs::scene_text(&self.project, text, id, number, heading),
            ProjectEdit::EditCharacter { id, name, .. } => {
                script_refs::character_text(&self.project, text, id, name)
            }
            _ => None,
        }
    }

    /// Adds an empty plan to the current scene and shows it; `name` empty: `Plan B` …
    /// Undoable. Returns the new plan's id.
    pub fn add_plan(&mut self, name: &str) -> Result<String, ServiceError> {
        let location = self
            .scene()
            .and_then(|s| scene_location::location_of(&self.project, &self.places, s))
            .map(|l| l.id.clone())
            .ok_or(ServiceError::NotShot)?;
        let mut next = self.project.clone();
        let at = project_edit::add_plan(&mut next, (self.scene, &location), name)?;
        self.seat_speakers(&mut next, self.scene, at);

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

        self.commit_step(next, "RenamePlan");
        Ok(())
    }

    /// Moves the plan `id` of the current scene one place; one undo step. At the first or last
    /// place nothing happens (no undo step, no error). The view stays on the plan it showed.
    pub fn move_plan(&mut self, id: &str, shift: PlanShift) -> Result<(), ServiceError> {
        let mut next = self.project.clone();
        project_edit::move_plan(&mut next, self.scene, id, shift)?;
        if next == self.project {
            return Ok(());
        }

        let on_view = self.scene().map(|s| s.id.clone());
        let plan_on_view = self.plan().map(|p| p.id.clone());
        self.commit_step(next, "MovePlan");
        self.follow_scene(on_view, plan_on_view);
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
        self.commit_step(next, "RemovePlan");
        self.follow_scene(on_view, plan_on_view);
        if was_shown {
            self.plan = before;
        }
        self.end_stale_look();
        Ok(())
    }

    /// Puts the characters who speak in the scene at `scene` of `next`, by the screenplay, on
    /// its new plan at `plan` (LH-09). Without a screenplay, or for a cue no character
    /// answers to, nobody is put.
    fn seat_speakers(&self, next: &mut Project, scene: usize, plan: usize) {
        let Some(speakers) = next
            .scenes
            .get(scene)
            .map(|s| script_refs::speakers(self.script_text(), s, next))
        else {
            return;
        };

        project_edit::seat_speakers(next, scene, plan, &speakers);
    }

    /// Commits `next`, which has a new plan at `at` in the current scene, and shows it.
    fn commit_new_plan(&mut self, next: Project, at: usize) -> String {
        self.commit_step(next, "AddPlan");
        self.plan = at;
        self.time_s = 0.0;
        self.selected_shot = None;
        self.end_stale_look();

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

/// The undo name of a project edit: the variant of the area's edit (`AddElement`), not the
/// area (`Breakdown`).
fn project_step(edit: &ProjectEdit) -> String {
    match edit {
        ProjectEdit::Names(inner) => step_name(inner),
        ProjectEdit::Library(inner) => step_name(inner),
        ProjectEdit::Template(inner) => step_name(inner),
        ProjectEdit::Breakdown(inner) => step_name(inner),
        ProjectEdit::Team(inner) => step_name(inner),
        other => step_name(other),
    }
}

fn plan_at(project: &Project, (scene, plan): (usize, usize)) -> Option<&FloorPlan> {
    project.scenes.get(scene)?.plans.get(plan)
}
