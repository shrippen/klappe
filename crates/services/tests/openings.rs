//! Doors and windows in walls (LH-10), on the example project: the south wall (index 2) runs
//! from (20, 13) to (0, 13) and has a door 0.9 m wide, 5 m from its start (x = 15); the north
//! wall (index 0) runs from (0, 0) to (20, 0) and has no opening. Walls are 0.4 m thick.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_model::{DoorSide, Hinge, Opening, OpeningKind};
use klappe_services::{
    Field, Hint, Layer, Lock, PlanEdit, Point, Role, ServiceError, Session, Source,
};

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe")
}

fn open() -> Session {
    Session::open(&example()).unwrap()
}

fn p(x_m: f64, y_m: f64) -> Point {
    Point::new(x_m, y_m)
}

fn opening(wall: usize, index: usize) -> Source {
    Source::Opening { wall, index }
}

fn openings(s: &Session, wall: usize) -> Vec<Opening> {
    s.project().locations[0].set.walls[wall].openings.clone()
}

fn bad(result: Result<(), ServiceError>, wanted: Field) -> bool {
    matches!(result, Err(ServiceError::BadValue { field, .. }) if field == wanted)
}

fn swing(side: Option<DoorSide>, hinge: Option<Hinge>) -> PlanEdit {
    PlanEdit::SetOpeningSwing {
        source: opening(2, 0),
        side,
        hinge,
    }
}

fn add(kind: OpeningKind, at: Point) -> PlanEdit {
    PlanEdit::AddOpening { kind, at }
}

#[test]
fn a_click_on_a_wall_sets_a_door_of_the_standard_width() {
    let mut s = open();

    s.edit_plan(add(OpeningKind::Door, p(8.0, 0.1))).unwrap();

    assert_eq!(
        openings(&s, 0),
        [Opening {
            kind: OpeningKind::Door,
            at_m: 8.0,
            width_m: 0.9,
            head_m: None,
            sill_m: None,
            side: DoorSide::Right,
            hinge: Hinge::Start,
        }]
    );
}

#[test]
fn a_window_is_wider_than_a_door() {
    let mut s = open();

    s.edit_plan(add(OpeningKind::Window, p(8.0, 0.0))).unwrap();

    assert_eq!(openings(&s, 0)[0].width_m, 1.2);
}

/// A click beside the wall, within its thickness and a pick radius, still takes it; the
/// opening lands on the wall's line.
#[test]
fn the_click_may_be_beside_the_wall() {
    let mut s = open();

    s.edit_plan(add(OpeningKind::Door, p(8.0, 0.4))).unwrap();

    assert_eq!(openings(&s, 0)[0].at_m, 8.0);
}

#[test]
fn a_click_far_from_any_wall_is_refused_and_leaves_no_undo_step() {
    let mut s = open();

    let result = s.edit_plan(add(OpeningKind::Door, p(10.0, 6.0)));

    assert!(bad(result, Field::Wall));
    assert!(!s.can_undo());
}

/// Near the corner the opening is pushed in: 0.45 m is half a door.
#[test]
fn an_opening_stays_inside_its_wall() {
    let mut s = open();

    s.edit_plan(add(OpeningKind::Door, p(0.05, 0.0))).unwrap();
    s.edit_plan(add(OpeningKind::Door, p(19.99, 0.0))).unwrap();

    let found = openings(&s, 0);
    assert_eq!(found[0].at_m, 0.45);
    assert_eq!(found[1].at_m, 19.55);
}

#[test]
fn an_opening_does_not_overlap_another() {
    let mut s = open();

    // The door of the south wall sits at 5 m, 0.45 either side: x from 14.55 to 15.45.
    let result = s.edit_plan(add(OpeningKind::Window, p(15.0, 13.0)));

    assert!(bad(result, Field::Wall));
    assert_eq!(openings(&s, 2).len(), 1);
}

#[test]
fn setting_an_opening_is_one_undo_step() {
    let mut s = open();
    let before = s.project().clone();

    s.edit_plan(add(OpeningKind::Door, p(8.0, 0.0))).unwrap();
    s.undo();

    assert_eq!(*s.project(), before);
    assert!(!s.can_undo());
}

