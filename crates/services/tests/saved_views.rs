//! Saved views (LH-119): a named choice of visible layers, cameras and figures, kept in the
//! project.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::{Layer, ServiceError, Session, Visibility};

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe")
}

fn open(name: &str) -> Session {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    copy_dir(&example(), &dir);
    Session::open(&dir).unwrap()
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
            continue;
        }
        std::fs::copy(entry.path(), target).unwrap();
    }
}

/// Which cameras and figures the plan on view draws.
fn drawn(s: &Session) -> (bool, bool, bool) {
    let sources: Vec<_> = s
        .display_list()
        .items
        .into_iter()
        .filter_map(|i| i.source)
        .collect();
    let has = |id: &str| {
        sources.iter().any(|x| match x {
            klappe_services::Source::Camera(c) => c == id,
            klappe_services::Source::Figure(c) => c == id,
            _ => false,
        })
    };

    (has("a"), has("b"), has("elisabeth"))
}

#[test]
fn a_view_remembers_layers_cameras_and_figures() {
    let mut s = open("views-save");
    s.set_layer(Layer::Notes, Visibility::Hidden);
    s.set_track("b", Visibility::Hidden);
    s.set_track("burgvogt", Visibility::Hidden);

    let id = s.save_view("Nur Kamera A").unwrap();

    let view = &s.saved_views()[0];
    assert_eq!(view.id, id);
    assert_eq!(view.name, "Nur Kamera A");
    assert!(!view.layers.contains(&"notes".to_owned()));
    assert!(view.layers.contains(&"set".to_owned()));
    assert_eq!(view.cameras, ["a"]);
    assert_eq!(view.figures, ["elisabeth"]);
}

#[test]
fn choosing_a_view_shows_what_it_names_and_hides_the_rest() {
    let mut s = open("views-apply");
    s.set_track("b", Visibility::Hidden);
    s.set_layer(Layer::Lights, Visibility::Hidden);
    let id = s.save_view("Ruhig").unwrap();
    s.set_track("b", Visibility::Shown);
    s.set_layer(Layer::Lights, Visibility::Shown);
    s.set_layer(Layer::Figures, Visibility::Hidden);
    assert_eq!(drawn(&s), (true, true, false));

    s.apply_view(&id).unwrap();

    assert_eq!(s.layer(Layer::Figures), Visibility::Shown);
    assert_eq!(s.layer(Layer::Lights), Visibility::Hidden);
    assert_eq!(drawn(&s), (true, false, true));
    // A view is a view setting: no undo step.
    assert!(s.can_undo());
    s.undo();
    assert!(!s.can_undo());
}

#[test]
fn saving_and_removing_are_undoable_and_a_name_replaces() {
    let mut s = open("views-edit");
    let id = s.save_view("Eins").unwrap();
    s.set_layer(Layer::Set, Visibility::Hidden);

    let again = s.save_view("eins").unwrap();

    assert_eq!(again, id);
    assert_eq!(s.saved_views().len(), 1);
    assert!(!s.saved_views()[0].layers.contains(&"set".to_owned()));
    s.remove_view(&id).unwrap();
    assert!(s.saved_views().is_empty());
    s.undo();
    assert_eq!(s.saved_views().len(), 1);
}

#[test]
fn a_view_needs_a_name_and_an_id_that_exists() {
    let mut s = open("views-refused");

    assert!(matches!(
        s.save_view("  "),
        Err(ServiceError::BadValue { .. })
    ));
    assert!(s.apply_view("nope").is_err());
    assert!(s.remove_view("nope").is_err());
    assert!(!s.can_undo());
}

/// Two names that read as one id get an id each.
#[test]
fn ids_are_made_from_the_names() {
    let mut s = open("views-ids");

    let a = s.save_view("Wide Shot").unwrap();
    let b = s.save_view("wide-shot").unwrap();

    assert_eq!(a, "wide-shot");
    assert_ne!(a, b);
}

#[test]
fn views_are_saved_in_the_project() {
    let mut s = open("views-file");
    s.set_track("b", Visibility::Hidden);
    s.save_view("Nur A").unwrap();
    s.save().unwrap();

    let text = std::fs::read_to_string(s.dir().join("projekt.toml")).unwrap();
    let again = Session::open(s.dir()).unwrap();

    assert!(text.contains("[[ansicht]]"), "{text}");
    assert_eq!(again.saved_views(), s.saved_views());
}
