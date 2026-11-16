//! The object library (LH-11, LH-12) and resizing objects (LH-25), on the example project:
//! its set has the table `tisch` (6 x 1.2 m at (10, 6.5)) and two pillars of 0.8 m.
#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};

use klappe_model::{LibraryItem, SetObject};
use klappe_services::{
    Field, Language, Layer, Lock, ObjectShape, PlanEdit, Point, ProjectEdit, Role, ServiceError,
    Session, Shape, Source,
};

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe")
}

fn open() -> Session {
    Session::open(&example()).unwrap()
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    dir
}

/// A copy of the example to save into.
fn copy(name: &str) -> PathBuf {
    let dir = scratch(name);
    copy_dir(&example(), &dir);

    dir
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
            continue;
        }
        fs::copy(entry.path(), target).unwrap();
    }
}

fn p(x_m: f64, y_m: f64) -> Point {
    Point::new(x_m, y_m)
}

fn objects(s: &Session) -> &[SetObject] {
    &s.project().locations[0].set.objects
}

fn place(s: &mut Session, kind: &str) -> SetObject {
    s.edit_plan(PlanEdit::AddObject {
        kind: kind.into(),
        at: p(2.0, 2.0),
    })
    .unwrap();

    objects(s).last().unwrap().clone()
}

/// The corners of a polygon; none for any other shape, which the tests then miss.
fn corners(shape: &Shape) -> &[Point] {
    match shape {
        Shape::Polygon { points } => points,
        _ => &[],
    }
}

fn entry(s: &Session, kind: &str) -> klappe_services::LibraryEntry {
    s.library().into_iter().find(|e| e.kind == kind).unwrap()
}

fn resize(s: &mut Session, id: &str, width_m: f64, depth_m: f64) -> Result<(), ServiceError> {
    s.edit_plan(PlanEdit::Resize {
        source: Source::Object(id.into()),
        width_m,
        depth_m,
    })
}

fn throne() -> LibraryItem {
    LibraryItem {
        kind: "thron".into(),
        group: String::new(),
        label: "Thron".into(),
        width_m: 0.9,
        depth_m: 0.8,
        shape: ObjectShape::Rect,
        height_m: None,
        form: None,
    }
}

fn pond() -> LibraryItem {
    LibraryItem {
        kind: "teich".into(),
        label: "Teich".into(),
        width_m: 6.0,
        depth_m: 4.0,
        shape: ObjectShape::Ellipse,
        ..throne()
    }
}

fn import(s: &mut Session, items: Vec<LibraryItem>) -> Result<(), ServiceError> {
    s.edit_project(ProjectEdit::ImportLibrary { items })
}

fn own_kinds(s: &Session) -> Vec<&str> {
    s.project()
        .library
        .iter()
        .map(|i| i.kind.as_str())
        .collect()
}

#[test]
fn a_new_object_has_the_size_and_shape_of_its_library_entry() {
    let mut s = open();

    let chair = place(&mut s, "chair");
    assert_eq!((chair.width_m, chair.depth_m), (0.45, 0.45));
    assert_eq!(chair.shape, ObjectShape::Rect);

    let horse = place(&mut s, "horse");
    assert_eq!((horse.width_m, horse.depth_m), (2.4, 0.7));
    assert_eq!(horse.shape, ObjectShape::Ellipse);

    let car = place(&mut s, "car");
    assert_eq!((car.width_m, car.depth_m), (4.5, 1.8));
    assert!(car.label.is_empty());
}

