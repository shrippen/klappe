//! The templates: built-in, and the project's own in `vorlagen.toml`.
#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};

use klappe_model::{Project, Template};
use klappe_store::{builtin_templates, load, load_backup, save, save_backup};

fn test_dir(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    dir
}

/// A built-in template under an own id and name.
fn own(id: &str, label: &str) -> Template {
    Template {
        id: id.into(),
        label: label.into(),
        ..builtin_templates()[0].clone()
    }
}

#[test]
fn own_templates_are_saved_and_loaded_with_the_project() {
    let dir = test_dir("templates-roundtrip");
    let mut project = Project::new("Eigene");
    project.templates = vec![
        own("mein-dialog", "Mein Dialog"),
        builtin_templates()[2].clone(),
    ];

    save(&dir, &project, None).unwrap();
    let text = fs::read_to_string(dir.join("vorlagen.toml")).unwrap();

    assert!(text.contains("[[vorlage]]") && text.contains("label = \"Mein Dialog\""));
    assert_eq!(load(&dir).unwrap().project.templates, project.templates);
}

#[test]
fn a_project_without_own_templates_has_no_file() {
    let dir = test_dir("templates-none");
    let mut project = Project::new("Eigene");
    project.templates = vec![own("a", "A")];

    save(&dir, &project, None).unwrap();
    assert!(dir.join("vorlagen.toml").exists());

    project.templates.clear();
    save(&dir, &project, None).unwrap();
    assert!(!dir.join("vorlagen.toml").exists());
    assert!(load(&dir).unwrap().project.templates.is_empty());
}

#[test]
fn the_backup_keeps_own_templates() {
    let dir = test_dir("templates-backup");
    let mut project = Project::new("Eigene");
    project.templates = vec![own("a", "A")];

    save_backup(&dir, &project, None).unwrap();

    let backup = load_backup(&dir).unwrap().unwrap();
    assert_eq!(backup.project.templates, project.templates);
}

#[test]
fn a_broken_template_file_is_refused() {
    let dir = test_dir("templates-broken");
    let mut project = Project::new("Eigene");
    project.templates = vec![own("a", "A")];
    save(&dir, &project, None).unwrap();

    for text in [
        "[[vorlage]]\nid = \"leer\"\n",
        "[[vorlag]]\nid = \"x\"\n",
        "titel = 1\n",
    ] {
        fs::write(dir.join("vorlagen.toml"), text).unwrap();
        assert!(load(&dir).is_err(), "{text}");
    }
}

/// TOML knows `nan` and `inf`; a number a mark cannot use would load but never save again.
#[test]
fn a_template_file_with_unusable_numbers_is_refused() {
    let dir = test_dir("templates-numbers");
    let mut project = Project::new("Eigene");
    project.templates = vec![own("a", "A")];
    save(&dir, &project, None).unwrap();
    let base = include_str!("../template/standard.toml");
    let camera_mark = "height_m = 1.6";

    for (from, to) in [
        ("at_s = 0.0", "at_s = nan"),
        ("x_m = -1.0", "x_m = inf"),
        ("facing_deg = 0.0", "facing_deg = nan"),
        ("facing_deg = 0.0", "facing_deg = 0.0\nhold_s = nan"),
        (
            "facing_deg = 0.0",
            "facing_deg = 0.0\nvia = { x_m = nan, y_m = 0.0 }",
        ),
        (camera_mark, "height_m = nan"),
        (camera_mark, "height_m = 0.0"),
        (camera_mark, "height_m = 1.6\ntilt_deg = inf"),
        (camera_mark, "height_m = 1.6\nlens_mm = nan"),
        (camera_mark, "height_m = 1.6\nlens_mm = 0.0"),
    ] {
        assert!(base.contains(from), "{from}");
        fs::write(dir.join("vorlagen.toml"), base.replacen(from, to, 1)).unwrap();
        assert!(load(&dir).is_err(), "{to}");
    }
}

/// A template that cannot be read back is not written.
#[test]
fn an_unusable_template_is_not_saved() {
    let dir = test_dir("templates-unusable");
    let mut project = Project::new("Eigene");
    project.templates = vec![own("a", "A"), own("a", "B")];

    assert!(save(&dir, &project, None).is_err());
}
