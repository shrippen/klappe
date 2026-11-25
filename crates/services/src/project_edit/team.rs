//! The team's edits (Besetzung/Team): persons, their contacts and the casting of characters.

use std::collections::HashMap;

use klappe_model::{free_random_id, Casting, Contact, Department, IdCase, Person, Project};

use crate::error::ServiceError;
use crate::field::Field;

/// Edits of the persons, the contacts and the casting.
#[derive(Debug, Clone, PartialEq)]
pub enum TeamEdit {
    /// A new person. Refused: an empty name.
    AddPerson {
        name: String,
        department: Option<Department>,
    },
    /// The person's fields and contact in one step; an empty contact removes it.
    EditPerson {
        id: String,
        name: String,
        department: Option<Department>,
        function: String,
        agency: String,
        contact: Contact,
    },
    /// Takes the person out of every casting and every element's answer too.
    RemovePerson {
        id: String,
    },
    /// The person is a proposal for the character.
    ProposeCast {
        character: String,
        person: String,
    },
    /// A new person in the cast department is a proposal for the character, in one step.
    /// Refused: an empty name, an unknown character.
    ProposeNew {
        character: String,
        name: String,
    },
    /// The person plays the character; a person not yet proposed is proposed, the other
    /// proposals stay. `None` fixes nobody.
    FixCast {
        character: String,
        person: Option<String>,
    },
    /// The person is no proposal any more; the fixed one is no longer fixed.
    DropProposal {
        character: String,
        person: String,
    },
    /// The character's number by hand; `None` gives it back to the book. The session refuses
    /// a number another character has.
    SetCastNumber {
        character: String,
        number: Option<u32>,
    },
    /// The numbers stay as they are now: the session stores them, new characters take the next.
    LockCastNumbers,
    UnlockCastNumbers,
}

/// What the team edits do.
pub(super) fn apply(project: &mut Project, edit: &TeamEdit) -> Result<(), ServiceError> {
    match edit {
        TeamEdit::AddPerson { name, department } => {
            add_person(project, name, *department).map(drop)
        }
        TeamEdit::EditPerson {
            id,
            name,
            department,
            function,
            agency,
            contact,
        } => edit_person(
            project,
            id,
            (name, *department),
            (function, agency, contact),
        ),
        TeamEdit::RemovePerson { id } => remove_person(project, id),
        TeamEdit::ProposeCast { character, person } => propose(project, character, person),
        TeamEdit::ProposeNew { character, name } => propose_new(project, character, name),
        TeamEdit::FixCast { character, person } => fix(project, character, person.as_deref()),
        TeamEdit::DropProposal { character, person } => drop_proposal(project, character, person),
        TeamEdit::SetCastNumber { character, number } => set_number(project, character, *number),
        TeamEdit::LockCastNumbers => {
            project.team.numbers_locked = true;
            Ok(())
        }
        TeamEdit::UnlockCastNumbers => {
            project.team.numbers_locked = false;
            Ok(())
        }
    }
}

pub(crate) fn check_person(project: &Project, id: &str) -> Result<(), ServiceError> {
    match project.team.person(id) {
        Some(_) => Ok(()),
        None => Err(ServiceError::NoPerson(id.to_owned())),
    }
}

fn check_character(project: &Project, id: &str) -> Result<(), ServiceError> {
    match project.character(id) {
        Some(_) => Ok(()),
        None => Err(ServiceError::NoCharacter(id.to_owned())),
    }
}

/// The trimmed name, not empty.
fn name_of(name: &str) -> Result<String, ServiceError> {
    match name.trim() {
        "" => Err(ServiceError::BadValue {
            field: Field::Name,
            value: String::new(),
        }),
        name => Ok(name.to_owned()),
    }
}

fn add_person(
    project: &mut Project,
    name: &str,
    department: Option<Department>,
) -> Result<String, ServiceError> {
    let name = name_of(name)?;
    let taken = project.team.persons.iter().map(|p| p.id.as_str());

    let id = free_random_id(taken, IdCase::Exact);
    project.team.persons.push(Person {
        id: id.clone(),
        name,
        department,
        function: String::new(),
        agency: String::new(),
    });

    Ok(id)
}

