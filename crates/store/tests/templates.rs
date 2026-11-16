//! The templates: built-in, and the project's own in `vorlagen.toml`.
#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};

use klappe_model::{Project, Template};
use klappe_store::{
    builtin_templates, load, load_backup, load_template_file, save, save_backup,
    save_template_file, StoreError,
};

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
        own("lange-fahrt", "Lange Fahrt"),
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
        // Finite, but beyond what the editor takes.
        ("x_m = -1.0", "x_m = 1e307"),
        ("x_m = -1.0", "x_m = -1001.0"),
        ("facing_deg = 0.0", "facing_deg = 0.0\nhold_s = -5.0"),
        (camera_mark, "height_m = 1e6"),
        (camera_mark, "height_m = 1.6\ntilt_deg = 1000.0"),
        (camera_mark, "height_m = 1.6\nlens_mm = 1e-300"),
        (camera_mark, "height_m = 1.6\nlens_mm = 1e9"),
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

/// An own template is listed by its label; an empty one would show a blank line.
#[test]
fn an_own_template_without_a_label_is_refused() {
    let dir = test_dir("templates-label");
    let mut project = Project::new("Eigene");
    project.templates = vec![own("a", " ")];

    assert!(save(&dir, &project, None).is_err());

    project.templates = vec![own("a", "A")];
    save(&dir, &project, None).unwrap();
    let text = fs::read_to_string(dir.join("vorlagen.toml")).unwrap();
    fs::write(
        dir.join("vorlagen.toml"),
        text.replace("label = \"A\"\n", ""),
    )
    .unwrap();

    assert!(load(&dir).is_err());
}

/// The camera's lens, sensor and frame stay within the editor's bounds too.
#[test]
fn a_camera_beyond_the_editor_bounds_is_refused() {
    let dir = test_dir("templates-camera");
    let base = builtin_templates()[0].clone();
    let sets: [fn(&mut Template); 3] = [
        |t| t.cameras[0].lens_mm = 1e-300,
        |t| t.cameras[0].aspect = 1e9,
        |t| t.cameras[0].sensor = klappe_model::Sensor::of_width_mm(1e9),
    ];

    for set in sets {
        let mut template = own("a", "A");
        template.cameras = base.cameras.clone();
        set(&mut template);
        let mut project = Project::new("Eigene");
        project.templates = vec![template];

        assert!(save(&dir, &project, None).is_err());
    }
}

#[test]
fn a_template_file_to_pass_on_reads_back_as_written() {
    let dir = test_dir("templates-file");
    let path = dir.join("dialoge.toml");
    let templates = vec![own("mein-dialog", "Mein Dialog"), own("fahrt", "Fahrt")];

    save_template_file(&path, &templates).unwrap();
    let text = fs::read_to_string(&path).unwrap();

    assert!(text.contains("[[vorlage]]"));
    assert_eq!(load_template_file(&path).unwrap(), templates);
}

#[test]
fn a_file_that_is_no_template_file_is_refused() {
    let dir = test_dir("templates-file-broken");

    let missing = load_template_file(&dir.join("fehlt.toml")).unwrap_err();
    assert!(matches!(missing, StoreError::Io { .. }), "{missing:?}");

    let other = dir.join("projekt.toml");
    fs::write(&other, "title = \"Kein Vorlagenfile\"\n").unwrap();
    let err = load_template_file(&other).unwrap_err();
    assert!(matches!(err, StoreError::Parse { .. }), "{err:?}");

    let unnamed = dir.join("namenlos.toml");
    save_template_file(&unnamed, &[own("a", "A")]).unwrap();
    let text = fs::read_to_string(&unnamed)
        .unwrap()
        .replace("\"A\"", "\" \"");
    fs::write(&unnamed, text).unwrap();
    let err = load_template_file(&unnamed).unwrap_err();
    assert!(matches!(err, StoreError::Parse { .. }), "{err:?}");
}

#[test]
fn an_unusable_template_is_not_written_to_a_file() {
    let dir = test_dir("templates-file-bad");
    let path = dir.join("schlecht.toml");

    let err = save_template_file(&path, &[own("a", " ")]).unwrap_err();

    assert!(matches!(err, StoreError::Encode { .. }), "{err:?}");
    assert!(!path.exists());
}
