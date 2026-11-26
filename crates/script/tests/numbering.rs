//! Scene numbers in the text (DB-33).
#![allow(clippy::unwrap_used)]

use klappe_script::{is_omitted, number_scenes, NumberMode, NumberRules, OMITTED_DE, OMITTED_EN};

fn rules<'a>(mode: NumberMode, locked: &'a [String]) -> NumberRules<'a> {
    NumberRules {
        mode,
        first: 1,
        omitted: OMITTED_EN,
        locked,
        holders: &[],
    }
}

/// The numbers of a Sperrstand.
fn lock(numbers: &[&str]) -> Vec<String> {
    numbers.iter().map(|n| (*n).to_owned()).collect()
}

fn numbered(text: &str, mode: NumberMode) -> String {
    number_scenes(text, &rules(mode, &[]))
}

#[test]
fn automatic_numbers_count_from_the_first_number() {
    let text = "INT. A - DAY #7#\n\nHe waits.\n\nINT. B - DAY\n\nINT. C - DAY #3#\n";
    let rules = NumberRules {
        first: 10,
        ..rules(NumberMode::Automatic, &[])
    };

    assert_eq!(
        number_scenes(text, &rules),
        "INT. A - DAY #10#\n\nHe waits.\n\nINT. B - DAY #11#\n\nINT. C - DAY #12#\n"
    );
}

#[test]
fn a_text_that_is_numbered_already_stays_as_it_is() {
    let text = "Title: X\n\nINT. A - DAY #1#\n\nHe waits.\n\nINT. B - DAY #2#\n";

    assert_eq!(numbered(text, NumberMode::Automatic), text);
    assert_eq!(numbered(text, NumberMode::Locked), text);
}

#[test]
fn locked_numbers_never_change() {
    let text = "INT. A - DAY #5#\n\nINT. B - DAY #9#\n";

    assert_eq!(numbered(text, NumberMode::Locked), text);
}

#[test]
fn a_new_scene_gets_the_next_letter() {
    let text = "INT. A - DAY #12#\n\nINT. NEW - DAY\n\nINT. NEWER - DAY\n\nINT. B - DAY #13#\n";

    assert_eq!(
        numbered(text, NumberMode::Locked),
        "INT. A - DAY #12#\n\nINT. NEW - DAY #12A#\n\nINT. NEWER - DAY #12B#\n\nINT. B - DAY #13#\n"
    );
}

#[test]
fn a_letter_in_use_is_skipped() {
    let text = "INT. A - DAY #12#\n\nINT. NEW - DAY\n\nINT. B - DAY #12A#\n";

    assert!(numbered(text, NumberMode::Locked).contains("INT. NEW - DAY #12B#"));
}

#[test]
fn scenes_before_the_first_number_count_in_front_of_it() {
    let text = "INT. X - DAY\n\nINT. Y - DAY\n\nINT. A - DAY #1#\n";

    assert_eq!(
        numbered(text, NumberMode::Locked),
        "INT. X - DAY #A1#\n\nINT. Y - DAY #B1#\n\nINT. A - DAY #1#\n"
    );
}

#[test]
fn a_copied_number_makes_the_scene_that_is_not_the_holder_an_inserted_one() {
    let text = "INT. A - DAY #1#\n\nINT. B - DAY #2#\n\nINT. B - DAY #2#\n\nINT. C - DAY #3#\n";
    let held_by = |holders: &[usize]| {
        let rules = NumberRules {
            holders,
            ..rules(NumberMode::Locked, &[])
        };

        number_scenes(text, &rules)
    };

    assert_eq!(
        held_by(&[1]),
        "INT. A - DAY #1#\n\nINT. B - DAY #2#\n\nINT. B - DAY #2A#\n\nINT. C - DAY #3#\n"
    );
    assert_eq!(
        held_by(&[2]),
        "INT. A - DAY #1#\n\nINT. B - DAY #1A#\n\nINT. B - DAY #2#\n\nINT. C - DAY #3#\n"
    );
}

#[test]
fn a_copied_number_without_one_known_holder_stays() {
    let text = "INT. A - DAY #1#\n\nINT. B - DAY #2#\n\nINT. B - DAY #2#\n";
    let both = NumberRules {
        holders: &[1, 2],
        ..rules(NumberMode::Locked, &[])
    };

    assert_eq!(numbered(text, NumberMode::Locked), text);
    assert_eq!(number_scenes(text, &both), text);
}

