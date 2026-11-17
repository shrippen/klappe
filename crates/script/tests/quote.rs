//! DB-64: quote anchors.
#![allow(clippy::unwrap_used)]

use klappe_script::{find_quote, parse, quote_at, Element, Found, LineKind};

fn scene(text: &str) -> Vec<Element> {
    parse(&format!("INT. SAAL\n\n{text}")).elements[1..].to_vec()
}

const BASE: &str = "Sie geht zum Fenster und schaut hinaus in den Hof.

ELISABETH
Ihr kommt spät.

BURGVOGT
(ohne aufzusehen)
Ihr kommt spät.

ELISABETH
Ihr kommt spät.
";

#[test]
fn the_quote_of_a_dialogue_names_the_speaker() {
    let s = scene(BASE);
    let q = quote_at(&s, 5).unwrap();

    assert_eq!(q.kind, LineKind::Dialogue);
    assert_eq!(q.character.as_deref(), Some("BURGVOGT"));
    assert_eq!(q.text, "Ihr kommt spät.");
    assert_eq!(q.occurrence, 0);
}

#[test]
fn equal_lines_are_counted() {
    let s = scene(BASE);

    // Elisabeth's two equal lines: the second is occurrence 1.
    assert_eq!(quote_at(&s, 2).unwrap().occurrence, 0);
    assert_eq!(quote_at(&s, 7).unwrap().occurrence, 1);
}

#[test]
fn a_page_break_has_no_quote() {
    assert_eq!(quote_at(&scene("===\n"), 0), None);
}

#[test]
fn an_exact_line_is_found_after_lines_were_inserted_before() {
    let s = scene(BASE);
    let q = quote_at(&s, 7).unwrap();
    let edited = scene(&format!("Es regnet.\n\nNoch eine Zeile.\n\n{BASE}"));

    assert_eq!(find_quote(&edited, &q), Found::At(9));
}

#[test]
fn the_occurrence_picks_among_equal_lines() {
    let s = scene(BASE);
    let first = quote_at(&s, 2).unwrap();
    let second = quote_at(&s, 7).unwrap();

    assert_eq!(find_quote(&s, &first), Found::At(2));
    assert_eq!(find_quote(&s, &second), Found::At(7));
}

#[test]
fn a_vanished_equal_line_is_ambiguous() {
    let s = scene(BASE);
    let second = quote_at(&s, 7).unwrap();
    let edited = scene("ELISABETH\nIhr kommt spät.\n\nBURGVOGT\nGuten Tag.\n");

    // Occurrence 1 of the line is gone; the one left may be either.
    assert_eq!(find_quote(&edited, &second), Found::Ambiguous(vec![1]));
}

#[test]
fn a_reworded_line_is_found_when_it_is_clearly_the_most_similar() {
    let s = scene(BASE);
    let q = quote_at(&s, 0).unwrap();
    let edited = scene(&BASE.replace("schaut hinaus in den Hof", "blickt hinaus in den Hof"));

    assert_eq!(find_quote(&edited, &q), Found::At(0));
}

#[test]
fn a_reworded_line_of_a_renamed_speaker_is_found() {
    let s = scene("ELISABETH\nDer Weg war vereist und lang.\n");
    let q = quote_at(&s, 1).unwrap();
    let edited = scene("ELISA\nDer Weg war vereist und sehr lang.\n");

    assert_eq!(find_quote(&edited, &q), Found::At(1));
}

#[test]
fn two_equally_similar_lines_are_ambiguous() {
    let s = scene("Er öffnet die schwere Tür zum Saal.\n");
    let q = quote_at(&s, 0).unwrap();
    let edited =
        scene("Er öffnet die schwere Tür zum Hof.\n\nEr öffnet die schwere Tür zum Keller.\n");

    assert_eq!(find_quote(&edited, &q), Found::Ambiguous(vec![0, 1]));
}

#[test]
fn a_rewritten_line_is_missing() {
    let s = scene(BASE);
    let q = quote_at(&s, 0).unwrap();

    assert_eq!(
        find_quote(&scene("Es klopft dreimal.\n"), &q),
        Found::Missing
    );
}

