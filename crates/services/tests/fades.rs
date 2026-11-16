//! Fading layers in steps (LH-15): the plan's items and the 3D view's bodies read the strength.
#![allow(clippy::unwrap_used)]

use std::path::Path;

use klappe_services::{Fade, Layer, SceneRole, Session};

const EXAMPLE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../testdata/beispiel.klappe"
);

fn example() -> Session {
    Session::open(Path::new(EXAMPLE)).unwrap()
}

#[test]
fn the_steps_are_100_60_and_30_percent() {
    let strengths: Vec<f64> = Fade::ALL.iter().map(|f| f.strength()).collect();
    let names: Vec<_> = Fade::ALL.iter().map(|f| f.name()).collect();

    assert_eq!(strengths, [1.0, 0.6, 0.3]);
    assert_eq!(names, ["100", "60", "30"]);
    for fade in Fade::ALL {
        assert_eq!(Fade::from_name(fade.name()), Some(fade));
    }
    assert_eq!(Fade::Full.next(), Fade::Medium);
    assert_eq!(Fade::Faint.next(), Fade::Full);
}

#[test]
fn a_faded_layer_fades_its_items_and_no_others() {
    let mut s = example();
    s.set_fade(Layer::Figures, Fade::Medium);

    let list = s.display_list();
    let fades = s.item_fades(&list);

    assert_eq!(fades.len(), list.items.len());
    let figure = list
        .items
        .iter()
        .position(|i| matches!(i.role, klappe_services::Role::Figure(_)))
        .unwrap();
    let wall = list
        .items
        .iter()
        .position(|i| i.role == klappe_services::Role::Wall)
        .unwrap();
    assert_eq!(fades[figure], 0.6);
    assert_eq!(fades[wall], 1.0);
}

/// Fading is a view setting: nothing to undo, nothing hidden, and back to full takes it away.
#[test]
fn fading_is_a_view_setting() {
    let mut s = example();
    let shown = s.display_list().items.len();

    s.set_fade(Layer::Set, Fade::Faint);

    assert!(!s.can_undo());
    assert!(!s.is_dirty());
    assert_eq!(s.display_list().items.len(), shown);
    assert_eq!(s.fade(Layer::Set), Fade::Faint);
    s.set_fade(Layer::Set, Fade::Full);
    assert!(s.item_fades(&s.display_list()).iter().all(|f| *f == 1.0));
}

#[test]
fn the_3d_view_reads_the_layer_of_each_body() {
    let mut s = example();
    s.set_fade(Layer::Set, Fade::Faint);
    s.set_fade(Layer::Cameras, Fade::Medium);

    assert_eq!(s.scene_fade(SceneRole::Floor), 0.3);
    assert_eq!(s.scene_fade(SceneRole::Wall), 0.3);
    assert_eq!(s.scene_fade(SceneRole::Object), 0.3);
    assert_eq!(s.scene_fade(SceneRole::Camera), 0.6);
    assert_eq!(
        s.scene_fade(SceneRole::Figure(klappe_services::FigureColor::Red)),
        1.0
    );
}
