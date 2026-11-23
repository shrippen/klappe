//! The breakdown (D-22): elements, the sheet of the scene on view and the objects of a set that
//! stand for an element. A refusal goes to the status line like every other edit.

use super::qobject;
use core::pin::Pin;
use cxx_qt_lib::QString;
use klappe_services::{
    BreakdownEdit, ElementCategory, ElementStatus, Field, ProjectEdit, ServiceError,
};

fn category_of(name: &QString) -> Result<ElementCategory, ServiceError> {
    let name = name.to_string();

    ElementCategory::from_name(&name).ok_or(ServiceError::BadValue {
        field: Field::Kind,
        value: name,
    })
}

fn status_of(name: &QString) -> Result<ElementStatus, ServiceError> {
    let name = name.to_string();

    ElementStatus::from_name(&name).ok_or(ServiceError::BadValue {
        field: Field::Status,
        value: name,
    })
}

impl qobject::Controller {
    pub(super) fn add_element(self: Pin<&mut Self>, category: &QString, name: &QString) -> bool {
        let edit = category_of(category).map(|category| {
            ProjectEdit::Breakdown(BreakdownEdit::AddElement {
                category,
                name: name.to_string(),
            })
        });
        self.commit_project(edit)
    }

    pub(super) fn edit_element(
        self: Pin<&mut Self>,
        id: &QString,
        category: &QString,
        name: &QString,
        description: &QString,
        status: &QString,
    ) -> bool {
        let edit = category_of(category).and_then(|category| {
            Ok(ProjectEdit::Breakdown(BreakdownEdit::EditElement {
                id: id.to_string(),
                category,
                name: name.to_string(),
                description: description.to_string(),
                status: status_of(status)?,
            }))
        });
        self.commit_project(edit)
    }

    pub(super) fn remove_element(self: Pin<&mut Self>, id: &QString) -> bool {
        self.commit_project(Ok(ProjectEdit::Breakdown(BreakdownEdit::RemoveElement {
            id: id.to_string(),
        })))
    }

    pub(super) fn set_sheet(
        self: Pin<&mut Self>,
        scene: &QString,
        story_day: &QString,
        prestop: &QString,
        planned: &QString,
        continuity: &QString,
        note: &QString,
    ) -> bool {
        self.commit_project(Ok(ProjectEdit::Breakdown(BreakdownEdit::SetSheet {
            scene: scene.to_string(),
            story_day: story_day.to_string(),
            prestop: prestop.to_string(),
            planned: planned.to_string(),
            continuity: continuity.to_string(),
            note: note.to_string(),
        })))
    }

    pub(super) fn add_element_to_scene(
        self: Pin<&mut Self>,
        scene: &QString,
        category: &QString,
        name: &QString,
    ) -> bool {
        let edit = category_of(category).map(|category| {
            ProjectEdit::Breakdown(BreakdownEdit::AddElementToScene {
                scene: scene.to_string(),
                category,
                name: name.to_string(),
            })
        });
        self.commit_project(edit)
    }

    pub(super) fn remove_scene_element(
        self: Pin<&mut Self>,
        scene: &QString,
        element: &QString,
    ) -> bool {
        self.commit_project(Ok(ProjectEdit::Breakdown(
            BreakdownEdit::RemoveSceneElement {
                scene: scene.to_string(),
                element: element.to_string(),
            },
        )))
    }

    pub(super) fn take_suggestion(
        self: Pin<&mut Self>,
        scene: &QString,
        element: &QString,
    ) -> bool {
        self.commit_project(Ok(ProjectEdit::Breakdown(BreakdownEdit::TakeSuggestion {
            scene: scene.to_string(),
            element: element.to_string(),
        })))
    }

    pub(super) fn exclude_suggestion(
        self: Pin<&mut Self>,
        scene: &QString,
        element: &QString,
    ) -> bool {
        self.commit_project(Ok(ProjectEdit::Breakdown(
            BreakdownEdit::ExcludeSuggestion {
                scene: scene.to_string(),
                element: element.to_string(),
            },
        )))
    }

    pub(super) fn add_scene_character(
        self: Pin<&mut Self>,
        scene: &QString,
        character: &QString,
    ) -> bool {
        self.commit_project(Ok(ProjectEdit::Breakdown(
            BreakdownEdit::AddSceneCharacter {
                scene: scene.to_string(),
                character: character.to_string(),
            },
        )))
    }

    pub(super) fn remove_scene_character(
        self: Pin<&mut Self>,
        scene: &QString,
        character: &QString,
    ) -> bool {
        self.commit_project(Ok(ProjectEdit::Breakdown(
            BreakdownEdit::RemoveSceneCharacter {
                scene: scene.to_string(),
                character: character.to_string(),
            },
        )))
    }

    pub(super) fn set_object_element(
        self: Pin<&mut Self>,
        location: &QString,
        object: &QString,
        element: &QString,
    ) -> bool {
        let element = element.to_string();

        self.commit_project(Ok(ProjectEdit::Breakdown(
            BreakdownEdit::SetObjectElement {
                location: location.to_string(),
                object: object.to_string(),
                element: Some(element).filter(|e| !e.is_empty()),
            },
        )))
    }
}
