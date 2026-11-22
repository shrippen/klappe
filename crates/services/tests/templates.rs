//! Templates (LH-07), on the example project: Elisabeth and Burgvogt have figures on its plan,
//! the cameras are A and B, camera B follows Burgvogt.
#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};

use klappe_services::{
    Field, Layer, Lock, PlanEdit, Point, ProjectEdit, ServiceError, Session, Source, TemplateEntry,
};

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe")
}

fn open() -> Session {
    Session::open(&example()).unwrap()
}

/// A session on a new scene of the example: an empty plan, no figure, no camera.
fn on_empty_plan() -> Session {
    let mut s = open();

    s.edit_project(ProjectEdit::AddScene {
        number: "13".into(),
        heading: "INT. SAAL".into(),
        location: "saengersaal".into(),
    })
    .unwrap();
    let id = s.project().scenes.last().unwrap().id.clone();
    s.select_scene(&id).unwrap();

    s
}

fn p(x_m: f64, y_m: f64) -> Point {
    Point::new(x_m, y_m)
}

fn near(a: Point, b: Point) -> bool {
    a.distance_m(b) < 1e-9
}

fn near_cm(a: Point, b: Point) -> bool {
    a.distance_m(b) < 0.02
}

/// The middle of the box around every place and bend of the marks.
fn middle_of(marks: &[(Point, Vec<Point>)]) -> Point {
    let points: Vec<Point> = marks
        .iter()
        .flat_map(|(at, via)| std::iter::once(*at).chain(via.iter().copied()))
        .collect();
    let span = |value: fn(&Point) -> f64| {
        let values = points.iter().map(value);
        let low = values.clone().fold(f64::MAX, f64::min);
        let high = values.fold(f64::MIN, f64::max);

        (low + high) / 2.0
    };

    Point::new(span(|q| q.x_m), span(|q| q.y_m))
}

fn characters(s: &Session) -> Vec<&str> {
    s.project()
        .characters
        .iter()
        .map(|c| c.id.as_str())
        .collect()
}

fn labels(s: &Session) -> Vec<&str> {
    let plan = s.plan().unwrap();

    plan.cameras.iter().map(|c| c.label.as_str()).collect()
}

#[test]
fn the_list_offers_the_three_built_in_templates() {
    let entries = open().templates();
    let seen: Vec<_> = entries
        .iter()
        .map(|e| (e.id.as_str(), e.figures, e.cameras, e.custom))
        .collect();

    assert_eq!(
        seen,
        [
            ("shot-reverse", 2, 3, false),
            ("triangle", 2, 3, false),
            ("long-take", 1, 1, false)
        ]
    );
    assert!(entries.iter().all(|e| e.label.is_empty()));
}

/// Both characters of the project stand on the plan already: the placeholders get characters
/// of their own, named by the placeholder, and the cameras take the letters that are free.
#[test]
fn placeholders_without_a_free_character_make_new_ones() {
    let mut s = open();
    let before = s.project().clone();

    s.apply_template("shot-reverse", p(10.0, 5.0)).unwrap();

    assert_eq!(characters(&s), ["elisabeth", "burgvogt", "a", "b"]);
    assert_eq!(s.project().characters[2].name, "A");
    assert_eq!(s.project().characters[2].short, "A");
    assert_ne!(
        s.project().characters[2].color,
        s.project().characters[0].color
    );

    let plan = s.plan().unwrap();
    let first = &plan.figure("a").unwrap().marks[0];
    assert!(near(first.at, p(9.0, 5.0)), "{:?}", first.at);
    assert_eq!(first.facing_deg, 0.0);
    assert!(near(plan.figure("b").unwrap().marks[0].at, p(11.0, 5.0)));

    assert_eq!(labels(&s), ["A", "B", "C", "D", "E"]);
    let d = plan.camera("d").unwrap();
    assert!(near(d.marks[0].at, p(12.4, 5.4)), "{:?}", d.marks[0].at);
    assert_eq!(d.lens_mm, 50.0);

    s.undo();
    assert_eq!(s.project(), &before);
    assert!(!s.can_undo());
}

/// A new scene has no figure yet: the placeholders take the project's characters in order.
#[test]
fn placeholders_take_the_characters_the_plan_has_no_figure_of() {
    let mut s = on_empty_plan();

    s.apply_template("triangle", p(8.0, 4.0)).unwrap();

    assert_eq!(characters(&s), ["elisabeth", "burgvogt"]);
    let plan = s.plan().unwrap();
    assert!(near(
        plan.figure("elisabeth").unwrap().marks[0].at,
        p(7.2, 4.0)
    ));
    assert!(near(
        plan.figure("burgvogt").unwrap().marks[0].at,
        p(8.8, 4.0)
    ));
    assert_eq!(labels(&s), ["A", "B", "C"]);
    assert!(plan.cameras.iter().all(|c| c.follows.is_none()));
}

