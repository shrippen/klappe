//! DB-41, DB-42, DB-44: comparing two states of a book.
#![allow(clippy::unwrap_used)]

use klappe_script::{compare, Change, Comparison, Place, Status};

/// A scene of five action lines that only this `tag` has.
fn scene(number: Option<&str>, head: &str, tag: &str) -> String {
    let num = number.map(|n| format!(" #{n}#")).unwrap_or_default();
    let mut s = format!("INT. {head} – TAG{num}\n\n");
    for i in 1..=5 {
        s.push_str(&format!(
            "{tag} geht durch den Raum und tut Ding {i} {tag}.\n\n"
        ));
    }
    s
}

const A: &str = "Alpha";
const B: &str = "Beta";
const C: &str = "Gamma";

fn book(scenes: &[(&str, &str)]) -> String {
    scenes
        .iter()
        .enumerate()
        .map(|(i, (head, tag))| scene(Some(&(i + 1).to_string()), head, tag))
        .collect()
}

fn statuses(c: &Comparison) -> Vec<(Status, Place)> {
    c.scenes.iter().map(|s| (s.status, s.place)).collect()
}

const THREE: [(&str, &str); 3] = [("SAAL", A), ("KÜCHE", B), ("HOF", C)];

#[test]
fn equal_books_are_all_the_same() {
    let text = book(&THREE);
    let c = compare(&text, &text);

    assert_eq!(statuses(&c), vec![(Status::Same, Place::Kept); 3]);
    assert!(c.marked_lines().is_empty());
}

#[test]
fn two_empty_books_have_no_scenes() {
    assert!(compare("", "").scenes.is_empty());
}

#[test]
fn from_an_empty_book_every_scene_is_new() {
    let c = compare("", &book(&THREE));

    assert_eq!(statuses(&c), vec![(Status::New, Place::Kept); 3]);
    assert!(c.scenes[0].lines.iter().all(|l| l.change == Change::Added));
}

#[test]
fn to_an_empty_book_every_scene_is_removed() {
    let c = compare(&book(&THREE), "");

    assert_eq!(statuses(&c), vec![(Status::Removed, Place::Kept); 3]);
    assert!(c.marked_lines().is_empty());
}

#[test]
fn a_scene_at_the_start_is_new_and_renumbering_changes_nothing() {
    let new = book(&[("NEU", "Delta"), ("SAAL", A), ("KÜCHE", B), ("HOF", C)]);
    let c = compare(&book(&THREE), &new);

    assert_eq!(
        statuses(&c),
        vec![
            (Status::New, Place::Kept),
            (Status::Same, Place::Kept),
            (Status::Same, Place::Kept),
            (Status::Same, Place::Kept),
        ]
    );
    assert_eq!(c.scenes[1].number.as_deref(), Some("2"));
}

#[test]
fn a_removed_scene_stands_after_the_one_before_it() {
    let c = compare(&book(&THREE), &book(&[("SAAL", A), ("HOF", C)]));

    assert_eq!(
        statuses(&c).iter().map(|s| s.0).collect::<Vec<_>>(),
        vec![Status::Same, Status::Removed, Status::Same]
    );
    assert_eq!(c.scenes[1].heading.as_deref(), Some("INT. KÜCHE – TAG"));
}

#[test]
fn a_removed_first_scene_comes_first() {
    let c = compare(&book(&THREE), &book(&[("KÜCHE", B), ("HOF", C)]));

    assert_eq!(c.scenes[0].status, Status::Removed);
    assert_eq!(c.scenes.len(), 3);
}

#[test]
fn a_changed_line_is_one_removed_and_one_added() {
    let old = book(&THREE);
    let new = old.replace(
        "Beta geht durch den Raum und tut Ding 3 Beta.",
        "Beta schläft.",
    );
    let c = compare(&old, &new);

    assert_eq!(c.scenes[1].status, Status::Changed);
    let lines = &c.scenes[1].lines;
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0].change, Change::Removed);
    assert_eq!(
        lines[0].text,
        "Beta geht durch den Raum und tut Ding 3 Beta."
    );
    assert_eq!(lines[1].change, Change::Added);
    assert_eq!(lines[1].text, "Beta schläft.");

    // The added line is the one in the new text.
    let at = new.lines().position(|l| l == "Beta schläft.").unwrap();
    assert_eq!(c.marked_lines(), vec![at]);
    assert_eq!(c.scenes[0].status, Status::Same);
}

#[test]
fn a_scene_that_only_moved_is_moved_and_unmarked() {
    let old = book(&THREE);
    let new = book(&[("KÜCHE", B), ("HOF", C), ("SAAL", A)]);
    let c = compare(&old, &new);

    assert_eq!(
        statuses(&c),
        vec![
            (Status::Same, Place::Kept),
            (Status::Same, Place::Kept),
            (Status::Same, Place::Moved),
        ]
    );
    assert!(c.marked_lines().is_empty());
}

#[test]
fn a_scene_that_moved_and_changed_is_both() {
    let old = book(&THREE);
    let new = book(&[("KÜCHE", B), ("HOF", C), ("SAAL", A)]).replace(
        "Alpha geht durch den Raum und tut Ding 1 Alpha.",
        "Alpha bleibt.",
    );
    let c = compare(&old, &new);

    assert_eq!(
        (c.scenes[2].status, c.scenes[2].place),
        (Status::Changed, Place::Moved)
    );
    assert_eq!(c.marked_lines().len(), 1);
}

#[test]
fn a_heading_that_changed_alone_is_a_change() {
    let old = book(&THREE);
    let new = old.replace("KÜCHE", "KELLER");
    let c = compare(&old, &new);

    assert_eq!(c.scenes[1].status, Status::Changed);
    assert_eq!(c.scenes[1].lines.len(), 2);
}

#[test]
fn twin_scenes_are_not_guessed() {
    // The same scene twice: align cannot tell which is which, so nothing is paired.
    let twin = scene(None, "SAAL", A);
    let old = format!("{twin}{}", scene(None, "HOF", C));
    let new = format!("{twin}{twin}");
    let c = compare(&old, &new);

    assert!(c.scenes.iter().any(|s| s.status == Status::New));
}
