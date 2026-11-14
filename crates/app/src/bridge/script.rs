//! Scenes, locations and characters of the project.

use super::qobject;
use crate::manage;
use core::pin::Pin;
use cxx_qt_lib::QString;
use klappe_services::{Field, FigureColor, ProjectEdit, ServiceError};

impl qobject::Controller {
    pub(super) fn add_scene(
        self: Pin<&mut Self>,
        number: &QString,
        heading: &QString,
        location: &QString,
    ) -> bool {
        self.commit_project(Ok(ProjectEdit::AddScene {
            number: number.to_string(),
            heading: heading.to_string(),
            location: location.to_string(),
        }))
    }

    pub(super) fn edit_scene(
        self: Pin<&mut Self>,
        id: &QString,
        number: &QString,
        heading: &QString,
        location: &QString,
    ) -> bool {
        self.commit_project(Ok(ProjectEdit::EditScene {
            id: id.to_string(),
            number: number.to_string(),
            heading: heading.to_string(),
            location: location.to_string(),
        }))
    }

    pub(super) fn remove_scene(self: Pin<&mut Self>, id: &QString) -> bool {
        self.commit_project(Ok(ProjectEdit::RemoveScene { id: id.to_string() }))
    }

    pub(super) fn add_location(
        self: Pin<&mut Self>,
        name: &QString,
        width: &QString,
        depth: &QString,
    ) -> bool {
        let edit = sizes(width, depth).map(|(width_m, depth_m)| ProjectEdit::AddLocation {
            name: name.to_string(),
            width_m,
            depth_m,
        });
        self.commit_project(edit)
    }

    pub(super) fn edit_location(
        self: Pin<&mut Self>,
        id: &QString,
        name: &QString,
        width: &QString,
        depth: &QString,
    ) -> bool {
        let edit = sizes(width, depth).map(|(width_m, depth_m)| ProjectEdit::EditLocation {
            id: id.to_string(),
            name: name.to_string(),
            width_m,
            depth_m,
        });
        self.commit_project(edit)
    }

    pub(super) fn remove_location(self: Pin<&mut Self>, id: &QString) -> bool {
        self.commit_project(Ok(ProjectEdit::RemoveLocation { id: id.to_string() }))
    }

    pub(super) fn add_character(
        self: Pin<&mut Self>,
        name: &QString,
        label: &QString,
        color: &QString,
        group: u32,
    ) -> bool {
        let edit = color_edit(color).map(|color| ProjectEdit::AddCharacter {
            name: name.to_string(),
            short: label.to_string(),
            color,
            group: manage::group_of(group),
        });
        self.commit_project(edit)
    }

    pub(super) fn edit_character(
        self: Pin<&mut Self>,
        id: &QString,
        name: &QString,
        label: &QString,
        color: &QString,
        group: u32,
    ) -> bool {
        let edit = color_edit(color).map(|color| ProjectEdit::EditCharacter {
            id: id.to_string(),
            name: name.to_string(),
            short: label.to_string(),
            color,
            group: manage::group_of(group),
        });
        self.commit_project(edit)
    }

    pub(super) fn remove_character(self: Pin<&mut Self>, id: &QString) -> bool {
        self.commit_project(Ok(ProjectEdit::RemoveCharacter { id: id.to_string() }))
    }
}

/// Width and depth typed in the location form.
fn sizes(width: &QString, depth: &QString) -> Result<(f64, f64), ServiceError> {
    manage::sizes(&width.to_string(), &depth.to_string())
}

/// The colour a form sends by its role name.
fn color_edit(name: &QString) -> Result<FigureColor, ServiceError> {
    let name = name.to_string();

    FigureColor::from_name(&name).ok_or(ServiceError::BadValue {
        field: Field::Color,
        value: name,
    })
}
