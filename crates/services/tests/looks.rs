//! The looks of the breakdown: a character's named costume and make-up, the look it wears in a
//! scene, and what the reports make of it. Each edit is one undo step.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_model::{BookState, Character, FigureColor, Project, Scene};
use klappe_services::{
    error_text, BreakdownEdit, ElementCategory, Field, Language, OnOutside, ProjectEdit, Report,
    ReportFormat, ServiceError, Session,
};

/// ANNA speaks in 14, BOB is silent there; scene 23 has nobody.
const BOOK: &str = "INT. WACHRAUM – NACHT #14#\n\nANNA\nHallo.\n\n\
EXT. HAFEN – NACHT #23#\n\nWasser.\n\n";

fn character(id: &str) -> Character {
    Character {
        id: id.into(),
        name: id.to_uppercase(),
        short: id[..1].to_uppercase(),
        color: FigureColor::ALL[0],
        group: None,
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

/// Scene 14 (`aaaa`) has ANNA (cue) and BOB (sheet); scene 23 (`bbbb`) nobody.
fn open_in(dir: &Path) -> Session {
    let mut project = Project::new("Test");
    project.characters = vec![character("anna"), character("bob")];
    project.scenes = vec![
        scene("aaaa", "14", "INT. WACHRAUM – NACHT"),
        scene("bbbb", "23", "EXT. HAFEN – NACHT"),
    ];
    klappe_store::save(dir, &project, Some(BOOK)).unwrap();

    let mut s = Session::open(dir).unwrap();
    s.select_scene("aaaa").unwrap();
    edit(
        &mut s,
        BreakdownEdit::AddSceneCharacter {
            scene: "aaaa".into(),
            character: "bob".into(),
        },
    );

    s
}

fn open(name: &str) -> Session {
    open_in(&dir_of(name))
}

fn run(s: &mut Session, edit: BreakdownEdit) -> Result<(), ServiceError> {
    s.edit_project(ProjectEdit::Breakdown(edit))
}

fn edit(s: &mut Session, edit: BreakdownEdit) {
    run(s, edit).unwrap();
}

fn put_on(
    s: &mut Session,
    scene: &str,
    character: &str,
    look: Option<&str>,
) -> Result<(), ServiceError> {
    run(
        s,
        BreakdownEdit::SetSceneLook {
            scene: scene.into(),
            character: character.into(),
            look: look.map(str::to_owned),
        },
    )
}

fn in_look(s: &mut Session, look: &str, element: &str) -> Result<(), ServiceError> {
    run(
        s,
        BreakdownEdit::AddLookElement {
            look: look.into(),
            element: element.into(),
        },
    )
}

fn worn(s: &Session, scene: &str, character: &str) -> Option<String> {
    s.project()
        .scene(scene)
        .unwrap()
        .sheet
        .looks
        .get(character)
        .cloned()
}

#[test]
fn a_look_is_added_renamed_removed_and_undone() {
    let mut s = open("looks-add");

    let id = s.add_look("anna", "  Ölzeug, trocken ").unwrap();
    assert_eq!(s.project().looks[0].id, id);
    assert_eq!(s.project().looks[0].name, "Ölzeug, trocken");
    assert_eq!(s.project().looks[0].character, "anna");
    assert_eq!(s.undo_step(), Some("AddLook"));

    edit(
        &mut s,
        BreakdownEdit::EditLook {
            id: id.clone(),
            name: "Ölzeug, nass".into(),
        },
    );
    assert_eq!(s.project().looks[0].name, "Ölzeug, nass");
    assert_eq!(s.undo_step(), Some("EditLook"));
    s.undo();
    assert_eq!(s.project().looks[0].name, "Ölzeug, trocken");

    edit(&mut s, BreakdownEdit::RemoveLook { id });
    assert!(s.project().looks.is_empty());
    assert_eq!(s.undo_step(), Some("RemoveLook"));
    s.undo();
    assert_eq!(s.project().looks.len(), 1);

    s.undo();
    assert!(s.project().looks.is_empty());
}

#[test]
fn a_look_needs_a_name_a_character_and_is_unique_per_character() {
    let mut s = open("looks-names");
    let id = s.add_look("anna", "Mantel").unwrap();

    assert!(matches!(
        s.add_look("anna", " mantel "),
        Err(ServiceError::BadValue { .. })
    ));
    assert!(matches!(
        s.add_look("anna", "  "),
        Err(ServiceError::BadValue { .. })
    ));
    assert!(matches!(
        s.add_look("nobody", "Mantel"),
        Err(ServiceError::NoCharacter(_))
    ));
    // Another character may have a look of the same name.
    s.add_look("bob", "Mantel").unwrap();

    let other = s.add_look("anna", "Kleid").unwrap();
    let clash = run(
        &mut s,
        BreakdownEdit::EditLook {
            id: other,
            name: "MANTEL".into(),
        },
    );
    assert!(matches!(clash, Err(ServiceError::BadValue { .. })));
    // Its own name again is no clash.
    edit(
        &mut s,
        BreakdownEdit::EditLook {
            id,
            name: "mantel".into(),
        },
    );
    let gone = run(
        &mut s,
        BreakdownEdit::EditLook {
            id: "weg".into(),
            name: "X".into(),
        },
    );
    assert!(matches!(gone, Err(ServiceError::NoLook(_))));
}

#[test]
fn only_costume_and_makeup_belong_to_a_look() {
    let mut s = open("looks-category");
    let look = s.add_look("anna", "Ölzeug, nass").unwrap();
    let coat = s.add_element(ElementCategory::Costume, "Ölmantel").unwrap();
    let blood = s.add_element(ElementCategory::Makeup, "Schramme").unwrap();
    let lamp = s.add_element(ElementCategory::Prop, "Öllampe").unwrap();

    in_look(&mut s, &look, &coat).unwrap();
    in_look(&mut s, &look, &blood).unwrap();
    in_look(&mut s, &look, &coat).unwrap();
    assert_eq!(s.project().looks[0].elements, [coat.clone(), blood.clone()]);
    assert_eq!(s.undo_step(), Some("AddLookElement"));

    let refused = in_look(&mut s, &look, &lamp).unwrap_err();
    assert!(matches!(refused, ServiceError::NotLookElement(_)));
    assert!(error_text(Language::German, &refused).contains("Öllampe"));
    assert!(error_text(Language::German, &refused).contains("Kostüm und Maskenbild"));
    assert!(error_text(Language::English, &refused).contains("costume and make-up"));
    assert!(matches!(
        in_look(&mut s, &look, "weg"),
        Err(ServiceError::NoElement(_))
    ));
    assert!(matches!(
        in_look(&mut s, "weg", &coat),
        Err(ServiceError::NoLook(_))
    ));
    assert_eq!(s.project().looks[0].elements.len(), 2);

    edit(
        &mut s,
        BreakdownEdit::RemoveLookElement {
            look: look.clone(),
            element: coat.clone(),
        },
    );
    assert_eq!(s.project().looks[0].elements, [blood]);
    assert_eq!(s.undo_step(), Some("RemoveLookElement"));
    s.undo();
    assert_eq!(s.project().looks[0].elements.len(), 2);
}

#[test]
fn a_character_wears_one_of_its_looks_in_a_scene() {
    let mut s = open("looks-scene");
    let wet = s.add_look("anna", "Ölzeug, nass").unwrap();
    let dry = s.add_look("anna", "Ölzeug, trocken").unwrap();
    let bobs = s.add_look("bob", "Anzug").unwrap();

    // ANNA has a cue, BOB is on the sheet.
    put_on(&mut s, "aaaa", "anna", Some(&wet)).unwrap();
    put_on(&mut s, "aaaa", "bob", Some(&bobs)).unwrap();
    assert_eq!(s.undo_step(), Some("SetSceneLook"));
    assert_eq!(worn(&s, "aaaa", "anna"), Some(wet.clone()));

    let sheet = s.overview().sheet.unwrap();
    assert_eq!(sheet.speaking[0].look, Some(wet.clone()));
    assert_eq!(sheet.present[0].look, Some(bobs.clone()));
    let rows = s.overview().looks;
    assert_eq!(rows.len(), 3);
    assert_eq!(
        (rows[0].id.as_str(), rows[0].character.as_str()),
        (wet.as_str(), "anna")
    );

    // Another one replaces it; it is one step.
    put_on(&mut s, "aaaa", "anna", Some(&dry)).unwrap();
    assert_eq!(worn(&s, "aaaa", "anna"), Some(dry));
    s.undo();
    assert_eq!(worn(&s, "aaaa", "anna"), Some(wet.clone()));

    // Not in scene 23; not BOB's look for ANNA (it is none of hers); no such look.
    let absent = put_on(&mut s, "bbbb", "anna", Some(&wet)).unwrap_err();
    assert!(matches!(
        absent,
        ServiceError::BadValue {
            field: Field::Character,
            ..
        }
    ));
    let foreign = put_on(&mut s, "aaaa", "anna", Some(&bobs)).unwrap_err();
    assert!(matches!(foreign, ServiceError::NoLook(_)));
    assert!(matches!(
        put_on(&mut s, "aaaa", "anna", Some("weg")),
        Err(ServiceError::NoLook(_))
    ));
    assert!(matches!(
        put_on(&mut s, "zzzz", "anna", Some(&wet)),
        Err(ServiceError::NoScene(_))
    ));
    assert_eq!(worn(&s, "bbbb", "anna"), None);

    put_on(&mut s, "aaaa", "anna", None).unwrap();
    assert_eq!(worn(&s, "aaaa", "anna"), None);
    assert_eq!(s.overview().sheet.unwrap().speaking[0].look, None);
}

#[test]
fn removing_things_cleans_up_the_looks() {
    let mut s = open("looks-clean");
    let wet = s.add_look("anna", "Ölzeug, nass").unwrap();
    let suit = s.add_look("bob", "Anzug").unwrap();
    let coat = s.add_element(ElementCategory::Costume, "Ölmantel").unwrap();
    in_look(&mut s, &wet, &coat).unwrap();
    put_on(&mut s, "aaaa", "anna", Some(&wet)).unwrap();
    put_on(&mut s, "aaaa", "bob", Some(&suit)).unwrap();

    // The element leaves the look.
    edit(&mut s, BreakdownEdit::RemoveElement { id: coat });
    assert!(s.project().looks[0].elements.is_empty());
    s.undo();
    assert_eq!(s.project().looks[0].elements.len(), 1);

    // BOB leaves the scene: its look goes from the sheet, the look stays.
    edit(
        &mut s,
        BreakdownEdit::RemoveSceneCharacter {
            scene: "aaaa".into(),
            character: "bob".into(),
        },
    );
    assert_eq!(worn(&s, "aaaa", "bob"), None);
    assert_eq!(s.project().looks.len(), 2);

    // The look goes from every sheet.
    edit(&mut s, BreakdownEdit::RemoveLook { id: wet.clone() });
    assert_eq!(worn(&s, "aaaa", "anna"), None);
    s.undo();
    assert_eq!(worn(&s, "aaaa", "anna"), Some(wet));

    // A deleted character leaves no look in a sheet.
    edit(
        &mut s,
        BreakdownEdit::AddSceneCharacter {
            scene: "aaaa".into(),
            character: "bob".into(),
        },
    );
    put_on(&mut s, "aaaa", "bob", Some(&suit)).unwrap();
    s.edit_project(ProjectEdit::RemoveCharacter { id: "bob".into() })
        .unwrap();
    assert_eq!(worn(&s, "aaaa", "bob"), None);
}

#[test]
fn a_removed_character_takes_its_looks_along() {
    let mut s = open("looks-removed");
    let suit = s.add_look("bob", "Anzug").unwrap();
    let wet = s.add_look("anna", "Ölzeug, nass").unwrap();
    put_on(&mut s, "aaaa", "bob", Some(&suit)).unwrap();

    s.edit_project(ProjectEdit::RemoveCharacter { id: "bob".into() })
        .unwrap();
    assert_eq!(s.undo_step(), Some("RemoveCharacter"));
    assert_eq!(s.project().looks.len(), 1);
    assert_eq!(s.project().looks[0].id, wet);

    // Ids come from the name: the next BOB starts without the old looks.
    s.edit_project(ProjectEdit::AddCharacter {
        name: "BOB".into(),
        short: "B".into(),
        color: FigureColor::ALL[0],
        group: None,
        size_m: None,
    })
    .unwrap();
    assert!(s.overview().looks.iter().all(|l| l.id != suit));
    s.add_look("bob", "Anzug").unwrap();
    s.undo();
    s.undo();

    // One undo step brings the look and its place in the sheet back.
    s.undo();
    assert_eq!(s.project().looks.len(), 2);
    assert_eq!(worn(&s, "aaaa", "bob"), Some(suit));
}

#[test]
fn an_element_leaves_the_looks_when_it_is_no_costume_or_makeup() {
    let mut s = open("looks-recategory");
    let look = s.add_look("anna", "Ölzeug, nass").unwrap();
    let coat = s.add_element(ElementCategory::Costume, "Ölmantel").unwrap();
    in_look(&mut s, &look, &coat).unwrap();
    let to = |category| BreakdownEdit::EditElement {
        id: coat.clone(),
        category,
        name: "Ölmantel".into(),
        description: String::new(),
        status: Default::default(),
        responsible: None,
    };

    // Costume to make-up: still fits.
    edit(&mut s, to(ElementCategory::Makeup));
    assert_eq!(s.project().looks[0].elements, std::slice::from_ref(&coat));

    edit(&mut s, to(ElementCategory::Prop));
    assert!(s.project().looks[0].elements.is_empty());
    assert_eq!(s.undo_step(), Some("EditElement"));
    s.undo();
    assert_eq!(s.project().looks[0].elements, [coat]);
}

#[test]
fn the_looks_survive_a_save_and_an_orphan_is_kept_but_not_offered() {
    let dir = dir_of("looks-save");
    let mut s = open_in(&dir);
    let wet = s.add_look("anna", "Ölzeug, nass").unwrap();
    let suit = s.add_look("bob", "Anzug").unwrap();
    let coat = s.add_element(ElementCategory::Costume, "Ölmantel").unwrap();
    in_look(&mut s, &wet, &coat).unwrap();
    put_on(&mut s, "aaaa", "anna", Some(&wet)).unwrap();
    assert!(!dir.join("breakdown/looks.toml").exists());
    s.save(OnOutside::Ask).unwrap();

    assert!(dir.join("breakdown/looks.toml").is_file());
    let again = Session::open(&dir).unwrap();
    assert_eq!(again.project().looks, s.project().looks);
    assert_eq!(worn(&again, "aaaa", "anna"), Some(wet.clone()));

    // BOB is gone from the project file (not removed in Klappe); his look loads, is saved
    // again, and is not offered.
    let mut project = again.project().clone();
    project.characters.retain(|c| c.id != "bob");
    klappe_store::save(&dir, &project, Some(BOOK)).unwrap();
    let orphaned = Session::open(&dir).unwrap();
    assert!(orphaned.project().looks.iter().any(|l| l.id == suit));
    let offered: Vec<String> = orphaned
        .overview()
        .looks
        .into_iter()
        .map(|l| l.id)
        .collect();
    assert_eq!(offered, [wet]);

    // Nothing left: the file goes.
    let mut empty = orphaned;
    for id in empty
        .project()
        .looks
        .iter()
        .map(|l| l.id.clone())
        .collect::<Vec<_>>()
    {
        edit(&mut empty, BreakdownEdit::RemoveLook { id });
    }
    empty.save(OnOutside::Ask).unwrap();
    assert!(!dir.join("breakdown/looks.toml").exists());
}

/// Anna wears two looks in scene 14: costume `Ölmantel` (also on the sheet) and make-up
/// `Schramme`; Bob wears none.
fn dressed(name: &str) -> Session {
    let mut s = open(name);
    let wet = s.add_look("anna", "Ölzeug, nass").unwrap();
    let coat = s.add_element(ElementCategory::Costume, "Ölmantel").unwrap();
    let scar = s.add_element(ElementCategory::Makeup, "Schramme").unwrap();
    in_look(&mut s, &wet, &coat).unwrap();
    in_look(&mut s, &wet, &scar).unwrap();
    edit(
        &mut s,
        BreakdownEdit::AddSceneElement {
            scene: "aaaa".into(),
            element: coat,
        },
    );
    put_on(&mut s, "aaaa", "anna", Some(&wet)).unwrap();

    s
}

fn rows(s: &mut Session, report: Report, name: &str) -> Vec<Vec<String>> {
    let path = dir_of(name).with_extension("csv");
    s.export_report(report, ReportFormat::Csv, &path, Language::German)
        .unwrap();
    let text = std::fs::read_to_string(&path).unwrap();
    let _ = std::fs::remove_file(&path);

    text.lines()
        .map(|l| l.split(',').map(str::to_owned).collect())
        .collect()
}

#[test]
fn the_cross_table_shows_the_look_instead_of_an_x() {
    let mut s = dressed("looks-cast");

    let rows = rows(&mut s, Report::Cast, "looks-cast-csv");

    // The cast number stands before the name (Besetzung/Team).
    assert_eq!(rows[0], ["Szene", "1  ANNA", "2  BOB"]);
    assert_eq!(rows[1], ["14", "\"Ölzeug", " nass\"", "X"]);
    assert_eq!(rows[2], ["23", "", ""]);
}

#[test]
fn the_extract_names_the_look_and_lists_its_elements_once() {
    let mut s = dressed("looks-extract");

    let table = rows(&mut s, Report::Extract, "looks-extract-csv");
    assert_eq!(
        table[1..],
        [
            ["14", "Kostüm", "Ölmantel", "offen"],
            ["14", "Maskenbild", "Schramme", "offen"]
        ]
    );

    let path = dir_of("looks-extract-file").with_extension("pdf");
    s.export_report(Report::Extract, ReportFormat::Pdf, &path, Language::German)
        .unwrap();
    let pdf = std::fs::read(&path).unwrap();
    let _ = std::fs::remove_file(&path);
    let text = String::from_utf8_lossy(&pdf);
    // The en dash is the octal escape 226 in the PDF's text.
    assert!(text.contains("(ANNA \\226 \\326lzeug, nass)"), "{text}");
    assert!(text.contains("(Maskenbild)"));
    assert!(text.contains("(Schramme"));
    // Ö is the octal escape 326: the coat is on the sheet and in the look, listed once.
    assert_eq!(text.matches("\\326lmantel").count(), 1, "{text}");
}

#[test]
fn a_look_of_a_character_not_in_the_scene_is_not_shown() {
    let mut s = dressed("looks-left");
    // Scene 14's sheet stays; ANNA no longer has a cue once the book loses it.
    s.set_script_text(&BOOK.replace("ANNA\nHallo.", "Hallo."));

    assert!(s.overview().sheet.unwrap().speaking.is_empty());
    let rows = rows(&mut s, Report::Cast, "looks-left-csv");
    assert_eq!(rows[1], ["14", "X"]);
}

#[test]
fn a_new_element_goes_into_a_look_in_one_step() {
    let mut s = open("looks-to-look");
    let wet = s.add_look("anna", "Ölzeug, nass").unwrap();
    let add = |s: &mut Session, name: &str| {
        run(
            s,
            BreakdownEdit::AddElementToLook {
                look: wet.clone(),
                category: ElementCategory::Costume,
                name: name.into(),
            },
        )
    };

    add(&mut s, "Ölmantel").unwrap();
    assert_eq!(s.project().elements.len(), 1);
    assert_eq!(s.project().looks[0].elements.len(), 1);
    assert_eq!(s.undo_step(), Some("AddElementToLook"));

    // One undo takes the element and its place in the look.
    s.undo();
    assert!(s.project().elements.is_empty());
    assert!(s.project().looks[0].elements.is_empty());

    // The same name, the case aside, is the same element, once in the look.
    add(&mut s, "Ölmantel").unwrap();
    add(&mut s, " ölmantel ").unwrap();
    assert_eq!(s.project().elements.len(), 1);
    assert_eq!(s.project().looks[0].elements.len(), 1);
}

#[test]
fn a_new_prop_does_not_go_into_a_look() {
    let mut s = open("looks-prop");
    let wet = s.add_look("anna", "Ölzeug, nass").unwrap();

    let refused = run(
        &mut s,
        BreakdownEdit::AddElementToLook {
            look: wet,
            category: ElementCategory::Prop,
            name: "Lampe".into(),
        },
    )
    .unwrap_err();

    assert!(matches!(refused, ServiceError::NotLookElement(_)));
    assert!(s.project().elements.is_empty());
}

#[test]
fn a_look_lists_no_scene_its_character_left() {
    let mut s = dressed("looks-left-scenes");
    assert_eq!(s.overview().looks[0].scenes.len(), 1);

    s.set_script_text(&BOOK.replace("ANNA\nHallo.", "Hallo."));

    assert!(s.overview().looks[0].scenes.is_empty());
}

#[test]
fn a_sheet_shows_the_elements_of_the_looks_worn_and_a_look_its_scenes() {
    let mut s = open("looks-sheet");
    let wet = s.add_look("anna", "Ölzeug, nass").unwrap();
    let coat = s.add_element(ElementCategory::Costume, "Ölmantel").unwrap();
    let blood = s.add_element(ElementCategory::Makeup, "Schramme").unwrap();
    in_look(&mut s, &wet, &coat).unwrap();
    in_look(&mut s, &wet, &blood).unwrap();

    // Not worn yet: nothing in the sheet, no scene for the look.
    let seen = s.overview();
    assert!(seen.sheet.unwrap().categories.is_empty());
    assert!(seen.looks[0].scenes.is_empty());

    put_on(&mut s, "aaaa", "anna", Some(&wet)).unwrap();
    let seen = s.overview();
    let categories = seen.sheet.unwrap().categories;
    let names: Vec<_> = categories.iter().map(|c| c.category).collect();
    assert_eq!(names, ["costume", "makeup"]);
    assert!(categories[0].elements.is_empty());
    assert_eq!(categories[0].from_looks[0].name, "Ölmantel");
    assert_eq!(categories[0].from_looks[0].figure, "ANNA");
    assert_eq!(seen.looks[0].scenes.len(), 1);
    assert_eq!(seen.looks[0].scenes[0].id, "aaaa");

    // The same element on the sheet itself is listed there as well.
    edit(
        &mut s,
        BreakdownEdit::AddElementToScene {
            scene: "aaaa".into(),
            category: ElementCategory::Costume,
            name: "Ölmantel".into(),
        },
    );
    let categories = s.overview().sheet.unwrap().categories;
    assert_eq!(categories[0].elements.len(), 1);
    assert_eq!(categories[0].from_looks.len(), 1);

    put_on(&mut s, "aaaa", "anna", None).unwrap();
    assert!(s.overview().looks[0].scenes.is_empty());
}
