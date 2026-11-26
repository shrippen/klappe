//! DB-41, DB-42, DB-44: comparing two states of a book.
#![allow(clippy::unwrap_used)]

use klappe_script::{compare, Comparison, Edit, ScenePlace, SceneStatus};

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

fn statuses(c: &Comparison) -> Vec<(SceneStatus, ScenePlace)> {
    c.scenes.iter().map(|s| (s.status, s.place)).collect()
}

const THREE: [(&str, &str); 3] = [("SAAL", A), ("KÜCHE", B), ("HOF", C)];

#[test]
fn equal_books_are_all_the_same() {
    let text = book(&THREE);
    let c = compare(&text, &text);

    assert_eq!(statuses(&c), vec![(SceneStatus::Same, ScenePlace::Kept); 3]);
    assert!(c.marked_lines().is_empty());
}

#[test]
fn two_empty_books_have_no_scenes() {
    assert!(compare("", "").scenes.is_empty());
}

#[test]
fn from_an_empty_book_every_scene_is_new() {
    let c = compare("", &book(&THREE));

    assert_eq!(statuses(&c), vec![(SceneStatus::New, ScenePlace::Kept); 3]);
    assert!(c.scenes[0].lines.iter().all(|l| l.change == Edit::Added));
}

#[test]
fn to_an_empty_book_every_scene_is_removed() {
    let c = compare(&book(&THREE), "");

    assert_eq!(
        statuses(&c),
        vec![(SceneStatus::Removed, ScenePlace::Kept); 3]
    );
    assert!(c.marked_lines().is_empty());
}

#[test]
fn a_scene_at_the_start_is_new_and_renumbering_changes_nothing() {
    let new = book(&[("NEU", "Delta"), ("SAAL", A), ("KÜCHE", B), ("HOF", C)]);
    let c = compare(&book(&THREE), &new);

    assert_eq!(
        statuses(&c),
        vec![
            (SceneStatus::New, ScenePlace::Kept),
            (SceneStatus::Same, ScenePlace::Kept),
            (SceneStatus::Same, ScenePlace::Kept),
            (SceneStatus::Same, ScenePlace::Kept),
        ]
    );
    assert_eq!(c.scenes[1].number.as_deref(), Some("2"));
}

#[test]
fn a_removed_scene_stands_after_the_one_before_it() {
    let c = compare(&book(&THREE), &book(&[("SAAL", A), ("HOF", C)]));

    assert_eq!(
        statuses(&c).iter().map(|s| s.0).collect::<Vec<_>>(),
        vec![SceneStatus::Same, SceneStatus::Removed, SceneStatus::Same]
    );
    assert_eq!(c.scenes[1].heading.as_deref(), Some("INT. KÜCHE – TAG"));
}

#[test]
fn a_removed_first_scene_comes_first() {
    let c = compare(&book(&THREE), &book(&[("KÜCHE", B), ("HOF", C)]));

    assert_eq!(c.scenes[0].status, SceneStatus::Removed);
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

    assert_eq!(c.scenes[1].status, SceneStatus::Changed);
    let lines = &c.scenes[1].lines;
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0].change, Edit::Removed);
    assert_eq!(
        lines[0].text,
        "Beta geht durch den Raum und tut Ding 3 Beta."
    );
    assert_eq!(lines[1].change, Edit::Added);
    assert_eq!(lines[1].text, "Beta schläft.");

    // The added line is the one in the new text.
    let at = new.lines().position(|l| l == "Beta schläft.").unwrap();
    assert_eq!(c.marked_lines(), vec![at]);
    assert_eq!(c.scenes[0].status, SceneStatus::Same);
}

#[test]
fn a_scene_that_only_moved_is_moved_and_unmarked() {
    let old = book(&THREE);
    let new = book(&[("KÜCHE", B), ("HOF", C), ("SAAL", A)]);
    let c = compare(&old, &new);

    assert_eq!(
        statuses(&c),
        vec![
            (SceneStatus::Same, ScenePlace::Kept),
            (SceneStatus::Same, ScenePlace::Kept),
            (SceneStatus::Same, ScenePlace::Moved),
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
        (SceneStatus::Changed, ScenePlace::Moved)
    );
    assert_eq!(c.marked_lines().len(), 1);
}

#[test]
fn a_heading_that_changed_alone_is_a_change() {
    let old = book(&THREE);
    let new = old.replace("KÜCHE", "KELLER");
    let c = compare(&old, &new);

    assert_eq!(c.scenes[1].status, SceneStatus::Changed);
    assert_eq!(c.scenes[1].lines.len(), 2);
}

#[test]
fn twin_scenes_are_not_guessed() {
    // The same scene twice: align cannot tell which is which, so nothing is paired.
    let twin = scene(None, "SAAL", A);
    let old = format!("{twin}{}", scene(None, "HOF", C));
    let new = format!("{twin}{twin}");
    let c = compare(&old, &new);

    assert!(c.scenes.iter().any(|s| s.status == SceneStatus::New));
}

#[test]
fn a_struck_heading_belongs_to_the_removed_scene() {
    let old = book(&THREE);
    let new = old.replace(&scene(Some("2"), "KÜCHE", B), ".ENTFÄLLT #2#\n\n");
    assert_ne!(old, new);
    let c = compare(&old, &new);

    let kinds: Vec<SceneStatus> = c.scenes.iter().map(|s| s.status).collect();
    assert_eq!(
        kinds,
        vec![SceneStatus::Same, SceneStatus::Removed, SceneStatus::Same]
    );
}

#[test]
fn notes_and_boneyards_are_not_compared() {
    let old = book(&THREE);
    let new = old
        .replace("Ding 1 Beta.\n", "Ding 1 Beta. [[Regie: lauter]]\n")
        .replace(
            "Ding 2 Gamma.\n",
            "Ding 2 Gamma.\n\n/* gestrichen\nzwei Zeilen */\n",
        )
        .replace("Ding 3 Alpha.\n", "Ding 3 Alpha.\n\n[[Eine Notiz.]]\n");
    assert_ne!(old, new);
    let c = compare(&old, &new);

    assert_eq!(statuses(&c), vec![(SceneStatus::Same, ScenePlace::Kept); 3]);
    assert!(c.marked_lines().is_empty());
}

/// Two edits in one scene read as two hunks: each removed line stands right above the line
/// that replaces it, not all removed lines first.
#[test]
fn each_removed_line_stands_above_its_replacement() {
    let old = book(&THREE);
    let new = old
        .replace("tut Ding 2 Beta.", "tut etwas anderes.")
        .replace("tut Ding 4 Beta.", "schläft.");
    let c = compare(&old, &new);

    let changes: Vec<_> = c.scenes[1]
        .lines
        .iter()
        .map(|l| (l.change, l.text.as_str()))
        .collect();
    assert_eq!(
        changes,
        vec![
            (
                Edit::Removed,
                "Beta geht durch den Raum und tut Ding 2 Beta."
            ),
            (
                Edit::Added,
                "Beta geht durch den Raum und tut etwas anderes."
            ),
            (
                Edit::Removed,
                "Beta geht durch den Raum und tut Ding 4 Beta."
            ),
            (Edit::Added, "Beta geht durch den Raum und schläft."),
        ]
    );
}
