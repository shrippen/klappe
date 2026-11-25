//! The team (DATENMODELL §3): `personen.toml` (persons, casting, whether the cast numbers are
//! locked) and `kontakte.toml` (how to reach the persons, a file of its own for the rights of
//! DATENMODELL §10). Both are optional.

use std::collections::{BTreeMap, HashSet};
use std::path::Path;

use klappe_model::{Casting, Contact, Person, Team};
use serde::{Deserialize, Serialize};

use crate::files;
use crate::{OptionalFile, StoreError, CONTACTS_FILE, PEOPLE_FILE};

/// `personen.toml`.
#[derive(Serialize, Deserialize)]
struct People {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    numbers_locked: bool,
    #[serde(default)]
    person: Vec<Person>,
    #[serde(default)]
    besetzung: Vec<Casting>,
}

/// `kontakte.toml`.
#[derive(Serialize, Deserialize)]
struct Contacts {
    #[serde(default)]
    kontakt: Vec<Entry>,
}

/// One `[[kontakt]]`: the contact of a person.
#[derive(Serialize, Deserialize)]
struct Entry {
    person: String,
    #[serde(flatten)]
    contact: Contact,
}

/// The team of the folder `dir`; ids that name no person or character are left in.
pub(crate) fn load(dir: &Path) -> Result<Team, StoreError> {
    let mut team = Team::default();

    let path = dir.join(PEOPLE_FILE);
    if files::exists(&path)? {
        let people: People = files::read_toml(&path)?;
        check_people(&people.person, &people.besetzung)
            .map_err(|message| StoreError::Parse { path, message })?;
        team.persons = people.person;
        team.castings = people.besetzung;
        team.numbers_locked = people.numbers_locked;
    }

    let path = dir.join(CONTACTS_FILE);
    if files::exists(&path)? {
        let contacts: Contacts = files::read_toml(&path)?;
        let entries: Vec<&str> = contacts.kontakt.iter().map(|e| e.person.as_str()).collect();
        check_ids(&entries).map_err(|message| StoreError::Parse { path, message })?;
        team.contacts = contacts
            .kontakt
            .into_iter()
            .map(|e| (e.person, e.contact))
            .collect();
    }

    Ok(team)
}

/// Ids are not empty and used once; one casting per character; a stored number is above 0.
fn check_people(persons: &[Person], castings: &[Casting]) -> Result<(), String> {
    let ids: Vec<&str> = persons.iter().map(|p| p.id.as_str()).collect();
    check_ids(&ids)?;

    let characters: Vec<&str> = castings.iter().map(|c| c.character.as_str()).collect();
    check_ids(&characters)?;
    if castings.iter().any(|c| c.cast_number == Some(0)) {
        return Err("a cast number is 1 or more".into());
    }

    Ok(())
}

fn check_ids(ids: &[&str]) -> Result<(), String> {
    let mut seen = HashSet::new();

    for id in ids {
        if id.trim().is_empty() {
            return Err("an entry has no id".into());
        }
        if !seen.insert(*id) {
            return Err(format!("{id}: the id is used twice"));
        }
    }

    Ok(())
}

/// `personen.toml` and `kontakte.toml`, each `None` when there is nothing for it.
pub(crate) fn encode(dir: &Path, team: &Team) -> Result<Vec<OptionalFile>, StoreError> {
    let path = dir.join(PEOPLE_FILE);
    let empty = team.persons.is_empty() && team.castings.is_empty() && !team.numbers_locked;
    let people = match empty {
        true => None,
        false => {
            check_people(&team.persons, &team.castings).map_err(|message| StoreError::Parse {
                path: path.clone(),
                message,
            })?;
            let file = People {
                numbers_locked: team.numbers_locked,
                person: team.persons.clone(),
                besetzung: team.castings.clone(),
            };
            Some(files::encode_toml(&path, &file)?)
        }
    };

    let contacts_path = dir.join(CONTACTS_FILE);
    let held: BTreeMap<&String, &Contact> = team
        .contacts
        .iter()
        .filter(|(_, c)| !c.is_empty())
        .collect();
    let contacts = match held.is_empty() {
        true => None,
        false => {
            let file = Contacts {
                kontakt: held
                    .into_iter()
                    .map(|(person, contact)| Entry {
                        person: person.clone(),
                        contact: contact.clone(),
                    })
                    .collect(),
            };
            Some(files::encode_toml(&contacts_path, &file)?)
        }
    };

    Ok(vec![(path, people), (contacts_path, contacts)])
}
