//! The reports of DB-61 (scenes, scripted locations, characters by scene, extract): the rows
//! as `services/` works them out, and the files they are written as.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_model::{BookState, Character, FigureColor, Location, Place, Project, Scene, SetPlan};
use klappe_services::{
    error_text, BreakdownEdit, ElementCategory, Language, ProjectEdit, Report, ReportFormat,
    Session,
};

/// ANNA speaks in 14; BOB is silent there; scene 23 has nobody. CARL is in no scene.
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

/// Two scenes; WACHRAUM is shot in the lighthouse, HAFEN nowhere yet. Scene 14 needs a
/// costume and two props, scene 23 a prop (none of the categories in between).
fn open(name: &str) -> Session {
    let dir = dir_of(name);
    let mut project = Project::new("Test");
    project.locations = vec![Location {
        id: "leuchtturm".into(),
        name: "Leuchtturm".into(),
        site: None,
        set: SetPlan::default(),
    }];
    project.characters = vec![character("anna"), character("bob"), character("carl")];
    project.scenes = vec![
        scene("aaaa", "14", "INT. WACHRAUM – NACHT"),
        scene("bbbb", "23", "EXT. HAFEN – NACHT"),
    ];
    klappe_store::save(&dir, &project, Some(BOOK)).unwrap();
    let places = [
        Place {
            id: "wach".into(),
            name: "WACHRAUM".into(),
            variants: Vec::new(),
            location: Some("leuchtturm".into()),
        },
        Place {
            id: "hafen".into(),
            name: "HAFEN".into(),
            variants: Vec::new(),
            location: None,
        },
    ];
    klappe_store::save_places(&dir, &places).unwrap();

    let mut s = Session::open(&dir).unwrap();
    let mantel = s.add_element(ElementCategory::Costume, "Mantel").unwrap();
    let lampe = s.add_element(ElementCategory::Prop, "Öllampe").unwrap();
    let brief = s.add_element(ElementCategory::Prop, "Brief").unwrap();
    for (scene, element) in [
        ("aaaa", &lampe),
        ("aaaa", &mantel),
        ("aaaa", &brief),
        ("bbbb", &brief),
    ] {
        s.edit_project(ProjectEdit::Breakdown(BreakdownEdit::AddSceneElement {
            scene: scene.into(),
            element: element.clone(),
        }))
        .unwrap();
    }
    s.edit_project(ProjectEdit::Breakdown(BreakdownEdit::AddSceneCharacter {
        scene: "aaaa".into(),
        character: "bob".into(),
    }))
    .unwrap();
    s.edit_project(ProjectEdit::Breakdown(BreakdownEdit::SetSheet {
        scene: "aaaa".into(),
        story_day: "Tag 3".into(),
        prestop: "1:35".into(),
        planned: "240".into(),
        continuity: String::new(),
        note: String::new(),
    }))
    .unwrap();

    s
}

/// Scene 14 is in the book (ANNA speaks); scene 99 is struck from it but still has a sheet with
/// CARL, an element and the place HAFEN. With `keep` false, 14 is struck too.
fn open_struck(name: &str, keep: bool) -> Session {
    let dir = dir_of(name);
    let mut project = Project::new("Test");
    project.characters = vec![character("anna"), character("carl")];
    let mut kept = scene("aaaa", "14", "INT. WACHRAUM – NACHT");
    let mut struck = scene("cccc", "99", "EXT. HAFEN – TAG");
    struck.state = BookState::NotInBook;
    if !keep {
        kept.state = BookState::NotInBook;
    }
    project.scenes = vec![kept, struck];
    let book = match keep {
        true => "INT. WACHRAUM – NACHT #14#\n\nANNA\nHallo.\n\n",
        false => "",
    };
    klappe_store::save(&dir, &project, Some(book)).unwrap();
    let place = |id: &str, name: &str| Place {
        id: id.into(),
        name: name.into(),
        variants: Vec::new(),
        location: None,
    };
    klappe_store::save_places(&dir, &[place("wach", "WACHRAUM"), place("hafen", "HAFEN")]).unwrap();

    let mut s = Session::open(&dir).unwrap();
    let brief = s.add_element(ElementCategory::Prop, "Brief").unwrap();
    s.edit_project(ProjectEdit::Breakdown(BreakdownEdit::AddSceneElement {
        scene: "cccc".into(),
        element: brief,
    }))
    .unwrap();
    s.edit_project(ProjectEdit::Breakdown(BreakdownEdit::AddSceneCharacter {
        scene: "cccc".into(),
        character: "carl".into(),
    }))
    .unwrap();

    s
}

