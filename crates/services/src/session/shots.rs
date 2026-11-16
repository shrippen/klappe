use std::path::Path;

use klappe_model::{AxisPair, Camera, FloorPlan, Movement, Shot};
use klappe_stellprobe::{
    axis_jumps, movement_in, ordered_shots, setup_choices, setup_target, shot_lens, AxisJump,
    ShotOrder,
};

use super::{Layer, Lock, Session};
use crate::edit::{self, ShotField};
use crate::error::ServiceError;
use crate::field::Field;
use crate::shot_edit;
use crate::views::{SetupOption, ShotRow, FALLBACK_ASPECT};

const AGAINST_SEPARATOR: &str = ", ";

/// The numbers of the shots `jump` is against, as "12.1, 12.2".
fn against_numbers(plan: &FloorPlan, jump: &AxisJump) -> String {
    let numbers: Vec<&str> = jump
        .against
        .iter()
        .filter_map(|id| plan.shots.iter().find(|s| s.id == *id))
        .map(|s| s.number.as_str())
        .collect();

    numbers.join(AGAINST_SEPARATOR)
}

impl Session {
    pub fn selected_shot(&self) -> Option<&str> {
        self.selected_shot.as_deref()
    }

    /// Selects a shot and moves the playhead to its start (LH-73).
    pub fn select_shot(&mut self, id: &str) -> Result<(), ServiceError> {
        let shot = self
            .plan()
            .and_then(|p| p.shots.iter().find(|s| s.id == id))
            .ok_or_else(|| ServiceError::NoShot(id.to_owned()))?;

        self.time_s = shot.from_s;
        self.selected_shot = Some(id.to_owned());
        Ok(())
    }

    /// The shot list of the current plan, in cut order.
    pub fn shots(&self) -> Vec<ShotRow> {
        self.shots_in(ShotOrder::Cut)
    }

    /// The shot list of the current plan in `order`; `setup` counts the same in both.
    pub fn shots_in(&self, order: ShotOrder) -> Vec<ShotRow> {
        let Some(plan) = self.plan() else {
            return Vec::new();
        };
        let jumps = axis_jumps(plan);

        ordered_shots(plan, order)
            .into_iter()
            .map(|(setup, shot)| {
                let camera = plan.camera(&shot.camera);
                let jump = jumps.iter().find(|j| j.shot == shot.id);
                let lens = camera.map(|c| shot_lens(c, shot));
                ShotRow {
                    id: shot.id.clone(),
                    number: shot.number.clone(),
                    camera: shot.camera.clone(),
                    camera_color: plan
                        .camera_color(&shot.camera)
                        .map(|c| c.name().to_owned())
                        .unwrap_or_default(),
                    short_name: shot.short_name.clone(),
                    image: shot.image.clone().unwrap_or_default(),
                    camera_label: camera.map(|c| c.label.clone()).unwrap_or_default(),
                    aspect: camera.map_or(FALLBACK_ASPECT, |c| c.aspect),
                    follows: camera.and_then(|c| c.follows.clone()).unwrap_or_default(),
                    size: shot.size.name().to_owned(),
                    angle: shot.angle.name().to_owned(),
                    movement: shot.movement.name().to_owned(),
                    movement_hint: movement_hint(camera, shot),
                    lens_mm: lens.map_or(shot.lens_mm.unwrap_or(0.0), |l| {
                        l.span_mm().map_or(l.from_mm, |(min_mm, _)| min_mm)
                    }),
                    lens_to_mm: lens.and_then(|l| l.span_mm()).map(|(_, max_mm)| max_mm),
                    lens_from_marks: lens.is_some_and(|l| l.from_marks),
                    own_lens_mm: shot.lens_mm,
                    from_s: shot.from_s,
                    to_s: shot.to_s,
                    description: shot.description.clone(),
                    props: shot.props.clone(),
                    gear: shot.gear.clone(),
                    crew: shot.crew.clone(),
                    note: shot.note.clone(),
                    axis_jump: jump.is_some(),
                    axis_against: jump.map(|j| against_numbers(plan, j)).unwrap_or_default(),
                    setup,
                    setup_of: shot.setup_of.clone().unwrap_or_default(),
                    setup_lost: shot.setup_of.is_some() && setup_target(plan, shot).is_none(),
                }
            })
            .collect()
    }