/// LH-35: camera support is placed with its footprint: a track at gauge, a crane as the circle
/// its arm sweeps.
#[test]
fn camera_support_is_placed_with_its_footprint() {
    let mut s = open();

    let track = place(&mut s, "track");
    assert_eq!((track.width_m, track.depth_m), (4.0, 0.62));
    assert_eq!(track.shape, ObjectShape::Rect);

    let crane = place(&mut s, "crane");
    assert_eq!((crane.width_m, crane.depth_m), (12.0, 12.0));
    assert_eq!(crane.shape, ObjectShape::Ellipse);

    let jib = place(&mut s, "jib");
    assert_eq!((jib.width_m, jib.depth_m), (6.0, 6.0));

    let dolly = place(&mut s, "dolly");
    assert_eq!((dolly.width_m, dolly.depth_m), (1.2, 0.8));

    let tripod = place(&mut s, "tripod");
    assert_eq!((tripod.width_m, tripod.depth_m), (1.0, 1.0));
    assert_eq!(tripod.shape, ObjectShape::Ellipse);
}

/// An old project may name a kind no library has; placing one still works.
#[test]
fn an_unknown_kind_is_a_one_metre_square() {
    let mut s = open();

    let zeppelin = place(&mut s, "zeppelin");

    assert_eq!((zeppelin.width_m, zeppelin.depth_m), (1.0, 1.0));
    assert_eq!(zeppelin.shape, ObjectShape::Rect);
}

#[test]
fn an_own_object_brings_its_size_shape_and_name() {
    let mut s = open();
    import(&mut s, vec![pond()]).unwrap();

    let placed = place(&mut s, "teich");

    assert_eq!((placed.width_m, placed.depth_m), (6.0, 4.0));
    assert_eq!(placed.shape, ObjectShape::Ellipse);
    assert_eq!(placed.label, "Teich");
}

#[test]
fn the_list_has_the_built_in_entries_then_the_own_ones() {
    let mut s = open();
    import(&mut s, vec![throne()]).unwrap();

    let list = s.library();
    let built_in = list.iter().filter(|e| !e.custom).count();

    assert!(built_in >= 14);
    assert!(list[..built_in]
        .iter()
        .all(|e| !e.custom && e.label.is_empty()));
    assert_eq!(list[built_in].kind, "thron");
    assert_eq!(list[built_in].label, "Thron");
    assert!(list[built_in].custom);
    assert_eq!(list.len(), built_in + 1);
}

/// Every built-in entry is named in both languages.
#[test]
fn every_built_in_entry_has_a_name_in_both_languages() {
    for entry in open().library() {
        let key = format!("objectKind.{}", entry.kind);
        for language in [Language::German, Language::English] {
            assert_ne!(language.text(&key), key, "{key}");
        }
    }
}

#[test]
fn an_ellipse_object_is_drawn_as_an_ellipse() {
    let mut s = open();
    let tree = place(&mut s, "tree");

    let list = s.display_list();
    let item = list
        .items
        .iter()
        .find(|i| i.source == Some(Source::Object(tree.id.clone())))
        .unwrap();

    assert_eq!(item.role, Role::Object);
    let points = corners(&item.shape);
    assert!(points.len() > 8);
    // 3 m wide at (2, 2): the rightmost corner is at x = 3.5.
    let right = points.iter().map(|c| c.x_m).fold(f64::MIN, f64::max);
    assert!((right - 3.5).abs() < 1e-9);
}

#[test]
fn a_placed_object_can_be_filed_in_the_library() {
    let mut s = open();

    s.edit_project(ProjectEdit::LibraryFromObject {
        location: "saengersaal".into(),
        object: "tisch".into(),
        name: "  Lange Tafel ".into(),
    })
    .unwrap();

    assert_eq!(
        s.project().library,
        vec![LibraryItem {
            kind: "lange-tafel".into(),
            group: String::new(),
            label: "Lange Tafel".into(),
            width_m: 6.0,
            depth_m: 1.2,
            shape: ObjectShape::Rect,
            height_m: None,
            form: None,
        }]
    );
    assert!(s.is_dirty());
    assert!(entry(&s, "lange-tafel").custom);

    s.undo();
    assert!(s.project().library.is_empty());
}

