//! The statistics (DB-53) and the dialogue of one character (DB-52): the rows as `services/`
//! works them out, read back from the CSV.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_model::{BookState, Character, FigureColor, Place, Project, Scene};
use klappe_script::length_text;
use klappe_services::{error_text, Language, Report, ReportFormat, ServiceError, Session};

/// ANNA is also called ANNI; ZED is no character. In 2 the second voice of a dual dialogue is
/// ANNA's. Scene 3 is struck from the book: in the boneyard.
const BOOK: &str =
    "INT. SAAL – TAG #1#\n\nANNA\n(leise)\nHallo.\nNoch eine Zeile.\n\nANNI\nNochmal.\n\n\
BOB\nJa.\n\nEXT. HOF – NACHT #2#\n\nBOB\nEins.\n\nANNA ^\nZwei.\n\nZED\nWer?\n\n\
/*\nEXT. HOF – TAG #3#\n\nBOB\nWeg.\n*/\n";

fn character(id: &str, aliases: &[&str]) -> Character {
    Character {
        id: id.into(),
        name: id.to_uppercase(),
        short: id[..1].to_uppercase(),
        color: FigureColor::ALL[0],
        group: None,
        size_m: None,
        aliases: aliases.iter().map(|a| a.to_string()).collect(),
    }
}

fn scene(id: &str, number: &str, heading: &str, state: BookState) -> Scene {
    Scene {
        id: id.into(),
        number: number.into(),
        heading: heading.into(),
        state,
        ..Scene::default()
    }
}

fn dir_of(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);

    dir
}

fn open(name: &str) -> Session {
    let dir = dir_of(name);
    let mut project = Project::new("Test");
    project.characters = vec![
        character("bob", &[]),
        character("anna", &["Anni"]),
        character("carl", &[]),
    ];
    project.scenes = vec![
        scene("aaaa", "1", "INT. SAAL – TAG", BookState::InBook),
        scene("bbbb", "2", "EXT. HOF – NACHT", BookState::InBook),
        scene("cccc", "3", "EXT. HOF – TAG", BookState::NotInBook),
    ];
    klappe_store::save(&dir, &project, Some(BOOK)).unwrap();
    let saal = Place {
        id: "saal".into(),
        name: "SAAL".into(),
        variants: Vec::new(),
        location: None,
    };
    klappe_store::save_places(&dir, &[saal]).unwrap();

    Session::open(&dir).unwrap()
}

/// The report's CSV, line by line.
fn lines(s: &mut Session, report: Report, name: &str) -> Vec<String> {
    let path = dir_of(name).with_extension("csv");
    s.export_report(report, ReportFormat::Csv, &path, Language::German)
        .unwrap();
    let text = std::fs::read_to_string(&path).unwrap();
    let _ = std::fs::remove_file(&path);

    text.lines().map(str::to_owned).collect()
}

fn dialogue(s: &mut Session, figure: &str, name: &str) -> Vec<String> {
    let path = dir_of(name).with_extension("csv");
    s.export_dialogue(figure, ReportFormat::Csv, &path, Language::German)
        .unwrap();
    let text = std::fs::read_to_string(&path).unwrap();
    let _ = std::fs::remove_file(&path);

    text.lines().map(str::to_owned).collect()
}

#[test]
fn replies_count_by_name_and_alias_and_the_dual_voice() {
    let mut s = open("stat-figures");

    let csv = lines(&mut s, Report::Statistics, "stat-figures-file");

    let at = csv.iter().position(|l| l == "Figuren").unwrap();
    assert_eq!(
        csv[at + 1],
        "Figur,Szenen mit Dialog,Repliken,Wörter,Anteil am Dialog (%)"
    );
    // ANNA 6 of 8 words, BOB 2: the struck scene's BOB and ZED are not counted.
    assert_eq!(csv[at + 2], "ANNA,2,3,6,\"75,0\"");
    assert_eq!(csv[at + 3], "BOB,2,2,2,\"25,0\"");
    assert_eq!(csv[at + 4], "");
}

#[test]
fn places_and_times_count_scenes_and_length() {
    let mut s = open("stat-places");
    let csv = lines(&mut s, Report::Statistics, "stat-places-file");
    let eighths: Vec<usize> = s
        .script_pages()
        .unwrap()
        .scenes
        .iter()
        .map(|span| span.eighths)
        .collect();

    let places = csv.iter().position(|l| l == "Spielorte").unwrap();
    assert_eq!(
        csv[places + 2..places + 4],
        [
            format!("SAAL,1,{}", length_text(eighths[0])),
            format!("ohne Spielort,1,{}", length_text(eighths[1])),
        ]
    );
    let times = csv.iter().position(|l| l == "Tageszeiten").unwrap();
    assert_eq!(
        csv[times + 2..times + 4],
        [
            format!("TAG,1,{}", length_text(eighths[0])),
            format!("NACHT,1,{}", length_text(eighths[1])),
        ]
    );
}

#[test]
fn the_total_is_the_layouts_length_and_minutes() {
    let mut s = open("stat-total");
    let csv = lines(&mut s, Report::Statistics, "stat-total-file");
    let pages = s.script_pages().unwrap();

    assert_eq!(csv[0], "Gesamtlänge");
    assert_eq!(csv[1], "Seiten,Minuten (geschätzt)");
    assert_eq!(csv[2], format!("{},1 Min.", pages.length));
}

#[test]
fn a_character_has_its_replies_in_book_order() {
    let mut s = open("dialog-anna");

    let csv = dialogue(&mut s, "anna", "dialog-anna-file");

    assert_eq!(
        csv,
        [
            "Szene,Klammer,Dialog",
            "1  INT. SAAL – TAG,(leise),Hallo. Noch eine Zeile.",
            "1  INT. SAAL – TAG,,Nochmal.",
            "2  EXT. HOF – NACHT,,Zwei.",
        ]
    );
}

