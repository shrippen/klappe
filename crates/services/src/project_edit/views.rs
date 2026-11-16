//! The project's saved views (LH-119) as project edits: save, remove.

use klappe_model::{Project, SavedView};

use super::{slug, unique_id};
use crate::error::ServiceError;
use crate::field::Field;

const VIEW_ID_FALLBACK: &str = "view";

/// Keeps `view` in the project: under the name it has already, it replaces that view (same
/// id); else it is a new one with an id made from the name. `view.id` is ignored. The id.
pub(crate) fn save_view(project: &mut Project, view: SavedView) -> Result<String, ServiceError> {
    let name = view.name.trim().to_owned();
    if name.is_empty() {
        return Err(ServiceError::BadValue {
            field: Field::Name,
            value: String::new(),
        });
    }

    let same = project
        .views
        .iter_mut()
        .find(|v| v.name.to_lowercase() == name.to_lowercase());
    if let Some(old) = same {
        *old = SavedView {
            id: old.id.clone(),
            name: old.name.clone(),
            ..view
        };

        return Ok(old.id.clone());
    }

    let id = unique_id(
        &slug(&name, VIEW_ID_FALLBACK),
        project.views.iter().map(|v| v.id.as_str()),
    );
    project.views.push(SavedView {
        id: id.clone(),
        name,
        ..view
    });

    Ok(id)
}

pub(crate) fn remove_view(project: &mut Project, id: &str) -> Result<(), ServiceError> {
    let before = project.views.len();
    project.views.retain(|v| v.id != id);
    if project.views.len() == before {
        return Err(ServiceError::Gone);
    }

    Ok(())
}
