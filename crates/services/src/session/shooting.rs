//! Where the scene on view is shot (DB-51, D-21): the list of scripted locations with their
//! shooting locations, a plan drawn on another set than the scene is shot at, a scene shot
//! nowhere yet, and shooting one scene elsewhere.

use super::Session;
use crate::error::ServiceError;
use crate::project_edit::ProjectEdit;
use crate::scene_location;
use crate::views::{Elsewhere, NotShot, PlaceRow};

impl Session {
    /// The scripted locations as `orte.toml` holds them, each with its shooting location and
    /// the scenes whose heading names it.
    pub(super) fn place_rows(&self) -> Vec<PlaceRow> {
        let project = &self.project;

        self.places
            .iter()
            .map(|p| {
                let location = p.location.as_deref().and_then(|id| project.location(id));
                let scenes = project
                    .scenes
                    .iter()
                    .filter(|s| {
                        scene_location::place_of(&self.places, s).is_some_and(|o| o.id == p.id)
                    })
                    .map(|s| s.number.clone())
                    .collect();

                PlaceRow {
                    id: p.id.clone(),
                    name: p.name.clone(),
                    variants: p.variants.clone(),
                    location: location.map(|l| l.id.clone()).unwrap_or_default(),
                    location_name: location.map(|l| l.name.clone()).unwrap_or_default(),
                    scenes,
                }
            })
            .collect()
    }

    /// How many scenes are shot at location `id`.
    pub(super) fn scenes_at(&self, id: &str) -> usize {
        self.project
            .scenes
            .iter()
            .filter(|s| {
                scene_location::location_of(&self.project, &self.places, s)
                    .is_some_and(|at| at.id == id)
            })
            .count()
    }

    /// The plan on view lies on another set than the scene is shot at now; `None` when they
    /// agree or the scene is shot nowhere.
    pub fn plan_elsewhere(&self) -> Option<Elsewhere> {
        let plan = self.plan()?;
        let shot_at = scene_location::location_of(&self.project, &self.places, self.scene()?)?;
        if shot_at.id == plan.location {
            return None;
        }
        let drawn_on = self.project.location(&plan.location)?;

        Some(Elsewhere {
            drawn_on: drawn_on.id.clone(),
            drawn_on_name: drawn_on.name.clone(),
            shot_at: shot_at.id.clone(),
            shot_at_name: shot_at.name.clone(),
        })
    }

    /// The scene on view has no plan and is shot nowhere yet; `None` otherwise.
    pub fn not_shot(&self) -> Option<NotShot> {
        let scene = self.scene()?;
        let shot = scene_location::location_of(&self.project, &self.places, scene).is_some();
        if shot || !scene.plans.is_empty() {
            return None;
        }
        let place = scene_location::place_of(&self.places, scene);

        Some(NotShot {
            place: place.map(|p| p.id.clone()).unwrap_or_default(),
            place_name: place.map(|p| p.name.clone()).unwrap_or_default(),
        })
    }

    /// Shoots the scene on view at `location`, unlike its scripted location (`None`: as its
    /// scripted location). Its plans stay where they are drawn. Undoable.
    pub fn shoot_scene_at(&mut self, location: Option<&str>) -> Result<(), ServiceError> {
        let scene = self.scene().ok_or(ServiceError::Gone)?;
        let edit = ProjectEdit::EditScene {
            id: scene.id.clone(),
            number: scene.number.clone(),
            heading: scene.heading.clone(),
            location: location.map(str::to_owned),
            sun: scene.sun,
        };

        self.edit_project(edit)
    }
}
