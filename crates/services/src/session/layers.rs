use klappe_model::Project;
use klappe_stellprobe::{Role, Source};

use super::{Layer, Lock, Session, Visibility};
use crate::error::ServiceError;
use crate::plan_edit::PlanEdit;

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
        if self.hidden_layers.contains(&layer_of(role)) {
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

/// Which layer switch an item of the display list belongs to.
pub(super) fn layer_of(role: Role) -> Layer {
    match role {
        Role::Background | Role::Wall | Role::Object => Layer::Set,
        Role::Light | Role::LightBeam => Layer::Lights,
        Role::Camera | Role::Frustum | Role::CameraPath | Role::CameraHandle => Layer::Cameras,
        Role::Note => Layer::Notes,
        Role::Drawing { .. } | Role::ArrowHead(_) => Layer::Drawings,
        Role::Axis
        | Role::Figure(_)
        | Role::GroupDot
        | Role::FigurePath(_)
        | Role::FigureHandle(_)
        | Role::Mark(_) => Layer::Figures,
    }
}
