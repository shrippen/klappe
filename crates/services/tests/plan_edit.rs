//! Editing the plan of the example project (`testdata/beispiel.klappe`), numbers worked out by
//! hand from `szenen/szene-012.json` and `drehorte/saengersaal.toml`.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::{
    Layer, LightKind, PlanEdit, Point, ServiceError, Session, Snapping, Source, Visibility,
};

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe")
}

fn open() -> Session {
    Session::open(&example()).unwrap()
}

fn copy(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    copy_dir(&example(), &dir);
    dir
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

fn p(x_m: f64, y_m: f64) -> Point {
    Point::new(x_m, y_m)
}

fn mark(character: &str, index: usize) -> Source {
    Source::Mark {
        character: character.into(),
        index,
    }
}

fn near(a: Point, b: Point) -> bool {
    a.distance_m(b) < 1e-9
}

fn location_set(s: &Session) -> &klappe_model::SetPlan {
    &s.project().locations[0].set
}

#[test]
fn moves_a_mark_and_undoes() {
    let mut s = open();

    s.edit_plan(PlanEdit::Move {
        source: mark("elisabeth", 1),
        to: p(11.0, 7.0),
    })
    .unwrap();
    let at = |s: &Session| s.plan().unwrap().figures[0].marks[1].at;

    assert_eq!(at(&s), p(11.0, 7.0));
    assert!(s.is_dirty());

    s.undo();
    assert_eq!(at(&s), p(10.0, 6.5));
    s.redo();
    assert_eq!(at(&s), p(11.0, 7.0));
}

/// Elisabeth's marks begin at 0, 6 and 14 s; at 9 s she has left the second.
#[test]
fn a_figure_moves_the_mark_it_stands_at() {
    let mut s = open();
    s.set_time(9.0);

    s.edit_plan(PlanEdit::Move {
        source: Source::Figure("elisabeth".into()),
        to: p(1.0, 2.0),
    })
    .unwrap();

    let marks = &s.plan().unwrap().figures[0].marks;
    assert_eq!(marks[0].at, p(3.0, 6.5));
    assert_eq!(marks[1].at, p(1.0, 2.0));
    assert_eq!(marks[2].at, p(16.0, 3.0));
}

/// Camera A has marks at 0 and 6 s; at 7 s the second counts, at 3 s the first.
#[test]
fn a_camera_moves_the_mark_it_stands_at() {
    let mut s = open();
    let camera_a = |s: &Session| s.plan().unwrap().cameras[0].marks.clone();

    s.set_time(3.0);
    s.edit_plan(PlanEdit::Move {
        source: Source::Camera("a".into()),
        to: p(1.0, 1.0),
    })
    .unwrap();
    assert_eq!(camera_a(&s)[0].at, p(1.0, 1.0));

    s.set_time(7.0);
    s.edit_plan(PlanEdit::Move {
        source: Source::Camera("a".into()),
        to: p(2.0, 2.0),
    })
    .unwrap();
    assert_eq!(camera_a(&s)[1].at, p(2.0, 2.0));
    assert_eq!(camera_a(&s)[0].at, p(1.0, 1.0));
}

#[test]
fn moves_object_light_and_wall() {
    let mut s = open();

    s.edit_plan(PlanEdit::Move {
        source: Source::Object("tisch".into()),
        to: p(11.0, 7.0),
    })
    .unwrap();
    s.edit_plan(PlanEdit::Move {
        source: Source::Light("led-1".into()),
        to: p(2.0, 3.0),
    })
    .unwrap();
    // Wall 0 runs (0, 0) to (20, 0), middle (10, 0): to (10, 2) it shifts by 2 down.
    s.edit_plan(PlanEdit::Move {
        source: Source::Wall(0),
        to: p(10.0, 2.0),
    })
    .unwrap();

    assert_eq!(location_set(&s).objects[0].at, p(11.0, 7.0));
    assert_eq!(s.plan().unwrap().lights[0].at, p(2.0, 3.0));
    assert_eq!(location_set(&s).walls[0].from, p(0.0, 2.0));
    assert_eq!(location_set(&s).walls[0].to, p(20.0, 2.0));
}

#[test]
fn turns_are_kept_within_a_full_turn() {
    let mut s = open();
    let facing = |s: &Session| s.plan().unwrap().figures[0].marks[0].facing_deg;
    let turn = |s: &mut Session, deg: f64| {
        s.edit_plan(PlanEdit::Turn {
            source: mark("elisabeth", 0),
            facing_deg: deg,
        })
        .unwrap();
    };

    turn(&mut s, -90.0);
    assert_eq!(facing(&s), 270.0);
    turn(&mut s, 450.0);
    assert_eq!(facing(&s), 90.0);
    turn(&mut s, 360.0);
    assert_eq!(facing(&s), 0.0);
}

#[test]
fn turns_object_light_and_wall() {
    let mut s = open();

    s.edit_plan(PlanEdit::Turn {
        source: Source::Object("tisch".into()),
        facing_deg: 30.0,
    })
    .unwrap();
    s.edit_plan(PlanEdit::Turn {
        source: Source::Light("led-1".into()),
        facing_deg: 45.0,
    })
    .unwrap();
    // Wall 1 runs (20, 0) to (20, 13): middle (20, 6.5), half length 6.5. Turned to 0° it
    // lies along x: (13.5, 6.5) to (26.5, 6.5).
    s.edit_plan(PlanEdit::Turn {
        source: Source::Wall(1),
        facing_deg: 0.0,
    })
    .unwrap();

    assert_eq!(location_set(&s).objects[0].rotation_deg, 30.0);
    assert_eq!(s.plan().unwrap().lights[0].facing_deg, 45.0);
    assert!(near(location_set(&s).walls[1].from, p(13.5, 6.5)));
    assert!(near(location_set(&s).walls[1].to, p(26.5, 6.5)));
}

/// Links: element 0 -> elisabeth 0, element 3 -> burgvogt 1.
#[test]
fn removing_a_mark_fixes_the_links() {
    let mut s = open();
    let links = |s: &Session| {
        s.plan()
            .unwrap()
            .links
            .iter()
            .map(|l| format!("{:?}", l.target))
            .collect::<Vec<_>>()
    };

    s.edit_plan(PlanEdit::Remove {
        source: mark("elisabeth", 0),
    })
    .unwrap();
    assert_eq!(s.plan().unwrap().figures[0].marks.len(), 2);
    assert_eq!(links(&s).len(), 2);
    assert!(links(&s)[1].contains("burgvogt"));

    // Burgvogt's mark 0 goes: the link to mark 1 now names mark 0.
    s.edit_plan(PlanEdit::Remove {
        source: mark("burgvogt", 0),
    })
    .unwrap();
    assert!(links(&s)[1].contains("index: 0"), "{:?}", links(&s));

    s.undo();
    assert!(links(&s)[1].contains("index: 1"));
}

#[test]
fn the_last_mark_takes_the_track_with_it() {
    let mut s = open();

    for _ in 0..2 {
        s.edit_plan(PlanEdit::Remove {
            source: mark("burgvogt", 0),
        })
        .unwrap();
    }

    let plan = s.plan().unwrap();
    assert!(plan.figure("burgvogt").is_none());
    assert_eq!(plan.axis, None);
    assert_eq!(plan.cameras[1].follows, None);
    assert!(plan
        .links
        .iter()
        .all(|l| !format!("{l:?}").contains("burgvogt")));
}

#[test]
fn removing_a_figure_clears_what_names_it() {
    let mut s = open();

    s.edit_plan(PlanEdit::Remove {
        source: Source::Figure("elisabeth".into()),
    })
    .unwrap();

    let plan = s.plan().unwrap();
    assert_eq!(plan.figures.len(), 1);
    assert_eq!(plan.axis, None);
    // Only the link to burgvogt's mark and the shot link stay.
    assert_eq!(plan.links.len(), 2);
}

#[test]
fn a_camera_in_use_stays() {
    let mut s = open();

    let err = s
        .edit_plan(PlanEdit::Remove {
            source: Source::Camera("b".into()),
        })
        .unwrap_err();

    // Review of PR 10: what the user reads is the camera's label and the shot's number.
    assert!(matches!(
        err,
        ServiceError::CameraInUse { ref camera, ref shot } if camera == "B" && shot == "12.3"
    ));
    assert_eq!(s.plan().unwrap().cameras.len(), 2);
    assert!(!s.can_undo());
}

#[test]
fn a_new_camera_can_go_again() {
    let mut s = open();
    s.edit_plan(PlanEdit::AddCamera {
        at: p(1.0, 1.0),
        facing_deg: 0.0,
    })
    .unwrap();

    s.edit_plan(PlanEdit::Remove {
        source: Source::Camera("c".into()),
    })
    .unwrap();

    assert_eq!(s.plan().unwrap().cameras.len(), 2);
}

#[test]
fn removes_object_light_and_wall() {
    let mut s = open();

    for source in [
        Source::Object("saeule-1".into()),
        Source::Light("led-1".into()),
        Source::Wall(0),
    ] {
        s.edit_plan(PlanEdit::Remove { source }).unwrap();
    }

    assert_eq!(location_set(&s).objects.len(), 2);
    assert_eq!(location_set(&s).walls.len(), 3);
    assert!(s.plan().unwrap().lights.is_empty());
}

/// Elisabeth's last mark: (16, 3), arrives at 14 s, no pause. A mark at (16, 7.2) is 4.2 m
/// away: 4.2 / 1.4 = 3 s, so 17 s, facing 90° (down).
#[test]
fn a_new_mark_arrives_after_the_walk() {
    let mut s = open();

    s.edit_plan(PlanEdit::AddMark {
        character: "elisabeth".into(),
        at: p(16.0, 7.2),
    })
    .unwrap();

    let marks = &s.plan().unwrap().figures[0].marks;
    assert_eq!(marks.len(), 4);
    assert!((marks[3].at_s - 17.0).abs() < 1e-9, "{}", marks[3].at_s);
    assert!((marks[3].facing_deg - 90.0).abs() < 1e-9);
}

/// Burgvogt's mark 0 (17, 9.5) arrives at 0 s and stays 5 s; 2.8 m up to (17, 6.7) take 2 s:
/// 0 + 5 + 2 = 7 s, facing 270° (up).
#[test]
fn a_new_mark_waits_for_the_pause() {
    let mut s = open();
    s.edit_plan(PlanEdit::Remove {
        source: mark("burgvogt", 1),
    })
    .unwrap();

    s.edit_plan(PlanEdit::AddMark {
        character: "burgvogt".into(),
        at: p(17.0, 6.7),
    })
    .unwrap();

    let marks = &s.plan().unwrap().figures[1].marks;
    assert!((marks[1].at_s - 7.0).abs() < 1e-9, "{}", marks[1].at_s);
    assert!((marks[1].facing_deg - 270.0).abs() < 1e-9);
}

#[test]
fn a_first_mark_starts_at_the_playhead() {
    let mut s = open();
    s.edit_plan(PlanEdit::Remove {
        source: Source::Figure("burgvogt".into()),
    })
    .unwrap();
    s.set_time(5.0);

    s.edit_plan(PlanEdit::AddMark {
        character: "burgvogt".into(),
        at: p(1.0, 1.0),
    })
    .unwrap();

    let track = s.plan().unwrap().figure("burgvogt").unwrap();
    assert_eq!(track.marks.len(), 1);
    assert_eq!(track.marks[0].at_s, 5.0);
    assert_eq!(track.marks[0].facing_deg, 0.0);
}

#[test]
fn a_mark_needs_a_known_character() {
    let mut s = open();

    let err = s
        .edit_plan(PlanEdit::AddMark {
            character: "nobody".into(),
            at: p(1.0, 1.0),
        })
        .unwrap_err();

    assert!(matches!(
        err,
        ServiceError::BadValue {
            field: "character",
            ..
        }
    ));
    assert!(!s.can_undo());
}

/// Cameras A and B exist, so the new one is C, with A's lens (35 mm) and its mark at the
/// playhead, facing -30° = 330°.
#[test]
fn a_new_camera_takes_the_next_letter_and_the_first_cameras_lens() {
    let mut s = open();
    s.set_time(7.0);

    s.edit_plan(PlanEdit::AddCamera {
        at: p(4.0, 5.0),
        facing_deg: -30.0,
    })
    .unwrap();

    let camera = &s.plan().unwrap().cameras[2];
    assert_eq!((camera.id.as_str(), camera.label.as_str()), ("c", "C"));
    assert_eq!(camera.lens_mm, 35.0);
    assert_eq!(camera.aspect, 2.39);
    assert_eq!(camera.marks.len(), 1);
    assert_eq!(camera.marks[0].at_s, 7.0);
    assert_eq!(camera.marks[0].at, p(4.0, 5.0));
    assert_eq!(camera.marks[0].facing_deg, 330.0);
    assert_eq!(camera.marks[0].height_m, 1.6);
}

#[test]
fn a_new_light_has_the_defaults_of_its_kind() {
    let mut s = open();

    s.edit_plan(PlanEdit::AddLight {
        kind: LightKind::Fresnel,
        at: p(2.0, 2.0),
    })
    .unwrap();

    let light = &s.plan().unwrap().lights[1];
    assert_eq!((light.id.as_str(), light.label.as_str()), ("l1", "L1"));
    assert_eq!(light.beam_deg, 30.0);
    assert_eq!(light.color_temp_k, 3200.0);
    assert_eq!(light.power_w, 650.0);
}

#[test]
fn a_new_object_and_wall_have_the_defaults() {
    let mut s = open();

    s.edit_plan(PlanEdit::AddObject {
        kind: "chair".into(),
        at: p(5.0, 5.0),
    })
    .unwrap();
    s.edit_plan(PlanEdit::AddWall {
        from: p(1.0, 1.0),
        to: p(4.0, 1.0),
    })
    .unwrap();

    let object = location_set(&s).objects.last().unwrap();
    assert_eq!(object.id, "o1");
    assert_eq!(object.kind, "chair");
    assert_eq!((object.width_m, object.depth_m), (1.0, 1.0));
    assert!(object.label.is_empty());
    assert_eq!(location_set(&s).walls.last().unwrap().thickness_m, 0.2);
}

#[test]
fn a_wall_of_no_length_is_refused() {
    let mut s = open();

    let err = s
        .edit_plan(PlanEdit::AddWall {
            from: p(1.0, 1.0),
            to: p(1.0, 1.0),
        })
        .unwrap_err();

    assert!(matches!(err, ServiceError::BadValue { field: "wall", .. }));
    assert!(!s.can_undo());
}

#[test]
fn numbers_that_are_not_finite_change_nothing() {
    let mut s = open();
    let before = s.project().clone();
    let source = mark("elisabeth", 0);

    let edits = [
        PlanEdit::Move {
            source: source.clone(),
            to: p(f64::NAN, 1.0),
        },
        PlanEdit::Move {
            source: source.clone(),
            to: p(1.0, f64::INFINITY),
        },
        PlanEdit::Turn {
            source,
            facing_deg: f64::NAN,
        },
        PlanEdit::AddCamera {
            at: p(1.0, 1.0),
            facing_deg: f64::INFINITY,
        },
        PlanEdit::AddWall {
            from: p(0.0, 0.0),
            to: p(f64::NAN, 0.0),
        },
    ];
    for edit in edits {
        let err = s.edit_plan(edit).unwrap_err();
        assert!(matches!(err, ServiceError::BadValue { .. }));
    }

    assert_eq!(*s.project(), before);
    assert!(!s.can_undo());
}

#[test]
fn an_unknown_source_is_refused() {
    let mut s = open();

    let err = s
        .edit_plan(PlanEdit::Move {
            source: Source::Wall(99),
            to: p(1.0, 1.0),
        })
        .unwrap_err();

    assert!(matches!(err, ServiceError::Gone));
}

/// Review of PR 10: a 27th camera has no letter left, said without code text.
#[test]
fn cameras_run_out_of_letters() {
    let mut s = open();
    let add = || PlanEdit::AddCamera {
        at: p(1.0, 1.0),
        facing_deg: 0.0,
    };

    while s.plan().unwrap().cameras.len() < 26 {
        s.edit_plan(add()).unwrap();
    }

    assert!(matches!(
        s.edit_plan(add()).unwrap_err(),
        ServiceError::NoFreeCamera
    ));
}

#[test]
fn an_edit_that_changes_nothing_is_no_step() {
    let mut s = open();

    s.edit_plan(PlanEdit::Move {
        source: mark("elisabeth", 0),
        to: p(3.0, 6.5),
    })
    .unwrap();

    assert!(!s.can_undo());
    assert!(!s.is_dirty());
}

/// The table (10, 6.5; 6 x 1.2) and Elisabeth's mark 2 at its middle: the mark is drawn later,
/// so it is in front.
#[test]
fn pick_takes_the_front_item() {
    let s = open();

    assert_eq!(s.pick(p(10.0, 6.5)), Some(mark("elisabeth", 1)));
    assert_eq!(s.pick(p(8.0, 6.5)), Some(Source::Object("tisch".into())));
    assert_eq!(
        s.pick(p(3.0, 6.5)),
        Some(Source::Figure("elisabeth".into()))
    );
}

/// The mark has radius 0.22 m and the pick radius adds 0.15: 0.36 m away still hits it, 0.38 m
/// reaches only the table below.
#[test]
fn pick_has_a_tolerance() {
    let s = open();

    assert_eq!(s.pick(p(10.36, 6.5)), Some(mark("elisabeth", 1)));
    assert_eq!(s.pick(p(10.38, 6.5)), Some(Source::Object("tisch".into())));
}

/// The 0.4 m wall at y = 0 reaches 0.2 m out, plus 0.15 m.
#[test]
fn pick_finds_walls_and_lights() {
    let s = open();

    assert_eq!(s.pick(p(10.0, -0.3)), Some(Source::Wall(0)));
    assert_eq!(s.pick(p(10.0, -0.4)), None);
    assert_eq!(s.pick(p(1.0, 10.0)), Some(Source::Light("led-1".into())));
}

/// Where only the view cones, the axis and the paths are, nothing can be taken.
#[test]
fn pick_skips_what_only_shows() {
    let s = open();

    assert_eq!(s.pick(p(10.0, 3.0)), None);
}

#[test]
fn pick_skips_what_is_hidden() {
    let mut s = open();

    s.set_layer(Layer::Set, Visibility::Hidden);
    assert_eq!(s.pick(p(8.0, 6.5)), None);

    s.set_track("elisabeth", Visibility::Hidden);
    assert_eq!(s.pick(p(3.0, 6.5)), None);
}

#[test]
fn snapping_is_on_by_default() {
    let mut s = open();

    // 0.1 / 0.25 = 0.4 -> 0; 0.13 / 0.25 = 0.52 -> 0.25; 1.1 -> 1.0; 2.4 -> 2.5.
    assert_eq!(s.snap(p(0.1, 0.13)), p(0.0, 0.25));
    assert_eq!(s.snap(p(1.1, 2.4)), p(1.0, 2.5));

    s.set_snapping(Snapping::Off);
    assert_eq!(s.snap(p(0.1, 0.13)), p(0.1, 0.13));
}

#[test]
fn edits_survive_saving_and_loading() {
    let dir = copy("plan-edit-save");
    let mut s = Session::open(&dir).unwrap();

    s.edit_plan(PlanEdit::AddMark {
        character: "elisabeth".into(),
        at: p(16.0, 7.2),
    })
    .unwrap();
    s.edit_plan(PlanEdit::Move {
        source: Source::Wall(0),
        to: p(10.0, 2.0),
    })
    .unwrap();
    s.edit_plan(PlanEdit::AddLight {
        kind: LightKind::Tube,
        at: p(3.0, 3.0),
    })
    .unwrap();
    s.edit_plan(PlanEdit::AddObject {
        kind: "chair".into(),
        at: p(5.0, 5.0),
    })
    .unwrap();
    s.edit_plan(PlanEdit::AddCamera {
        at: p(4.0, 5.0),
        facing_deg: 100.0,
    })
    .unwrap();
    s.edit_plan(PlanEdit::Remove {
        source: Source::Light("led-1".into()),
    })
    .unwrap();
    s.save().unwrap();

    let again = Session::open(&dir).unwrap();
    assert_eq!(again.project(), s.project());
}
