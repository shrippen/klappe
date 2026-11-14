//! The object library: built-in, per project (`bibliothek.toml`) and as a file to pass on.
#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use klappe_model::{LibraryItem, ObjectShape, Project};
use klappe_store::{
    builtin_library, load, load_backup, load_library, save, save_backup, save_library, StoreError,
};

const EXAMPLE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../testdata/beispiel.klappe"
);

fn test_dir(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    dir
}

fn own(kind: &str, label: &str, width_m: f64, depth_m: f64, shape: ObjectShape) -> LibraryItem {
    LibraryItem {
        kind: kind.into(),
        label: label.into(),
        width_m,
        depth_m,
        shape,
        height_m: None,
    }
}

fn throne() -> LibraryItem {
    own("thron", "Thron", 0.9, 0.8, ObjectShape::Rect)
}

fn pond() -> LibraryItem {
    own("teich", "Teich", 6.0, 4.0, ObjectShape::Ellipse)
}

#[test]
fn the_built_in_library_has_what_the_lastenheft_names() {
    let kinds: Vec<&str> = builtin_library().iter().map(|i| i.kind.as_str()).collect();

    for wanted in [
        "table",
        "chair",
        "bed",
        "sofa",
        "door",
        "window",
        "pillar",
        "fireplace",
        "stairs",
        "car",
        "carriage",
        "horse",
        "tree",
        "platform",
    ] {
        assert!(kinds.contains(&wanted), "{wanted} is missing");
    }
}

#[test]
fn every_built_in_item_has_a_usable_size_and_a_kind_of_its_own() {
    let items = builtin_library();

    for (index, item) in items.iter().enumerate() {
        assert!(item.width_m > 0.0 && item.width_m <= 100.0, "{}", item.kind);
        assert!(item.depth_m > 0.0 && item.depth_m <= 100.0, "{}", item.kind);
        assert!(item.label.is_empty(), "{} has a label", item.kind);
        assert!(items[..index].iter().all(|o| o.kind != item.kind));
    }
}

/// LH-17: a built-in height is one an object can have; the flat ones have none.
#[test]
fn every_built_in_height_is_usable() {
    for item in builtin_library() {
        let height_m = item.height_m.unwrap_or(1.0);

        assert!(height_m > 0.0 && height_m <= 100.0, "{}", item.kind);
    }
    let find = |kind: &str| builtin_library().iter().find(|i| i.kind == kind).unwrap();

    assert_eq!(find("table").height_m, Some(0.75));
    assert_eq!(find("door").height_m, Some(2.1));
}

#[test]
fn a_round_pillar_and_a_square_table() {
    let find = |kind: &str| builtin_library().iter().find(|i| i.kind == kind).unwrap();

    assert_eq!(find("pillar").shape, ObjectShape::Ellipse);
    assert_eq!(find("table").shape, ObjectShape::Rect);
    assert_eq!((find("table").width_m, find("table").depth_m), (1.6, 0.9));
}

#[test]
fn own_objects_are_saved_and_loaded_with_the_project() {
    let dir = test_dir("library-roundtrip");
    let mut project = Project::new("Eigene");
    project.library = vec![throne(), pond()];

    save(&dir, &project, None).unwrap();
    let text = fs::read_to_string(dir.join("bibliothek.toml")).unwrap();

    assert!(text.contains("[[objekt]]") && text.contains("label = \"Thron\""));
    assert_eq!(load(&dir).unwrap().project.library, vec![throne(), pond()]);
}

/// LH-17: the height of an own object is kept in `bibliothek.toml`, and not written when
/// the object has none.
#[test]
fn the_height_of_an_own_object_round_trips() {
    let dir = test_dir("library-height");
    let tall = LibraryItem {
        height_m: Some(1.1),
        ..throne()
    };
    let mut project = Project::new("Eigene");
    project.library = vec![tall.clone(), pond()];

    save(&dir, &project, None).unwrap();
    let text = fs::read_to_string(dir.join("bibliothek.toml")).unwrap();

    assert_eq!(text.matches("height_m").count(), 1, "{text}");
    assert_eq!(load(&dir).unwrap().project.library, vec![tall, pond()]);
}