#[test]
fn letters_stop_at_z_and_go_on_with_two() {
    let text = "INT. A - DAY #12Z#\n\nINT. NEW - DAY\n\nINT. B - DAY #13#\n";
    let lower = text.replace("12Z", "12z");

    assert!(numbered(text, NumberMode::Locked).contains("INT. NEW - DAY #12AA#"));
    assert!(numbered(&lower, NumberMode::Locked).contains("INT. NEW - DAY #12AA#"));
}

#[test]
fn a_scene_before_a_prefixed_number_sorts_in_front_of_it() {
    let text = "INT. X - DAY\n\nINT. Y - DAY\n\nINT. A - DAY #A1#\n\nINT. B - DAY #1#\n";

    assert_eq!(
        numbered(text, NumberMode::Locked),
        "INT. X - DAY #A0#\n\nINT. Y - DAY #A0A#\n\nINT. A - DAY #A1#\n\nINT. B - DAY #1#\n"
    );
}

#[test]
fn a_number_between_two_prefixed_ones_takes_letters_behind() {
    let text = "INT. A - DAY #A1#\n\nINT. NEW - DAY\n\nINT. B - DAY #B1#\n";

    assert!(numbered(text, NumberMode::Locked).contains("INT. NEW - DAY #A1A#"));
}

#[test]
fn with_no_free_number_between_the_next_free_one_is_taken() {
    let text = "INT. A - DAY #12#\n\nINT. NEW - DAY\n\nINT. B - DAY #12A#\n";

    assert!(numbered(text, NumberMode::Locked).contains("INT. NEW - DAY #12B#"));
}

#[test]
fn crlf_lines_keep_their_return() {
    let text = "INT. A - DAY #1#\r\n\r\nINT. B - DAY\r\n\r\nINT. C - DAY #5#\r\n";

    assert_eq!(
        numbered(text, NumberMode::Locked),
        "INT. A - DAY #1#\r\n\r\nINT. B - DAY #1A#\r\n\r\nINT. C - DAY #5#\r\n"
    );
    assert_eq!(
        numbered("INT. A - DAY\r\n", NumberMode::Automatic),
        "INT. A - DAY #1#\r\n"
    );
}

#[test]
fn a_struck_scene_stays_as_a_heading_in_front_of_the_next_number() {
    let text = "INT. A - DAY #1#\n\nHe waits.\n\nINT. C - DAY #3#\n";
    let locked = lock(&["1", "2", "3"]);

    let out = number_scenes(text, &rules(NumberMode::Locked, &locked));

    assert_eq!(
        out,
        "INT. A - DAY #1#\n\nHe waits.\n\n.OMITTED #2#\n\nINT. C - DAY #3#\n"
    );
    // A second run finds it there and adds nothing; the heading is not renumbered.
    assert_eq!(
        number_scenes(&out, &rules(NumberMode::Locked, &locked)),
        out
    );
}

#[test]
fn struck_scenes_go_in_front_and_at_the_end() {
    let text = "INT. A - DAY #2#\n";
    let locked = lock(&["1", "2", "9", "10"]);
    let rules = NumberRules {
        omitted: OMITTED_DE,
        ..rules(NumberMode::Locked, &locked)
    };

    assert_eq!(
        number_scenes(text, &rules),
        ".ENTFÄLLT #1#\n\nINT. A - DAY #2#\n\n.ENTFÄLLT #9#\n\n.ENTFÄLLT #10#\n"
    );
}

/// A number the text carries on a struck heading is not struck again, and the struck heading
/// is a place to put the next one in front of.
#[test]
fn a_struck_heading_in_the_text_counts_as_present() {
    let text = "INT. A - DAY #1#\n\n.OMITTED #3#\n\nINT. D - DAY #4#\n";
    let locked = lock(&["1", "2", "3", "4"]);

    assert_eq!(
        number_scenes(text, &rules(NumberMode::Locked, &locked)),
        "INT. A - DAY #1#\n\n.OMITTED #2#\n\n.OMITTED #3#\n\nINT. D - DAY #4#\n"
    );
}

/// A number carried twice is struck once.
#[test]
fn a_struck_number_listed_twice_is_inserted_once() {
    let text = "INT. A - DAY #1#\n";
    let locked = lock(&["1", "2", "2"]);

    assert_eq!(
        number_scenes(text, &rules(NumberMode::Locked, &locked)),
        "INT. A - DAY #1#\n\n.OMITTED #2#\n"
    );
}