#[test]
fn a_filed_object_keeps_its_current_size_and_shape() {
    let mut s = open();
    let tree = place(&mut s, "tree");
    resize(&mut s, &tree.id, 5.0, 4.0).unwrap();

    s.edit_project(ProjectEdit::LibraryFromObject {
        location: "saengersaal".into(),
        object: tree.id,
        name: "Alte Eiche".into(),
    })
    .unwrap();

    let item = &s.project().library[0];
    assert_eq!((item.width_m, item.depth_m), (5.0, 4.0));
    assert_eq!(item.shape, ObjectShape::Ellipse);
}

/// Kinds are unique over the built-in and the own entries: `Table` would be `table` again.
#[test]
fn a_filed_object_gets_a_kind_of_its_own() {
    let mut s = open();
    let mut file = |name: &str| {
        s.edit_project(ProjectEdit::LibraryFromObject {
            location: "saengersaal".into(),
            object: "tisch".into(),
            name: name.into(),
        })
        .unwrap();
    };

    file("Table");
    file("Tisch");
    file("Tisch");

    assert_eq!(own_kinds(&s), ["table-2", "tisch", "tisch-2"]);
}

#[test]
fn filing_needs_a_name_and_an_object() {
    let mut s = open();
    let mut file = |location: &str, object: &str, name: &str| {
        s.edit_project(ProjectEdit::LibraryFromObject {
            location: location.into(),
            object: object.into(),
            name: name.into(),
        })
    };

    assert!(matches!(
        file("saengersaal", "tisch", "   "),
        Err(ServiceError::BadValue {
            field: Field::Name,
            ..
        })
    ));
    assert!(matches!(
        file("saengersaal", "o99", "X"),
        Err(ServiceError::Gone)
    ));
    assert!(matches!(
        file("nirgends", "tisch", "X"),
        Err(ServiceError::NoLocation(_))
    ));
    assert!(s.project().library.is_empty());
    assert!(!s.is_dirty());
}

#[test]
fn an_import_adds_new_entries_once() {
    let mut s = open();

    import(&mut s, vec![throne(), pond()]).unwrap();
    assert_eq!(own_kinds(&s), ["thron", "teich"]);

    // The same file again adds nothing and costs no undo step.
    import(&mut s, vec![throne(), pond()]).unwrap();
    assert_eq!(own_kinds(&s), ["thron", "teich"]);
    s.undo();
    assert!(s.project().library.is_empty());
}

#[test]
fn an_import_does_not_overwrite_what_has_the_kind() {
    let mut s = open();
    import(&mut s, vec![throne()]).unwrap();
    let bigger = LibraryItem {
        width_m: 1.2,
        ..throne()
    };
    let table = LibraryItem {
        kind: "table".into(),
        label: "Mein Tisch".into(),
        width_m: 3.0,
        depth_m: 1.0,
        ..throne()
    };

    import(&mut s, vec![bigger, table]).unwrap();

    assert_eq!(own_kinds(&s), ["thron", "thron-2", "table-2"]);
    assert_eq!(entry(&s, "thron").width_m, 0.9);
    assert_eq!(entry(&s, "thron-2").width_m, 1.2);
    assert_eq!(entry(&s, "table").width_m, 1.6);
    assert_eq!(entry(&s, "table-2").label, "Mein Tisch");
}

#[test]
fn an_import_without_names_uses_the_kind() {
    let mut s = open();
    let nameless = LibraryItem {
        label: String::new(),
        ..throne()
    };

    import(&mut s, vec![nameless]).unwrap();

    assert_eq!(s.project().library[0].label, "thron");
}

#[test]
fn an_import_with_an_unusable_entry_changes_nothing() {
    let mut s = open();
    let flat = LibraryItem {
        depth_m: 0.0,
        ..pond()
    };
    let nan = LibraryItem {
        width_m: f64::NAN,
        ..pond()
    };
    let blank = LibraryItem {
        kind: " ".into(),
        group: String::new(),
        ..pond()
    };

    for bad in [flat, nan, blank] {
        assert!(import(&mut s, vec![throne(), bad]).is_err());
        assert!(s.project().library.is_empty());
    }
    assert!(!s.is_dirty());
}

