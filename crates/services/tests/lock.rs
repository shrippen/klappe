//! Locking layers (LH-15), on the example project: the table `tisch` and the walls are on
//! the set, Elisabeth's second mark is at (10, 6.5) on the figures, `led-1` stands at
//! (1, 10), the note `n1` at (3, 11.5), the arrow `d1`
//! from (6, 10) to (6, 7.5).
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::{
    Dash, DrawingShape, FigureColor, Fill, Layer, LightKind, Lock, PlanEdit, Point, Role,
    ServiceError, Session, Source, Visibility,
};

const LAYERS: [Layer; 6] = [
    Layer::Set,
    Layer::Figures,
    Layer::Cameras,
    Layer::Lights,
    Layer::Notes,
    Layer::Drawings,
];

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe")
}

fn open() -> Session {
    Session::open(&example()).unwrap()
}

fn p(x_m: f64, y_m: f64) -> Point {
    Point::new(x_m, y_m)
}

fn locked(layer: Layer) -> Session {
    let mut s = open();
    s.set_lock(layer, Lock::Locked);
    s
}

fn is_locked(result: Result<(), ServiceError>, layer: Layer) -> bool {
    matches!(result, Err(ServiceError::Locked(l)) if l == layer)
}

#[test]
fn no_layer_is_locked_at_first() {
    let s = open();

    for layer in LAYERS {
        assert_eq!(s.lock(layer), Lock::Unlocked);
    }
}

#[test]
fn a_lock_can_be_set_and_taken_off() {
    let mut s = open();

    s.set_lock(Layer::Lights, Lock::Locked);
    assert_eq!(s.lock(Layer::Lights), Lock::Locked);
    assert_eq!(s.lock(Layer::Set), Lock::Unlocked);

    s.set_lock(Layer::Lights, Lock::Unlocked);
    assert_eq!(s.lock(Layer::Lights), Lock::Unlocked);
}

#[test]
fn pick_takes_nothing_from_a_locked_layer() {
    let mut s = open();
    let table = p(8.0, 6.5);
    let light = p(1.0, 10.0);
    let note = p(3.0, 11.5);
    assert_eq!(s.pick(table), Some(Source::Object("tisch".into())));

    s.set_lock(Layer::Set, Lock::Locked);
    assert_eq!(s.pick(table), None);
    assert_eq!(s.pick(light), Some(Source::Light("led-1".into())));

    s.set_lock(Layer::Lights, Lock::Locked);
    s.set_lock(Layer::Notes, Lock::Locked);
    assert_eq!(s.pick(light), None);
    assert_eq!(s.pick(note), None);

    s.set_lock(Layer::Set, Lock::Unlocked);
    assert_eq!(s.pick(table), Some(Source::Object("tisch".into())));
}

/// What lies under a locked item can be taken: the figure's mark stands on the table.
#[test]
fn pick_reaches_through_a_locked_layer() {
    let mut s = open();
    let on_table = p(10.0, 6.5);
    assert!(matches!(s.pick(on_table), Some(Source::Mark { .. })));

    s.set_lock(Layer::Figures, Lock::Locked);

    assert_eq!(s.pick(on_table), Some(Source::Object("tisch".into())));
}

#[test]
fn a_locked_layer_is_still_drawn() {
    let mut s = open();
    let before = s.display_list();

    for layer in LAYERS {
        s.set_lock(layer, Lock::Locked);
    }

    assert_eq!(s.display_list(), before);
}

#[test]
fn locking_and_hiding_are_independent() {
    let mut s = open();

    s.set_lock(Layer::Notes, Lock::Locked);
    s.set_layer(Layer::Notes, Visibility::Hidden);
    s.set_lock(Layer::Notes, Lock::Unlocked);

    assert_eq!(s.layer(Layer::Notes), Visibility::Hidden);
    assert!(s.display_list().items.iter().all(|i| i.role != Role::Note));
}

