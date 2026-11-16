use super::wrap::wrap;
use super::*;
use crate::parse;

/// Pages of `text` with `body_lines` rows each, English words.
fn lay(text: &str, body_lines: usize) -> Pagination {
    let mut options = PageOptions::new(PageSize::Letter, Words::ENGLISH);
    options.body_lines = body_lines;

    paginate(&parse(text).elements, &options)
}

/// `(row, x, text)` of every line of page `n` (from 1).
fn page(p: &Pagination, n: usize) -> Vec<(usize, usize, String)> {
    p.pages[n - 1]
        .lines
        .iter()
        .map(|l| (l.row, l.x, l.text.clone()))
        .collect()
}

fn row(row: usize, x: usize, text: &str) -> (usize, usize, String) {
    (row, x, text.to_string())
}

#[test]
fn wraps_at_spaces_and_cuts_long_words() {
    let text = "aaa bbb ccc";

    assert_eq!(wrap(text, 7), [0..7, 8..11]);
    assert_eq!(wrap(text, 20), vec![0..11]);
    assert_eq!(wrap("abcdefgh ij", 3), [0..3, 3..6, 6..8, 9..11]);
    assert!(wrap("", 10).is_empty());
}

#[test]
fn a_page_starts_without_blank_rows() {
    let p = lay("int. room - day\n\nShort action.\n", 10);

    // The two blank rows before the heading are dropped at the top; one blank row before
    // the action stays.
    assert_eq!(
        page(&p, 1),
        [row(0, 0, "INT. ROOM - DAY"), row(2, 0, "Short action.")]
    );
    assert_eq!(p.scenes[0].lines, 3);
}

#[test]
fn a_heading_never_stands_alone_at_the_page_end() {
    // 7 rows of action fill rows 0 to 6; the heading would take row 9 (two blanks, row 9
    // is the last), with nothing of its scene below.
    let text = "a\nb\nc\nd\ne\nf\ng\n\nINT. NEXT - DAY\n\nx\ny\nz\n";
    let p = lay(text, 10);

    assert_eq!(p.pages.len(), 2);
    assert!(page(&p, 1).iter().all(|(_, _, t)| t != "INT. NEXT - DAY"));
    assert_eq!(page(&p, 2)[0], row(0, 0, "INT. NEXT - DAY"));
    assert_eq!(p.scenes[0].page, 2);
    assert_eq!(p.scenes[0].row, 0);
}

#[test]
fn a_heading_stays_when_two_lines_follow_it() {
    // Space for the heading (rows 7-9 are 2 blanks and the heading at 9 is too late), so
    // with 4 rows of action ahead the heading sits at row 6 and its two lines follow.
    let text = "a\nb\nc\nd\n\nINT. NEXT - DAY\n\nx\ny\nz\n";
    let p = lay(text, 14);

    // rows: a-d 0-3, blanks 4-5, heading 6, blank 7, x 8, y 9, z 10
    assert_eq!(p.pages.len(), 1);
    assert_eq!(page(&p, 1)[4], row(6, 0, "INT. NEXT - DAY"));
}

#[test]
fn action_splits_between_sentences_and_keeps_two_rows() {
    // The first action takes rows 0-4; the second starts at row 6 and has room for 4 rows.
    // Cutting after the fourth sentence would leave one row for the next page, so the cut
    // is after the third.
    let text = "a\nb\nc\nd\ne\n\nOne.\nTwo.\nThree.\nFour.\nFive.\n";
    let p = lay(text, 10);

    let first: Vec<_> = page(&p, 1).into_iter().skip(5).collect();
    assert_eq!(
        first,
        [row(6, 0, "One."), row(7, 0, "Two."), row(8, 0, "Three.")]
    );
    assert_eq!(page(&p, 2), [row(0, 0, "Four."), row(1, 0, "Five.")]);
}

#[test]
fn dialogue_splits_with_more_and_contd() {
    let text = "a\nb\nc\nd\ne\n\nANNA\nD1.\nD2.\nD3.\nD4.\n";
    let p = lay(text, 10);

    let first: Vec<_> = page(&p, 1).into_iter().skip(5).collect();
    assert_eq!(
        first,
        [
            row(6, 22, "ANNA"),
            row(7, 10, "D1."),
            row(8, 10, "D2."),
            row(9, 22, "(MORE)")
        ]
    );
    assert_eq!(
        page(&p, 2),
        [
            row(0, 22, "ANNA (CONT'D)"),
            row(1, 10, "D3."),
            row(2, 10, "D4.")
        ]
    );
}

