//! DB-65: matching two script states.
#![allow(clippy::unwrap_used)]

use klappe_script::{align, parse, scene_states, Outcome, SceneState};

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

fn matched(scene: usize) -> Outcome {
    Outcome::Matched { scene }
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

    assert_eq!(al.old, vec![matched(0), matched(1), matched(2)]);
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

    assert_eq!(al.old, vec![matched(1), matched(2), matched(3)]);
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

    assert_eq!(al.old[0], matched(1));
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

    assert_eq!(al.old, vec![matched(1), matched(2), matched(0)]);
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

    assert_eq!(al.old, vec![matched(0), matched(1), matched(2)]);
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

    assert_eq!(align(&old, &parse(&new)).old[1], matched(1));
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

    assert_eq!(align(&old, &parse(&new)).old[1], matched(1));
}

#[test]
fn a_heavily_rewritten_scene_with_number_and_heading_is_offered() {
    let old = old_of(&three());
    let new = [
        scene(Some("1"), "SAAL", A),
        scene(Some("2"), "KÜCHE", "Omega"),
        scene(Some("3"), "HOF", C),
    ]
    .concat();

    // Number and heading alone are no proof: the user is asked.
    assert_eq!(
        align(&old, &parse(&new)).old[1],
        Outcome::Ambiguous(vec![1])
    );
}

#[test]
fn a_rewritten_scene_with_a_reworded_heading_keeps_only_the_number() {
    let old = old_of(&three());
    let new = [
        scene(Some("1"), "SAAL", A),
        scene(Some("2"), "KELLER", "Omega"),
        scene(Some("3"), "HOF", C),
    ]
    .concat();
    let al = align(&old, &parse(&new));

    // The number is all that is left: offered, not taken.
    assert_eq!(al.old[1], Outcome::Ambiguous(vec![1]));
    assert_eq!(al.unmatched, vec![1]);
}

#[test]
fn a_scene_split_in_half_is_ambiguous_even_with_the_number_on_one_half() {
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

    // The first half has exactly half of the scene's lines: not more than half.
    assert_eq!(al.old[1], Outcome::Ambiguous(vec![1, 2]));
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

    assert_eq!(al.old, vec![Outcome::Ambiguous(vec![0, 1])]);
}

#[test]
fn a_duplicated_scene_cannot_tell_the_original_from_the_copy() {
    let old = old_of(&three());
    let new = [
        scene(Some("1"), "SAAL", A),
        scene(Some("2"), "KÜCHE", B),
        scene(Some("2A"), "KÜCHE", B),
        scene(Some("3"), "HOF", C),
    ]
    .concat();
    let al = align(&old, &parse(&new));

    // Numbers never confirm: both are offered, the rest is matched.
    assert_eq!(al.old[1], Outcome::Ambiguous(vec![1, 2]));
    assert_eq!(al.old[0], matched(0));
    assert_eq!(al.old[2], matched(3));
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

    assert_eq!(al.old[0], matched(0));
    assert_eq!(al.old[1], Outcome::Ambiguous(vec![1, 2]));
}

#[test]
fn a_deleted_scene_is_not_found() {
    let old = old_of(&three());
    let new = [scene(Some("1"), "SAAL", A), scene(Some("3"), "HOF", C)].concat();
    let al = align(&old, &parse(&new));

    assert_eq!(al.old, vec![matched(0), Outcome::NotFound, matched(1)]);
    assert!(al.unmatched.is_empty());
}

#[test]
fn two_identical_scenes_keep_their_numbers() {
    let text = [scene(Some("1"), "FLUR", A), scene(Some("2"), "FLUR", A)].concat();
    let al = align(&old_of(&text), &parse(&text));

    assert_eq!(al.old, vec![matched(0), matched(1)]);
}

