//! Besetzung/Team: persons, contacts, the casting of characters, cast numbers and the contact
//! list; each edit is one undo step and the team survives a save.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_model::{BookState, Character, Contact, FigureColor, Project, Scene};
use klappe_services::{
    BreakdownEdit, Department, ElementCategory, Language, OnOutside, ProjectEdit, Report,
    ReportFormat, ServiceError, Session, TeamEdit,
};

/// ANNA is in all three scenes, WÄRTER in 1 and 2, BOB in 2 and 3; CHRIS is in none. CROWD is a
/// group of extras the sheet of scene 3 names.
const BOOK: &str = "INT. WACHRAUM – TAG #1#\n\nANNA\nHallo.\n\nWÄRTER\nHalt.\n\n\
INT. HAFEN – NACHT #2#\n\nANNA\nHi.\n\nWÄRTER\nWer da?\n\nBOB\nIch.\n\n\
EXT. DECK – TAG #3#\n\nANNA\nLos.\n\nBOB\nJa.\n\n";

fn character(id: &str, name: &str, group: Option<u32>) -> Character {
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

fn scene(id: &str, number: &str, heading: &str) -> Scene {
    Scene {
        id: id.into(),
        number: number.into(),
        heading: heading.into(),
        state: BookState::InBook,
        ..Scene::default()
    }
}

fn dir_of(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);

    dir
}

