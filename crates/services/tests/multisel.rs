//! Taking several elements at once (LH-08), on the example project: the pillars `saeule-1` at
//! (6, 4) and `saeule-2` at (14, 4), the table `tisch` at (8 .. 12, 6.5), the lamp `led-1` at
//! (1, 10), the note `n1` at (3, 11.5), Elisabeth's three marks, camera `b` that follows
//! Burgvogt.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::{Layer, Lock, PlanEdit, Point, ServiceError, Session, Snapping, Source};

/// The grid the example snaps to.
const GRID_M: f64 = 0.25;

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe")
}

fn open() -> Session {
    let mut s = Session::open(&example()).unwrap();
    s.set_snapping(Snapping::Off);
    s
}

fn p(x_m: f64, y_m: f64) -> Point {
    Point::new(x_m, y_m)
}

fn object(id: &str) -> Source {
    Source::Object(id.into())
}

fn mark(character: &str, index: usize) -> Source {
    Source::Mark {
        character: character.into(),
        index,
    }
}

fn pillars() -> Vec<Source> {
    vec![object("saeule-1"), object("saeule-2")]
}

fn near(a: Point, b: Point) -> bool {
    a.distance_m(b) < 1e-9
}

fn object_at(s: &Session, id: &str) -> Point {
    let location = s.project().location("saengersaal").unwrap();

    location.set.objects.iter().find(|o| o.id == id).unwrap().at
}

fn rotation_of(s: &Session, id: &str) -> f64 {
    let location = s.project().location("saengersaal").unwrap();

    location
        .set
        .objects
        .iter()
        .find(|o| o.id == id)
        .unwrap()
        .rotation_deg
}

fn mark_at(s: &Session, character: &str, index: usize) -> Point {
    s.plan().unwrap().figure(character).unwrap().marks[index].at
}

#[test]
fn a_box_takes_what_lies_wholly_in_it() {
    let s = open();

    let both = s.pick_in(p(5.0, 3.0), p(15.0, 5.0));
    // Corners in any order.
    let same = s.pick_in(p(15.0, 5.0), p(5.0, 3.0));

    assert!(both.contains(&object("saeule-1")));
    assert!(both.contains(&object("saeule-2")));
    assert!(!both.contains(&object("tisch")));
    assert_eq!(both, same);
}

#[test]
fn a_box_leaves_out_what_it_only_touches() {
    let s = open();

    // The pillar reaches from 5.6 to 6.4; the box ends at 6.0.
    let found = s.pick_in(p(5.0, 3.0), p(6.0, 5.0));

    assert!(!found.contains(&object("saeule-1")));
}

#[test]
fn a_box_takes_nothing_from_a_locked_layer() {
    let mut s = open();
    let all = (p(0.0, 0.0), p(20.0, 13.0));
    assert!(s.pick_in(all.0, all.1).contains(&object("saeule-1")));

    s.set_lock(Layer::Set, Lock::Locked);
    let found = s.pick_in(all.0, all.1);

    assert!(!found.contains(&object("saeule-1")));
    assert!(found.contains(&Source::Light("led-1".into())));
}

#[test]
fn a_group_moves_by_one_distance_in_one_undo_step() {
    let mut s = open();
    let sources = vec![
        object("saeule-1"),
        object("saeule-2"),
        Source::Light("led-1".into()),
    ];

    s.edit_plan(PlanEdit::ShiftGroup {
        sources,
        dx_m: 1.0,
        dy_m: 0.5,
    })
    .unwrap();

    assert!(near(object_at(&s, "saeule-1"), p(7.0, 4.5)));
    assert!(near(object_at(&s, "saeule-2"), p(15.0, 4.5)));
    assert!(near(s.plan().unwrap().lights[0].at, p(2.0, 10.5)));

    s.undo();

    assert!(near(object_at(&s, "saeule-1"), p(6.0, 4.0)));
    assert!(near(object_at(&s, "saeule-2"), p(14.0, 4.0)));
    assert!(near(s.plan().unwrap().lights[0].at, p(1.0, 10.0)));
}

/// The first one lands on the grid; the rest keep their place to it.
#[test]
fn a_group_snaps_by_its_first_element() {
    let mut s = open();
    s.set_snapping(Snapping::On);

    s.edit_plan(PlanEdit::ShiftGroup {
        sources: pillars(),
        dx_m: 0.3,
        dy_m: 0.0,
    })
    .unwrap();

    assert!(near(object_at(&s, "saeule-1"), p(6.25, 4.0)));
    assert!(near(object_at(&s, "saeule-2"), p(14.25, 4.0)));
}