#[test]
fn a_long_take_camera_follows_the_figure_of_its_placeholder() {
    let mut s = on_empty_plan();

    s.apply_template("long-take", p(2.0, 2.0)).unwrap();

    let plan = s.plan().unwrap();
    let camera = plan.camera("a").unwrap();
    assert_eq!(camera.follows.as_deref(), Some("elisabeth"));
    assert_eq!(camera.marks.len(), 4);
    assert_eq!(plan.figure("elisabeth").unwrap().marks.len(), 4);
    assert!(near(camera.marks[0].at, p(-0.5, 2.0)));
    assert!(near(
        plan.figure("elisabeth").unwrap().marks[3].at,
        p(9.0, 5.0)
    ));
    assert_eq!(plan.figure("elisabeth").unwrap().marks[3].at_s, 7.1);
}

/// The same template twice: the second time no free character is left, the cameras go on.
#[test]
fn a_second_placing_adds_more_without_clashing() {
    let mut s = on_empty_plan();

    s.apply_template("long-take", p(2.0, 2.0)).unwrap();
    s.apply_template("long-take", p(2.0, 8.0)).unwrap();

    assert_eq!(characters(&s), ["elisabeth", "burgvogt"]);
    assert_eq!(labels(&s), ["A", "B"]);
    assert_eq!(
        s.plan().unwrap().camera("b").unwrap().follows.as_deref(),
        Some("burgvogt")
    );
}

/// With every character on the plan, a second placing names its new figures freely: the
/// placeholder's name is taken by the first placing.
#[test]
fn a_full_plan_takes_the_template_twice() {
    let mut s = open();

    s.apply_template("shot-reverse", p(10.0, 5.0)).unwrap();
    s.apply_template("shot-reverse", p(10.0, 5.0)).unwrap();

    let names: Vec<_> = s
        .project()
        .characters
        .iter()
        .map(|c| c.name.as_str())
        .collect();
    assert!(
        names.contains(&"A 2") && names.contains(&"B 2"),
        "{names:?}"
    );
}

#[test]
fn a_locked_layer_refuses_the_template() {
    for layer in [Layer::Figures, Layer::Cameras] {
        let mut s = open();
        let before = s.project().clone();
        s.set_lock(layer, Lock::Locked);

        let result = s.apply_template("shot-reverse", p(10.0, 5.0));

        assert!(matches!(result, Err(ServiceError::Locked(l)) if l == layer));
        assert_eq!(s.project(), &before);
    }

    // The other layers do not matter.
    let mut s = open();
    s.set_lock(Layer::Set, Lock::Locked);
    assert!(s.apply_template("shot-reverse", p(10.0, 5.0)).is_ok());
}

#[test]
fn an_unknown_template_or_point_is_refused() {
    let mut s = open();

    assert!(matches!(
        s.apply_template("nope", p(1.0, 1.0)),
        Err(ServiceError::BadValue {
            field: Field::Kind,
            ..
        })
    ));
    assert!(s.apply_template("triangle", p(f64::NAN, 1.0)).is_err());
    assert!(!s.can_undo());
}

#[test]
fn the_middle_of_the_set_is_where_a_template_goes_by_default() {
    let s = open();
    let set = &s.project().locations[0].set;

    assert_eq!(s.plan_middle(), p(set.width_m / 2.0, set.depth_m / 2.0));
}

fn own(s: &Session) -> Vec<TemplateEntry> {
    s.templates().into_iter().filter(|e| e.custom).collect()
}

/// The plan's figures and cameras are filed around the middle of their marks, camera B keeps
/// following Burgvogt through his placeholder.
#[test]
fn a_plan_is_filed_as_a_template() {
    let mut s = open();

    s.save_template("Mein Dialog").unwrap();

    let entry = &own(&s)[0];
    assert_eq!(
        (entry.id.as_str(), entry.label.as_str()),
        ("mein-dialog", "Mein Dialog")
    );
    assert_eq!((entry.figures, entry.cameras), (2, 2));

    let template = &s.project().templates[0];
    assert_eq!(template.figures[0].slot, "Elisabeth");
    let b = template.cameras.iter().find(|c| c.id == "b").unwrap();
    assert_eq!(b.follows.as_deref(), Some("Burgvogt"));

    let figures = template
        .figures
        .iter()
        .flat_map(|f| f.marks.iter().map(|m| (m.at, m.via.clone())));
    let cameras = template
        .cameras
        .iter()
        .flat_map(|c| c.marks.iter().map(|m| (m.at, m.via.clone())));
    let middle = middle_of(&figures.chain(cameras).collect::<Vec<_>>());
    assert!(near_cm(middle, p(0.0, 0.0)), "{middle:?}");

    s.undo();
    assert!(s.project().templates.is_empty());
}

