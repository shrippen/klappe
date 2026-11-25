//! What the team tests share: a project with a book, a session on it and some edits.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_model::{BookState, Character, Contact, FigureColor, Project, Scene};
use klappe_services::{BreakdownEdit, ProjectEdit, Session, TeamEdit};

/// ANNA is in all three scenes, WÄRTER in 1 and 2, BOB in 2 and 3; CHRIS is in none. CROWD is a
/// group of extras the sheet of scene 3 names.
pub const BOOK: &str = "INT. WACHRAUM – TAG #1#\n\nANNA\nHallo.\n\nWÄRTER\nHalt.\n\n\
INT. HAFEN – NACHT #2#\n\nANNA\nHi.\n\nWÄRTER\nWer da?\n\nBOB\nIch.\n\n\
EXT. DECK – TAG #3#\n\nANNA\nLos.\n\nBOB\nJa.\n\n";

pub fn character(id: &str, name: &str, group: Option<u32>) -> Character {
    Character {
        id: id.into(),
        name: name.into(),
        short: name[..1].into(),
        color: FigureColor::ALL[0],
        group,
        size_m: None,
        aliases: Vec::new(),
    }
}

pub fn scene(id: &str, number: &str, heading: &str) -> Scene {
    Scene {
        id: id.into(),
        number: number.into(),
        heading: heading.into(),
        state: BookState::InBook,
        ..Scene::default()
    }
}

pub fn dir_of(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);

    dir
}

pub fn project() -> Project {
    let mut project = Project::new("Test");
    project.characters = vec![
        character("anna", "ANNA", None),
        character("waerter", "WÄRTER", None),
        character("bob", "BOB", None),
        character("crowd", "CROWD", Some(12)),
        character("chris", "CHRIS", None),
    ];
    project.scenes = vec![
        scene("aaaa", "1", "INT. WACHRAUM – TAG"),
        scene("bbbb", "2", "INT. HAFEN – NACHT"),
        scene("cccc", "3", "EXT. DECK – TAG"),
    ];

    project
}

pub fn open_with(name: &str, project: &Project, book: &str) -> Session {
    let dir = dir_of(name);
    klappe_store::save(&dir, project, Some(book)).unwrap();

    Session::open(&dir).unwrap()
}

/// The sheet of scene 3 names the extras.
pub fn open(name: &str) -> Session {
    let mut s = open_with(name, &project(), BOOK);
    s.edit_project(ProjectEdit::Breakdown(BreakdownEdit::AddSceneCharacter {
        scene: "cccc".into(),
        character: "crowd".into(),
    }))
    .unwrap();

    s
}

pub fn team(s: &mut Session, edit: TeamEdit) {
    s.edit_project(ProjectEdit::Team(edit)).unwrap();
}

/// Sets the contact of the person and leaves its other fields as they are.
pub fn set_contact(s: &mut Session, id: &str, contact: Contact) {
    let person = s.project().team.person(id).cloned().unwrap();
    team(
        s,
        TeamEdit::EditPerson {
            id: id.into(),
            name: person.name,
            department: person.department,
            function: person.function,
            agency: person.agency,
            contact,
        },
    );
}