/// Review of PR 31: the element first in the list is the one under the pointer and snaps,
/// even when it is a camera, which moves after the objects.
#[test]
fn a_group_snaps_by_the_camera_held() {
    let mut s = open();
    // Off the grid first, so the pillar and the camera snap apart.
    s.edit_plan(PlanEdit::ShiftGroup {
        sources: vec![object("saeule-1")],
        dx_m: 0.1,
        dy_m: 0.0,
    })
    .unwrap();
    s.set_snapping(Snapping::On);
    let pillar_before = object_at(&s, "saeule-1");

    s.edit_plan(PlanEdit::ShiftGroup {
        sources: vec![Source::Camera("b".into()), object("saeule-1")],
        dx_m: 0.3,
        dy_m: 0.1,
    })
    .unwrap();

    let camera = s.plan().unwrap().camera("b").unwrap().marks[0].at;
    let on_grid = |v: f64| ((v / GRID_M).round() * GRID_M - v).abs() < 1e-9;
    assert!(on_grid(camera.x_m) && on_grid(camera.y_m), "{camera:?}");
    assert!(!near(object_at(&s, "saeule-1"), pillar_before));
}

#[test]
fn a_group_turns_about_its_middle() {
    let mut s = open();

    s.edit_plan(PlanEdit::SpinGroup {
        sources: pillars(),
        by_deg: 90.0,
    })
    .unwrap();

    // Their middle is (10, 4); a quarter turn clockwise on the plan.
    assert!(near(object_at(&s, "saeule-1"), p(10.0, 0.0)));
    assert!(near(object_at(&s, "saeule-2"), p(10.0, 8.0)));
    assert_eq!(rotation_of(&s, "saeule-1"), 90.0);
    assert_eq!(rotation_of(&s, "saeule-2"), 90.0);
}

/// Review of what a group of a figure and its camera does: the figure carries the camera
/// that follows it, and the camera is in the group, too. It must not move twice.
#[test]
fn a_following_camera_in_the_group_moves_once() {
    let mut s = open();
    let camera_before = s.plan().unwrap().camera("b").unwrap().marks[0].at;

    s.edit_plan(PlanEdit::ShiftGroup {
        sources: vec![mark("burgvogt", 0), Source::Camera("b".into())],
        dx_m: 1.0,
        dy_m: 0.0,
    })
    .unwrap();

    let camera = &s.plan().unwrap().camera("b").unwrap().marks[0];
    assert!(near(
        camera.at,
        p(camera_before.x_m + 1.0, camera_before.y_m)
    ));
    assert!(near(mark_at(&s, "burgvogt", 0), p(18.0, 9.5)));
}

#[test]
fn a_figure_and_its_mark_move_once() {
    let mut s = open();

    s.edit_plan(PlanEdit::ShiftGroup {
        sources: vec![Source::Figure("elisabeth".into()), mark("elisabeth", 0)],
        dx_m: 1.0,
        dy_m: 0.0,
    })
    .unwrap();

    assert!(near(mark_at(&s, "elisabeth", 0), p(4.0, 6.5)));
}

#[test]
fn a_group_with_a_locked_layer_is_refused_whole() {
    let mut s = open();
    s.set_lock(Layer::Lights, Lock::Locked);
    let sources = vec![object("saeule-1"), Source::Light("led-1".into())];
    let edits = [
        PlanEdit::ShiftGroup {
            sources: sources.clone(),
            dx_m: 1.0,
            dy_m: 0.0,
        },
        PlanEdit::SpinGroup {
            sources: sources.clone(),
            by_deg: 15.0,
        },
        PlanEdit::RemoveGroup {
            sources: sources.clone(),
        },
        PlanEdit::Duplicate {
            sources,
            dx_m: 0.5,
            dy_m: 0.5,
        },
    ];

    for edit in edits {
        let result = s.edit_plan(edit.clone());

        assert!(
            matches!(result, Err(ServiceError::Locked(Layer::Lights))),
            "{edit:?}"
        );
        assert!(matches!(
            s.preview(&edit),
            Err(ServiceError::Locked(Layer::Lights))
        ));
    }
    assert!(!s.is_dirty());
}