/// Filed, saved, opened again and placed: the figures bind to the characters by order and the
/// middle of the placed marks is the insertion point.
#[test]
fn an_own_template_survives_saving_and_can_be_placed() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("services-templates");
    let _ = fs::remove_dir_all(&dir);
    copy_dir(&example(), &dir);

    let mut s = Session::open(&dir).unwrap();
    s.save_template("Mein Dialog").unwrap();
    s.save(klappe_services::OnOutside::Overwrite).unwrap();
    assert!(dir.join("vorlagen.toml").exists());

    let mut again = Session::open(&dir).unwrap();
    assert_eq!(again.project().templates, s.project().templates);

    again
        .edit_project(ProjectEdit::AddScene {
            number: "13".into(),
            heading: "INT. SAAL".into(),
            location: "saengersaal".into(),
        })
        .unwrap();
    let id = again.project().scenes.last().unwrap().id.clone();
    again.select_scene(&id).unwrap();
    again.apply_template("mein-dialog", p(10.0, 10.0)).unwrap();

    let plan = again.plan().unwrap();
    assert_eq!(plan.figures.len(), 2);
    assert_eq!(plan.cameras.len(), 2);
    assert_eq!(
        plan.camera("b").unwrap().follows.as_deref(),
        Some("burgvogt")
    );

    let figures = plan
        .figures
        .iter()
        .flat_map(|f| f.marks.iter().map(|m| (m.at, m.via.clone())));
    let cameras = plan
        .cameras
        .iter()
        .flat_map(|c| c.marks.iter().map(|m| (m.at, m.via.clone())));
    let middle = middle_of(&figures.chain(cameras).collect::<Vec<_>>());
    assert!(near_cm(middle, p(10.0, 10.0)), "{middle:?}");
}

#[test]
fn a_template_name_is_free_but_its_id_is_not_taken_twice() {
    let mut s = open();

    s.save_template("Dreieck").unwrap();
    s.save_template("Dreieck").unwrap();
    s.save_template("Shot Reverse").unwrap();

    let ids: Vec<_> = s
        .project()
        .templates
        .iter()
        .map(|t| t.id.as_str())
        .collect();
    assert_eq!(ids, ["dreieck", "dreieck-2", "shot-reverse-2"]);
    // An own template hides the built-in one of its id; here none does.
    assert_eq!(s.templates().len(), 6);
}

#[test]
fn nothing_to_file_and_no_name_are_refused() {
    let mut s = on_empty_plan();

    assert!(matches!(
        s.save_template("Leer"),
        Err(ServiceError::EmptyTemplate)
    ));

    let mut s = open();
    assert!(matches!(
        s.save_template("  "),
        Err(ServiceError::BadValue {
            field: Field::Name,
            ..
        })
    ));
    assert!(s.project().templates.is_empty());
}

/// Rounding to the centimetre must not turn a far place into `inf`: such a plan is not filed,
/// and the project stays saveable.
#[test]
fn a_plan_beyond_the_set_is_not_filed() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("services-templates-far");
    let _ = fs::remove_dir_all(&dir);
    copy_dir(&example(), &dir);

    let mut s = Session::open(&dir).unwrap();
    s.edit_plan(PlanEdit::Move {
        source: Source::Mark {
            character: "burgvogt".into(),
            index: 1,
        },
        to: p(1e307, 0.0),
    })
    .unwrap();
    s.edit_plan(PlanEdit::Move {
        source: Source::Mark {
            character: "burgvogt".into(),
            index: 0,
        },
        to: p(-1e307, 0.0),
    })
    .unwrap();

    assert!(matches!(
        s.save_template("Wieder"),
        Err(ServiceError::BadValue {
            field: Field::Point,
            ..
        })
    ));
    assert!(s.project().templates.is_empty());
    s.save(klappe_services::OnOutside::Overwrite).unwrap();
}

/// A camera on the floor (height 0), which the editor takes, is filed as a template.
#[test]
fn a_camera_on_the_floor_is_filed() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("services-templates-floor");
    let _ = fs::remove_dir_all(&dir);
    copy_dir(&example(), &dir);

    let mut s = Session::open(&dir).unwrap();
    s.edit_plan(PlanEdit::SetCameraAim {
        source: Source::Camera("a".into()),
        height_m: 0.0,
        tilt_deg: 0.0,
    })
    .unwrap();

    s.save_template("Boden").unwrap();
    assert_eq!(s.project().templates.len(), 1);
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

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    dir
}