#[test]
fn the_words_follow_the_project() {
    let text = "a\nb\nc\nd\ne\n\nANNA\nD1.\nD2.\nD3.\nD4.\n";
    let mut options = PageOptions::new(PageSize::A4, Words::GERMAN);
    options.body_lines = 10;
    let p = paginate(&parse(text).elements, &options);

    assert_eq!(page(&p, 1)[8], row(9, 22, "(WEITER)"));
    assert_eq!(page(&p, 2)[0], row(0, 22, "ANNA (FORTS.)"));
}

#[test]
fn a_dialogue_of_two_rows_is_not_cut() {
    // The name and one row fit, the second row would not: the dialogue moves as a whole.
    let text = "a\nb\nc\nd\ne\nf\ng\n\nANNA\nOne.\nTwo.\n";
    let p = lay(text, 10);

    assert_eq!(p.pages.len(), 2);
    assert_eq!(page(&p, 2)[0], row(0, 22, "ANNA"));
}

#[test]
fn a_dialogue_breaks_before_a_parenthetical() {
    let text = "a\nb\nc\nd\n\nANNA\nOne.\nTwo.\n(softly)\nThree.\nFour.\n";
    let p = lay(text, 10);

    // rows: a-d 0-3, blank 4, name 5, One 6, Two 7, then (MORE) at 8; "(softly)" and
    // what follows is on page 2 under the name with (CONT'D).
    assert_eq!(page(&p, 1).last(), Some(&row(8, 22, "(MORE)")));
    assert_eq!(page(&p, 2)[0], row(0, 22, "ANNA (CONT'D)"));
    assert_eq!(page(&p, 2)[1], row(1, 16, "(softly)"));
}

#[test]
fn dual_dialogue_stands_side_by_side() {
    let text = "ANNA\nLeft one two.\n\nBEN ^\nRight.\n";
    let p = lay(text, 10);

    assert_eq!(
        page(&p, 1),
        [
            row(0, 6, "ANNA"),
            row(0, 37, "BEN"),
            row(1, 0, "Left one two."),
            row(1, 31, "Right."),
        ]
    );
}

#[test]
fn a_forced_page_break_starts_a_page() {
    let p = lay("First.\n\n===\n\nSecond.\n", 10);

    assert_eq!(p.pages.len(), 2);
    assert_eq!(page(&p, 2), [row(0, 0, "Second.")]);
}

#[test]
fn contd_after_action_is_optional() {
    let text = "ANNA\nHi.\n\nShe waits.\n\nANNA\nAgain.\n";
    let off = lay(text, 20);
    let mut options = PageOptions::new(PageSize::A4, Words::ENGLISH);
    options.body_lines = 20;
    options.contd_after_action = ContdAfterAction::On;
    let on = paginate(&parse(text).elements, &options);

    let cues = |p: &Pagination| -> Vec<String> {
        page(p, 1)
            .into_iter()
            .filter(|(_, x, _)| *x == 22)
            .map(|(_, _, t)| t)
            .collect()
    };
    assert_eq!(cues(&off), ["ANNA", "ANNA"]);
    assert_eq!(cues(&on), ["ANNA", "ANNA (CONT'D)"]);
}

#[test]
fn transition_and_centered_are_set_in_their_places() {
    let p = lay(">THE END<\n\nCUT TO:\n", 10);

    // 7 characters centred in 60 columns (53 free, 26 to the left), a 7-character transition at the right edge.
    assert_eq!(page(&p, 1), [row(0, 26, "THE END"), row(2, 53, "CUT TO:")]);
}

#[test]
fn a_scene_is_measured_in_eighths_of_a_page() {
    // A heading (1 row) and an action of 18 rows after a blank: 20 rows of 55, 20/55 of a
    // page is 2.9 eighths, rounded up to 3.
    let lines = (0..18)
        .map(|i| format!("l{i}"))
        .collect::<Vec<_>>()
        .join("\n");
    let text = format!("INT. A - DAY\n\n{lines}\n");
    let p = lay(&text, PageSize::Letter.body_lines());

    assert_eq!(p.scenes[0].lines, 20);
    assert_eq!(p.scenes[0].eighths, 3);
    assert_eq!(length_text(3), "3/8");
}