    /// The setups shot `id` can be assigned to (LH-77), counted without that shot.
    pub fn setup_options(&self, id: &str) -> Vec<SetupOption> {
        let Some(plan) = self.plan() else {
            return Vec::new();
        };

        setup_choices(plan, id)
            .into_iter()
            .map(Into::into)
            .collect()
    }

    /// Changes one field of a shot; undoable (LH-06).
    pub fn edit_shot(
        &mut self,
        id: &str,
        field: ShotField,
        value: &str,
    ) -> Result<(), ServiceError> {
        let mut next = self.project.clone();
        let plan = next
            .scenes
            .get_mut(self.scene)
            .and_then(|s| s.plans.get_mut(self.plan))
            .ok_or_else(|| ServiceError::NoShot(id.to_owned()))?;
        if field == ShotField::Setup {
            shot_edit::check_setup(plan, id, value)?;
        }
        let shot = plan
            .shots
            .iter_mut()
            .find(|s| s.id == id)
            .ok_or_else(|| ServiceError::NoShot(id.to_owned()))?;

        edit::apply(shot, &plan.cameras, field, value)?;
        if next == self.project {
            return Ok(());
        }

        self.commit(next);
        Ok(())
    }

    /// Gives shot `id` the storyboard picture `file` (PNG or JPEG), copied into the project's
    /// `bilder/`, or takes it away with `None`; undoable (LH-74). The copy stays in `bilder/`,
    /// it is the user's file. Refused while the camera layer is locked, like any edit of a shot.
    pub fn set_shot_image(&mut self, id: &str, file: Option<&Path>) -> Result<(), ServiceError> {
        self.check_shot_lock()?;

        let image = match file {
            Some(file) => Some(klappe_store::add_image(&self.dir, file)?),
            None => None,
        };
        let mut next = self.project.clone();
        let plan = self.plan_in(&mut next)?;
        let shot = plan
            .shots
            .iter_mut()
            .find(|s| s.id == id)
            .ok_or_else(|| ServiceError::NoShot(id.to_owned()))?;

        shot.image = image;
        if next == self.project {
            return Ok(());
        }

        self.commit(next);
        Ok(())
    }

    /// Adds a shot at the playhead and selects it; undoable. Its camera is `camera`, else the
    /// selected shot's, else the first (LH-70). Refused while the camera layer is locked: a
    /// shot belongs to its camera.
    pub fn add_shot(&mut self, camera: Option<&str>) -> Result<String, ServiceError> {
        self.check_shot_lock()?;

        let camera = self.camera_for_shot(camera)?;
        let scene = self.scene_number();
        let numbering = self.project.numbering;
        let at_s = self.time_s;
        let mut next = self.project.clone();
        let plan = self.plan_in(&mut next)?;

        let id = shot_edit::add(plan, numbering, &scene, &camera, at_s);
        self.commit(next);
        self.selected_shot = Some(id.clone());
        Ok(id)
    }

    /// Copies a shot with a new number and selects the copy; undoable. A copy is how one
    /// setup yields several shots (LH-70).
    pub fn copy_shot(&mut self, id: &str) -> Result<String, ServiceError> {
        self.check_shot_lock()?;

        let scene = self.scene_number();
        let numbering = self.project.numbering;
        let mut next = self.project.clone();
        let plan = self.plan_in(&mut next)?;

        let copy = shot_edit::copy(plan, numbering, &scene, id)?;
        self.commit(next);
        self.selected_shot = Some(copy.clone());
        Ok(copy)
    }

    /// Copies a shot as the next version of its number (12.3 to 12.3A) and selects the copy;
    /// undoable (LH-76).
    pub fn version_shot(&mut self, id: &str) -> Result<String, ServiceError> {
        self.check_shot_lock()?;

        let scene = self.scene_number();
        let numbering = self.project.numbering;
        let mut next = self.project.clone();
        let plan = self.plan_in(&mut next)?;

        let copy = shot_edit::version(plan, numbering, &scene, id)?;
        self.commit(next);
        self.selected_shot = Some(copy.clone());
        Ok(copy)
    }

