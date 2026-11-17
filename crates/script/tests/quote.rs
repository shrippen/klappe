//! DB-64: quote anchors.
#![allow(clippy::unwrap_used)]

use klappe_script::{parse, quote_at, scene_states, Element, Found, LineKind, Quote, Script};

/// Looks the quote up in `new`; `old` is the scene the quote was made in.
fn find_in(old: &[Element], new: &[Element], q: &Quote) -> Found {
    let state = scene_states(&Script {
        title_page: vec![],
        elements: old.to_vec(),
    })
    .pop()
    .unwrap();
    klappe_script::find_quote(&state, new, q)
}

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

    assert_eq!(find_in(&s, &edited, &q), Found::At(9));
}

#[test]
fn the_occurrence_picks_among_equal_lines() {
    let s = scene(BASE);
    let first = quote_at(&s, 2).unwrap();
    let second = quote_at(&s, 7).unwrap();

    assert_eq!(find_in(&s, &s, &first), Found::At(2));
    assert_eq!(find_in(&s, &s, &second), Found::At(7));
}

#[test]
fn a_vanished_equal_line_is_missing() {
    let s = scene(BASE);
    let second = quote_at(&s, 7).unwrap();
    let edited = scene("ELISABETH\nIhr kommt spät.\n\nBURGVOGT\nGuten Tag.\n");

    // Occurrence 1 of the line is gone; the one left stands before the new line.
    assert_eq!(find_in(&s, &edited, &second), Found::Missing);
}

#[test]
fn a_reworded_line_is_found_when_it_is_clearly_the_most_similar() {
    let s = scene(BASE);
    let q = quote_at(&s, 0).unwrap();
    let edited = scene(&BASE.replace("schaut hinaus in den Hof", "blickt hinaus in den Hof"));

    assert_eq!(find_in(&s, &edited, &q), Found::At(0));
}

#[test]
fn a_reworded_line_of_a_renamed_speaker_is_offered() {
    let s = scene("ELISABETH\nDer Weg war vereist und lang.\n");
    let q = quote_at(&s, 1).unwrap();
    let edited = scene("ELISA\nDer Weg war vereist und sehr lang.\n");

    // A new speaker may be a rename or another line: the user confirms.
    assert_eq!(find_in(&s, &edited, &q), Found::Ambiguous(vec![1]));
}

#[test]
fn two_equally_similar_lines_are_ambiguous() {
    let s = scene("Er öffnet die schwere Tür zum Saal.\n");
    let q = quote_at(&s, 0).unwrap();
    let edited =
        scene("Er öffnet die schwere Tür zum Hof.\n\nEr öffnet die schwere Tür zum Keller.\n");

    assert_eq!(find_in(&s, &edited, &q), Found::Ambiguous(vec![0, 1]));
}

#[test]
fn a_rewritten_line_is_missing() {
    let s = scene(BASE);
    let q = quote_at(&s, 0).unwrap();

    assert_eq!(
        find_in(&s, &scene("Es klopft dreimal.\n"), &q),
        Found::Missing
    );
}

#[test]
fn a_line_of_another_kind_is_no_candidate() {
    let q = quote_at(&scene("Ihr kommt spät.\n"), 0).unwrap();
    let edited = scene("ELISABETH\nIhr kommt spät.\n");

    assert_eq!(q.kind, LineKind::Action);
    assert_eq!(
        find_in(&scene("Ihr kommt spät.\n"), &edited, &q),
        Found::Missing
    );
}

#[test]
fn equal_lines_inserted_before_the_last_line_do_not_shift_it() {
    // Review: ANNA "Ja." and BEN inserted before; the old second "Ja." is now the third.
    let old = "ANNA\nJa.\n\nBEN\nNein.\n\nANNA\nJa.\n";
    let q = quote_at(&scene(old), 5).unwrap();
    let new = scene(&format!("ANNA\nJa.\n\nBEN\nHm.\n\n{old}"));

    // The "Nein." before the line is still there, so the line after it is the one.
    assert_eq!(find_in(&scene(old), &new, &q), Found::At(9));
}