fn ids(s: &Session) -> Vec<String> {
    own(s).into_iter().map(|e| e.id).collect()
}

fn rename(id: &str, name: &str) -> ProjectEdit {
    ProjectEdit::EditTemplate {
        id: id.into(),
        name: name.into(),
    }
}

fn remove(id: &str) -> ProjectEdit {
    ProjectEdit::RemoveTemplate { id: id.into() }
}

#[test]
fn an_own_template_is_renamed_and_the_rename_undone() {
    let mut s = open();
    s.save_template("Mein Dialog").unwrap();

    s.edit_project(rename("mein-dialog", "  Zwiegespräch "))
        .unwrap();

    let entry = &own(&s)[0];
    assert_eq!(
        (entry.id.as_str(), entry.label.as_str()),
        ("mein-dialog", "Zwiegespräch")
    );

    s.undo();
    assert_eq!(own(&s)[0].label, "Mein Dialog");
}

#[test]
fn a_blank_name_or_an_unknown_template_is_refused_on_rename() {
    let mut s = open();
    s.save_template("Mein Dialog").unwrap();

    assert!(matches!(
        s.edit_project(rename("mein-dialog", "  ")),
        Err(ServiceError::BadValue {
            field: Field::Name,
            ..
        })
    ));
    assert!(matches!(
        s.edit_project(rename("nix", "Nix")),
        Err(ServiceError::Gone)
    ));
    assert_eq!(own(&s)[0].label, "Mein Dialog");
}

#[test]
fn an_own_template_is_deleted_and_the_deletion_undone() {
    let mut s = open();
    s.save_template("Eins").unwrap();
    s.save_template("Zwei").unwrap();

    s.edit_project(remove("eins")).unwrap();
    assert_eq!(ids(&s), ["zwei"]);

    s.undo();
    assert_eq!(ids(&s), ["eins", "zwei"]);
}

#[test]
fn a_built_in_template_cannot_be_renamed_or_deleted() {
    let mut s = open();

    assert!(matches!(
        s.edit_project(rename("triangle", "Mein Dreieck")),
        Err(ServiceError::Gone)
    ));
    assert!(matches!(
        s.edit_project(remove("triangle")),
        Err(ServiceError::Gone)
    ));
    assert_eq!(s.templates().len(), 3);
    assert!(!s.is_dirty());
}

#[test]
fn a_template_file_goes_from_one_project_to_another() {
    let mut first = open();
    first.save_template("Eins").unwrap();
    first.save_template("Zwei").unwrap();
    let file = scratch("templates-pass-on").join("dialoge.toml");

    assert_eq!(first.export_templates(&file).unwrap(), 2);
    let mut second = open();

    assert_eq!(second.import_templates(&file).unwrap(), 2);
    assert_eq!(second.project().templates, first.project().templates);

    // Passed on twice, it adds nothing; one undo step takes the first import back.
    assert_eq!(second.import_templates(&file).unwrap(), 0);
    second.undo();
    assert!(ids(&second).is_empty());
}

#[test]
fn an_imported_template_whose_id_is_taken_gets_another_id() {
    let mut first = open();
    first.save_template("Dreieck").unwrap();
    first.save_template("Triangle").unwrap();
    let file = scratch("templates-clash").join("dialoge.toml");
    first.export_templates(&file).unwrap();

    let mut second = open();
    second.save_template("Dreieck").unwrap();

    assert_eq!(second.import_templates(&file).unwrap(), 1);

    // The first file entry equals the own `dreieck` and is skipped; `triangle` clashes with
    // the built-in one.
    assert_eq!(ids(&second), ["dreieck", "triangle-2"]);
    assert_eq!(second.templates().len(), 5);
}

#[test]
fn there_is_nothing_to_export_without_own_templates() {
    let s = open();
    let file = scratch("templates-nothing").join("leer.toml");

    assert!(matches!(
        s.export_templates(&file),
        Err(ServiceError::EmptyLibrary)
    ));
    assert!(!file.exists());
}

#[test]
fn a_broken_template_file_is_refused_and_changes_nothing() {
    let mut s = open();
    let dir = scratch("templates-broken");

    let missing = dir.join("fehlt.toml");
    assert!(s.import_templates(&missing).is_err());

    let other = dir.join("projekt.toml");
    fs::write(&other, "title = \"Projekt\"\n").unwrap();
    assert!(s.import_templates(&other).is_err());

    assert!(s.project().templates.is_empty());
    assert!(!s.is_dirty());
}