#[test]
fn dual_dialogue_is_in_both_characters_dialogue() {
    let mut s = open("dialog-bob");

    let csv = dialogue(&mut s, "bob", "dialog-bob-file");

    assert_eq!(
        csv[1..],
        ["1  INT. SAAL – TAG,,Ja.", "2  EXT. HOF – NACHT,,Eins."]
    );
}

#[test]
fn the_choice_lists_who_speaks_most_first() {
    let s = open("dialog-choice");

    let figures = s.dialogue_figures();

    let seen: Vec<(&str, usize)> = figures.iter().map(|f| (f.id.as_str(), f.replies)).collect();
    // CARL says nothing in the book.
    assert_eq!(seen, [("anna", 3), ("bob", 2)]);
}

#[test]
fn the_dialogue_needs_a_character_of_the_project() {
    let mut s = open("dialog-none");
    let path = dir_of("dialog-none-file").with_extension("csv");

    let unknown = s.export_dialogue("nobody", ReportFormat::Csv, &path, Language::German);
    let unchosen = s.export_report(Report::Dialogue, ReportFormat::Csv, &path, Language::German);

    assert!(matches!(unknown, Err(ServiceError::NoCharacter(_))));
    assert!(matches!(unchosen, Err(ServiceError::NoCharacter(_))));
}

#[test]
fn a_book_without_scenes_has_neither() {
    let dir = dir_of("stat-empty");
    let mut project = Project::new("Test");
    project.characters = vec![character("anna", &[])];
    klappe_store::save(&dir, &project, Some("")).unwrap();
    let mut s = Session::open(&dir).unwrap();
    let path = dir_of("stat-empty-file").with_extension("csv");

    let statistics = s.export_report(
        Report::Statistics,
        ReportFormat::Csv,
        &path,
        Language::German,
    );
    let dialogue = s.export_dialogue("anna", ReportFormat::Csv, &path, Language::German);

    for result in [statistics, dialogue] {
        let error = result.unwrap_err();
        assert_eq!(error_text(Language::German, &error), "Keine Szenen im Buch");
        assert_eq!(
            error_text(Language::English, &error),
            "No scenes in the book"
        );
    }
    assert!(s.dialogue_figures().is_empty());
}

#[test]
fn every_format_holds_the_tables() {
    let mut s = open("stat-formats");

    for format in ReportFormat::ALL {
        let path = dir_of("stat-formats-file").with_extension(format.name());
        s.export_report(Report::Statistics, format, &path, Language::English)
            .unwrap();
        let bytes = std::fs::read(&path).unwrap();
        let _ = std::fs::remove_file(&path);

        assert!(bytes.len() > 100, "{}", format.name());
        // ODS and XLSX are stored zips and CSV is text; a PDF compresses nothing here either.
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.contains("Total length"), "{}", format.name());
    }
}

/// One scene of the character ANNA with `body` as her cue block, then BOB's two replies.
fn open_one(name: &str, body: &str) -> Session {
    let dir = dir_of(name);
    let mut project = Project::new("Test");
    project.characters = vec![character("anna", &[]), character("bob", &[])];
    project.scenes = vec![scene("aaaa", "1", "INT. SAAL – TAG", BookState::InBook)];
    let book = format!("INT. SAAL – TAG #1#\n\n{body}\n\nBOB\nJa.\n\nBOB\nJa.\n\n");
    klappe_store::save(&dir, &project, Some(&book)).unwrap();

    Session::open(&dir).unwrap()
}

#[test]
fn one_long_reply_outweighs_two_short_ones() {
    let long = "ANNA\nEs war einmal ein sehr langer Satz, der auf einer einzigen Zeile steht.";
    let mut s = open_one("stat-words", long);

    let csv = lines(&mut s, Report::Statistics, "stat-words-file");

    let at = csv.iter().position(|l| l == "Figuren").unwrap();
    assert!(csv[at + 2].starts_with("ANNA,1,1,13,"), "{}", csv[at + 2]);
    assert!(csv[at + 3].starts_with("BOB,1,2,2,"), "{}", csv[at + 3]);
}

#[test]
fn marks_and_notes_are_no_words() {
    let mut s = open_one("stat-plain", "ANNA\n*Laut* [[Regie]] hallo.");

    let csv = lines(&mut s, Report::Statistics, "stat-plain-file");
    let anna = dialogue(&mut s, "anna", "stat-plain-dialog");

    // Two words each: BOB's two replies put him first.
    assert!(csv.iter().any(|l| l.starts_with("ANNA,1,1,2,")), "{csv:?}");
    assert_eq!(anna[1], "1  INT. SAAL – TAG,,Laut hallo.");
}

#[test]
fn a_bracket_in_the_middle_keeps_its_place() {
    let mut s = open_one("stat-mid", "ANNA\nEins.\n(leise)\nZwei.");

    let csv = dialogue(&mut s, "anna", "stat-mid-file");

    assert_eq!(
        csv[1..],
        [
            "1  INT. SAAL – TAG,,Eins.",
            "1  INT. SAAL – TAG,(leise),Zwei."
        ]
    );
}

#[test]
fn a_character_is_found_by_id_name_or_alias() {
    let s = open("dialog-find");

    for given in ["anna", "ANNA", "anni", "Anni"] {
        assert_eq!(s.figure_named(given).as_deref(), Some("anna"), "{given}");
    }
    assert_eq!(s.figure_named("nobody"), None);
}
