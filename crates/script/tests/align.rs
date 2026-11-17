//! DB-65: matching two script states.
#![allow(clippy::unwrap_used)]

use klappe_script::{align, parse, scene_states, How, Outcome, SceneState};

/// A scene of five action lines that only this `tag` has, plus one dialogue.
fn scene(number: Option<&str>, head: &str, tag: &str) -> String {
    let num = number.map(|n| format!(" #{n}#")).unwrap_or_default();
    let mut s = format!("INT. {head} – TAG{num}\n\n");
    for i in 1..=5 {
        s.push_str(&format!(
            "{tag} geht durch den Raum und tut Ding {i} {tag}.\n\n"
        ));
    }
    s.push_str(&format!("ELISABETH\nDas ist {tag}, nicht wahr?\n\n"));
    s
}

fn old_of(text: &str) -> Vec<SceneState> {
    scene_states(&parse(text))
}

fn matched(scene: usize, how: How) -> Outcome {
    Outcome::Matched { scene, how }
}

const A: &str = "Alpha";
const B: &str = "Beta";
const C: &str = "Gamma";

fn three() -> String {
    [
        scene(Some("1"), "SAAL", A),
        scene(Some("2"), "KÜCHE", B),
        scene(Some("3"), "HOF", C),
    ]
    .concat()
}

#[test]
fn an_unchanged_script_matches_by_number() {
    let al = align(&old_of(&three()), &parse(&three()));

    assert_eq!(
        al.old,
        vec![
            matched(0, How::Number),
            matched(1, How::Number),
            matched(2, How::Number)
        ]
    );
    assert!(al.unmatched.is_empty());
}

#[test]
fn a_scene_inserted_before_shifts_the_numbers() {
    let old = old_of(&three());
    let new = [
        scene(Some("1"), "NEU", "Delta"),
        scene(Some("2"), "SAAL", A),
        scene(Some("3"), "KÜCHE", B),
        scene(Some("4"), "HOF", C),
    ]
    .concat();
    let al = align(&old, &parse(&new));

    assert_eq!(
        al.old,
        vec![
            matched(1, How::Content),
            matched(2, How::Content),
            matched(3, How::Content)
        ]
    );
    assert_eq!(al.unmatched, vec![0]);
}

#[test]
fn a_scene_inserted_before_with_the_same_heading_does_not_steal_the_number() {
    let old = old_of(&three());
    let new = [
        scene(Some("1"), "SAAL", "Delta"),
        scene(Some("2"), "SAAL", A),
        scene(Some("3"), "KÜCHE", B),
        scene(Some("4"), "HOF", C),
    ]
    .concat();
    let al = align(&old, &parse(&new));

    assert_eq!(al.old[0], matched(1, How::Content));
    assert_eq!(al.unmatched, vec![0]);
}

#[test]
fn a_moved_scene_keeps_its_number() {
    let old = old_of(&three());
    let new = [
        scene(Some("3"), "HOF", C),
        scene(Some("1"), "SAAL", A),
        scene(Some("2"), "KÜCHE", B),
    ]
    .concat();
    let al = align(&old, &parse(&new));

    assert_eq!(
        al.old,
        vec![
            matched(1, How::Number),
            matched(2, How::Number),
            matched(0, How::Number)
        ]
    );
}

#[test]
fn a_scene_renumbered_by_another_program_is_found_by_content() {
    let old = old_of(&three());
    let new = [
        scene(Some("10"), "SAAL", A),
        scene(Some("20"), "KÜCHE", B),
        scene(Some("30"), "HOF", C),
    ]
    .concat();
    let al = align(&old, &parse(&new));

    assert_eq!(
        al.old,
        vec![
            matched(0, How::Content),
            matched(1, How::Content),
            matched(2, How::Content)
        ]
    );
}

#[test]
fn a_scene_that_lost_its_number_is_found_by_content() {
    let old = old_of(&three());
    let new = [
        scene(None, "SAAL", A),
        scene(None, "KÜCHE", B),
        scene(None, "HOF", C),
    ]
    .concat();

    assert_eq!(align(&old, &parse(&new)).old[1], matched(1, How::Content));
}

#[test]
fn a_reworded_heading_is_confirmed_by_content() {
    let old = old_of(&three());
    let new = [
        scene(Some("1"), "SAAL", A),
        scene(Some("2"), "GROSSE KÜCHE", B),
        scene(Some("3"), "HOF", C),
    ]
    .concat();

    assert_eq!(align(&old, &parse(&new)).old[1], matched(1, How::Number));
}

#[test]
fn a_heavily_rewritten_scene_is_confirmed_by_number_and_heading() {
    let old = old_of(&three());
    let new = [
        scene(Some("1"), "SAAL", A),
        scene(Some("2"), "KÜCHE", "Omega"),
        scene(Some("3"), "HOF", C),
    ]
    .concat();

    assert_eq!(align(&old, &parse(&new)).old[1], matched(1, How::Number));
}