#[test]
fn a_library_file_goes_from_one_project_to_another() {
    let mut first = open();
    import(&mut first, vec![throne(), pond()]).unwrap();
    let file = scratch("library-pass-on").join("mittelalter.toml");

    assert_eq!(first.export_library(&file).unwrap(), 2);
    let mut second = open();

    assert_eq!(second.import_library(&file).unwrap(), 2);
    assert_eq!(second.project().library, first.project().library);
    assert_eq!(second.import_library(&file).unwrap(), 0);
}

#[test]
fn there_is_nothing_to_export_without_own_objects() {
    let s = open();
    let file = scratch("library-nothing").join("leer.toml");

    assert!(matches!(
        s.export_library(&file),
        Err(ServiceError::EmptyLibrary)
    ));
    assert!(!file.exists());
}

#[test]
fn a_broken_library_file_is_refused_and_changes_nothing() {
    let mut s = open();
    let dir = scratch("library-broken");
    let files = [
        ("kein-toml.toml", "[[objekt]\nkind"),
        (
            "zu-klein.toml",
            "[[objekt]]\nkind = \"x\"\nwidth_m = 0.0\ndepth_m = 1.0\n",
        ),
        ("fehlt.toml", ""),
    ];

    for (name, text) in files {
        let path = dir.join(name);
        if name != "fehlt.toml" {
            fs::write(&path, text).unwrap();
        }

        let err = s.import_library(&path).unwrap_err();

        assert!(matches!(err, ServiceError::Store(_)), "{name}");
        assert!(err.to_string().contains(name), "{err}");
    }
    assert!(s.project().library.is_empty());
    assert!(!s.is_dirty());
}

#[test]
fn own_objects_and_placed_shapes_survive_a_save() {
    let dir = copy("library-save");
    let mut s = Session::open(&dir).unwrap();
    import(&mut s, vec![pond()]).unwrap();
    place(&mut s, "teich");
    s.save().unwrap();

    let again = Session::open(&dir).unwrap();

    assert_eq!(again.project().library, vec![pond()]);
    let placed = objects(&again).last().unwrap();
    assert_eq!(
        (placed.label.as_str(), placed.shape.clone()),
        ("Teich", ObjectShape::Ellipse)
    );
    assert_eq!(entry(&again, "teich").width_m, 6.0);
}

#[test]
fn an_object_is_resized_and_undone() {
    let mut s = open();

    resize(&mut s, "tisch", 2.0, 0.5).unwrap();
    let table = |s: &Session| (objects(s)[0].width_m, objects(s)[0].depth_m);

    assert_eq!(table(&s), (2.0, 0.5));
    assert!(s.is_dirty());
    s.undo();
    assert_eq!(table(&s), (6.0, 1.2));
    s.redo();
    assert_eq!(table(&s), (2.0, 0.5));
}

/// The table at (10, 6.5) shrunk to 2 x 0.5 m reaches from x = 9 to 11.
#[test]
fn a_resize_shows_in_the_display_list_and_in_the_preview() {
    let mut s = open();
    let edit = PlanEdit::Resize {
        source: Source::Object("tisch".into()),
        width_m: 2.0,
        depth_m: 0.5,
    };
    let span = |list: klappe_services::DisplayList| {
        let item = list
            .items
            .into_iter()
            .find(|i| i.source == Some(Source::Object("tisch".into())))
            .unwrap();
        let xs: Vec<f64> = corners(&item.shape).iter().map(|c| c.x_m).collect();
        (
            xs.iter().cloned().fold(f64::MAX, f64::min),
            xs.iter().cloned().fold(f64::MIN, f64::max),
        )
    };

    assert_eq!(span(s.preview(&edit).unwrap()), (9.0, 11.0));
    assert_eq!(span(s.display_list()), (7.0, 13.0));
    assert!(!s.is_dirty());

    s.edit_plan(edit).unwrap();
    assert_eq!(span(s.display_list()), (9.0, 11.0));
}

