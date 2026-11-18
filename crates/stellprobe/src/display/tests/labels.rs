use super::*;

/// A label of a figure (LH-118), kept in the plan.
fn label() -> Note {
    Note {
        id: "t1".into(),
        at: Point::new(5.0, 6.0),
        text: "Auftritt".into(),
        width_m: 2.4,
        rotation_deg: 0.0,
        bound: Some("figure:elisabeth".into()),
    }
}

/// A plan's labels are drawn as notes, on top, and a selected one has the width grip of a note.
#[test]
fn plan_labels_are_drawn_as_notes_with_their_grip() {
    let mut p = plan();
    p.labels = vec![label()];

    let list = render(&project(), &set(), &p, 0.0, &|_| None);
    let last = list.items.last().unwrap();

    assert_eq!(last.role, Role::Note);
    assert_eq!(last.label.as_deref(), Some("Auftritt"));
    assert_eq!(last.source, Some(Source::Note("t1".into())));

    let grips = handles(&project(), &set(), &p, &Source::Note("t1".into()));

    assert_eq!(grips.len(), 1);
    assert_eq!(grips[0].source, Some(Source::NoteWidth("t1".into())));
}
