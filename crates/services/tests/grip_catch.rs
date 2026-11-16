//! Which grip a click takes when several catch it, and the grips too small to show.
#![allow(clippy::unwrap_used)]

use klappe_services::{ObjectGrip, PlanEdit, Point, Role, Session, Shape, Snapping, Source};
use std::path::{Path, PathBuf};

fn open(points: &[(f64, f64)]) -> Session {
    let dir: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe");
    let mut s = Session::open(&dir).unwrap();
    s.set_snapping(Snapping::Off);
    s.edit_plan(PlanEdit::AddShape {
        points: points.iter().map(|(x, y)| Point::new(*x, *y)).collect(),
    })
    .unwrap();

    s
}

fn form(s: &Session) -> Source {
    let objects = &s.project().locations[0].set.objects;

    Source::Object(objects.last().unwrap().id.clone())
}

fn point(s: &Session, n: usize) -> Source {
    let objects = &s.project().locations[0].set.objects;

    Source::ObjectGrip {
        object: objects.last().unwrap().id.clone(),
        grip: ObjectGrip::Point(n),
    }
}

/// Points 0 and 1 lie 0.3 m apart, so both catch a click between them; the later one is on
/// top, but the click is nearer point 0.
#[test]
fn the_nearest_of_two_grips_takes_the_click() {
    let s = open(&[(4.0, 5.0), (4.3, 5.0), (4.0, 9.0)]);
    let selected = form(&s);

    assert_eq!(
        s.pick_for(Point::new(4.1, 5.0), Some(&selected)),
        Some(point(&s, 0))
    );
    assert_eq!(
        s.pick_for(Point::new(4.2, 5.0), Some(&selected)),
        Some(point(&s, 1))
    );
}

/// A 0.45 m form: the click on its middle is 0.32 m from each corner grip (out of reach) and
/// would be 0.225 m from the middle of each side, in reach of that grip, if it were shown.
#[test]
fn a_small_form_can_be_clicked_in_its_middle() {
    let s = open(&[(4.0, 5.0), (4.45, 5.0), (4.45, 5.45), (4.0, 5.45)]);
    let selected = form(&s);

    assert_eq!(
        s.pick_for(Point::new(4.225, 5.225), Some(&selected)),
        Some(selected)
    );
}

/// A thin form, 0.4 m x 2 m: a click inside, 0.2 m from the middle of its long side, is out of
/// reach of that grip (radius 0.1) and takes the form.
#[test]
fn a_thin_form_can_be_clicked_beside_a_side_grip() {
    let s = open(&[(4.0, 5.0), (4.4, 5.0), (4.4, 7.0), (4.0, 7.0)]);
    let selected = form(&s);

    assert_eq!(
        s.pick_for(Point::new(4.2, 6.0), Some(&selected)),
        Some(selected)
    );
}

/// A 0.35 m square: its middle is 0.247 m from each corner, in reach of the corner grip only
/// by the pick tolerance, which does not count inside the form.
#[test]
fn a_tiny_form_can_be_clicked_in_its_middle() {
    let s = open(&[(4.0, 5.0), (4.35, 5.0), (4.35, 5.35), (4.0, 5.35)]);
    let selected = form(&s);

    assert_eq!(
        s.pick_for(Point::new(4.175, 5.175), Some(&selected)),
        Some(selected)
    );
}

/// Outside the form the tolerance stays: 0.25 m from the corner, beside the form.
#[test]
fn a_corner_grip_still_catches_beside_the_form() {
    let s = open(&[(4.0, 5.0), (4.35, 5.0), (4.35, 5.35), (4.0, 5.35)]);
    let selected = form(&s);

    assert_eq!(
        s.pick_for(Point::new(3.8, 4.85), Some(&selected)),
        Some(point(&s, 0))
    );
}

fn example() -> Session {
    let dir: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe");

    Session::open(&dir).unwrap()
}

/// The note `n1` of the example set, 2.4 m wide: just inside its card, 0.2 m from the width
/// grip, a click takes the card; on the grip it takes the grip.
#[test]
fn a_note_is_clicked_inside_beside_its_grip() {
    let s = example();
    let note = Source::Note("n1".into());
    let grip = s
        .display_list_for(Some(&note))
        .items
        .iter()
        .find_map(|i| match (&i.role, &i.shape) {
            (Role::NoteHandle, Shape::Circle { center, .. }) => Some(*center),
            _ => None,
        })
        .unwrap();

    assert_eq!(
        s.pick_for(Point::new(grip.x_m - 0.2, grip.y_m), Some(&note)),
        Some(note)
    );
    assert_eq!(
        s.pick_for(grip, Some(&Source::Note("n1".into()))),
        Some(Source::NoteWidth("n1".into()))
    );
}

/// The arrow `d1` from (6, 10) to (6, 7.5): a click on its stroke 0.15 m short of the end
/// takes the drawing, one 0.3 m beyond the end still takes the end point.
#[test]
fn a_drawing_is_clicked_on_its_stroke_beside_its_grip() {
    let s = example();
    let arrow = Source::Drawing("d1".into());

    assert_eq!(
        s.pick_click(Point::new(6.0, 7.65), Some(&arrow)),
        Some(arrow.clone())
    );
    assert_eq!(
        s.pick_for(Point::new(6.0, 7.2), Some(&arrow)),
        Some(Source::DrawingPoint {
            drawing: "d1".into(),
            index: 1
        })
    );
}