#[test]
fn a_size_must_be_a_usable_number() {
    let mut s = open();

    for (w, d) in [
        (0.0, 1.0),
        (1.0, -1.0),
        (f64::NAN, 1.0),
        (1.0, f64::INFINITY),
        (5000.0, 1.0),
        (0.04, 1.0),
    ] {
        let err = resize(&mut s, "tisch", w, d).unwrap_err();
        assert!(matches!(
            err,
            ServiceError::BadValue {
                field: Field::Size,
                ..
            }
        ));
    }
    assert_eq!((objects(&s)[0].width_m, objects(&s)[0].depth_m), (6.0, 1.2));
    assert!(!s.is_dirty());
}

#[test]
fn only_objects_have_a_size_to_change() {
    let mut s = open();

    for source in [
        Source::Wall(0),
        Source::Camera("a".into()),
        Source::Figure("elisabeth".into()),
    ] {
        let err = s
            .edit_plan(PlanEdit::Resize {
                source,
                width_m: 2.0,
                depth_m: 2.0,
            })
            .unwrap_err();
        assert!(matches!(
            err,
            ServiceError::BadValue {
                field: Field::Source,
                ..
            }
        ));
    }
    assert!(matches!(
        resize(&mut s, "o99", 2.0, 2.0),
        Err(ServiceError::Gone)
    ));
}

#[test]
fn a_locked_set_refuses_a_resize_and_the_other_layers_do_not() {
    let mut s = open();
    s.set_lock(Layer::Set, Lock::Locked);

    assert!(matches!(
        resize(&mut s, "tisch", 2.0, 2.0),
        Err(ServiceError::Locked(Layer::Set))
    ));
    assert_eq!(objects(&s)[0].width_m, 6.0);

    // Filing in the library changes no set: it works behind the lock.
    s.edit_project(ProjectEdit::LibraryFromObject {
        location: "saengersaal".into(),
        object: "tisch".into(),
        name: "Tafel".into(),
    })
    .unwrap();
    assert_eq!(s.project().library.len(), 1);
}

/// Review of PR 28: an object with a size no library takes (here a hand-edited depth of 0)
/// went into the library and then every save and autosave failed on `bibliothek.toml`.
#[test]
fn an_unusable_object_does_not_go_into_the_library() {
    let dir = copy("unusable-object");
    let file = dir.join("drehorte/saengersaal.toml");
    let text = fs::read_to_string(&file).unwrap();
    fs::write(&file, text.replacen("depth_m = 1.2", "depth_m = 0.0", 1)).unwrap();
    let mut s = Session::open(&dir).unwrap();

    let laid = s.edit_project(ProjectEdit::LibraryFromObject {
        location: "saengersaal".into(),
        object: "tisch".into(),
        name: "Tisch flach".into(),
    });

    assert!(
        matches!(laid, Err(ServiceError::BadValue { .. })),
        "{laid:?}"
    );
    assert!(s.project().library.is_empty());
    assert!(s.save().is_ok());
}

fn file_throne(s: &mut Session) {
    import(s, vec![throne()]).unwrap();
}

fn rename(kind: &str, name: &str, group: &str) -> ProjectEdit {
    ProjectEdit::EditLibraryItem {
        kind: kind.into(),
        name: name.into(),
        group: group.into(),
    }
}

/// LH-12: built-in entries have a group, own ones start without.
#[test]
fn the_list_carries_the_groups() {
    let mut s = open();
    file_throne(&mut s);

    assert_eq!(entry(&s, "table").group, "furniture");
    assert_eq!(entry(&s, "thron").group, "");
}

#[test]
fn an_own_object_is_renamed_and_grouped_in_one_step() {
    let mut s = open();
    file_throne(&mut s);

    s.edit_project(rename("thron", "  Königsthron ", " Requisiten "))
        .unwrap();

    let item = &s.project().library[0];
    assert_eq!(
        (item.label.as_str(), item.group.as_str()),
        ("Königsthron", "Requisiten")
    );
    assert_eq!(item.kind, "thron");
    assert_eq!(entry(&s, "thron").group, "Requisiten");

    s.undo();
    assert_eq!(s.project().library, vec![throne()]);
}

