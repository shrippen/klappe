use klappe_model::Project;
use klappe_stellprobe::{Role, Source};

use super::{Layer, Lock, Session, Visibility};
use crate::error::ServiceError;
use crate::plan_edit::{layer_of_source, PlanEdit};

impl Session {
    pub fn set_layer(&mut self, layer: Layer, visibility: Visibility) {
        match visibility {
            Visibility::Shown => self.hidden_layers.remove(&layer),
            Visibility::Hidden => self.hidden_layers.insert(layer),
        };
    }

    pub fn layer(&self, layer: Layer) -> Visibility {
        if self.hidden_layers.contains(&layer) {
            return Visibility::Hidden;
        }
        Visibility::Shown
    }

    /// Locks or unlocks a layer for this session: its elements cannot be picked, and edits
    /// that change or add something on it are refused with `ServiceError::Locked`. The lock
    /// is a view setting like `set_layer`, not part of the project.
    pub fn set_lock(&mut self, layer: Layer, lock: Lock) {
        match lock {
            Lock::Unlocked => self.locked_layers.remove(&layer),
            Lock::Locked => self.locked_layers.insert(layer),
        };
    }

    pub fn lock(&self, layer: Layer) -> Lock {
        if self.locked_layers.contains(&layer) {
            return Lock::Locked;
        }
        Lock::Unlocked
    }

    /// Refuses `edit` when it changes a locked layer.
    pub(super) fn check_lock(&self, edit: &PlanEdit) -> Result<(), ServiceError> {
        match edit
            .layers()
            .into_iter()
            .find(|layer| self.lock(*layer) == Lock::Locked)
        {
            Some(layer) => Err(ServiceError::Locked(layer)),
            None => Ok(()),
        }
    }

    /// Refuses a result `next` in which a figure has another size while the figure layer is
    /// locked (LH-25): the size is part of how the figure stands on the plan.
    pub(super) fn check_figure_sizes(&self, next: &Project) -> Result<(), ServiceError> {
        if self.lock(Layer::Figures) == Lock::Unlocked {
            return Ok(());
        }

        let changed = next.characters.iter().any(|c| {
            self.project
                .character(&c.id)
                .is_some_and(|old| old.size_m != c.size_m)
        });
        match changed {
            true => Err(ServiceError::Locked(Layer::Figures)),
            false => Ok(()),
        }
    }

    /// Refuses a result `next` in which a note has changed while the notes layer is locked: a
    /// bound note follows its element (LH-118), or lets go of one that is gone.
    pub(super) fn check_notes(&self, next: &Project) -> Result<(), ServiceError> {
        if self.lock(Layer::Notes) == Lock::Unlocked {
            return Ok(());
        }

        let notes = |project: &Project| {
            let location = &project.scenes.get(self.scene)?.location;
            let set = &project.locations.iter().find(|l| l.id == *location)?.set;
            Some(set.notes.clone())
        };
        if notes(next) == notes(&self.project) {
            return Ok(());
        }

        Err(ServiceError::Locked(Layer::Notes))
    }

    /// Refuses a result `next` in which a locked camera layer has moved: a figure drags its
    /// coupled cameras along (LH-37), and a camera left behind would break distance and angle.
    pub(super) fn check_cameras(&self, next: &Project) -> Result<(), ServiceError> {
        if self.lock(Layer::Cameras) == Lock::Unlocked {
            return Ok(());
        }

        let marks = |project: &Project| {
            let plan = project.scenes.get(self.scene)?.plans.get(self.plan)?;
            Some(
                plan.cameras
                    .iter()
                    .map(|c| c.marks.clone())
                    .collect::<Vec<_>>(),
            )
        };
        if marks(next) == marks(&self.project) {
            return Ok(());
        }

        Err(ServiceError::Locked(Layer::Cameras))
    }

    /// Shows or hides one figure, camera or light in plan and timeline.
    pub fn set_track(&mut self, id: &str, visibility: Visibility) {
        match visibility {
            Visibility::Shown => self.hidden_tracks.remove(id),
            Visibility::Hidden => self.hidden_tracks.insert(id.to_owned()),
        };
    }

    pub(super) fn shows(&self, role: Role, source: Option<&Source>) -> bool {
        if self.hidden_layers.contains(&layer_of(role, source)) {
            return false;
        }

        let id = match source {
            Some(Source::Camera(id) | Source::Light(id) | Source::Figure(id)) => id,
            Some(Source::Mark { character, .. } | Source::FigureVia { character, .. }) => character,
            Some(Source::CameraVia { camera, .. }) => camera,
            _ => return true,
        };
        !self.hidden_tracks.contains(id)
    }
}

/// Which layer switch an item of the display list belongs to. A grip is on its element's.
pub(super) fn layer_of(role: Role, source: Option<&Source>) -> Layer {
    match role {
        Role::Handle => source.map_or(Layer::Set, layer_of_source),
        Role::Background
        | Role::Wall
        | Role::Door
        | Role::Window
        | Role::Object
        | Role::Height { .. } => Layer::Set,
        Role::Light | Role::LightBeam => Layer::Lights,
        Role::Camera | Role::Frustum | Role::CameraPath { .. } | Role::CameraHandle => {
            Layer::Cameras
        }
        Role::Note | Role::Picture => Layer::Notes,
        Role::Drawing { .. } | Role::ArrowHead(_) => Layer::Drawings,
        Role::Axis
        | Role::Figure(_)
        | Role::GroupDot
        | Role::FigurePath(_)
        | Role::FigureHandle(_)
        | Role::Mark(_) => Layer::Figures,
    }
}
