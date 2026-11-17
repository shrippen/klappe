//! The grip that sets the width of a note's card (LH-16).
#![allow(clippy::unwrap_used)]

use klappe_model::{FloorPlan, Note, Point, Project, SetPlan};
use klappe_stellprobe::{handles, Role, Shape, Source};

fn set(width_m: f64) -> SetPlan {
    SetPlan {
        notes: vec![Note {
            id: "n1".into(),
            at: Point::new(3.0, 4.0),
            text: "Vorhang schließen".into(),
            width_m,
            bound: None,
        }],
        ..Default::default()
    }
}

/// The grip of a selected note sits in the middle of the card's right edge, whatever the
/// width; one line of text makes the card 0.4 m high.
#[test]
fn a_selected_note_has_its_width_grip_at_the_right_edge() {
    let project = Project::new("Test");
    let note = Source::Note("n1".into());

    for (width_m, edge_m) in [(4.0, 7.0), (2.4, 5.4)] {
        let grips = handles(
            &project,
            &set(width_m),
            &FloorPlan::new("a", "Plan A"),
            &note,
        );

        assert_eq!(grips.len(), 1);
        assert_eq!(grips[0].role, Role::Handle);
        assert_eq!(grips[0].source, Some(Source::NoteWidth("n1".into())));
        assert!(matches!(
            grips[0].shape,
            Shape::Circle { center, .. } if center == Point::new(edge_m, 4.2)
        ));
    }
}

/// A grip follows the note while it is dragged, and other elements have none of it.
#[test]
fn the_grip_stays_while_dragged_and_other_elements_have_none() {
    let project = Project::new("Test");
    let plan = FloorPlan::new("a", "Plan A");
    let grip = Source::NoteWidth("n1".into());

    assert_eq!(handles(&project, &set(2.4), &plan, &grip).len(), 1);
    assert!(handles(&project, &set(2.4), &plan, &Source::Wall(0)).is_empty());
    assert!(handles(&project, &set(2.4), &plan, &Source::Note("zz".into())).is_empty());
}