#[test]
fn changing_what_is_on_a_locked_layer_is_refused() {
    let cases = [
        (Layer::Set, Source::Object("tisch".into())),
        (Layer::Set, Source::Wall(0)),
        (
            Layer::Figures,
            Source::Mark {
                character: "elisabeth".into(),
                index: 1,
            },
        ),
        (Layer::Figures, Source::Figure("elisabeth".into())),
        (Layer::Cameras, Source::Camera("a".into())),
        (Layer::Lights, Source::Light("led-1".into())),
        (Layer::Notes, Source::Note("n1".into())),
        (Layer::Drawings, Source::Drawing("d1".into())),
    ];

    for (layer, source) in cases {
        let mut s = locked(layer);
        let edits = [
            PlanEdit::Move {
                source: source.clone(),
                to: p(5.0, 5.0),
            },
            PlanEdit::Shift {
                source: source.clone(),
                dx_m: 1.0,
                dy_m: 1.0,
            },
            PlanEdit::Spin {
                source: source.clone(),
                by_deg: 15.0,
            },
            PlanEdit::Turn {
                source: source.clone(),
                facing_deg: 15.0,
            },
            PlanEdit::Remove {
                source: source.clone(),
            },
        ];

        for edit in edits {
            assert!(is_locked(s.edit_plan(edit.clone()), layer), "{edit:?}");
            assert!(matches!(s.preview(&edit), Err(ServiceError::Locked(l)) if l == layer));
        }
        assert!(!s.is_dirty(), "{source:?}");
        assert!(!s.can_undo());
    }
}

#[test]
fn adding_to_a_locked_layer_is_refused() {
    let at = p(5.0, 5.0);
    let cases = [
        (
            Layer::Set,
            PlanEdit::AddObject {
                kind: "chair".into(),
                at,
            },
        ),
        (
            Layer::Set,
            PlanEdit::AddWall {
                from: at,
                to: p(6.0, 5.0),
            },
        ),
        (
            Layer::Figures,
            PlanEdit::AddMark {
                character: "elisabeth".into(),
                at,
            },
        ),
        (
            Layer::Cameras,
            PlanEdit::AddCamera {
                at,
                facing_deg: 0.0,
            },
        ),
        (
            Layer::Lights,
            PlanEdit::AddLight {
                kind: LightKind::Tube,
                at,
            },
        ),
        (
            Layer::Notes,
            PlanEdit::AddNote {
                at,
                text: "x".into(),
            },
        ),
        (
            Layer::Drawings,
            PlanEdit::AddDrawing {
                shape: DrawingShape::Line {
                    from: at,
                    to: p(6.0, 5.0),
                },
                color: FigureColor::Red,
                dash: Dash::Solid,
                fill: Fill::Outline,
            },
        ),
        (
            Layer::Notes,
            PlanEdit::EditNote {
                id: "n1".into(),
                text: "y".into(),
            },
        ),
    ];

    for (layer, edit) in cases {
        let mut s = locked(layer);

        assert!(is_locked(s.edit_plan(edit.clone()), layer), "{edit:?}");
        assert!(!s.is_dirty(), "{edit:?}");
    }
}

/// The background belongs to the set.
#[test]
fn the_background_follows_the_set_lock() {
    let mut s = locked(Layer::Set);

    for edit in [
        PlanEdit::MoveBackground {
            dx_m: 1.0,
            dy_m: 0.0,
        },
        PlanEdit::RemoveBackground,
        PlanEdit::Calibrate {
            from: p(0.0, 0.0),
            to: p(1.0, 0.0),
            distance_m: 2.0,
        },
        PlanEdit::SetBackground {
            source: PathBuf::from("none.png"),
        },
    ] {
        assert!(is_locked(s.edit_plan(edit.clone()), Layer::Set), "{edit:?}");
    }
}

#[test]
fn other_layers_stay_editable_and_unlocking_allows_again() {
    let mut s = locked(Layer::Set);
    let wall = PlanEdit::AddWall {
        from: p(1.0, 1.0),
        to: p(2.0, 1.0),
    };

    s.edit_plan(PlanEdit::AddNote {
        at: p(5.0, 5.0),
        text: "x".into(),
    })
    .unwrap();
    assert!(is_locked(s.edit_plan(wall.clone()), Layer::Set));

    s.set_lock(Layer::Set, Lock::Unlocked);
    s.edit_plan(wall).unwrap();
    assert_eq!(s.project().locations[0].set.walls.len(), 5);
}

/// A lock keeps edits out, not the history: undo and redo still work.
#[test]
fn undo_works_on_a_locked_layer() {
    let mut s = open();
    s.edit_plan(PlanEdit::Remove {
        source: Source::Note("n1".into()),
    })
    .unwrap();

    s.set_lock(Layer::Notes, Lock::Locked);
    s.undo();

    assert_eq!(s.project().locations[0].set.notes.len(), 1);
}
