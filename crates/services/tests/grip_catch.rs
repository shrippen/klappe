//! Which grip a click takes when several catch it, and the grips too small to show.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use klappe_services::{ObjectGrip, PlanEdit, Point, Session, Snapping, Source};

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