#[test]
fn a_rewritten_scene_with_a_reworded_heading_is_not_found() {
    let old = old_of(&three());
    let new = [
        scene(Some("1"), "SAAL", A),
        scene(Some("2"), "KELLER", "Omega"),
        scene(Some("3"), "HOF", C),
    ]
    .concat();
    let al = align(&old, &parse(&new));

    assert_eq!(al.old[1], Outcome::NotFound);
    assert_eq!(al.unmatched, vec![1]);
}

#[test]
fn a_scene_split_in_two_keeps_the_number_on_the_first_half() {
    let old = old_of(&three());
    let first: String = scene(Some("2"), "KÜCHE", B)
        .lines()
        .take(7)
        .collect::<Vec<_>>()
        .join("\n");
    let rest = format!(
        "\n\nINT. KÜCHE – ABEND\n\n{}\n",
        scene(None, "X", B)
            .lines()
            .skip(7)
            .collect::<Vec<_>>()
            .join("\n")
    );
    let new = [
        scene(Some("1"), "SAAL", A),
        format!("{first}{rest}"),
        scene(Some("3"), "HOF", C),
    ]
    .concat();
    let al = align(&old, &parse(&new));

    assert_eq!(al.old[1], matched(1, How::Number));
    assert_eq!(al.unmatched, vec![2]);
}

#[test]
fn a_scene_split_in_two_without_numbers_is_ambiguous() {
    let old = old_of(&[scene(None, "KÜCHE", B)].concat());
    let lines: Vec<String> = scene(None, "KÜCHE", B)
        .lines()
        .map(str::to_string)
        .collect();
    let new = format!(
        "{}\n\nINT. KÜCHE – ABEND\n\n{}\n",
        lines[..7].join("\n"),
        lines[7..].join("\n")
    );
    let al = align(&old, &parse(&new));

    // Best first: the larger half scores higher.
    assert_eq!(al.old, vec![Outcome::Ambiguous(vec![1, 0])]);
}

#[test]
fn a_duplicated_scene_matches_the_original_and_leaves_the_copy() {
    let old = old_of(&three());
    let new = [
        scene(Some("1"), "SAAL", A),
        scene(Some("2"), "KÜCHE", B),
        scene(Some("2A"), "KÜCHE", B),
        scene(Some("3"), "HOF", C),
    ]
    .concat();
    let al = align(&old, &parse(&new));

    assert_eq!(al.old[1], matched(1, How::Number));
    assert_eq!(al.unmatched, vec![2]);
}

#[test]
fn a_duplicated_scene_without_numbers_is_ambiguous() {
    let old = old_of(&[scene(None, "SAAL", A), scene(None, "KÜCHE", B)].concat());
    let new = [
        scene(None, "SAAL", A),
        scene(None, "KÜCHE", B),
        scene(None, "KÜCHE", B),
    ]
    .concat();
    let al = align(&old, &parse(&new));

    assert_eq!(al.old[0], matched(0, How::Content));
    assert_eq!(al.old[1], Outcome::Ambiguous(vec![1, 2]));
}

#[test]
fn a_deleted_scene_is_not_found() {
    let old = old_of(&three());
    let new = [scene(Some("1"), "SAAL", A), scene(Some("3"), "HOF", C)].concat();
    let al = align(&old, &parse(&new));

    assert_eq!(
        al.old,
        vec![
            matched(0, How::Number),
            Outcome::NotFound,
            matched(1, How::Number)
        ]
    );
    assert!(al.unmatched.is_empty());
}

#[test]
fn two_identical_scenes_keep_their_numbers() {
    let text = [scene(Some("1"), "FLUR", A), scene(Some("2"), "FLUR", A)].concat();
    let al = align(&old_of(&text), &parse(&text));

    assert_eq!(
        al.old,
        vec![matched(0, How::Number), matched(1, How::Number)]
    );
}

#[test]
fn two_identical_scenes_renumbered_are_ambiguous() {
    let old = old_of(&[scene(Some("1"), "FLUR", A), scene(Some("2"), "FLUR", A)].concat());
    let new = [scene(Some("8"), "FLUR", A), scene(Some("9"), "FLUR", A)].concat();
    let al = align(&old, &parse(&new));

    assert_eq!(al.old, vec![Outcome::Ambiguous(vec![0, 1]); 2]);
}

#[test]
fn two_old_twins_and_one_new_scene_are_ambiguous() {
    let old = old_of(&[scene(None, "FLUR", A), scene(None, "FLUR", A)].concat());
    let al = align(&old, &parse(&scene(None, "FLUR", A)));

    assert_eq!(al.old, vec![Outcome::Ambiguous(vec![0]); 2]);
}

#[test]
fn fingerprints_ignore_case_spacing_and_notes() {
    let a = old_of("INT. A\n\nEr  geht.\n\n[[Note]]\n");
    let b = old_of("INT. A\n\ner geht.\n");

    assert_eq!(a[0].prints, b[0].prints);
}