#[test]
fn an_opening_slides_along_its_wall() {
    let mut s = open();

    // The south wall runs towards smaller x, so x = 10 is 10 m from its start.
    s.edit_plan(PlanEdit::Move {
        source: opening(2, 0),
        to: p(10.0, 12.0),
    })
    .unwrap();

    assert_eq!(openings(&s, 2)[0].at_m, 10.0);
    s.undo();
    assert_eq!(openings(&s, 2)[0].at_m, 5.0);
}

/// A drag by a distance carries the opening along the wall by the part that runs along it.
#[test]
fn a_drag_slides_it_by_the_distance_along_the_wall() {
    let mut s = open();

    s.edit_plan(PlanEdit::Shift {
        source: opening(2, 0),
        dx_m: -3.0,
        dy_m: -4.0,
    })
    .unwrap();

    assert_eq!(openings(&s, 2)[0].at_m, 8.0);
    assert!(s.can_undo());
}

#[test]
fn sliding_into_another_opening_is_refused() {
    let mut s = open();
    s.edit_plan(add(OpeningKind::Window, p(10.0, 13.0)))
        .unwrap();

    let result = s.edit_plan(PlanEdit::Move {
        source: opening(2, 0),
        to: p(10.0, 13.0),
    });

    assert!(bad(result, Field::Wall));
}

#[test]
fn an_opening_cannot_be_turned() {
    let mut s = open();
    let before = s.project().clone();

    s.edit_plan(PlanEdit::Turn {
        source: opening(2, 0),
        facing_deg: 90.0,
    })
    .unwrap();

    assert_eq!(*s.project(), before);
}

#[test]
fn the_width_changes_about_the_centre() {
    let mut s = open();

    s.edit_plan(PlanEdit::SetOpeningWidth {
        source: opening(2, 0),
        width_m: 1.5,
    })
    .unwrap();

    let door = &openings(&s, 2)[0];
    assert_eq!((door.at_m, door.width_m), (5.0, 1.5));
    s.undo();
    assert_eq!(openings(&s, 2)[0].width_m, 0.9);
}

#[test]
fn a_width_that_does_not_fit_is_refused() {
    let mut s = open();

    for width_m in [0.0, -1.0, f64::NAN, 25.0, 0.1, 0.29] {
        let result = s.edit_plan(PlanEdit::SetOpeningWidth {
            source: opening(2, 0),
            width_m,
        });
        assert!(bad(result, Field::Width), "{width_m}");
    }

    // The window at 11 m (10.4 to 11.6 along the wall) is in the way of a 12 m wide door,
    // which would be pushed in to span 0 to 12.
    s.edit_plan(add(OpeningKind::Window, p(9.0, 13.0))).unwrap();
    let before = s.project().clone();
    let result = s.edit_plan(PlanEdit::SetOpeningWidth {
        source: opening(2, 0),
        width_m: 12.0,
    });
    assert!(bad(result, Field::Wall));
    assert_eq!(*s.project(), before);
}

#[test]
fn removing_an_opening_closes_the_gap_and_undoes() {
    let mut s = open();

    s.edit_plan(PlanEdit::Remove {
        source: opening(2, 0),
    })
    .unwrap();

    assert!(openings(&s, 2).is_empty());
    s.undo();
    assert_eq!(openings(&s, 2).len(), 1);
}

#[test]
fn removing_a_wall_takes_its_openings_and_a_missing_opening_is_gone() {
    let mut s = open();

    s.edit_plan(PlanEdit::Remove {
        source: Source::Wall(2),
    })
    .unwrap();
    let result = s.edit_plan(PlanEdit::Remove {
        source: opening(2, 0),
    });

    assert!(matches!(result, Err(ServiceError::Gone)));
}

/// The opening of a wall in a group follows the wall: moving both along the wall keeps the
/// opening where it is on the wall, not where the drag would slide it.
#[test]
fn a_group_with_a_wall_and_its_opening_moves_as_one() {
    let mut s = open();

    s.edit_plan(PlanEdit::ShiftGroup {
        sources: vec![opening(2, 0), Source::Wall(2)],
        dx_m: 2.0,
        dy_m: 0.0,
    })
    .unwrap();

    let set = &s.project().locations[0].set;
    assert_eq!(set.walls[2].from.x_m, 22.0);
    assert_eq!(set.walls[2].openings[0].at_m, 5.0);
}

