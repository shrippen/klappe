//! Scene numbers in the text (DB-33).
#![allow(clippy::unwrap_used)]

use klappe_script::{
    is_omitted, number_scenes, NumberMode, NumberRules, Struck, OMITTED_DE, OMITTED_EN,
};

fn rules(mode: NumberMode, struck: &[Struck]) -> NumberRules<'_> {
    NumberRules {
        mode,
        first: 1,
        omitted: OMITTED_EN,
        struck,
    }
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
fn locked_without_any_number_stays_as_it_is() {
    let text = "INT. X - DAY\n\nINT. Y - DAY\n";
    let struck = [Struck {
        after: Some(0),
        number: "2".into(),
    }];

    assert_eq!(numbered(text, NumberMode::Locked), text);
    assert_eq!(
        number_scenes(text, &rules(NumberMode::Locked, &struck)),
        text
    );
}

#[test]
fn a_copied_number_makes_the_later_scene_an_inserted_one() {
    let text = "INT. A - DAY #1#\n\nINT. B - DAY #2#\n\nINT. B - DAY #2#\n\nINT. C - DAY #3#\n";

    assert_eq!(
        numbered(text, NumberMode::Locked),
        "INT. A - DAY #1#\n\nINT. B - DAY #2#\n\nINT. B - DAY #2A#\n\nINT. C - DAY #3#\n"
    );
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
fn a_struck_scene_stays_as_a_heading_behind_its_neighbour() {
    let text = "INT. A - DAY #1#\n\nHe waits.\n\nINT. C - DAY #3#\n";
    let struck = [Struck {
        after: Some(0),
        number: "2".into(),
    }];

    let out = number_scenes(text, &rules(NumberMode::Locked, &struck));

    assert_eq!(
        out,
        "INT. A - DAY #1#\n\nHe waits.\n\n.OMITTED #2#\n\nINT. C - DAY #3#\n"
    );
    // A second run finds it there and adds nothing; the heading is not renumbered.
    assert_eq!(
        number_scenes(&out, &rules(NumberMode::Locked, &struck)),
        out
    );
}

#[test]
fn struck_scenes_go_in_front_and_at_the_end() {
    let text = "INT. A - DAY #2#\n";
    let struck = [
        Struck {
            after: Some(0),
            number: "9".into(),
        },
        Struck {
            after: None,
            number: "1".into(),
        },
        Struck {
            after: Some(0),
            number: "10".into(),
        },
    ];
    let rules = NumberRules {
        omitted: OMITTED_DE,
        ..rules(NumberMode::Locked, &struck)
    };

    assert_eq!(
        number_scenes(text, &rules),
        ".ENTFÄLLT #1#\n\nINT. A - DAY #2#\n\n.ENTFÄLLT #9#\n\n.ENTFÄLLT #10#\n"
    );
}

#[test]
fn automatic_counting_passes_a_struck_heading() {
    let text = "INT. A - DAY #1#\n\n.OMITTED #2#\n\nINT. C - DAY #3#\n\nINT. D - DAY\n";

    assert_eq!(
        numbered(text, NumberMode::Automatic),
        "INT. A - DAY #1#\n\n.OMITTED #2#\n\nINT. C - DAY #2#\n\nINT. D - DAY #3#\n"
    );
}

#[test]
fn both_words_are_struck_headings() {
    assert!(is_omitted("ENTFÄLLT") && is_omitted("omitted") && !is_omitted("INT. A"));
}
