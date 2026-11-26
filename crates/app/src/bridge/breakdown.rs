//! The breakdown (D-22): elements, the sheet of the scene on view and the objects of a set that
//! stand for an element. A refusal goes to the status line like every other edit.

use super::qobject;
use core::pin::Pin;
use cxx_qt_lib::QString;
use klappe_services::{
    BreakdownEdit, ElementCategory, ElementStatus, Field, MarkTarget, ProjectEdit, ServiceError,
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
    /// The new element's id; empty when it was refused.
    pub(super) fn add_element(
        mut self: Pin<&mut Self>,
        category: &QString,
        name: &QString,
    ) -> QString {
        let category = match category_of(category) {
            Ok(category) => category,
            Err(e) => return self.made(Some(Err(e))),
        };
        let result = self
            .as_mut()
            .session()
            .map(|s| s.add_element(category, &name.to_string()));

        self.made(result)
    }

    pub(super) fn edit_element(
        self: Pin<&mut Self>,
        id: &QString,
        category: &QString,
        name: &QString,
        description: &QString,
        status: &QString,
        responsible: &QString,
    ) -> bool {
        let edit = category_of(category).and_then(|category| {
            Ok(ProjectEdit::Breakdown(BreakdownEdit::EditElement {
                id: id.to_string(),
                category,
                name: name.to_string(),
                description: description.to_string(),
                status: status_of(status)?,
                responsible: Some(responsible.to_string()).filter(|id| !id.is_empty()),
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
        mut self: Pin<&mut Self>,
        scene: &QString,
        story_day: &QString,
        prestop: &QString,
        planned: &QString,
        continuity: &QString,
        note: &QString,
    ) -> QString {
        let edit = ProjectEdit::Breakdown(BreakdownEdit::SetSheet {
            scene: scene.to_string(),
            story_day: story_day.to_string(),
            prestop: prestop.to_string(),
            planned: planned.to_string(),
            continuity: continuity.to_string(),
            note: note.to_string(),
        });
        let result = self.as_mut().session().map(|s| s.edit_project(edit));

        // The page marks the field that was refused.
        let refused = match &result {
            Some(Err(ServiceError::BadValue { field, .. })) => field.name(),
            _ => "",
        };
        let refused = QString::from(refused);
        self.report(result);
        refused
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

    pub(super) fn mark_text(
        mut self: Pin<&mut Self>,
        scene: &QString,
        line: i32,
        prefix: &QString,
        text: &QString,
        element: &QString,
        category: &QString,
    ) -> bool {
        let target = match element.is_empty() {
            true => category_of(category).map(MarkTarget::New),
            false => Ok(MarkTarget::Element(element.to_string())),
        };
        let Ok(line) = usize::try_from(line) else {
            return false;
        };
        let (prefix, text) = (prefix.to_string(), text.to_string());
        let result = self
            .as_mut()
            .session()
            .map(|s| s.mark_text(&scene.to_string(), (line, &prefix, &text), target?));

        self.report(result)
    }

    pub(super) fn forget_lost(
        mut self: Pin<&mut Self>,
        scene: &QString,
        element: &QString,
    ) -> bool {
        let result = self
            .as_mut()
            .session()
            .map(|s| s.forget_lost(&scene.to_string(), &element.to_string()));

        self.report(result)
    }

    /// The new look's id; empty when it was refused.
    pub(super) fn add_look(
        mut self: Pin<&mut Self>,
        character: &QString,
        name: &QString,
    ) -> QString {
        let result = self
            .as_mut()
            .session()
            .map(|s| s.add_look(&character.to_string(), &name.to_string()));

        self.made(result)
    }

    pub(super) fn edit_look(self: Pin<&mut Self>, id: &QString, name: &QString) -> bool {
        self.commit_project(Ok(ProjectEdit::Breakdown(BreakdownEdit::EditLook {
            id: id.to_string(),
            name: name.to_string(),
        })))
    }

    pub(super) fn remove_look(self: Pin<&mut Self>, id: &QString) -> bool {
        self.commit_project(Ok(ProjectEdit::Breakdown(BreakdownEdit::RemoveLook {
            id: id.to_string(),
        })))
    }

    pub(super) fn add_element_to_look(
        self: Pin<&mut Self>,
        look: &QString,
        category: &QString,
        name: &QString,
    ) -> bool {
        let edit = category_of(category).map(|category| {
            ProjectEdit::Breakdown(BreakdownEdit::AddElementToLook {
                look: look.to_string(),
                category,
                name: name.to_string(),
            })
        });
        self.commit_project(edit)
    }

    pub(super) fn remove_look_element(
        self: Pin<&mut Self>,
        look: &QString,
        element: &QString,
    ) -> bool {
        self.commit_project(Ok(ProjectEdit::Breakdown(
            BreakdownEdit::RemoveLookElement {
                look: look.to_string(),
                element: element.to_string(),
            },
        )))
    }

    pub(super) fn set_scene_look(
        self: Pin<&mut Self>,
        scene: &QString,
        character: &QString,
        look: &QString,
    ) -> bool {
        let look = look.to_string();

        self.commit_project(Ok(ProjectEdit::Breakdown(BreakdownEdit::SetSceneLook {
            scene: scene.to_string(),
            character: character.to_string(),
            look: Some(look).filter(|l| !l.is_empty()),
        })))
    }
}
