//! The scripted locations: `orte.toml` (DB-51, FORMAT.md).
#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};

use klappe_model::{Place, Project};
use klappe_store::{load_places, save, save_places};

const EXAMPLE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../testdata/beispiel.klappe"
);

fn place(id: &str, name: &str, variants: &[&str]) -> Place {
    Place {
        id: id.into(),
        name: name.into(),
        variants: variants.iter().map(|v| (*v).into()).collect(),
    }
}

fn dir(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&dir);

    dir
}

#[test]
fn the_example_has_its_place() {
    assert_eq!(
        load_places(Path::new(EXAMPLE)).unwrap(),
        [place("k3m9q2xa", "SÄNGERSAAL", &[])]
    );
}

#[test]
fn places_survive_and_the_file_exists_only_with_them() {
    let dir = dir("places-roundtrip");
    save(&dir, &Project::new("T"), None).unwrap();
    assert!(!dir.join("orte.toml").exists());

    let places = vec![place("a", "SÄNGERSAAL", &["SAAL"]), place("b", "HOF", &[])];
    save_places(&dir, &places).unwrap();
    assert_eq!(load_places(&dir).unwrap(), places);
    let text = fs::read_to_string(dir.join("orte.toml")).unwrap();
    assert!(
        text.contains("[[ort]]") && text.contains("variants = [\"SAAL\"]"),
        "{text}"
    );
    assert_eq!(text.matches("variants").count(), 1, "{text}");

    save_places(&dir, &[]).unwrap();
    assert!(!dir.join("orte.toml").exists());
}

#[test]
fn save_places_writes_nothing_else() {
    let dir = dir("places-only");
    save(&dir, &Project::new("T"), None).unwrap();
    let before = fs::read(dir.join("projekt.toml")).unwrap();

    save_places(&dir, &[place("a", "HOF", &[])]).unwrap();

    assert_eq!(load_places(&dir).unwrap(), [place("a", "HOF", &[])]);
    assert_eq!(fs::read(dir.join("projekt.toml")).unwrap(), before);
}

#[test]
fn a_name_on_two_places_is_refused_on_save_and_load() {
    let dir = dir("places-refused");
    let both = vec![place("a", "Straße", &[]), place("b", "X", &["STRASSE"])];

    save(&dir, &Project::new("T"), None).unwrap();
    assert!(save_places(&dir, &both).is_err());

    fs::write(
        dir.join("orte.toml"),
        "[[ort]]\nid = \"a\"\nname = \"Straße\"\n\n[[ort]]\nid = \"b\"\nname = \"STRASSE\"\n",
    )
    .unwrap();
    assert!(load_places(&dir).is_err());
}

#[test]
fn an_empty_name_or_a_repeated_id_is_refused() {
    let dir = dir("places-bad");
    save(&dir, &Project::new("T"), None).unwrap();

    assert!(save_places(&dir, &[place("a", " ", &[])]).is_err());
    assert!(save_places(&dir, &[place("a", "X", &[]), place("a", "Y", &[])]).is_err());
    assert!(save_places(&dir, &[place("a", "X", &[""])]).is_err());
}

/// `orte.toml` is not the project's: a save never writes or removes it.
#[test]
fn a_project_save_never_touches_orte_toml() {
    let dir = dir("places-kept");
    save(&dir, &Project::new("T"), None).unwrap();
    save_places(&dir, &[place("a", "HOF", &[])]).unwrap();
    let before = fs::read(dir.join("orte.toml")).unwrap();

    save(&dir, &Project::new("T"), None).unwrap();

    assert_eq!(fs::read(dir.join("orte.toml")).unwrap(), before);
}
