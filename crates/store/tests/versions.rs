//! The locked versions: `drehbuch/fassungen.toml` and `drehbuch/fassungen/<id>.fountain` (DB-40).
#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};

use klappe_model::{PageStart, RevisionColor, Version};
use klappe_store::{
    load_version_text, load_versions, remove_version_text, save_version_text, save_versions,
    StoreError,
};

fn test_dir(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    dir
}

fn version(id: &str, name: &str, date: &str, color: RevisionColor) -> Version {
    Version {
        id: id.into(),
        name: name.into(),
        date: date.into(),
        color,
        pages: Vec::new(),
    }
}

/// Writes `text` as the list of `dir`.
fn write_list(dir: &Path, text: &str) {
    fs::create_dir_all(dir.join("drehbuch")).unwrap();
    fs::write(dir.join("drehbuch/fassungen.toml"), text).unwrap();
}

#[test]
fn no_file_is_no_version() {
    let dir = test_dir("versions-missing");

    assert_eq!(load_versions(&dir).unwrap(), Vec::new());
}

#[test]
fn the_list_keeps_its_order() {
    let dir = test_dir("versions-roundtrip");
    let list = vec![
        version(
            "aaaaaaaa",
            "Drehfassung",
            "2026-11-25",
            RevisionColor::White,
        ),
        version("bbbbbbbb", "Nachdreh", "2027-01-02", RevisionColor::Blue),
    ];

    save_versions(&dir, &list).unwrap();

    let text = fs::read_to_string(dir.join("drehbuch/fassungen.toml")).unwrap();
    assert!(text.contains("farbe = \"blue\""), "{text}");
    assert_eq!(load_versions(&dir).unwrap(), list);
}

#[test]
fn the_pages_of_a_version_come_back() {
    let dir = test_dir("versions-pages");
    let page = |label: &str, line, offset| PageStart {
        label: label.into(),
        line,
        offset,
    };
    let mut locked = version("aaaaaaaa", "Dreh", "2026-11-25", RevisionColor::White);
    locked.pages = vec![page("1", 0, 0), page("2", 58, 17), page("2A", 90, 0)];
    let old = version("bbbbbbbb", "Alt", "2026-11-26", RevisionColor::Blue);

    save_versions(&dir, &[locked.clone(), old.clone()]).unwrap();

    // The column is only written when a page starts within a line.
    let text = fs::read_to_string(dir.join("drehbuch/fassungen.toml")).unwrap();
    assert!(text.contains("nummer = \"2A\""), "{text}");
    assert_eq!(text.matches("spalte").count(), 1, "{text}");
    assert_eq!(load_versions(&dir).unwrap(), vec![locked, old]);
}

#[test]
fn a_version_without_pages_locks_none() {
    let dir = test_dir("versions-no-pages");
    write_list(
        &dir,
        "[[fassung]]\nid = \"a\"\nname = \"N\"\ndatum = \"2026-11-25\"\nfarbe = \"blue\"\n",
    );

    let versions = load_versions(&dir).unwrap();

    assert!(versions[0].pages.is_empty());
}

#[test]
fn a_page_without_its_line_is_refused() {
    let dir = test_dir("versions-bad-page");
    write_list(
        &dir,
        "[[fassung]]\nid = \"a\"\nname = \"N\"\ndatum = \"2026-11-25\"\nfarbe = \"blue\"\n\
         [[fassung.seiten]]\nnummer = \"1\"\n",
    );

    let error = load_versions(&dir).unwrap_err();

    assert!(matches!(error, StoreError::Parse { .. }), "{error}");
}

#[test]
fn the_text_comes_back_as_it_was_locked() {
    let dir = test_dir("versions-text");
    let text = "INT. SAAL - TAG #1#\r\n\r\nEr wartet.\n";

    save_version_text(&dir, "aaaaaaaa", text).unwrap();

    assert!(dir.join("drehbuch/fassungen/aaaaaaaa.fountain").exists());
    assert_eq!(load_version_text(&dir, "aaaaaaaa").unwrap(), text);

    remove_version_text(&dir, "aaaaaaaa").unwrap();
    remove_version_text(&dir, "aaaaaaaa").unwrap();
    assert!(load_version_text(&dir, "aaaaaaaa").is_err());
}

#[test]
fn conflict_marks_are_refused_with_their_line() {
    let dir = test_dir("versions-conflict");
    write_list(&dir, "[[fassung]]\nid = \"a\"\n<<<<<<< HEAD\n");

    let error = load_versions(&dir).unwrap_err();

    assert!(
        matches!(error, StoreError::Conflict { line: 3, .. }),
        "{error}"
    );
}

#[test]
fn a_wrong_type_is_refused() {
    let dir = test_dir("versions-type");
    let entry = |extra: &str| format!("[[fassung]]\nid = \"a\"\nname = \"N\"\n{extra}\n");

    for list in [
        entry("datum = 5\nfarbe = \"blue\""),
        entry("datum = 2026-11-25\nfarbe = \"blue\""),
        entry("datum = \"2026-11-25\"\nfarbe = \"purple\""),
        entry("datum = \"2026-11-25\"\nfarbe = \"blue\"\nextra = 1"),
        entry("farbe = \"blue\""),
    ] {
        write_list(&dir, &list);

        let error = load_versions(&dir).unwrap_err();
        assert!(matches!(error, StoreError::Parse { .. }), "{list}: {error}");
    }
}

#[test]
fn a_date_that_does_not_exist_is_refused() {
    let dir = test_dir("versions-date");
    write_list(
        &dir,
        "[[fassung]]\nid = \"a\"\nname = \"N\"\ndatum = \"2026-02-30\"\nfarbe = \"white\"\n",
    );

    assert!(matches!(
        load_versions(&dir).unwrap_err(),
        StoreError::Parse { .. }
    ));
}

#[test]
fn an_unsafe_or_doubled_id_is_refused() {
    let dir = test_dir("versions-id");
    let entry = |id: &str| {
        format!(
            "[[fassung]]\nid = \"{id}\"\nname = \"N\"\ndatum = \"2026-11-25\"\nfarbe = \"white\"\n"
        )
    };

    write_list(&dir, &entry("../x"));
    assert!(matches!(
        load_versions(&dir).unwrap_err(),
        StoreError::BadId { .. }
    ));

    write_list(&dir, &format!("{}{}", entry("a"), entry("A")));
    assert!(matches!(
        load_versions(&dir).unwrap_err(),
        StoreError::DuplicateId { .. }
    ));

    assert!(load_version_text(&dir, "../x").is_err());
}
