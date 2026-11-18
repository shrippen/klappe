//! Saved views (LH-119): a name for a choice of visible layers, cameras and figures. Saving
//! and removing one is a change of the project (undoable); choosing one is a view setting,
//! like the layer switches it sets.
//!
//! ```text
//! save_view("Nur Kameras") ──► SavedView { layers, cameras, figures } ──► projekt.toml
//! apply_view(id)           ──► hidden layers and tracks of the plan on view
//! ```

use std::collections::HashSet;

use klappe_model::SavedView;

use super::{Layer, Session, Visibility};
use crate::error::ServiceError;
use crate::project_edit;

impl Session {
    /// The saved views, in the order they were made.
    pub fn saved_views(&self) -> &[SavedView] {
        &self.project.views
    }

    /// Saves what the plan shows now (visible layers, cameras and figures of the plan on
    /// view) under `name`; a view of that name is replaced. Undoable. Returns the view's id.
    pub fn save_view(&mut self, name: &str) -> Result<String, ServiceError> {
        let shown = |id: &String| !self.plan_view.track_hidden(id);
        let plan = self.plan();
        let view = SavedView {
            id: String::new(),
            name: name.to_owned(),
            layers: Layer::ALL
                .into_iter()
                .filter(|l| self.layer(*l) == Visibility::Shown)
                .map(|l| l.name().to_owned())
                .collect(),
            cameras: plan
                .map(|p| p.cameras.iter().map(|c| c.id.clone()).collect::<Vec<_>>())
                .unwrap_or_default()
                .into_iter()
                .filter(shown)
                .collect(),
            figures: plan
                .map(|p| {
                    p.figures
                        .iter()
                        .map(|f| f.character.clone())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
                .into_iter()
                .filter(shown)
                .collect(),
        };

        let mut next = self.project.clone();
        let id = project_edit::save_view(&mut next, view)?;
        if next != self.project {
            self.commit_step(next, "SaveView");
        }

        Ok(id)
    }

    /// Shows the layers, cameras and figures the view `id` names and hides the rest of the
    /// plan on view; the lamps and other plans' tracks stay as they are.
    pub fn apply_view(&mut self, id: &str) -> Result<(), ServiceError> {
        let view = self
            .project
            .views
            .iter()
            .find(|v| v.id == id)
            .cloned()
            .ok_or(ServiceError::Gone)?;

        let named: HashSet<&str> = view.layers.iter().map(String::as_str).collect();
        for layer in Layer::ALL {
            let visibility = match named.contains(layer.name()) {
                true => Visibility::Shown,
                false => Visibility::Hidden,
            };
            self.set_layer(layer, visibility);
        }

        let tracks: Vec<(String, bool)> = self
            .plan()
            .map(|p| {
                let cameras = p.cameras.iter().map(|c| (c.id.clone(), &view.cameras));
                let figures = p
                    .figures
                    .iter()
                    .map(|f| (f.character.clone(), &view.figures));

                cameras
                    .chain(figures)
                    .map(|(id, shown)| (id.clone(), shown.contains(&id)))
                    .collect()
            })
            .unwrap_or_default();
        for (id, shown) in tracks {
            let visibility = match shown {
                true => Visibility::Shown,
                false => Visibility::Hidden,
            };
            self.set_track(&id, visibility);
        }

        Ok(())
    }

    /// Deletes the saved view `id`; undoable.
    pub fn remove_view(&mut self, id: &str) -> Result<(), ServiceError> {
        let mut next = self.project.clone();
        project_edit::remove_view(&mut next, id)?;
        self.commit_step(next, "RemoveView");

        Ok(())
    }
}
