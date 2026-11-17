//! The outline (DB-20, DB-21): tree, collapse, filter and moving blocks.

use crate::{move_entry, outline, parse, visible, write, Element, Entry, EntryKind, Way};

const TEXT: &str = "Title: X

# Akt I

= Der Anfang.

## Ankunft

INT. SAAL - TAG #1#

= Sie kommt an.

Text eins.

INT. HOF - TAG #2#

Text zwei.

## Abreise

INT. TOR - NACHT #3#

Text drei.

# Akt II

INT. KELLER - TAG #4#

Text vier.
";

fn lines(text: &str) -> Vec<String> {
    text.split('\n').map(str::to_string).collect()
}

fn texts(entries: &[Entry]) -> Vec<&str> {
    entries.iter().map(|e| e.text.as_str()).collect()
}

/// Applies the move and returns the new text.
fn moved(text: &str, text_of: &str, way: Way) -> Option<String> {
    let entries = outline(text);
    let index = entries.iter().position(|e| e.text == text_of)?;
    let lines = lines(text);
    let m = move_entry(&lines, &entries, index, way)?;
    let mut out = lines;
    out.splice(m.from..m.to, m.lines);

    Some(out.join("\n"))
}

#[test]
fn outline_is_a_tree_of_sections_scenes_and_synopses() {
    let entries = outline(TEXT);

    assert_eq!(
        texts(&entries),
        [
            "Akt I",
            "Der Anfang.",
            "Ankunft",
            "INT. SAAL - TAG",
            "Sie kommt an.",
            "INT. HOF - TAG",
            "Abreise",
            "INT. TOR - NACHT",
            "Akt II",
            "INT. KELLER - TAG"
        ]
    );
    let depths: Vec<usize> = entries.iter().map(|e| e.depth).collect();
    assert_eq!(depths, [0, 1, 1, 2, 3, 2, 1, 2, 0, 1]);
    assert_eq!(entries[0].kind, EntryKind::Section(1));
    assert_eq!(entries[2].kind, EntryKind::Section(2));
    assert_eq!(entries[3].number.as_deref(), Some("1"));
    assert_eq!(entries[5].scene, Some(1));
    // A section's block holds what is inside; a scene's ends at the next scene.
    assert_eq!((entries[0].line, entries[0].end), (2, 24));
    assert_eq!((entries[2].line, entries[2].end), (6, 18));
    assert_eq!((entries[3].line, entries[3].end), (8, 14));
}

#[test]
fn a_collapsed_section_hides_what_is_in_it() {
    let entries = outline(TEXT);
    let none = |_: &Entry| false;
    assert_eq!(visible(&entries, &none, "").len(), entries.len());

    let akt_one = |e: &Entry| e.text == "Akt I";
    assert_eq!(visible(&entries, &akt_one, ""), [0, 8, 9]);

    let ankunft = |e: &Entry| e.text == "Ankunft";
    assert_eq!(visible(&entries, &ankunft, ""), [0, 1, 2, 6, 7, 8, 9]);
}

#[test]
fn filter_matches_heading_synopsis_and_number_with_their_sections() {
    let entries = outline(TEXT);
    let collapsed = |_: &Entry| true;

    // The synopsis hit shows its scene's section and ignores the collapse.
    assert_eq!(visible(&entries, &collapsed, "KOMMT"), [0, 2, 4]);
    assert_eq!(visible(&entries, &collapsed, "keller"), [8, 9]);
    assert_eq!(visible(&entries, &collapsed, "#4").len(), 0);
    assert_eq!(visible(&entries, &collapsed, "3"), [0, 6, 7]);
    assert!(visible(&entries, &collapsed, "gibtsnicht").is_empty());
}

#[test]
fn a_scene_moves_down_with_its_text_and_keeps_its_number() {
    let out = moved(TEXT, "INT. SAAL - TAG", Way::Down).unwrap_or_default();
    let expected = TEXT
        .replace(
            "INT. SAAL - TAG #1#\n\n= Sie kommt an.\n\nText eins.\n\n",
            "",
        )
        .replace(
            "Text zwei.\n\n",
            "Text zwei.\n\nINT. SAAL - TAG #1#\n\n= Sie kommt an.\n\nText eins.\n\n",
        );

    assert_eq!(out, expected);
    let numbers: Vec<Option<String>> = outline(&out).iter().map(|e| e.number.clone()).collect();
    assert_eq!(numbers[3..6], [Some("2".into()), Some("1".into()), None]);
}

#[test]
fn moving_up_undoes_moving_down() {
    let down = moved(TEXT, "INT. SAAL - TAG", Way::Down).unwrap_or_default();

    assert_eq!(
        moved(&down, "INT. SAAL - TAG", Way::Up).as_deref(),
        Some(TEXT)
    );
}

#[test]
fn a_scene_steps_over_a_section_heading() {
    // The first scene of "Abreise" goes up into "Ankunft".
    let out = moved(TEXT, "INT. TOR - NACHT", Way::Up).unwrap_or_default();
    let entries = outline(&out);
    let at = |t: &str| entries.iter().position(|e| e.text == t).unwrap_or(0);

    assert!(at("INT. TOR - NACHT") < at("Abreise"));
    assert_eq!(entries[at("INT. TOR - NACHT")].depth, 2);
    assert_eq!(out.matches("Text drei.").count(), 1);
}

#[test]
fn a_section_moves_with_everything_in_it() {
    let out = moved(TEXT, "Abreise", Way::Up).unwrap_or_default();
    let entries = outline(&out);

    assert_eq!(
        texts(&entries)[2..8],
        [
            "Abreise",
            "INT. TOR - NACHT",
            "Ankunft",
            "INT. SAAL - TAG",
            "Sie kommt an.",
            "INT. HOF - TAG"
        ]
    );
    assert_eq!(moved(&out, "Abreise", Way::Down).as_deref(), Some(TEXT));
    // Acts swap too, to the end of the file without a trailing blank line.
    let acts = moved(TEXT, "Akt I", Way::Down).unwrap_or_default();
    assert!(acts.starts_with("Title: X\n\n# Akt II\n"));
    assert!(acts.ends_with("Text drei.\n"));
}

#[test]
fn nothing_moves_past_its_ends_or_out_of_its_parent() {
    assert_eq!(moved(TEXT, "Ankunft", Way::Up), None, "first of its parent");
    assert_eq!(
        moved(TEXT, "Abreise", Way::Down),
        None,
        "last of its parent"
    );
    assert_eq!(moved(TEXT, "Akt I", Way::Up), None);
    assert_eq!(moved(TEXT, "Der Anfang.", Way::Down), None, "synopsis");
    assert_eq!(
        moved(TEXT, "INT. KELLER - TAG", Way::Down),
        None,
        "last scene"
    );
}

#[test]
fn a_move_leaves_the_elements_as_a_reordering() {
    let before = parse(TEXT).elements;
    let after = parse(&moved(TEXT, "Akt I", Way::Down).unwrap_or_default()).elements;
    let (mut a, mut b) = (before.clone(), after.clone());
    let key = |e: &Element| format!("{e:?}");
    a.sort_by_key(key);
    b.sort_by_key(key);

    assert_eq!(a, b);
    assert_ne!(before, after);
}

#[test]
fn sections_and_synopses_round_trip() {
    let script = parse(TEXT);
    let again = parse(&write(&script));

    assert_eq!(again, script);
    assert_eq!(outline(&write(&script)), outline(TEXT));
}
