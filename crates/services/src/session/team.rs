//! The team as the surfaces see it (Besetzung/Team): the persons, who plays which character
//! under which number, and the checks of the edits that need the book.

use std::collections::{HashMap, HashSet};

use klappe_model::{Character, Department, ElementCategory, Person, Project};
use klappe_script::Element as Line;

use super::Session;
use crate::cast_number::{self, Appearance, Numbers};
use crate::error::ServiceError;
use crate::project_edit::{self, ProjectEdit, TeamEdit};
use crate::views::{CastingRow, PersonRow};

impl Session {
    /// The persons with their contact, in the order they were made.
    pub(super) fn person_rows(&self) -> Vec<PersonRow> {
        let team = &self.project.team;

        team.persons
            .iter()
            .map(|p| person_row(p, team.contacts.get(&p.id).cloned().unwrap_or_default()))
            .collect()
    }

    /// The persons for an element of `category` to be put in charge of: those of the
    /// department that usually answers for it first, then the others, each in the order they
    /// were made.
    pub fn persons_for(&self, category: ElementCategory) -> Vec<PersonRow> {
        let department = Department::of_category(category);
        let (mut own, rest): (Vec<PersonRow>, Vec<PersonRow>) = self
            .person_rows()
            .into_iter()
            .partition(|p| p.department == Some(department.name()));
        own.extend(rest);

        own
    }

    /// The casting of every character of the project: persons it knows, the cast number and
    /// the scenes of the book it is in.
    pub(super) fn casting_rows(&self) -> Vec<CastingRow> {
        let numbers = self.cast_numbers();
        let team = &self.project.team;
        let known = |id: &&String| team.person(id).is_some();
        let seen = self.scene_counts();

        self.project
            .characters
            .iter()
            .zip(seen)
            .map(|(c, (scenes, _))| {
                let casting = team.casting(&c.id);

                CastingRow {
                    character: c.id.clone(),
                    fixed: casting
                        .and_then(|k| k.fixed.clone())
                        .filter(|id| known(&id)),
                    proposed: casting
                        .map(|k| k.proposed.iter().filter(known).cloned().collect())
                        .unwrap_or_default(),
                    number: numbers.get(&c.id).copied(),
                    stored: casting.is_some_and(|k| k.cast_number.is_some()),
                    scenes,
                }
            })
            .collect()
    }

    /// The number of every character that has one: stored, or worked out from the book.
    pub(super) fn cast_numbers(&self) -> HashMap<String, u32> {
        self.numbers_in(&self.project)
    }

    /// Like `cast_numbers`, for the team of `project`, a copy of this session's.
    fn numbers_in(&self, project: &Project) -> HashMap<String, u32> {
        let seen = self.scene_counts();
        let appearances: Vec<Appearance> = project
            .characters
            .iter()
            .zip(seen)
            .map(|(c, (scenes, first))| appearance(project, c, scenes, first))
            .collect();
        let mode = match project.team.numbers_locked {
            true => Numbers::Locked,
            false => Numbers::Moving,
        };

        cast_number::cast_numbers(&appearances, mode)
    }

    /// For each character of the project, in how many scenes of the book it is and the place
    /// of its first one (`usize::MAX` for none): its cues and the sheet's characters, as in
    /// the report of the cast. One pass over the book.
    fn scene_counts(&self) -> Vec<(usize, usize)> {
        let characters = &self.project.characters;
        let book = self.script.scenes();
        let mut counts = vec![(0, usize::MAX); characters.len()];
        let mut named: HashMap<&str, Option<usize>> = HashMap::new();

        for (place, (entry, scene)) in self.book_scenes().enumerate() {
            let mut here: HashSet<usize> = HashSet::new();

            let view = self.book_scene_of(entry).and_then(|n| book.get(n));
            for element in view.map(|v| v.elements).unwrap_or_default() {
                let Line::Character { name, .. } = element else {
                    continue;
                };
                let found = named
                    .entry(name)
                    .or_insert_with(|| characters.iter().position(|c| c.is_called(name)));
                here.extend(*found);
            }
            for id in &scene.sheet.characters {
                here.extend(characters.iter().position(|c| c.id == *id));
            }

            for at in here {
                counts[at].0 += 1;
                counts[at].1 = counts[at].1.min(place);
            }
        }

        counts
    }

    /// A cast number another character has is refused.
    pub(super) fn check_cast_number(&self, edit: &ProjectEdit) -> Result<(), ServiceError> {
        let ProjectEdit::Team(TeamEdit::SetCastNumber {
            character,
            number: Some(number),
        }) = edit
        else {
            return Ok(());
        };

        let taken = self
            .cast_numbers()
            .iter()
            .any(|(id, n)| id != character && n == number);
        match taken {
            true => Err(ServiceError::CastNumberTaken(*number)),
            false => Ok(()),
        }
    }

    /// Locking the numbers stores the ones the characters have now, as the book gives them.
    pub(super) fn store_numbers(&self, edit: &ProjectEdit, next: &mut Project) {
        if !matches!(edit, ProjectEdit::Team(TeamEdit::LockCastNumbers)) {
            return;
        }

        project_edit::store_numbers(next, &self.cast_numbers());
    }
}

fn appearance<'a>(
    project: &Project,
    character: &'a Character,
    scenes: usize,
    first: usize,
) -> Appearance<'a> {
    Appearance {
        id: &character.id,
        name: &character.name,
        extra: character.group.is_some(),
        scenes,
        first,
        stored: project
            .team
            .casting(&character.id)
            .and_then(|k| k.cast_number),
    }
}

fn person_row(person: &Person, contact: klappe_model::Contact) -> PersonRow {
    PersonRow {
        id: person.id.clone(),
        name: person.name.clone(),
        department: person.department.map(Department::name),
        function: person.function.clone(),
        agency: person.agency.clone(),
        phone: contact.phone,
        mail: contact.mail,
        address: contact.address,
    }
}