#[test]
fn a_line_of_another_kind_is_no_candidate() {
    let q = quote_at(&scene("Ihr kommt spät.\n"), 0).unwrap();
    let edited = scene("ELISABETH\nIhr kommt spät.\n");

    assert_eq!(q.kind, LineKind::Action);
    assert_eq!(find_quote(&edited, &q), Found::Missing);
}

#[test]
fn equal_lines_inserted_before_the_last_line_are_ambiguous() {
    // Review: ANNA "Ja." and BEN inserted before; the old second "Ja." is now the third.
    let old = "ANNA\nJa.\n\nBEN\nNein.\n\nANNA\nJa.\n";
    let q = quote_at(&scene(old), 5).unwrap();
    let new = scene(&format!("ANNA\nJa.\n\nBEN\nHm.\n\n{old}"));

    // The line at the scene's end looks the same whether pairs were added before or
    // after it: no pick.
    assert_eq!(find_quote(&new, &q), Found::Ambiguous(vec![1, 5, 9]));
}

#[test]
fn equal_lines_inserted_before_and_after_are_told_apart_by_neighbours() {
    let q = quote_at(&scene("ANNA\nJa.\n\nEs klopft.\n"), 1).unwrap();
    let new =
        scene("ANNA\nJa.\n\nEr nickt.\n\nANNA\nJa.\n\nEs klopft.\n\nANNA\nJa.\n\nSie geht.\n");

    assert_eq!(find_quote(&new, &q), Found::At(4));
}

#[test]
fn equal_lines_with_the_same_neighbours_are_ambiguous() {
    let q = quote_at(&scene("ANNA\nJa.\n\nEs klopft.\n"), 1).unwrap();
    let new = scene("ANNA\nJa.\n\nEs klopft.\n\nANNA\nJa.\n\nEs klopft.\n");

    assert_eq!(find_quote(&new, &q), Found::Ambiguous(vec![1, 4]));
}

#[test]
fn neighbours_break_a_tie_between_similar_lines() {
    let q = quote_at(
        &scene("Es regnet.\n\nEr öffnet die schwere Tür zum Saal.\n\nEs klopft.\n"),
        1,
    )
    .unwrap();
    let new = scene(
        "Es schneit.\n\nEr öffnet die schwere Tür zum Hof.\n\nEs klopft.\n\n\
         Er öffnet die schwere Tür zum Keller.\n\nEs donnert.\n",
    );

    assert_eq!(find_quote(&new, &q), Found::At(1));
}

#[test]
fn dash_variants_and_spaces_in_a_line_are_equal() {
    let q = quote_at(&scene("Er wartet – lange.\n"), 0).unwrap();

    assert_eq!(
        find_quote(&scene("Er  wartet — lange.\n"), &q),
        Found::At(0)
    );
}

#[test]
fn an_equal_line_appended_right_behind_the_anchor_is_ambiguous() {
    // The anchor was the last line; the neighbours of both lines are then partly equal lines.
    let q = quote_at(&scene("Tür.\n\nEs klopft.\n"), 1).unwrap();
    let new = scene("Tür.\n\nEs klopft.\n\nEs klopft.\n");

    assert_eq!(find_quote(&new, &q), Found::Ambiguous(vec![1, 2]));
}

#[test]
fn an_equal_line_inserted_right_before_the_anchor_is_ambiguous() {
    let q = quote_at(&scene("Es klopft.\n\nTür.\n"), 0).unwrap();
    let new = scene("Es klopft.\n\nEs klopft.\n\nTür.\n");

    assert_eq!(find_quote(&new, &q), Found::Ambiguous(vec![0, 1]));
}

#[test]
fn a_dialogue_appended_at_the_scene_end_is_ambiguous() {
    // The anchor is the only "Ja."; an equal one follows at the end. Both have the cue
    // above and the scene's edge below.
    let q = quote_at(&scene("ANNA\nJa.\n"), 1).unwrap();
    let new = scene("ANNA\nJa.\n\nBEN\nHm.\n\nANNA\nJa.\n");

    assert_eq!(find_quote(&new, &q), Found::Ambiguous(vec![1, 5]));
}