/// The report as CSV, read back as rows of fields (none of the cells holds a comma or a quote).
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

fn write(s: &mut Session, report: Report, format: ReportFormat, name: &str) -> Vec<u8> {
    let path = dir_of(name).with_extension(format.name());
    s.export_report(report, format, &path, Language::German)
        .unwrap();
    let bytes = std::fs::read(&path).unwrap();
    let _ = std::fs::remove_file(&path);

    bytes
}

fn sheets_in(pdf: &[u8]) -> usize {
    String::from_utf8_lossy(pdf)
        .matches("/Type /Page /")
        .count()
}

#[test]
fn the_scene_report_lists_number_heading_length_and_figures() {
    let mut s = open("report-scenes");

    let rows = rows(&mut s, Report::Scenes, "report-scenes-csv");

    assert_eq!(rows[0], ["Nummer", "Kopf", "Länge (Seiten)", "Figuren"]);
    // The length is the layout's, in eighths of a page: a short scene is not empty.
    assert_eq!(&rows[1][..2], ["14", "INT. WACHRAUM – NACHT"]);
    assert!(!rows[1][2].is_empty());
    // A speaking figure first, then one without dialog from the sheet; the CSV quotes the list.
    assert_eq!(rows[1][3..], ["\"ANNA", " BOB\""]);
    assert_eq!(rows[2][..2], ["23", "EXT. HAFEN – NACHT"]);
    assert_eq!(rows[2].len(), 4);
    assert_eq!(rows[2][3], "");
}

#[test]
fn the_place_report_names_the_shooting_location() {
    let mut s = open("report-places");

    let rows = rows(&mut s, Report::Places, "report-places-csv");

    assert_eq!(rows[0], ["Spielort", "Drehort", "Szenen"]);
    assert_eq!(rows[1], ["WACHRAUM", "Leuchtturm", "14"]);
    assert_eq!(rows[2], ["HAFEN", "", "23"]);
}

#[test]
fn the_cross_table_marks_who_is_in_which_scene() {
    let mut s = open("report-cast");

    let rows = rows(&mut s, Report::Cast, "report-cast-csv");

    // CARL is in no scene and has no column.
    assert_eq!(rows[0], ["Szene", "1  ANNA", "2  BOB"]);
    assert_eq!(rows[1], ["14", "X", "X"]);
    assert_eq!(rows[2], ["23", "", ""]);
}

#[test]
fn the_extract_as_a_table_has_a_row_per_element_of_a_scene() {
    let mut s = open("report-extract");

    let rows = rows(&mut s, Report::Extract, "report-extract-csv");

    assert_eq!(rows[0], ["Szene", "Kategorie", "Element", "Status"]);
    // By category in the sheet's order; categories without elements are not there.
    assert_eq!(
        rows[1..],
        [
            ["14", "Kostüm", "Mantel", "offen"],
            ["14", "Requisite", "Öllampe", "offen"],
            ["14", "Requisite", "Brief", "offen"],
            ["23", "Requisite", "Brief", "offen"],
        ]
    );
}