#[test]
fn two_identical_scenes_renumbered_keep_their_order() {
    let old = old_of(&[scene(Some("1"), "FLUR", A), scene(Some("2"), "FLUR", A)].concat());
    let new = [scene(Some("8"), "FLUR", A), scene(Some("9"), "FLUR", A)].concat();
    let al = align(&old, &parse(&new));

    // The scripts are equal around the twins, so their order pairs them.
    assert_eq!(al.old, vec![matched(0), matched(1)]);
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

/// A scene of the given action lines.
fn lines_scene(number: Option<&str>, head: &str, lines: &[String]) -> String {
    let num = number.map(|n| format!(" #{n}#")).unwrap_or_default();
    let mut s = format!("INT. {head} – TAG{num}\n\n");
    for l in lines {
        s.push_str(&format!("{l}\n\n"));
    }
    s
}

/// `n` lines "<tag> Zeile i", all different from the lines of other tags.
fn tagged(tag: &str, n: usize) -> Vec<String> {
    (0..n)
        .map(|i| format!("{tag} Zeile {i} mit Text"))
        .collect()
}

fn dialogue_scene(number: &str, head: &str, lines: &[String]) -> String {
    let mut s = format!("INT. {head} – TAG #{number}#\n\n");
    for l in lines {
        s.push_str(&format!("ELISABETH\n{l}\n\n"));
    }
    s
}

#[test]
fn a_deleted_twin_heading_does_not_steal_the_number() {
    // Review: two KÜCHE scenes 2 and 3; scene 2 is deleted and the rest renumbered.
    let old = old_of(
        &[
            scene(Some("1"), "SAAL", A),
            scene(Some("2"), "KÜCHE", B),
            scene(Some("3"), "KÜCHE", C),
        ]
        .concat(),
    );
    let new = [scene(Some("1"), "SAAL", A), scene(Some("2"), "KÜCHE", C)].concat();
    let al = align(&old, &parse(&new));

    assert_eq!(al.old, vec![matched(0), Outcome::NotFound, matched(1)]);
    assert!(al.unmatched.is_empty());
}

#[test]
fn a_deleted_first_twin_renumbered_is_found_by_content() {
    let old = old_of(&[scene(Some("1"), "KÜCHE", B), scene(Some("2"), "KÜCHE", C)].concat());
    let new = scene(Some("1"), "KÜCHE", C);
    let al = align(&old, &parse(&new));

    assert_eq!(al.old, vec![Outcome::NotFound, matched(0)]);
}

#[test]
fn similar_dialogue_scenes_do_not_steal_the_number() {
    // Review: 14 of 16 elements shared (Dice 0.75), same heading, same number.
    let first: Vec<String> = (1..=8).map(|i| format!("Satz {i}")).collect();
    let mut second = first.clone();
    for (i, l) in second.iter_mut().enumerate().skip(4) {
        *l = format!("Anderer Satz {i}");
    }
    let old = old_of(
        &[
            dialogue_scene("2", "KÜCHE", &first),
            dialogue_scene("3", "KÜCHE", &second),
        ]
        .concat(),
    );
    let new = dialogue_scene("2", "KÜCHE", &second);
    let al = align(&old, &parse(&new));

    // Number and heading say 2, the content says 3: no silent pick, both are asked.
    assert_eq!(al.old, vec![Outcome::Ambiguous(vec![0]); 2]);
}

#[test]
fn an_inserted_scene_with_the_same_heading_does_not_take_a_rewritten_original() {
    let old = old_of(&scene(Some("1"), "SAAL", A));
    let new = [
        scene(Some("1"), "SAAL", "Delta"),
        scene(Some("2"), "SAAL", "Omega"),
    ]
    .concat();
    let al = align(&old, &parse(&new));

    assert!(
        !matches!(al.old[0], Outcome::Matched { .. }),
        "{:?}",
        al.old
    );
}

#[test]
fn a_near_twin_is_ambiguous() {
    // Review: one candidate at 0.8, the next at 0.6.
    let base = tagged("Basis", 10);
    let mut near = base.clone();
    near[8] = "Neu eins".into();
    near[9] = "Neu zwei".into();
    let mut far = base.clone();
    for (i, l) in far.iter_mut().enumerate().skip(6) {
        *l = format!("Fern {i}");
    }
    let old = old_of(&lines_scene(None, "ALT", &base));
    let new = [
        lines_scene(None, "EINS", &near),
        lines_scene(None, "ZWEI", &far),
    ]
    .concat();
    let al = align(&old, &parse(&new));

    // The 0.6 scene shares no line that is unique; the 0.8 one is offered, not taken.
    assert_eq!(al.old, vec![Outcome::Ambiguous(vec![0])]);
}

#[test]
fn a_clear_lead_over_the_second_is_a_match() {
    let base = tagged("Basis", 10);
    let mut near = base.clone();
    near[9] = "Neu".into();
    let old = old_of(&lines_scene(None, "ALT", &base));
    let new = [
        lines_scene(None, "EINS", &near),
        lines_scene(None, "ZWEI", &tagged("Fremd", 10)),
    ]
    .concat();

    assert_eq!(align(&old, &parse(&new)).old[0], matched(0));
}

#[test]
fn a_heading_only_scene_matches_by_number_and_heading() {
    let text = "INT. SAAL – TAG #1#\n\nINT. HOF – TAG #2#\n\n";

    assert_eq!(
        align(&old_of(text), &parse(text)).old,
        vec![matched(0), matched(1)]
    );
}

#[test]
fn dash_variants_and_spaces_in_headings_are_equal() {
    let old = old_of("INT. SAAL – TAG #1#\n\nEr geht.\n");
    let new = "INT.  SAAL — TAG #1#\n\nEr geht.\n";

    assert_eq!(align(&old, &parse(new)).old, vec![matched(0)]);
}

fn dialogue(number: &str, head: &str, lines: &[&str]) -> String {
    let lines: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
    let mut s = format!("INT. {head} – TAG #{number}#\n\n");
    for (i, l) in lines.iter().enumerate() {
        let who = if i % 2 == 0 { "ANNA" } else { "BEN" };
        s.push_str(&format!("{who}\n{l}\n\n"));
    }
    s
}

#[test]
fn a_cue_only_overlap_does_not_beat_the_content() {
    // Round 2: ANNA/BEN scene with the same heading inserted before; the original was
    // rewritten. The cue lines alone must not make the inserted scene the match.
    let old = old_of(
        &[
            scene(Some("1"), "SAAL", A),
            dialogue("2", "KÜCHE", &["Eins.", "Zwei.", "Drei.", "Vier."]),
        ]
        .concat(),
    );
    let new = [
        scene(Some("1"), "SAAL", A),
        dialogue(
            "2",
            "KÜCHE",
            &["Neu eins.", "Neu zwei.", "Neu drei.", "Neu vier."],
        ),
        dialogue("3", "KÜCHE", &["Fünf.", "Sechs.", "Sieben.", "Acht."]),
    ]
    .concat();
    let al = align(&old, &parse(&new));

    assert!(
        !matches!(al.old[1], Outcome::Matched { .. }),
        "{:?}",
        al.old
    );
}

#[test]
fn twins_with_a_scene_inserted_before_are_not_guessed() {
    // Round 2: two equal scenes, one scene inserted before (scores 1.4 against 1.1).
    let old = old_of(&[scene(Some("1"), "AUTO X", A), scene(Some("2"), "AUTO X", A)].concat());
    let new = [
        scene(Some("1"), "NEU", "Delta"),
        scene(Some("2"), "AUTO X", A),
        scene(Some("3"), "AUTO X", A),
    ]
    .concat();
    let al = align(&old, &parse(&new));

    assert!(
        al.old.iter().all(|o| !matches!(o, Outcome::Matched { .. })),
        "{:?}",
        al.old
    );
}