#[test]
fn automatic_removes_the_struck_headings() {
    let text =
        "INT. A - DAY #1#\n\n.OMITTED #2#\n\nINT. C - DAY #3#\n\nINT. D - DAY\n\n.ENTFÄLLT #9#\n";

    assert_eq!(
        numbered(text, NumberMode::Automatic),
        "INT. A - DAY #1#\n\nINT. C - DAY #2#\n\nINT. D - DAY #3#\n"
    );
}

#[test]
fn both_words_are_struck_headings() {
    assert!(is_omitted("ENTFÄLLT") && is_omitted("omitted") && !is_omitted("INT. A"));
}

/// A number without the `A12B` pattern in front of a new scene is no reason to take `1`.
#[test]
fn a_new_scene_skips_a_number_in_use_behind_a_patternless_one() {
    let text = "INT. A - DAY\n\nINT. B - DAY #I#\n\nINT. C - DAY #1#\n";
    let once = numbered(text, NumberMode::Locked);

    assert_eq!(
        once,
        "INT. A - DAY #1A#\n\nINT. B - DAY #I#\n\nINT. C - DAY #1#\n"
    );
    assert_eq!(numbered(&once, NumberMode::Locked), once);
}

#[test]
fn a_new_scene_before_a_dotted_number_skips_the_ones_in_use() {
    let text = "INT. A - DAY\n\nINT. B - DAY #1.1#\n\nINT. C - DAY\n";
    let once = numbered(text, NumberMode::Locked);

    assert_eq!(
        once,
        "INT. A - DAY #1#\n\nINT. B - DAY #1.1#\n\nINT. C - DAY #1A#\n"
    );
    assert_eq!(numbered(&once, NumberMode::Locked), once);
}

#[test]
fn inserted_struck_headings_use_the_line_ending_of_the_book() {
    let text = "INT. A - DAY #1#\r\n\r\nINT. B - DAY #3#\r\n";
    let locked = lock(&["1", "2", "3", "4"]);

    assert_eq!(
        number_scenes(text, &rules(NumberMode::Locked, &locked)),
        "INT. A - DAY #1#\r\n\r\n.OMITTED #2#\r\n\r\nINT. B - DAY #3#\r\n\r\n.OMITTED #4#\r\n"
    );
}

/// Review round 4: a struck last scene in a CRLF book without a final newline went behind
/// a bare `\n`.
#[test]
fn a_struck_heading_at_the_end_of_a_crlf_text_without_newline_keeps_crlf() {
    let text = "INT. A - DAY #1#\r\n\r\nAnna.";
    let locked = lock(&["1", "2"]);

    assert_eq!(
        number_scenes(text, &rules(NumberMode::Locked, &locked)),
        "INT. A - DAY #1#\r\n\r\nAnna.\r\n\r\n.OMITTED #2#"
    );
}

/// Review round 4: removing the struck headings of a CRLF book left a single `\r` at the end.
#[test]
fn automatic_removes_struck_headings_of_a_crlf_book_cleanly() {
    let text =
        "INT. A - DAY #1#\r\n\r\n.OMITTED #2#\r\n\r\nINT. B - DAY #3#\r\n\r\n.OMITTED #4#\r\n";

    assert_eq!(
        numbered(text, NumberMode::Automatic),
        "INT. A - DAY #1#\r\n\r\nINT. B - DAY #2#\r\n"
    );
}

/// Review round 4: a struck heading without a number (from another program) stays so.
#[test]
fn a_struck_heading_without_a_number_gets_none_while_locked() {
    let text = "INT. A - DAY #1#\n\n.OMITTED\n\nINT. B - DAY #2#\n";
    let locked = lock(&["1", "2"]);

    assert_eq!(
        number_scenes(text, &rules(NumberMode::Locked, &locked)),
        text
    );
}

#[test]
fn a_number_replaces_only_a_closed_mark_at_the_end() {
    let text = "INT. A - DAY   #7#\n\nINT. #1 B\n\nINT. C ##\n\nINT. D #a b#\n";

    assert_eq!(
        numbered(text, NumberMode::Automatic),
        "INT. A - DAY #1#\n\nINT. #1 B #2#\n\nINT. C ## #3#\n\nINT. D #a b# #4#\n"
    );
}