#[test]
fn an_empty_group_takes_an_object_out_of_its_group() {
    let mut s = open();
    file_throne(&mut s);
    s.edit_project(rename("thron", "Thron", "Requisiten"))
        .unwrap();

    s.edit_project(rename("thron", "Thron", "  ")).unwrap();

    assert_eq!(entry(&s, "thron").group, "");
}

#[test]
fn a_rename_needs_a_name_and_an_own_object() {
    let mut s = open();
    file_throne(&mut s);

    assert!(matches!(
        s.edit_project(rename("thron", "  ", "")),
        Err(ServiceError::BadValue {
            field: Field::Name,
            ..
        })
    ));
    // A built-in entry is not the project's to change, nor is one that is not there.
    assert!(matches!(
        s.edit_project(rename("table", "Tafel", "")),
        Err(ServiceError::Gone)
    ));
    assert!(matches!(
        s.edit_project(rename("nix", "Nix", "")),
        Err(ServiceError::Gone)
    ));
    assert_eq!(s.project().library, vec![throne()]);
}

#[test]
fn an_own_object_is_deleted_and_the_deletion_undone() {
    let mut s = open();
    import(&mut s, vec![throne(), pond()]).unwrap();

    s.edit_project(ProjectEdit::RemoveLibraryItem {
        kind: "thron".into(),
    })
    .unwrap();

    assert_eq!(own_kinds(&s), vec!["teich"]);
    assert!(s.library().iter().all(|e| e.kind != "thron"));

    s.undo();
    assert_eq!(own_kinds(&s), vec!["thron", "teich"]);
}

#[test]
fn a_built_in_object_cannot_be_deleted() {
    let mut s = open();
    file_throne(&mut s);

    let result = s.edit_project(ProjectEdit::RemoveLibraryItem {
        kind: "table".into(),
    });

    assert!(matches!(result, Err(ServiceError::Gone)));
    assert_eq!(entry(&s, "table").uses, 1);
}

/// What the plan uses is counted for the question before a delete.
#[test]
fn an_entry_counts_the_objects_placed_from_it() {
    let mut s = open();
    file_throne(&mut s);
    assert_eq!(entry(&s, "thron").uses, 0);

    place(&mut s, "thron");
    place(&mut s, "thron");

    assert_eq!(entry(&s, "thron").uses, 2);
    assert_eq!(entry(&s, "table").uses, 1);
}

/// Placed objects keep size, shape and name when their entry goes or is renamed; a deleted
/// kind is then like an unknown one: a new object of it is a one metre square.
#[test]
fn placed_objects_survive_their_entry() {
    let mut s = open();
    file_throne(&mut s);
    let placed = place(&mut s, "thron");

    s.edit_project(rename("thron", "Sessel", "")).unwrap();
    assert_eq!(objects(&s).last().unwrap().label, "Thron");

    s.edit_project(ProjectEdit::RemoveLibraryItem {
        kind: "thron".into(),
    })
    .unwrap();

    let kept = objects(&s).last().unwrap();
    assert_eq!(kept, &placed);
    let again = place(&mut s, "thron");
    assert_eq!((again.width_m, again.depth_m), (1.0, 1.0));
}

#[test]
fn the_group_goes_into_a_library_file_and_back() {
    let mut s = open();
    let dir = scratch("library-group-file");
    let path = dir.join("bibliothek.toml");
    file_throne(&mut s);
    s.edit_project(rename("thron", "Thron", "Requisiten"))
        .unwrap();
    s.export_library(&path).unwrap();

    let mut other = open();
    other.import_library(&path).unwrap();

    assert_eq!(entry(&other, "thron").group, "Requisiten");
}

#[test]
fn every_built_in_group_has_a_name_in_both_languages() {
    for entry in open().library() {
        let key = format!("objectGroup.{}", entry.group);
        for language in [Language::German, Language::English] {
            assert_ne!(language.text(&key), key, "{key}");
        }
    }
}