#[test]
fn a_project_without_own_objects_has_no_library_file() {
    let dir = test_dir("library-none");
    let mut project = Project::new("Eigene");
    project.library = vec![throne()];
    save(&dir, &project, None).unwrap();
    assert!(dir.join("bibliothek.toml").exists());

    project.library.clear();
    save(&dir, &project, None).unwrap();

    assert!(!dir.join("bibliothek.toml").exists());
    assert!(load(&dir).unwrap().project.library.is_empty());
}

/// An older project has no `bibliothek.toml`: it loads with an empty library and saves
/// without one.
#[test]
fn an_older_project_loads_and_saves_unchanged() {
    let dir = test_dir("library-older");
    let loaded = load(Path::new(EXAMPLE)).unwrap();
    assert!(loaded.project.library.is_empty());

    save(&dir, &loaded.project, loaded.script.as_deref()).unwrap();

    assert!(!dir.join("bibliothek.toml").exists());
    assert_eq!(load(&dir).unwrap(), loaded);
}

#[test]
fn a_library_file_to_pass_on_reads_back_as_written() {
    let dir = test_dir("library-file");
    let path = dir.join("mittelalter.toml");

    save_library(&path, &[throne(), pond()]).unwrap();

    assert_eq!(load_library(&path).unwrap(), vec![throne(), pond()]);
}

/// Tests run side by side: each file gets a folder of its own.
fn parse_error(text: &str) -> String {
    static COUNT: AtomicUsize = AtomicUsize::new(0);
    let dir = test_dir(&format!(
        "library-broken-{}",
        COUNT.fetch_add(1, Ordering::SeqCst)
    ));
    let path = dir.join("kaputt.toml");
    fs::write(&path, text).unwrap();

    let err = load_library(&path).unwrap_err();

    assert!(matches!(err, StoreError::Parse { .. }), "{err:?}");
    err.to_string()
}

#[test]
fn broken_library_files_are_refused_with_the_reason() {
    assert!(!parse_error("[[objekt]\nkind = ").is_empty());
    assert!(parse_error("[[objekt]]\nkind = \"x\"\nwidth_m = 1.0\n").contains("depth_m"));
    assert!(
        parse_error("[[objekt]]\nkind = \"x\"\nwidth_m = \"breit\"\ndepth_m = 1.0\n")
            .contains("width_m")
    );
    assert!(parse_error(
        "[[objekt]]\nkind = \"x\"\nwidth_m = 1.0\ndepth_m = 1.0\nshape = \"star\"\n"
    )
    .contains("star"));
}

#[test]
fn items_that_cannot_be_used_refuse_the_whole_file() {
    let item = |kind: &str, w: &str, d: &str| {
        format!("[[objekt]]\nkind = \"{kind}\"\nwidth_m = {w}\ndepth_m = {d}\n")
    };

    assert!(parse_error(&item("x", "0.0", "1.0")).contains("not usable"));
    assert!(parse_error(&item("x", "1.0", "-2.0")).contains("not usable"));
    assert!(parse_error(&item("x", "nan", "1.0")).contains("not usable"));
    assert!(parse_error(&item("x", "inf", "1.0")).contains("not usable"));
    assert!(parse_error(&item("x", "5000.0", "1.0")).contains("not usable"));
    assert!(parse_error(&item("  ", "1.0", "1.0")).contains("kind is empty"));

    // A height is more than nothing and at most 100 m (LH-17).
    let tall = |h: &str| format!("{}height_m = {h}\n", item("x", "1.0", "1.0"));
    for bad in ["0.0", "-1.0", "nan", "inf", "100.5"] {
        assert!(parse_error(&tall(bad)).contains("height"), "{bad}");
    }

    // A good item before a bad one does not save the file.
    let text = format!("{}{}", item("ok", "1.0", "1.0"), item("bad", "0.0", "1.0"));
    assert!(parse_error(&text).contains("bad"));

    let twice = format!("{}{}", item("ok", "1.0", "1.0"), item("ok", "2.0", "2.0"));
    assert!(parse_error(&twice).contains("twice"));
}