#[test]
fn a_group_is_removed_with_the_indices_it_had() {
    let mut s = open();
    let walls = s.project().location("saengersaal").unwrap().set.walls.len();

    s.edit_plan(PlanEdit::RemoveGroup {
        sources: vec![
            Source::Wall(0),
            Source::Wall(2),
            mark("elisabeth", 0),
            mark("elisabeth", 1),
            object("saeule-1"),
        ],
    })
    .unwrap();

    let location = s.project().location("saengersaal").unwrap();
    assert_eq!(location.set.walls.len(), walls - 2);
    assert!(location.set.objects.iter().all(|o| o.id != "saeule-1"));
    let marks = &s.plan().unwrap().figure("elisabeth").unwrap().marks;
    assert_eq!(marks.len(), 1);
    assert!(near(marks[0].at, p(16.0, 3.0)));

    s.undo();

    let location = s.project().location("saengersaal").unwrap();
    assert_eq!(location.set.walls.len(), walls);
    assert_eq!(
        s.plan().unwrap().figure("elisabeth").unwrap().marks.len(),
        3
    );
}

/// A figure that goes takes its marks with it; removing them again would find nothing.
#[test]
fn removing_a_figure_and_its_marks_works() {
    let mut s = open();

    s.edit_plan(PlanEdit::RemoveGroup {
        sources: vec![
            mark("elisabeth", 1),
            Source::Figure("elisabeth".into()),
            Source::FigureVia {
                character: "elisabeth".into(),
                index: 2,
            },
        ],
    })
    .unwrap();

    assert!(s.plan().unwrap().figure("elisabeth").is_none());
    assert!(s.plan().unwrap().figure("burgvogt").is_some());
}

#[test]
fn a_copy_has_its_own_ids_and_sits_apart() {
    let mut s = open();
    let before = s.display_list();

    s.edit_plan(PlanEdit::Duplicate {
        sources: vec![
            object("saeule-1"),
            Source::Light("led-1".into()),
            Source::Note("n1".into()),
            Source::Drawing("d1".into()),
            Source::Camera("a".into()),
            Source::Wall(0),
        ],
        dx_m: 0.5,
        dy_m: 0.5,
    })
    .unwrap();

    let location = s.project().location("saengersaal").unwrap();
    assert_eq!(location.set.objects.len(), 4);
    let copy = location.set.objects.last().unwrap();
    assert_ne!(copy.id, "saeule-1");
    assert!(near(copy.at, p(6.5, 4.5)));
    assert!(near(object_at(&s, "saeule-1"), p(6.0, 4.0)));

    let plan = s.plan().unwrap();
    assert_eq!(plan.lights.len(), 2);
    assert_eq!(plan.cameras.len(), 3);
    let camera = plan.cameras.last().unwrap();
    assert_eq!(camera.id, "c");
    assert_eq!(camera.label, "C");
    assert!(near(camera.marks[0].at, p(8.5, 11.5)));
    assert_eq!(plan.drawings.len(), 2);
    assert_eq!(location.set.notes.len(), 2);
    assert_eq!(location.set.walls.len(), 5);

    s.undo();

    assert_eq!(s.display_list(), before);
}

#[test]
fn a_figure_has_no_copy() {
    let mut s = open();

    let result = s.edit_plan(PlanEdit::Duplicate {
        sources: vec![Source::Figure("elisabeth".into()), mark("elisabeth", 1)],
        dx_m: 0.5,
        dy_m: 0.5,
    });

    assert!(matches!(result, Err(ServiceError::BadValue { .. })));
    assert!(!s.is_dirty());
}

#[test]
fn an_element_that_is_gone_refuses_the_group() {
    let mut s = open();

    let result = s.edit_plan(PlanEdit::ShiftGroup {
        sources: vec![object("saeule-1"), object("weg")],
        dx_m: 1.0,
        dy_m: 0.0,
    });

    assert!(matches!(result, Err(ServiceError::Gone)));
    assert!(near(object_at(&s, "saeule-1"), p(6.0, 4.0)));
}

#[test]
fn a_group_edit_refuses_numbers_that_are_not() {
    let mut s = open();

    let shift = s.edit_plan(PlanEdit::ShiftGroup {
        sources: pillars(),
        dx_m: f64::NAN,
        dy_m: 0.0,
    });
    let spin = s.edit_plan(PlanEdit::SpinGroup {
        sources: pillars(),
        by_deg: f64::INFINITY,
    });

    assert!(matches!(shift, Err(ServiceError::BadValue { .. })));
    assert!(matches!(spin, Err(ServiceError::BadValue { .. })));
}
