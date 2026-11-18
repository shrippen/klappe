//! The spoken dialogue of a scene (LH-64), on the example project.
#![allow(clippy::unwrap_used)]

use std::path::Path;

use klappe_services::{Language, Session};

const EXAMPLE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../testdata/beispiel.klappe"
);

#[test]
fn the_dialogue_is_read_where_its_cue_is_linked() {
    let s = Session::open(Path::new(EXAMPLE)).unwrap();

    let speech = s.speech(Language::German);

    let lines: Vec<_> = speech
        .lines
        .iter()
        .map(|l| (l.at_s, l.text.as_str()))
        .collect();
    assert_eq!(
        lines,
        [(6.0, "Ihr kommt spät."), (9.0, "Der Weg war lang.")]
    );
}

#[test]
fn the_voice_speaks_the_surfaces_language_while_the_project_has_none() {
    let s = Session::open(Path::new(EXAMPLE)).unwrap();

    assert_eq!(s.speech(Language::German).language, "de");
    assert_eq!(s.speech(Language::English).language, "en");
}

#[test]
fn the_voice_speaks_the_projects_language() {
    let dir = std::env::temp_dir().join(format!("klappe-speech-{}", std::process::id()));
    let s = Session::create(&dir, "Sprache", Language::English).unwrap();

    let language = s.speech(Language::German).language;

    std::fs::remove_dir_all(&dir).unwrap();
    assert_eq!(language, "en");
}