#[test]
fn a_missing_library_file_is_an_error_naming_it() {
    let dir = test_dir("library-missing");

    let err = load_library(&dir.join("fehlt.toml")).unwrap_err();

    assert!(matches!(err, StoreError::Io { .. }));
    assert!(err.to_string().contains("fehlt.toml"));
}

#[test]
fn a_broken_project_library_stops_the_project_from_opening() {
    let dir = test_dir("library-project-broken");
    save(&dir, &Project::new("Eigene"), None).unwrap();
    fs::write(dir.join("bibliothek.toml"), "[[objekt]]\nkind = \"x\"\n").unwrap();

    let err = load(&dir).unwrap_err();

    assert!(matches!(err, StoreError::Parse { .. }));
    assert!(err.to_string().contains("bibliothek.toml"));
}

#[test]
fn unusable_items_are_not_written() {
    let dir = test_dir("library-unwritable");
    let path = dir.join("nie.toml");
    let bad = own("x", "X", f64::NAN, 1.0, ObjectShape::Rect);

    let err = save_library(&path, &[throne(), bad.clone()]).unwrap_err();
    assert!(matches!(err, StoreError::Encode { .. }));
    assert!(!path.exists());

    let mut project = Project::new("Eigene");
    project.library = vec![bad];
    assert!(save(&dir, &project, None).is_err());
}

/// An object whose kind no library knows is kept as it is; one with a shape keeps it.
#[test]
fn objects_keep_their_unknown_kind_and_their_shape() {
    let dir = test_dir("library-objects");
    let mut loaded = load(Path::new(EXAMPLE)).unwrap();
    let set = &mut loaded.project.locations[0].set;
    set.objects[0].kind = "zeppelin".into();
    set.objects[1].shape = ObjectShape::Ellipse;

    save(&dir, &loaded.project, loaded.script.as_deref()).unwrap();
    let text = fs::read_to_string(dir.join("drehorte/saengersaal.toml")).unwrap();

    assert_eq!(text.matches("shape = \"ellipse\"").count(), 1);
    assert!(!text.contains("shape = \"rect\""));

    let again = load(&dir).unwrap().project;
    let objects = &again.locations[0].set.objects;
    assert_eq!(objects[0].kind, "zeppelin");
    assert_eq!(objects[0].shape, ObjectShape::Rect);
    assert_eq!(objects[1].shape, ObjectShape::Ellipse);
}

/// The backup is a whole project folder: own objects survive a crash too.
#[test]
fn the_backup_keeps_own_objects() {
    let dir = test_dir("library-backup");
    let mut project = Project::new("Eigene");
    project.library = vec![throne()];

    save_backup(&dir, &project, None).unwrap();

    let backup = load_backup(&dir).unwrap().unwrap();
    assert_eq!(backup.project.library, vec![throne()]);
    assert!(!dir.join("bibliothek.toml").exists());
}

/// Review of PR 28: any TOML file read as an empty library, so importing `projekt.toml` or a
/// file with a misspelt `[[objkt]]` said "nothing new" instead of "not a library".
#[test]
fn a_file_that_is_no_library_is_an_error() {
    let dir = test_dir("no-library");
    for (name, text) in [
        ("typo.toml", "[[objkt]]\nkind = \"thron\"\n"),
        ("projekt.toml", "title = \"Wartburg\"\n"),
    ] {
        let path = dir.join(name);
        fs::write(&path, text).unwrap();

        assert!(load_library(&path).is_err(), "{name}");
    }
}