#[test]
fn a_tiny_scene_is_an_eighth_and_a_full_page_is_eight() {
    let p = lay("INT. A - DAY\n", 55);
    assert_eq!(p.scenes[0].eighths, 1);

    let lines = (0..53)
        .map(|i| format!("l{i}"))
        .collect::<Vec<_>>()
        .join("\n");
    let p = lay(&format!("INT. A - DAY\n\n{lines}\n"), 55);
    assert_eq!(p.scenes[0].lines, 55);
    assert_eq!(p.scenes[0].eighths, 8);
    assert_eq!(p.eighths(), 8);
    assert_eq!(length_text(8), "1");
    assert_eq!(length_text(19), "2 3/8");
    assert!((p.minutes() - 1.0).abs() < f64::EPSILON);
}

#[test]
fn the_paper_decides_the_rows_of_a_page() {
    assert_eq!(PageSize::Letter.body_lines(), 55);
    assert_eq!(PageSize::A4.body_lines(), 59);
}

#[test]
fn lines_point_back_to_their_element_and_offset() {
    let p = lay("One. Two.\nThree.\n", 10);
    let lines = &p.pages[0].lines;

    assert_eq!(
        lines
            .iter()
            .map(|l| (l.element, l.offset))
            .collect::<Vec<_>>(),
        [(0, 0), (0, 10)]
    );
}

#[test]
fn a_scene_number_is_carried_on_its_heading_line() {
    let p = lay("INT. A - DAY #12#\n\nText.\n", 10);

    assert_eq!(p.pages[0].lines[0].number.as_deref(), Some("12"));
}

/// The demo and example screenplays lay out without panics, within the page.
#[test]
fn the_example_screenplays_lay_out() {
    let texts = [
        include_str!("../../../../testdata/beispiel.klappe/drehbuch.fountain"),
        include_str!("../../../../demo/beispiel-de.klappe/drehbuch.fountain"),
        include_str!("../../../../demo/beispiel-en.klappe/drehbuch.fountain"),
    ];

    for text in texts {
        for size in [PageSize::A4, PageSize::Letter] {
            let p = paginate(
                &parse(text).elements,
                &PageOptions::new(size, Words::GERMAN),
            );

            assert!(!p.pages.is_empty());
            assert!(!p.scenes.is_empty());
            for page in &p.pages {
                assert!(page.lines.iter().all(|l| l.row < size.body_lines()));
            }
        }
    }
}

/// A block taller than a page with no cut place is cut at rows, not lost or looped on.
#[test]
fn a_block_taller_than_a_page_is_cut_at_rows() {
    let words = (0..12)
        .map(|i| format!("w{i}"))
        .collect::<Vec<_>>()
        .join("\n");
    let p = lay(&format!("ANNA\nx\n\nBEN ^\n{words}\n"), 5);

    assert!(p.pages.len() >= 3);
    let total: usize = p.pages.iter().map(|pg| pg.lines.len()).sum();
    assert_eq!(total, 1 + 1 + 1 + 12);
}

#[test]
fn more_and_the_repeated_name_are_marked_as_added() {
    let p = lay("a\nb\nc\nd\ne\n\nANNA\nD1.\nD2.\nD3.\nD4.\n", 10);
    let added = |n: usize| -> Vec<String> {
        p.pages[n]
            .lines
            .iter()
            .filter(|l| l.added)
            .map(|l| l.text.clone())
            .collect()
    };

    assert_eq!(added(0), ["(MORE)"]);
    assert_eq!(added(1), ["ANNA (CONT'D)"]);
}

#[test]
fn a_sentence_does_not_end_at_an_abbreviation() {
    use super::block::sentence_starts;

    let text = "Dr. Meyer sah Hr. Roth, z. B. Mrs. Lee. Dann ging er. Nr. 5 blieb.";
    let starts: Vec<&str> = sentence_starts(text).iter().map(|&i| &text[i..]).collect();

    assert_eq!(starts, ["Dann ging er. Nr. 5 blieb.", "Nr. 5 blieb."]);
}
