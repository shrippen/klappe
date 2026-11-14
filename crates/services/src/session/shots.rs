use klappe_stellprobe::{axis_jumps, ordered_shots, ShotOrder};

use super::{Layer, Lock, Session};
use crate::edit::{self, ShotField};
use crate::error::ServiceError;
use crate::field::Field;
use crate::shot_edit;
use crate::views::ShotRow;

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
                ShotRow {
                    id: shot.id.clone(),
                    number: shot.number.clone(),
                    camera: shot.camera.clone(),
                    camera_label: camera.map(|c| c.label.clone()).unwrap_or_default(),
                    follows: camera.and_then(|c| c.follows.clone()).unwrap_or_default(),
                    size: shot.size.name().to_owned(),
                    angle: shot.angle.name().to_owned(),
                    movement: shot.movement.name().to_owned(),
                    lens_mm: shot.lens_mm.or(camera.map(|c| c.lens_mm)).unwrap_or(0.0),
                    own_lens_mm: shot.lens_mm,
                    from_s: shot.from_s,
                    to_s: shot.to_s,
                    description: shot.description.clone(),
                    props: shot.props.clone(),
                    gear: shot.gear.clone(),
                    crew: shot.crew.clone(),
                    note: shot.note.clone(),
                    axis_jump: jumps.contains(&shot.id),
                    setup,
                }
            })
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

    /// Adds a shot at the playhead and selects it; undoable. Its camera is `camera`, else the
    /// selected shot's, else the first (LH-70). Refused while the camera layer is locked: a
    /// shot belongs to its camera.
    pub fn add_shot(&mut self, camera: Option<&str>) -> Result<String, ServiceError> {
        self.check_shot_lock()?;

        let camera = self.camera_for_shot(camera)?;
        let scene = self.scene_number();
        let at_s = self.time_s;
        let mut next = self.project.clone();
        let plan = self.plan_in(&mut next)?;

        let id = shot_edit::add(plan, &scene, &camera, at_s);
        self.commit(next);
        self.selected_shot = Some(id.clone());
        Ok(id)
    }

    /// Copies a shot with a new number and selects the copy; undoable. A copy is how one
    /// setup yields several shots (LH-70).
    pub fn copy_shot(&mut self, id: &str) -> Result<String, ServiceError> {
        self.check_shot_lock()?;

        let scene = self.scene_number();
        let mut next = self.project.clone();
        let plan = self.plan_in(&mut next)?;

        let copy = shot_edit::copy(plan, &scene, id)?;
        self.commit(next);
        self.selected_shot = Some(copy.clone());
        Ok(copy)
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
