//! Aliases and names that are no figure (DB-50) survive a save and a load, and are not
//! written when empty.
#![allow(clippy::unwrap_used)]

use std::path::Path;

use klappe_model::{Character, FigureColor, Project};

#[test]
fn aliases_and_not_figures_round_trip() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("aliases");
    let _ = std::fs::remove_dir_all(&dir);
    let mut project = Project::new("Test");
    project.characters.push(Character {
        id: "elisabeth".into(),
        name: "Elisabeth".into(),
        short: "E".into(),
        color: FigureColor::Purple,
        group: None,
        size_m: None,
        aliases: vec!["LIESL".into()],
    });
    project.not_figures = vec!["ALLE".into(), "STIMME".into()];

    klappe_store::save(&dir, &project, None).unwrap();

    let loaded = klappe_store::load(&dir).unwrap().project;
    assert_eq!(loaded.characters, project.characters);
    assert_eq!(loaded.not_figures, project.not_figures);
    let file = std::fs::read_to_string(dir.join("figuren.toml")).unwrap();
    assert!(
        file.starts_with("not_figures"),
        "keys before the tables: {file}"
    );
}

#[test]
fn nothing_is_written_when_there_are_none() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("aliases-none");
    let _ = std::fs::remove_dir_all(&dir);

    klappe_store::save(&dir, &Project::new("Test"), None).unwrap();

    let file = std::fs::read_to_string(dir.join("figuren.toml")).unwrap();
    assert!(
        !file.contains("not_figures") && !file.contains("aliases"),
        "{file}"
    );
}

#[test]
fn an_empty_alias_does_not_load() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("aliases-empty");
    let _ = std::fs::remove_dir_all(&dir);
    klappe_store::save(&dir, &Project::new("Test"), None).unwrap();
    std::fs::write(
        dir.join("figuren.toml"),
        "[[figur]]\nid = \"a\"\nname = \"A\"\nshort = \"A\"\ncolor = \"red\"\naliases = [\" \"]\n",
    )
    .unwrap();

    assert!(klappe_store::load(&dir).is_err());
}

fn figure_file(name: &str, body: &str) -> std::path::PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    klappe_store::save(&dir, &Project::new("Test"), None).unwrap();
    std::fs::write(dir.join("figuren.toml"), body).unwrap();

    dir
}

#[test]
fn a_name_or_alias_of_two_figures_does_not_load() {
    let figure = |id: &str, name: &str, aliases: &str| {
        format!(
            "[[figur]]\nid = \"{id}\"\nname = \"{name}\"\nshort = \"{id}\"\ncolor = \"red\"\naliases = [{aliases}]\n"
        )
    };
    let cases = [
        // name twice, in another case
        figure("a", "Anna", "") + &figure("b", "ANNA", ""),
        // alias of one is the name of another
        figure("a", "Anna", "\"Bert\"") + &figure("b", "bert", ""),
        // alias twice
        figure("a", "Anna", "\"Liesl\"") + &figure("b", "Bert", "\"LIESL\""),
    ];

    for (i, body) in cases.iter().enumerate() {
        let dir = figure_file(&format!("aliases-twice-{i}"), body);

        assert!(klappe_store::load(&dir).is_err(), "case {i}: {body}");
    }
}
