//! A grip is on the layer of the element it belongs to: hiding the layer hides the grip,
//! locking it makes the grip untakeable. On the example project: the table `tisch`, the lamp
//! `led-1`, the note `n1`, the arrow `d1` and the door of wall 2.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::{
    Layer, LightGrip, Lock, ObjectGrip, OpeningEnd, Point, Session, Shape, Snapping, Source,
    Visibility,
};

fn open() -> Session {
    let dir: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe");
    let mut s = Session::open(&dir).unwrap();
    s.set_snapping(Snapping::Off);

    s
}

/// The element, one of its grips and the layer both are on.
fn cases() -> Vec<(Source, Source, Layer)> {
    vec![
        (
            Source::Object("tisch".into()),
            Source::ObjectGrip {
                object: "tisch".into(),
                grip: ObjectGrip::Width,
            },
            Layer::Set,
        ),
        (
            Source::Opening { wall: 2, index: 0 },
            Source::OpeningEdge {
                wall: 2,
                index: 0,
                end: OpeningEnd::End,
            },
            Layer::Set,
        ),
        (
            Source::Light("led-1".into()),
            Source::LightGrip {
                light: "led-1".into(),
                grip: LightGrip::Beam,
            },
            Layer::Lights,
        ),
        (
            Source::Note("n1".into()),
            Source::NoteWidth("n1".into()),
            Layer::Notes,
        ),
        (
            Source::Drawing("d1".into()),
            Source::DrawingPoint {
                drawing: "d1".into(),
                index: 0,
            },
            Layer::Drawings,
        ),
    ]
}

fn place(s: &Session, selected: &Source, grip: &Source) -> Option<Point> {
    s.display_list_for(Some(selected))
        .items
        .iter()
        .find(|i| i.source.as_ref() == Some(grip))
        .and_then(|i| match i.shape {
            Shape::Circle { center, .. } => Some(center),
            _ => None,
        })
}

#[test]
fn a_grip_shows_while_its_element_is_selected() {
    let s = open();

    for (selected, grip, _) in cases() {
        assert!(place(&s, &selected, &grip).is_some(), "{grip:?}");
    }
}

#[test]
fn hiding_a_layer_hides_the_grips_on_it() {
    for (selected, grip, layer) in cases() {
        let mut s = open();
        s.set_layer(layer, Visibility::Hidden);

        assert_eq!(place(&s, &selected, &grip), None, "{grip:?}");
    }
}

#[test]
fn a_locked_layer_does_not_give_up_its_grips() {
    for (selected, grip, layer) in cases() {
        let mut s = open();
        let at = place(&s, &selected, &grip).unwrap();
        assert_eq!(
            s.pick_for(at, Some(&selected)),
            Some(grip.clone()),
            "{grip:?}"
        );

        s.set_lock(layer, Lock::Locked);

        assert_ne!(
            s.pick_for(at, Some(&selected)),
            Some(grip.clone()),
            "{grip:?}"
        );
    }
}
