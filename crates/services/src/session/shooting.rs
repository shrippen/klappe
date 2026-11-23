//! Where the scene on view is shot (DB-51, D-21): the list of scripted locations with their
//! shooting locations, a plan drawn on another set than the scene is shot at, a scene shot
//! nowhere yet, and shooting one scene elsewhere.

use klappe_model::{Location, Place, Scene};

use super::Session;
use crate::error::ServiceError;
use crate::project_edit::ProjectEdit;
use crate::scene_location;
use crate::views::{Elsewhere, NotShot, PlaceRow};

impl Session {
    /// The scripted location and the shooting location of each scene, in project order.
    pub(super) fn shot_scenes(&self) -> (Vec<Option<&Place>>, Vec<Option<&Location>>) {
        let places = scene_location::places_of(&self.places, &self.project.scenes);
        let shot = self
            .project
            .scenes
            .iter()
            .zip(&places)
            .map(|(s, p)| scene_location::shot_at(&self.project, s, *p))
            .collect();

        (places, shot)
    }

    /// The scripted locations as `orte.toml` holds them, each with its shooting location and
    /// the scenes whose heading names it; `of_scenes` is the place of each scene.
    pub(super) fn place_rows(&self, of_scenes: &[Option<&Place>]) -> Vec<PlaceRow> {
        let project = &self.project;

        self.places
            .iter()
            .map(|p| {
                let location = p.location.as_deref().and_then(|id| project.location(id));
                let scenes = project
                    .scenes
                    .iter()
                    .zip(of_scenes)
                    .filter(|(_, o)| o.is_some_and(|o| o.id == p.id))
                    .map(|(s, _)| s.number.clone())
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

    /// An edit of the project as the session takes it: a scene's own location that is its
    /// scripted location's is none of its own (FORMAT.md).
    pub(super) fn plain_own(&self, edit: ProjectEdit) -> ProjectEdit {
        let same = |heading: &str, location: &Option<String>| {
            let scene = Scene {
                heading: heading.to_owned(),
                ..Scene::default()
            };
            let place = scene_location::place_of(&self.places, &scene);
            location.is_some() && place.is_some_and(|p| p.location == *location)
        };

        match edit {
            ProjectEdit::AddScene {
                number,
                heading,
                location,
            } if same(&heading, &location) => ProjectEdit::AddScene {
                number,
                heading,
                location: None,
            },
            ProjectEdit::EditScene {
                id,
                number,
                heading,
                location,
                sun,
            } if same(&heading, &location) => ProjectEdit::EditScene {
                id,
                number,
                heading,
                location: None,
                sun,
            },
            other => other,
        }
    }

    /// A location the scenes are shot at, or with a plan on it, stays: the list counts alike.
    pub(super) fn check_location_use(&self, edit: &ProjectEdit) -> Result<(), ServiceError> {
        let ProjectEdit::RemoveLocation { id } = edit else {
            return Ok(());
        };
        let (_, shot) = self.shot_scenes();
        let scenes = scene_location::scenes_using(&self.project, &shot, id);
        if scenes == 0 {
            return Ok(());
        }

        Err(ServiceError::LocationInUse {
            id: id.clone(),
            scenes,
        })
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