fn edit_person(
    project: &mut Project,
    id: &str,
    (name, department): (&str, Option<Department>),
    (function, agency, contact): (&str, &str, &Contact),
) -> Result<(), ServiceError> {
    check_person(project, id)?;
    let name = name_of(name)?;

    let Some(person) = project.team.persons.iter_mut().find(|p| p.id == id) else {
        return Ok(());
    };
    person.name = name;
    person.department = department;
    person.function = function.trim().to_owned();
    person.agency = agency.trim().to_owned();
    set_contact(project, id, contact);

    Ok(())
}

/// Takes the person out of the team, the castings, the contacts and the elements.
fn remove_person(project: &mut Project, id: &str) -> Result<(), ServiceError> {
    check_person(project, id)?;

    let team = &mut project.team;
    team.persons.retain(|p| p.id != id);
    team.contacts.remove(id);
    for casting in &mut team.castings {
        casting.proposed.retain(|p| p != id);
        if casting.fixed.as_deref() == Some(id) {
            casting.fixed = None;
        }
    }
    team.castings.retain(|c| !c.is_empty());

    for element in &mut project.elements {
        if element.responsible.as_deref() == Some(id) {
            element.responsible = None;
        }
    }

    Ok(())
}

/// Stores the trimmed contact of the person; an empty one removes it.
fn set_contact(project: &mut Project, person: &str, contact: &Contact) {
    let contact = Contact {
        phone: contact.phone.trim().to_owned(),
        mail: contact.mail.trim().to_owned(),
        address: contact.address.trim().to_owned(),
    };
    match contact.is_empty() {
        true => project.team.contacts.remove(person),
        false => project.team.contacts.insert(person.to_owned(), contact),
    };
}

/// The casting of the character, made if there is none.
fn casting_mut<'a>(project: &'a mut Project, character: &str) -> &'a mut Casting {
    let castings = &mut project.team.castings;
    let at = match castings.iter().position(|c| c.character == character) {
        Some(at) => at,
        None => {
            castings.push(Casting {
                character: character.to_owned(),
                ..Casting::default()
            });
            castings.len() - 1
        }
    };

    &mut castings[at]
}

/// Stores `numbers` in the castings, one made for a character that has none yet.
pub(crate) fn store_numbers(project: &mut Project, numbers: &HashMap<String, u32>) {
    for (character, number) in numbers {
        casting_mut(project, character).cast_number = Some(*number);
    }
    tidy(project);
}

/// A casting that says nothing has no entry.
fn tidy(project: &mut Project) {
    project.team.castings.retain(|c| !c.is_empty());
}

fn propose(project: &mut Project, character: &str, person: &str) -> Result<(), ServiceError> {
    check_character(project, character)?;
    check_person(project, person)?;

    let casting = casting_mut(project, character);
    if !casting.proposed.iter().any(|p| p == person) {
        casting.proposed.push(person.to_owned());
    }

    Ok(())
}

fn propose_new(project: &mut Project, character: &str, name: &str) -> Result<(), ServiceError> {
    check_character(project, character)?;

    let person = add_person(project, name, Some(Department::Cast))?;

    propose(project, character, &person)
}

fn fix(project: &mut Project, character: &str, person: Option<&str>) -> Result<(), ServiceError> {
    check_character(project, character)?;
    let Some(person) = person else {
        casting_mut(project, character).fixed = None;
        tidy(project);

        return Ok(());
    };
    check_person(project, person)?;

    let casting = casting_mut(project, character);
    if !casting.proposed.iter().any(|p| p == person) {
        casting.proposed.push(person.to_owned());
    }
    casting.fixed = Some(person.to_owned());

    Ok(())
}

fn drop_proposal(project: &mut Project, character: &str, person: &str) -> Result<(), ServiceError> {
    check_character(project, character)?;

    let casting = casting_mut(project, character);
    casting.proposed.retain(|p| p != person);
    if casting.fixed.as_deref() == Some(person) {
        casting.fixed = None;
    }
    tidy(project);

    Ok(())
}

fn set_number(
    project: &mut Project,
    character: &str,
    number: Option<u32>,
) -> Result<(), ServiceError> {
    check_character(project, character)?;
    if number == Some(0) {
        return Err(ServiceError::BadValue {
            field: Field::Number,
            value: "0".into(),
        });
    }

    casting_mut(project, character).cast_number = number;
    tidy(project);

    Ok(())
}