fn project() -> Project {
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

fn open_with(name: &str, project: &Project, book: &str) -> Session {
    let dir = dir_of(name);
    klappe_store::save(&dir, project, Some(book)).unwrap();

    Session::open(&dir).unwrap()
}

/// The sheet of scene 3 names the extras.
fn open(name: &str) -> Session {
    let mut s = open_with(name, &project(), BOOK);
    s.edit_project(ProjectEdit::Breakdown(BreakdownEdit::AddSceneCharacter {
        scene: "cccc".into(),
        character: "crowd".into(),
    }))
    .unwrap();

    s
}

fn team(s: &mut Session, edit: TeamEdit) {
    s.edit_project(ProjectEdit::Team(edit)).unwrap();
}

fn refused(s: &mut Session, edit: TeamEdit) -> ServiceError {
    s.edit_project(ProjectEdit::Team(edit)).unwrap_err()
}

/// The cast number the overview gives a character.
fn number(s: &Session, character: &str) -> Option<u32> {
    let castings = s.overview().castings;

    castings.iter().find(|c| c.character == character)?.number
}

fn numbers(s: &Session) -> Vec<Option<u32>> {
    ["anna", "waerter", "bob", "crowd", "chris"]
        .map(|c| number(s, c))
        .into()
}

fn propose(s: &mut Session, character: &str, person: &str) {
    team(
        s,
        TeamEdit::ProposeCast {
            character: character.into(),
            person: person.into(),
        },
    );
}

fn casting(s: &Session, character: &str) -> klappe_model::Casting {
    s.project().team.casting(character).cloned().unwrap()
}

#[test]
fn a_person_is_added_edited_and_removed() {
    let mut s = open("team-person");

    let id = s.add_person("  Ines ", Some(Department::Lighting)).unwrap();
    assert_eq!(s.undo_step(), Some("AddPerson"));
    let rows = s.overview().persons;
    assert_eq!(
        (rows[0].name.as_str(), rows[0].department),
        ("Ines", Some("lighting"))
    );

    team(
        &mut s,
        TeamEdit::EditPerson {
            id: id.clone(),
            name: "Ines Weber".into(),
            department: Some(Department::Camera),
            function: " Oberbeleuchter ".into(),
            agency: "Licht & Co".into(),
        },
    );
    assert_eq!(s.undo_step(), Some("EditPerson"));
    let row = s.overview().persons[0].clone();
    assert_eq!(row.name, "Ines Weber");
    assert_eq!(row.department, Some("camera"));
    assert_eq!(row.function, "Oberbeleuchter");
    assert_eq!(row.agency, "Licht & Co");

    team(&mut s, TeamEdit::RemovePerson { id: id.clone() });
    assert_eq!(s.undo_step(), Some("RemovePerson"));
    assert!(s.overview().persons.is_empty());

    s.undo();
    assert_eq!(s.overview().persons.len(), 1);
    s.undo();
    assert_eq!(s.overview().persons[0].name, "Ines");
    s.undo();
    assert!(s.overview().persons.is_empty());
}

#[test]
fn a_person_needs_a_name_and_must_exist() {
    let mut s = open("team-person-refused");

    assert!(matches!(
        s.add_person("  ", None),
        Err(ServiceError::BadValue { .. })
    ));
    assert!(matches!(
        refused(&mut s, TeamEdit::RemovePerson { id: "x".into() }),
        ServiceError::NoPerson(_)
    ));
    assert!(matches!(
        refused(
            &mut s,
            TeamEdit::ProposeCast {
                character: "anna".into(),
                person: "x".into()
            }
        ),
        ServiceError::NoPerson(_)
    ));
}

/// A removed person leaves every casting and every element they answered for.
#[test]
fn removing_a_person_cleans_castings_and_elements() {
    let mut s = open("team-remove-clean");
    let ines = s.add_person("Ines", None).unwrap();
    let jonas = s.add_person("Jonas", None).unwrap();
    propose(&mut s, "anna", &ines);
    propose(&mut s, "anna", &jonas);
    team(
        &mut s,
        TeamEdit::FixCast {
            character: "anna".into(),
            person: Some(ines.clone()),
        },
    );
    propose(&mut s, "bob", &ines);
    let lamp = s.add_element(ElementCategory::Prop, "Lampe").unwrap();
    s.edit_project(ProjectEdit::Breakdown(BreakdownEdit::SetResponsible {
        element: lamp,
        person: Some(ines.clone()),
    }))
    .unwrap();
    team(
        &mut s,
        TeamEdit::SetContact {
            person: ines.clone(),
            contact: Contact {
                phone: "1".into(),
                ..Contact::default()
            },
        },
    );

    team(&mut s, TeamEdit::RemovePerson { id: ines });

    let anna = casting(&s, "anna");
    assert_eq!((anna.fixed, anna.proposed), (None, vec![jonas]));
    assert!(s.project().team.casting("bob").is_none());
    assert_eq!(s.project().elements[0].responsible, None);
    assert!(s.project().team.contacts.is_empty());
    assert_eq!(s.overview().elements[0].responsible, None);

    s.undo();
    assert_eq!(
        casting(&s, "anna").fixed.as_deref(),
        Some(s.project().team.persons[0].id.as_str())
    );
}

#[test]
fn a_responsible_person_is_set_and_undone() {
    let mut s = open("team-responsible");
    let ines = s.add_person("Ines", Some(Department::Props)).unwrap();
    let lamp = s.add_element(ElementCategory::Prop, "Lampe").unwrap();

    let set = |person: Option<String>| {
        ProjectEdit::Breakdown(BreakdownEdit::SetResponsible {
            element: lamp.clone(),
            person,
        })
    };
    s.edit_project(set(Some(ines.clone()))).unwrap();
    assert_eq!(s.undo_step(), Some("SetResponsible"));
    assert_eq!(s.overview().elements[0].responsible, Some(ines));

    assert!(matches!(
        s.edit_project(set(Some("x".into()))),
        Err(ServiceError::NoPerson(_))
    ));

    s.undo();
    assert_eq!(s.overview().elements[0].responsible, None);
}

/// The persons of the department that fits the category come first.
#[test]
fn the_matching_department_is_offered_first() {
    let mut s = open("team-offer");
    s.add_person("Anke", Some(Department::Camera)).unwrap();
    s.add_person("Berta", None).unwrap();
    s.add_person("Carl", Some(Department::Costume)).unwrap();
    s.add_person("Dora", Some(Department::Costume)).unwrap();

    let names = |category| {
        let rows = s.persons_for(category);

        rows.into_iter().map(|p| p.name).collect::<Vec<_>>()
    };

    assert_eq!(
        names(ElementCategory::Costume),
        ["Carl", "Dora", "Anke", "Berta"]
    );
    assert_eq!(
        names(ElementCategory::CameraLight),
        ["Anke", "Berta", "Carl", "Dora"]
    );
}

#[test]
fn proposals_are_fixed_and_dropped() {
    let mut s = open("team-casting");
    let ines = s.add_person("Ines", Some(Department::Cast)).unwrap();
    let jonas = s.add_person("Jonas", Some(Department::Cast)).unwrap();

    propose(&mut s, "anna", &ines);
    propose(&mut s, "anna", &ines);
    assert_eq!(s.undo_step(), Some("ProposeCast"));
    assert_eq!(casting(&s, "anna").proposed, [ines.as_str()]);

    // Fixing a person who is not yet proposed proposes them; the others stay.
    team(
        &mut s,
        TeamEdit::FixCast {
            character: "anna".into(),
            person: Some(jonas.clone()),
        },
    );
    let anna = casting(&s, "anna");
    assert_eq!(anna.fixed.as_deref(), Some(jonas.as_str()));
    assert_eq!(anna.proposed, [ines.clone(), jonas.clone()]);

    // Dropping another proposal leaves the fixed one; dropping the fixed one clears it.
    team(
        &mut s,
        TeamEdit::DropProposal {
            character: "anna".into(),
            person: ines.clone(),
        },
    );
    assert_eq!(casting(&s, "anna").fixed.as_deref(), Some(jonas.as_str()));
    team(
        &mut s,
        TeamEdit::DropProposal {
            character: "anna".into(),
            person: jonas.clone(),
        },
    );
    assert!(s.project().team.casting("anna").is_none());

    // Fixing nobody keeps the proposal.
    propose(&mut s, "anna", &ines);
    team(
        &mut s,
        TeamEdit::FixCast {
            character: "anna".into(),
            person: Some(ines.clone()),
        },
    );
    team(
        &mut s,
        TeamEdit::FixCast {
            character: "anna".into(),
            person: None,
        },
    );
    let anna = casting(&s, "anna");
    assert_eq!((anna.fixed, anna.proposed), (None, vec![ines.clone()]));

    let row = &s.overview().castings[0];
    assert_eq!(row.proposed, [ines]);
}

#[test]
fn a_casting_needs_a_character() {
    let mut s = open("team-casting-character");
    let ines = s.add_person("Ines", None).unwrap();

    let edit = TeamEdit::ProposeCast {
        character: "nobody".into(),
        person: ines,
    };

    assert!(matches!(
        refused(&mut s, edit),
        ServiceError::NoCharacter(_)
    ));
}

/// By scenes, most first; WÄRTER before BOB (the same scenes, earlier first); the extras and
/// CHRIS (no scene) last or none.
#[test]
fn cast_numbers_follow_the_scenes() {
    let s = open("team-numbers");

    assert_eq!(numbers(&s), [Some(1), Some(2), Some(3), None, Some(4)]);
    let anna = &s.overview().castings[0];
    assert_eq!((anna.scenes, anna.stored), (3, false));
    assert_eq!(s.overview().castings[3].scenes, 1);
    assert!(!s.overview().numbers_locked);
}

#[test]
fn a_number_by_hand_stays_and_gives_way() {
    let mut s = open("team-number-hand");

    team(
        &mut s,
        TeamEdit::SetCastNumber {
            character: "bob".into(),
            number: Some(5),
        },
    );
    assert_eq!(s.undo_step(), Some("SetCastNumber"));

    // BOB keeps 5; the others take the lowest left.
    assert_eq!(numbers(&s), [Some(1), Some(2), Some(5), None, Some(3)]);
    assert!(s.overview().castings[2].stored);

    team(
        &mut s,
        TeamEdit::SetCastNumber {
            character: "bob".into(),
            number: None,
        },
    );
    assert_eq!(numbers(&s), [Some(1), Some(2), Some(3), None, Some(4)]);
    assert!(s.project().team.castings.is_empty());
}

#[test]
fn a_number_another_character_has_stored_is_refused() {
    let mut s = open("team-number-taken");

    // 9 is free; 0 is no number.
    team(
        &mut s,
        TeamEdit::SetCastNumber {
            character: "bob".into(),
            number: Some(9),
        },
    );
    let taken = refused(
        &mut s,
        TeamEdit::SetCastNumber {
            character: "anna".into(),
            number: Some(9),
        },
    );
    assert!(matches!(taken, ServiceError::CastNumberTaken(9)));
    for language in [Language::German, Language::English] {
        let text = klappe_services::error_text(language, &taken);
        assert!(text.contains('9'));
        assert!(!text.contains("error."));
    }
    assert!(matches!(
        refused(
            &mut s,
            TeamEdit::SetCastNumber {
                character: "anna".into(),
                number: Some(0)
            }
        ),
        ServiceError::BadValue { .. }
    ));
    assert_eq!(number(&s, "bob"), Some(9));

    // The number it has itself is no clash.
    team(
        &mut s,
        TeamEdit::SetCastNumber {
            character: "bob".into(),
            number: Some(9),
        },
    );
}

/// Unlocked, a number only the book gives another character is free to take by hand: that
/// character gives way.
#[test]
fn a_number_only_the_book_gives_can_be_taken() {
    let mut s = open("team-number-book");
    assert_eq!(number(&s, "anna"), Some(1));

    team(
        &mut s,
        TeamEdit::SetCastNumber {
            character: "bob".into(),
            number: Some(1),
        },
    );

    assert_eq!(numbers(&s), [Some(2), Some(3), Some(1), None, Some(4)]);
}

/// Locked, the numbers stay as they are; a new character takes the next; unlocked, the
/// stored numbers still hold.
#[test]
fn locked_numbers_count_on() {
    let mut s = open("team-number-lock");
    team(&mut s, TeamEdit::LockCastNumbers);
    assert_eq!(s.undo_step(), Some("LockCastNumbers"));

    assert!(s.overview().numbers_locked);
    assert_eq!(numbers(&s), [Some(1), Some(2), Some(3), None, Some(4)]);
    assert!(s
        .overview()
        .castings
        .iter()
        .filter(|c| c.number.is_some())
        .all(|c| c.stored));

    // A character with more scenes than any still comes after the others.
    s.edit_project(ProjectEdit::AddCharacter {
        name: "Dora".into(),
        short: "D".into(),
        color: FigureColor::Blue,
        group: None,
        size_m: None,
    })
    .unwrap();
    assert_eq!(number(&s, "dora"), Some(5));

    team(&mut s, TeamEdit::UnlockCastNumbers);
    assert!(!s.overview().numbers_locked);
    assert_eq!(number(&s, "anna"), Some(1));
    assert_eq!(number(&s, "dora"), Some(5));

    // Undo takes back the unlock, the character and the lock with the numbers it stored.
    s.undo();
    s.undo();
    s.undo();
    assert!(!s.overview().numbers_locked);
    assert!(s.project().team.castings.is_empty());
}

#[test]
fn contacts_are_kept_apart_from_the_persons() {
    let dir = dir_of("team-contact");
    klappe_store::save(&dir, &project(), Some(BOOK)).unwrap();
    let mut s = Session::open(&dir).unwrap();
    let ines = s.add_person("Ines", Some(Department::Lighting)).unwrap();

    team(
        &mut s,
        TeamEdit::SetContact {
            person: ines.clone(),
            contact: Contact {
                phone: " 0151 1234 ".into(),
                mail: "ines@example.org".into(),
                address: "Hauptstr. 1\n10115 Berlin".into(),
            },
        },
    );
    assert_eq!(s.undo_step(), Some("SetContact"));
    let row = s.overview().persons[0].clone();
    assert_eq!(row.phone, "0151 1234");
    assert_eq!(row.address, "Hauptstr. 1\n10115 Berlin");

    s.save(OnOutside::Ask).unwrap();
    let people = std::fs::read_to_string(dir.join("personen.toml")).unwrap();
    let contacts = std::fs::read_to_string(dir.join("kontakte.toml")).unwrap();
    assert!(people.contains("Ines"));
    assert!(!people.contains("0151"));
    assert!(!people.contains("Berlin"));
    assert!(!people.contains("example.org"));
    assert!(contacts.contains("0151 1234"));
    assert!(!contacts.contains("Ines"));

    // An empty contact removes the file again.
    team(
        &mut s,
        TeamEdit::SetContact {
            person: ines,
            contact: Contact::default(),
        },
    );
    s.save(OnOutside::Ask).unwrap();
    assert!(!dir.join("kontakte.toml").exists());
}

/// A casting naming a character or person the project lacks loads, is kept by a save and
/// is not offered.
#[test]
fn an_orphan_casting_is_kept() {
    let dir = dir_of("team-orphan");
    let mut orphaned = project();
    orphaned.team.persons = vec![klappe_model::Person {
        id: "p1".into(),
        name: "Ines".into(),
        department: None,
        function: String::new(),
        agency: String::new(),
    }];
    orphaned.team.castings = vec![
        klappe_model::Casting {
            character: "gone".into(),
            fixed: Some("p1".into()),
            proposed: vec!["p1".into()],
            cast_number: Some(3),
        },
        klappe_model::Casting {
            character: "anna".into(),
            fixed: Some("nobody".into()),
            proposed: vec!["nobody".into(), "p1".into()],
            cast_number: None,
        },
    ];
    klappe_store::save(&dir, &orphaned, Some(BOOK)).unwrap();

    let mut s = Session::open(&dir).unwrap();
    let anna = s.overview().castings[0].clone();
    assert_eq!(anna.fixed, None);
    assert_eq!(anna.proposed, ["p1"]);
    assert_eq!(s.overview().castings.len(), 5);

    // Any edit and a save keep what the offer leaves out.
    s.add_person("Jonas", None).unwrap();
    s.save(OnOutside::Ask).unwrap();
    let again = klappe_store::load(&dir).unwrap().project;
    assert_eq!(again.team.castings, orphaned.team.castings);
}

fn contact_rows(s: &mut Session, name: &str, language: Language) -> Vec<Vec<String>> {
    let path = dir_of(name).with_extension("csv");
    s.export_report(Report::Contacts, ReportFormat::Csv, &path, language)
        .unwrap();
    let text = std::fs::read_to_string(&path).unwrap();

    text.trim_end()
        .split("\r\n")
        .map(|l| vec![l.to_owned()])
        .collect()
}

/// Sorted by department (none last), then by name; the address, which runs over two lines,
/// is quoted.
#[test]
fn the_contact_list_is_a_report() {
    let mut s = open("team-contacts-report");
    let a = s.add_person("Zora", Some(Department::Camera)).unwrap();
    s.add_person("Ben", Some(Department::Camera)).unwrap();
    s.add_person("Yves", Some(Department::Direction)).unwrap();
    s.add_person("Nobody", None).unwrap();
    team(
        &mut s,
        TeamEdit::SetContact {
            person: a,
            contact: Contact {
                phone: "0151".into(),
                mail: "z@example.org".into(),
                address: "Weg 1\n10115 Berlin".into(),
            },
        },
    );

    let lines = contact_rows(&mut s, "team-contacts-de", Language::German);

    assert_eq!(lines[0], ["Name,Department,Funktion,Telefon,Mail,Adresse"]);
    assert_eq!(lines[1], ["Yves,Regie,,,,"]);
    assert_eq!(lines[2], ["Ben,Kamera,,,,"]);
    assert_eq!(
        lines[3],
        ["Zora,Kamera,,0151,z@example.org,\"Weg 1\n10115 Berlin\""]
    );
    assert_eq!(lines[4], ["Nobody,,,,,"]);

    let lines = contact_rows(&mut s, "team-contacts-en", Language::English);
    assert_eq!(lines[0], ["Name,Department,Function,Phone,Mail,Address"]);
    assert_eq!(lines[1], ["Yves,Direction,,,,"]);

    for format in [ReportFormat::Pdf, ReportFormat::Ods, ReportFormat::Xlsx] {
        let path = dir_of("team-contacts-file").with_extension(format.name());
        s.export_report(Report::Contacts, format, &path, Language::German)
            .unwrap();
        assert!(std::fs::metadata(&path).unwrap().len() > 0);
    }
}

/// The contact list does not depend on the book; the scene reports still do.
#[test]
fn the_contact_list_needs_no_scenes() {
    let mut s = open_with("team-contacts-no-scenes", &Project::new("Leer"), "");
    s.add_person("Ines", Some(Department::Sound)).unwrap();

    let lines = contact_rows(&mut s, "team-contacts-empty-book", Language::German);
    assert_eq!(lines[1], ["Ines,Ton,,,,"]);

    let path = dir_of("team-scenes-empty-book").with_extension("csv");
    for report in [
        Report::Scenes,
        Report::Places,
        Report::Cast,
        Report::Extract,
    ] {
        let result = s.export_report(report, ReportFormat::Csv, &path, Language::German);

        assert!(
            matches!(result, Err(ServiceError::NoBookScenes)),
            "{report:?}"
        );
    }
}

/// The head of the cast report shows the number before the name; the scenes report does not.
#[test]
fn the_cast_report_shows_the_number() {
    let mut s = open("team-cast-head");
    team(
        &mut s,
        TeamEdit::SetCastNumber {
            character: "bob".into(),
            number: Some(7),
        },
    );
    let path = dir_of("team-cast-report").with_extension("csv");
    s.export_report(Report::Cast, ReportFormat::Csv, &path, Language::German)
        .unwrap();
    let text = std::fs::read_to_string(&path).unwrap();
    let head = text.lines().next().unwrap();

    // The group has no number; the others are in the order of the project.
    assert_eq!(head, "Szene,1  ANNA,2  WÄRTER,7  BOB,CROWD");

    let path = dir_of("team-scenes-report").with_extension("csv");
    s.export_report(Report::Scenes, ReportFormat::Csv, &path, Language::German)
        .unwrap();
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("ANNA, WÄRTER"));
    assert!(!text.contains("1  ANNA"));
}

