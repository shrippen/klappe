use std::path::Path;

use klappe_model::Point;

use super::{Layer, Lock, Session};
use crate::error::ServiceError;
use crate::project_edit::ProjectEdit;
use crate::templates;
use crate::views::TemplateEntry;

impl Session {
    /// What the template list offers: the built-in templates, then the project's own (LH-07).
    pub fn templates(&self) -> Vec<TemplateEntry> {
        templates::entries(&self.project.templates)
    }

    /// The middle of the set the plan on view is drawn on: where a template goes down unless
    /// the user says otherwise. The origin when the scene has no set.
    pub fn plan_middle(&self) -> Point {
        let set = self
            .scene()
            .and_then(|scene| self.project.location(&scene.location))
            .map(|location| &location.set);

        set.map_or(Point::default(), |set| {
            Point::new(set.width_m / 2.0, set.depth_m / 2.0)
        })
    }

    /// Puts the template `id` on the plan on view, its insertion point at `at`; one undo step.
    /// Refused when the figure or the camera layer is locked.
    pub fn apply_template(&mut self, id: &str, at: Point) -> Result<(), ServiceError> {
        for layer in [Layer::Figures, Layer::Cameras] {
            if self.lock(layer) == Lock::Locked {
                return Err(ServiceError::Locked(layer));
            }
        }

        let (scene, plan) = self.on_view()?;

        self.edit_project(ProjectEdit::PlaceTemplate {
            scene,
            plan,
            template: id.to_owned(),
            at,
        })
    }

    /// Files the figures and cameras of the plan on view as a template of the project under
    /// `name`; one undo step. A plan with neither is refused.
    pub fn save_template(&mut self, name: &str) -> Result<(), ServiceError> {
        let (scene, plan) = self.on_view()?;

        self.edit_project(ProjectEdit::SaveTemplate {
            scene,
            plan,
            name: name.to_owned(),
        })
    }

    /// Takes the templates of the file `path` into the project; one undo step. Returns how many
    /// it added: templates the project has already are not added again. A file with an unusable
    /// template is refused as a whole and changes nothing.
    pub fn import_templates(&mut self, path: &Path) -> Result<usize, ServiceError> {
        let items = klappe_store::load_template_file(path)?;
        let before = self.project.templates.len();

        self.edit_project(ProjectEdit::ImportTemplates { items })?;

        Ok(self.project.templates.len() - before)
    }

    /// Writes the project's own templates to the file `path`, to pass on. Returns how many it
    /// wrote; none to write is an error.
    pub fn export_templates(&self, path: &Path) -> Result<usize, ServiceError> {
        if self.project.templates.is_empty() {
            return Err(ServiceError::EmptyLibrary);
        }

        klappe_store::save_template_file(path, &self.project.templates)?;

        Ok(self.project.templates.len())
    }

    /// The ids of the scene and the plan on view.
    fn on_view(&self) -> Result<(String, String), ServiceError> {
        let (Some(scene), Some(plan)) = (self.scene(), self.plan()) else {
            return Err(ServiceError::Gone);
        };

        Ok((scene.id.clone(), plan.id.clone()))
    }
}