#[test]
fn every_table_is_a_pdf_on_landscape_sheets() {
    let mut s = open("report-pdf");

    for report in [
        Report::Scenes,
        Report::Places,
        Report::Cast,
        Report::Extract,
    ] {
        let pdf = write(&mut s, report, ReportFormat::Pdf, "report-pdf-file");

        assert!(pdf.starts_with(b"%PDF"), "{}", report.name());
        let text = String::from_utf8_lossy(&pdf);
        let landscape = text.contains("/MediaBox [0 0 841.89 595.28]");
        assert_eq!(landscape, report != Report::Extract, "{}", report.name());
    }
}

#[test]
fn the_extract_pdf_has_a_portrait_page_per_scene_and_no_empty_category() {
    let mut s = open("report-extract-pdf");

    let pdf = write(
        &mut s,
        Report::Extract,
        ReportFormat::Pdf,
        "report-extract-file",
    );

    assert_eq!(sheets_in(&pdf), 2);
    let text = String::from_utf8_lossy(&pdf);
    assert!(text.contains("/MediaBox [0 0 595.28 841.89]"));
    assert!(text.contains("(Spieltag: Tag 3"));
    assert!(text.contains("Vorstopp: 1:35"));
    assert!(text.contains("(Requisite)"));
    // ü is the octal escape 374 in the PDF's text.
    assert!(text.contains("(Kost\\374m)"));
    // Nobody needs sound in this film.
    assert!(!text.contains("(Ton)"));
}

#[test]
fn ods_and_xlsx_hold_the_cells() {
    let mut s = open("report-sheets");

    for format in [ReportFormat::Ods, ReportFormat::Xlsx] {
        let bytes = write(&mut s, Report::Cast, format, "report-sheets-file");

        // A zip, its entries stored: the cells are in the bytes.
        assert!(bytes.starts_with(b"PK\x03\x04"));
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.contains("ANNA"), "{}", format.name());
        assert!(text.contains("Figuren je Szene"), "{}", format.name());
    }
}

#[test]
fn the_overview_offers_the_reports() {
    let s = open("report-overview");

    let overview = s.overview();

    assert_eq!(
        overview.reports,
        ["szenen", "spielorte", "figuren", "auszug", "kontakte"]
    );
    assert_eq!(overview.report_formats, ["pdf", "csv", "ods", "xlsx"]);
}

#[test]
fn struck_scenes_are_in_no_report() {
    let mut s = open_struck("report-struck", true);

    let scenes = rows(&mut s, Report::Scenes, "report-struck-scenes");
    let places = rows(&mut s, Report::Places, "report-struck-places");
    let cast = rows(&mut s, Report::Cast, "report-struck-cast");
    let extract = rows(&mut s, Report::Extract, "report-struck-extract");
    let pdf = write(
        &mut s,
        Report::Extract,
        ReportFormat::Pdf,
        "report-struck-pdf",
    );

    assert_eq!(scenes.len(), 2);
    assert_eq!(scenes[1][0], "14");
    assert_eq!(places[1..], [["WACHRAUM", "", "14"], ["HAFEN", "", ""]]);
    assert_eq!(cast, [["Szene", "1  ANNA"], ["14", "X"]]);
    assert_eq!(extract.len(), 1);
    assert_eq!(sheets_in(&pdf), 1);
}

#[test]
fn a_book_without_scenes_has_no_report() {
    let mut s = open_struck("report-none", false);
    let path = dir_of("report-none-file").with_extension("csv");

    // The contact list lists no scenes and so has none missing.
    for report in Report::ALL.into_iter().filter(|r| *r != Report::Contacts) {
        let result = s.export_report(report, ReportFormat::Csv, &path, Language::German);

        let error = result.unwrap_err();
        assert_eq!(
            error_text(Language::German, &error),
            "Keine Szenen im Buch",
            "{}",
            report.name()
        );
        assert_eq!(
            error_text(Language::English, &error),
            "No scenes in the book"
        );
    }
}