#[test]
fn every_department_has_its_text() {
    for department in Department::ALL {
        let key = format!("department.{}", department.name());

        for language in [Language::German, Language::English] {
            assert_ne!(language.text(&key), key);
        }
    }
}

/// The team is part of the folder: outside changes to its files are told apart by kind.
#[test]
fn the_team_files_are_outside_changes() {
    let dir = dir_of("team-outside");
    klappe_store::save(&dir, &project(), Some(BOOK)).unwrap();
    let mut s = Session::open(&dir).unwrap();

    let mut other = project();
    other.team.persons.push(klappe_model::Person {
        id: "p1".into(),
        name: "Ines".into(),
        department: None,
        function: String::new(),
        agency: String::new(),
    });
    other.team.contacts.insert(
        "p1".into(),
        Contact {
            phone: "1".into(),
            ..Contact::default()
        },
    );
    klappe_store::save(&dir, &other, None).unwrap();

    let found = s.check().unwrap();
    let kinds: Vec<_> = found.taken.iter().map(|f| f.kind).collect();

    assert!(kinds.contains(&klappe_services::FileKind::Team));
    assert!(kinds.contains(&klappe_services::FileKind::Contacts));
    assert_eq!(s.overview().persons.len(), 1);
}

/// Ids come from the name: a later CHRIS must not inherit the casting of the removed one.
#[test]
fn removing_a_character_drops_its_casting() {
    let mut s = open("team-remove-character");
    let ines = s.add_person("Ines", Some(Department::Cast)).unwrap();
    propose(&mut s, "chris", &ines);

    s.edit_project(ProjectEdit::RemoveCharacter { id: "chris".into() })
        .unwrap();

    assert!(s.project().team.casting("chris").is_none());

    s.undo();
    assert_eq!(casting(&s, "chris").proposed, [ines.as_str()]);
}

/// A casting whose character left `figuren.toml` from outside keeps its number: no one else
/// gets it, so locking can still be saved.
#[test]
fn the_number_of_an_orphan_casting_is_taken() {
    let mut project = project();
    project.team.castings.push(klappe_model::Casting {
        character: "gone".into(),
        cast_number: Some(1),
        ..Default::default()
    });
    let mut s = open_with("team-orphan-number", &project, BOOK);

    assert_eq!(number(&s, "anna"), Some(2));

    team(&mut s, TeamEdit::LockCastNumbers);
    s.save(OnOutside::Ask).unwrap();
}