#[test]
fn a_copied_wall_carries_its_openings() {
    let mut s = open();

    s.edit_plan(PlanEdit::Duplicate {
        sources: vec![Source::Wall(2), opening(2, 0)],
        dx_m: 0.0,
        dy_m: 1.0,
    })
    .unwrap();

    let set = &s.project().locations[0].set;
    assert_eq!(set.walls.len(), 5);
    assert_eq!(set.walls[4].openings.len(), 1);
    assert_eq!(set.walls[2].openings.len(), 1);
}

/// The wall has a gap where the door is, and the door is drawn there.
#[test]
fn the_display_list_has_the_gap_and_the_door() {
    let s = open();
    let list = s.display_list();
    let wall_pieces = |index: usize| {
        list.items
            .iter()
            .filter(|i| i.source == Some(Source::Wall(index)))
            .count()
    };

    assert_eq!(wall_pieces(0), 1);
    assert_eq!(wall_pieces(2), 2);
    assert!(list
        .items
        .iter()
        .any(|i| i.role == Role::Door && i.source == Some(opening(2, 0))));
}

#[test]
fn a_click_in_the_door_picks_the_opening() {
    let s = open();

    // In the swing of the door: hinge at (15.45, 13), 0.9 m leaf.
    let picked = s.pick(p(15.0, 12.7));

    assert_eq!(picked, Some(opening(2, 0)));
}

#[test]
fn a_locked_set_layer_refuses_every_opening_edit() {
    let mut s = open();
    s.set_lock(Layer::Set, Lock::Locked);
    let edits = [
        add(OpeningKind::Door, p(8.0, 0.0)),
        PlanEdit::Move {
            source: opening(2, 0),
            to: p(10.0, 13.0),
        },
        PlanEdit::Shift {
            source: opening(2, 0),
            dx_m: 1.0,
            dy_m: 0.0,
        },
        PlanEdit::SetOpeningWidth {
            source: opening(2, 0),
            width_m: 1.0,
        },
        swing(Some(DoorSide::Left), None),
        PlanEdit::Remove {
            source: opening(2, 0),
        },
    ];

    for edit in edits {
        let result = s.edit_plan(edit.clone());
        assert!(
            matches!(result, Err(ServiceError::Locked(Layer::Set))),
            "{edit:?}"
        );
    }
    assert!(!s.can_undo());
}

#[test]
fn openings_survive_saving_and_opening_again() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("openings-save");
    let _ = std::fs::remove_dir_all(&dir);
    copy_dir(&example(), &dir);

    let mut s = Session::open(&dir).unwrap();
    s.edit_plan(add(OpeningKind::Window, p(8.0, 0.0))).unwrap();
    s.save(klappe_services::OnOutside::Overwrite).unwrap();

    let again = Session::open(&dir).unwrap();
    assert_eq!(openings(&again, 0).len(), 1);
    assert_eq!(openings(&again, 0)[0].kind, OpeningKind::Window);
    assert_eq!(openings(&again, 2).len(), 1);
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

/// The door of the south wall starts as right and start; a swing edit changes what it names and
/// keeps the other, in one undo step each, and the card's hint says so.
#[test]
fn a_door_swing_is_set_in_one_undo_step() {
    let mut s = open();

    s.edit_plan(swing(Some(DoorSide::Left), None)).unwrap();
    s.edit_plan(swing(None, Some(Hinge::End))).unwrap();

    let door = &openings(&s, 2)[0];
    assert_eq!((door.side, door.hinge), (DoorSide::Left, Hinge::End));
    assert!(matches!(
        s.opening_hint(2, 0),
        Some(Hint::Opening {
            side: DoorSide::Left,
            hinge: Hinge::End,
            ..
        })
    ));

    s.undo();
    let door = &openings(&s, 2)[0];
    assert_eq!((door.side, door.hinge), (DoorSide::Left, Hinge::Start));
    s.undo();
    let door = &openings(&s, 2)[0];
    assert_eq!((door.side, door.hinge), (DoorSide::Right, Hinge::Start));
    assert!(!s.can_undo());
}

#[test]
fn a_window_has_no_swing() {
    let mut s = open();
    s.edit_plan(add(OpeningKind::Window, p(8.0, 0.0))).unwrap();

    let result = s.edit_plan(PlanEdit::SetOpeningSwing {
        source: opening(0, 0),
        side: Some(DoorSide::Left),
        hinge: None,
    });

    assert!(bad(result, Field::Source));
    assert_eq!(openings(&s, 0)[0].side, DoorSide::Right);
}