#[test]
fn equal_lines_inserted_before_and_after_are_ambiguous() {
    let q = quote_at(&scene("ANNA\nJa.\n\nEs klopft.\n"), 1).unwrap();
    let new =
        scene("ANNA\nJa.\n\nEr nickt.\n\nANNA\nJa.\n\nEs klopft.\n\nANNA\nJa.\n\nSie geht.\n");

    assert_eq!(
        find_in(&scene("ANNA\nJa.\n\nEs klopft.\n"), &new, &q),
        Found::Ambiguous(vec![1, 4])
    );
}

#[test]
fn equal_lines_with_the_same_neighbours_are_ambiguous() {
    let q = quote_at(&scene("ANNA\nJa.\n\nEs klopft.\n"), 1).unwrap();
    let new = scene("ANNA\nJa.\n\nEs klopft.\n\nANNA\nJa.\n\nEs klopft.\n");

    assert_eq!(
        find_in(&scene("ANNA\nJa.\n\nEs klopft.\n"), &new, &q),
        Found::Ambiguous(vec![1, 4])
    );
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

    assert_eq!(
        find_in(
            &scene("Es regnet.\n\nEr öffnet die schwere Tür zum Saal.\n\nEs klopft.\n"),
            &new,
            &q
        ),
        Found::At(1)
    );
}

#[test]
fn dash_variants_and_spaces_in_a_line_are_equal() {
    let q = quote_at(&scene("Er wartet – lange.\n"), 0).unwrap();

    let old = scene("Er wartet – lange.\n");
    assert_eq!(
        find_in(&old, &scene("Er  wartet — lange.\n"), &q),
        Found::At(0)
    );
}

#[test]
fn an_equal_line_appended_right_behind_the_anchor_is_ambiguous() {
    // The anchor was the last line; the neighbours of both lines are then partly equal lines.
    let q = quote_at(&scene("Tür.\n\nEs klopft.\n"), 1).unwrap();
    let new = scene("Tür.\n\nEs klopft.\n\nEs klopft.\n");

    assert_eq!(
        find_in(&scene("Tür.\n\nEs klopft.\n"), &new, &q),
        Found::Ambiguous(vec![1, 2])
    );
}

#[test]
fn an_equal_line_inserted_right_before_the_anchor_is_ambiguous() {
    let q = quote_at(&scene("Es klopft.\n\nTür.\n"), 0).unwrap();
    let new = scene("Es klopft.\n\nEs klopft.\n\nTür.\n");

    assert_eq!(
        find_in(&scene("Es klopft.\n\nTür.\n"), &new, &q),
        Found::Ambiguous(vec![0, 1])
    );
}

#[test]
fn a_dialogue_appended_at_the_scene_end_is_ambiguous() {
    // The anchor is the only "Ja."; an equal one follows at the end. Both have the cue
    // above and the scene's edge below.
    let q = quote_at(&scene("ANNA\nJa.\n"), 1).unwrap();
    let new = scene("ANNA\nJa.\n\nBEN\nHm.\n\nANNA\nJa.\n");

    assert_eq!(
        find_in(&scene("ANNA\nJa.\n"), &new, &q),
        Found::Ambiguous(vec![1, 5])
    );
}

#[test]
fn swapped_exchanges_are_not_guessed_by_order() {
    // Round 2: two exchanges swapped; the "Ja." lines are equal, only the neighbours differ.
    let old = "ANNA\nJa.\n\nBEN\nNein.\n\nANNA\nJa.\n\nBEN\nDoch.\n";
    let new = "ANNA\nJa.\n\nBEN\nDoch.\n\nANNA\nJa.\n\nBEN\nNein.\n";
    let q = quote_at(&scene(old), 1).unwrap();

    assert_eq!(
        find_in(&scene(old), &scene(new), &q),
        Found::Ambiguous(vec![1, 5])
    );
}

#[test]
fn a_gap_that_spans_paired_lines_is_ambiguous() {
    // Round 3: "Drei." moved up between the neighbours of both "Ja." lines; the gap
    // between their partners then holds paired lines, and both quotes landed on one line.
    let old = scene("Eins.\n\nANNA\nJa.\n\nZwei.\n\nDrei.\n\nANNA\nJa.\n");
    let new = scene("Eins.\n\nDrei.\n\nANNA\nJa.\n\nZwei.\n");

    for index in [2, 6] {
        let q = quote_at(&old, index).unwrap();
        assert_eq!(
            find_in(&old, &new, &q),
            Found::Ambiguous(vec![3]),
            "line {index}"
        );
    }
}
