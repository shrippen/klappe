//! Besetzung/Team: the contact list and the cast number in the reports.
#![allow(clippy::unwrap_used)]

mod team_common;

use klappe_model::{Contact, Project};
use klappe_services::{
    Department, Language, Report, ReportFormat, ServiceError, Session, TeamEdit,
};
use team_common::{dir_of, open, open_with, set_contact, team};

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
    set_contact(
        &mut s,
        &a,
        Contact {
            phone: "0151".into(),
            mail: "z@example.org".into(),
            address: "Weg 1\n10115 Berlin".into(),
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