    /// Numbers the shots of the plan one after the other in cut order, versions keeping their
    /// letters; one undo step (LH-76).
    pub fn renumber_shots(&mut self) -> Result<(), ServiceError> {
        self.check_shot_lock()?;

        let scene = self.scene_number();
        let numbering = self.project.numbering;
        let mut next = self.project.clone();
        let plan = self.plan_in(&mut next)?;

        shot_edit::renumber(plan, numbering, &scene);
        if next == self.project {
            return Ok(());
        }

        self.commit(next);
        Ok(())
    }

    /// Deletes a shot and the script links to it; undoable. If it was selected, the next
    /// shot in the list is, else the previous.
    pub fn delete_shot(&mut self, id: &str) -> Result<(), ServiceError> {
        self.check_shot_lock()?;

        let mut next = self.project.clone();
        let plan = self.plan_in(&mut next)?;

        let neighbour = shot_edit::remove(plan, id)?;
        self.commit(next);
        if self.selected_shot.as_deref() == Some(id) {
            self.selected_shot = neighbour;
        }
        Ok(())
    }

    /// Sets the characters the 180° axis of the plan runs between (LH-40), or takes the
    /// choice away with `None`; undoable. Both need a track in the plan, and they differ.
    /// Refused while the figure layer is locked, which holds the axis.
    pub fn set_axis(&mut self, pair: Option<(&str, &str)>) -> Result<(), ServiceError> {
        if self.lock(Layer::Figures) == Lock::Locked {
            return Err(ServiceError::Locked(Layer::Figures));
        }

        let mut next = self.project.clone();
        let plan = self.plan_in(&mut next)?;
        plan.axis = match pair {
            None => None,
            Some((a, b)) => {
                let known = |id: &str| plan.figure(id).is_some();
                if a == b || !known(a) || !known(b) {
                    return Err(ServiceError::BadValue {
                        field: Field::Character,
                        value: format!("{a}, {b}"),
                    });
                }

                Some(AxisPair {
                    a: a.to_owned(),
                    b: b.to_owned(),
                })
            }
        };
        if next == self.project {
            return Ok(());
        }

        self.commit(next);
        Ok(())
    }

    /// Shots belong to cameras: no new, copied or deleted shot while they are locked.
    fn check_shot_lock(&self) -> Result<(), ServiceError> {
        if self.lock(Layer::Cameras) == Lock::Locked {
            return Err(ServiceError::Locked(Layer::Cameras));
        }

        Ok(())
    }

    /// The camera id a new shot gets: `camera` if asked for, else the selected shot's, else
    /// the first of the plan.
    fn camera_for_shot(&self, camera: Option<&str>) -> Result<String, ServiceError> {
        let plan = self.plan().ok_or(ServiceError::NoCamera)?;
        let wanted = camera.map(str::to_owned).or_else(|| {
            let id = self.selected_shot.as_deref()?;
            plan.shots
                .iter()
                .find(|s| s.id == id)
                .map(|s| s.camera.clone())
        });
        let Some(wanted) = wanted else {
            return plan
                .cameras
                .first()
                .map(|c| c.id.clone())
                .ok_or(ServiceError::NoCamera);
        };

        match plan.camera(&wanted) {
            Some(c) => Ok(c.id.clone()),
            None => Err(ServiceError::BadValue {
                field: Field::Camera,
                value: wanted,
            }),
        }
    }
}

/// The movement the camera's marks suggest for a shot that sets none; empty when there is
/// nothing to suggest.
fn movement_hint(camera: Option<&Camera>, shot: &Shot) -> String {
    let Some(camera) = camera.filter(|_| shot.movement == Movement::Static) else {
        return String::new();
    };

    match movement_in(camera, shot) {
        Movement::Static => String::new(),
        movement => movement.name().to_owned(),
    }
}
