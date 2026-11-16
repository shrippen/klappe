//! Height ergänzen (LH-93): objects without a height, the next one after the selection, and
//! the 3D view following it. The example's set has three, in the order `tisch`, `saeule-1`,
//! `saeule-2`.
#![allow(clippy::unwrap_used)]

use std::path::Path;

use klappe_services::{PlanEdit, Session, Source};

const EXAMPLE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../testdata/beispiel.klappe"
);
const SIZE: (f64, f64) = (320.0, 240.0);
/// Half of the stand-in height of an object without one (0.9 m).
const HALF_STAND_IN_M: f64 = 0.45;

fn example() -> Session {
    Session::open(Path::new(EXAMPLE)).unwrap()
}

fn set_height(s: &mut Session, id: &str, height_m: Option<f64>) {
    let edit = PlanEdit::SetObjectHeight {
        source: Source::Object(id.into()),
        height_m,
    };
    s.edit_plan(edit).unwrap();
}

#[test]
fn the_count_follows_the_heights() {
    let mut s = example();
    assert_eq!(s.missing_heights(), 3);

    set_height(&mut s, "tisch", Some(0.8));
    assert_eq!(s.missing_heights(), 2);

    set_height(&mut s, "tisch", None);
    assert_eq!(s.missing_heights(), 3);
}

#[test]
fn the_next_object_without_height_goes_round() {
    let mut s = example();
    set_height(&mut s, "saeule-1", Some(3.0));

    assert_eq!(s.next_missing_height(None).as_deref(), Some("tisch"));
    assert_eq!(
        s.next_missing_height(Some("tisch")).as_deref(),
        Some("saeule-2")
    );
    assert_eq!(
        s.next_missing_height(Some("saeule-1")).as_deref(),
        Some("saeule-2")
    );
    assert_eq!(
        s.next_missing_height(Some("saeule-2")).as_deref(),
        Some("tisch")
    );
    assert_eq!(
        s.next_missing_height(Some("gone")).as_deref(),
        Some("tisch")
    );
}

/// With one left, the search comes back to the selected object itself.
#[test]
fn the_last_one_is_its_own_next() {
    let mut s = example();
    set_height(&mut s, "saeule-1", Some(3.0));
    set_height(&mut s, "saeule-2", Some(3.0));

    assert_eq!(
        s.next_missing_height(Some("tisch")).as_deref(),
        Some("tisch")
    );

    set_height(&mut s, "tisch", Some(0.8));
    assert_eq!(s.next_missing_height(Some("tisch")), None);
    assert_eq!(s.missing_heights(), 0);
}

/// The middle of the 3D picture goes to the object, at half the stand-in height.
#[test]
fn the_3d_view_follows_the_object() {
    let mut s = example();
    s.toggle_view();
    s.aim_at_object("saeule-2");

    let middle = s
        .free_level_at((SIZE.0 / 2.0, SIZE.1 / 2.0), SIZE, HALF_STAND_IN_M)
        .unwrap();

    assert!((middle.x_m - 14.0).abs() < 1e-6, "{middle:?}");
    assert!((middle.y_m - 4.0).abs() < 1e-6, "{middle:?}");
}

/// What a height drag starts from (LH-95): the stand-in for an object without a height, the
/// own height once set, the wall's top (the room's 2.6 m without its own); nothing for the rest.
#[test]
fn a_height_drag_starts_from_what_stands() {
    let mut s = example();
    let table = Source::Object("tisch".into());
    assert!((s.height_of(&table).unwrap() - 0.9).abs() < 1e-9);

    set_height(&mut s, "tisch", Some(0.8));
    assert!((s.height_of(&table).unwrap() - 0.8).abs() < 1e-9);
    assert!((s.height_of(&Source::Wall(0)).unwrap() - 2.6).abs() < 1e-9);
    assert!(s.height_of(&Source::Wall(99)).is_none());
    assert!(s.height_of(&Source::Object("fort".into())).is_none());
    assert!(s.height_of(&Source::Note("x".into())).is_none());
}

/// A click on an object's height caption takes the object: the caption lies on top of its body
/// but belongs to nothing. The pillar `saeule-1` (0.8 m, middle (6, 4), named) has its caption
/// 0.3 m below the middle.
#[test]
fn a_click_on_the_height_takes_the_object() {
    let s = example();
    let caption = klappe_model::Point::new(6.0, 4.3);

    assert_eq!(s.pick(caption), Some(Source::Object("saeule-1".into())));
}
