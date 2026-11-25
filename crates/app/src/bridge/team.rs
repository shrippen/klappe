//! The team (Besetzung/Team): persons, their contacts, the casting of characters and the
//! person who answers for an element. A refusal goes to the status line like every other edit.

use super::q;
use super::qobject;
use super::view::to_json;
use core::pin::Pin;
use cxx_qt_lib::QString;
use klappe_services::{
    BreakdownEdit, Contact, Department, ElementCategory, Field, ProjectEdit, ServiceError, TeamEdit,
};

/// A department's name as the catalogue has it; empty is none.
fn department_of(name: &QString) -> Result<Option<Department>, ServiceError> {
    let name = name.to_string();
    if name.is_empty() {
        return Ok(None);
    }

    Department::from_name(&name)
        .map(Some)
        .ok_or(ServiceError::BadValue {
            field: Field::Kind,
            value: name,
        })
}

/// The number as typed; empty gives the character back to the book.
fn number_of(text: &QString) -> Result<Option<u32>, ServiceError> {
    let text = text.to_string();
    let text = text.trim();
    if text.is_empty() {
        return Ok(None);
    }

    text.parse().map(Some).map_err(|_| ServiceError::BadValue {
        field: Field::Number,
        value: text.to_owned(),
    })
}

/// An id; empty is nobody.
fn person_of(id: &QString) -> Option<String> {
    Some(id.to_string()).filter(|id| !id.is_empty())
}

impl qobject::Controller {
    /// The new person's id; empty when it was refused.
    pub(super) fn add_person(
        mut self: Pin<&mut Self>,
        name: &QString,
        department: &QString,
    ) -> QString {
        let department = match department_of(department) {
            Ok(department) => department,
            Err(e) => return self.made(Some(Err(e))),
        };
        let result = self
            .as_mut()
            .session()
            .map(|s| s.add_person(&name.to_string(), department));

        self.made(result)
    }

    pub(super) fn edit_person(
        self: Pin<&mut Self>,
        id: &QString,
        name: &QString,
        department: &QString,
        function: &QString,
        agency: &QString,
    ) -> bool {
        let edit = department_of(department).map(|department| {
            ProjectEdit::Team(TeamEdit::EditPerson {
                id: id.to_string(),
                name: name.to_string(),
                department,
                function: function.to_string(),
                agency: agency.to_string(),
            })
        });
        self.commit_project(edit)
    }

    pub(super) fn remove_person(self: Pin<&mut Self>, id: &QString) -> bool {
        self.commit_project(Ok(ProjectEdit::Team(TeamEdit::RemovePerson {
            id: id.to_string(),
        })))
    }

    pub(super) fn set_contact(
        self: Pin<&mut Self>,
        person: &QString,
        phone: &QString,
        mail: &QString,
        address: &QString,
    ) -> bool {
        self.commit_project(Ok(ProjectEdit::Team(TeamEdit::SetContact {
            person: person.to_string(),
            contact: Contact {
                phone: phone.to_string().trim().to_owned(),
                mail: mail.to_string().trim().to_owned(),
                address: address.to_string().trim().to_owned(),
            },
        })))
    }

    pub(super) fn propose_cast(
        self: Pin<&mut Self>,
        character: &QString,
        person: &QString,
    ) -> bool {
        self.commit_project(Ok(ProjectEdit::Team(TeamEdit::ProposeCast {
            character: character.to_string(),
            person: person.to_string(),
        })))
    }

    pub(super) fn fix_cast(self: Pin<&mut Self>, character: &QString, person: &QString) -> bool {
        self.commit_project(Ok(ProjectEdit::Team(TeamEdit::FixCast {
            character: character.to_string(),
            person: person_of(person),
        })))
    }

    pub(super) fn drop_proposal(
        self: Pin<&mut Self>,
        character: &QString,
        person: &QString,
    ) -> bool {
        self.commit_project(Ok(ProjectEdit::Team(TeamEdit::DropProposal {
            character: character.to_string(),
            person: person.to_string(),
        })))
    }

    pub(super) fn set_cast_number(
        self: Pin<&mut Self>,
        character: &QString,
        number: &QString,
    ) -> bool {
        let edit = number_of(number).map(|number| {
            ProjectEdit::Team(TeamEdit::SetCastNumber {
                character: character.to_string(),
                number,
            })
        });
        self.commit_project(edit)
    }

    pub(super) fn lock_numbers(self: Pin<&mut Self>) -> bool {
        self.commit_project(Ok(ProjectEdit::Team(TeamEdit::LockCastNumbers)))
    }

    pub(super) fn unlock_numbers(self: Pin<&mut Self>) -> bool {
        self.commit_project(Ok(ProjectEdit::Team(TeamEdit::UnlockCastNumbers)))
    }

    pub(super) fn set_responsible(
        self: Pin<&mut Self>,
        element: &QString,
        person: &QString,
    ) -> bool {
        self.commit_project(Ok(ProjectEdit::Breakdown(BreakdownEdit::SetResponsible {
            element: element.to_string(),
            person: person_of(person),
        })))
    }

    /// The persons, those of the department that answers for `category` first, as JSON.
    pub(super) fn persons_for(mut self: Pin<&mut Self>, category: &QString) -> QString {
        let category = ElementCategory::from_name(&category.to_string());
        let rows = match (category, self.as_mut().session()) {
            (Some(category), Some(s)) => s.persons_for(category),
            _ => Vec::new(),
        };

        q(&to_json(&rows))
    }
}
