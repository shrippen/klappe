//! Emphasis toggled over a selection of the source (D-36, D-37).

use crate::emphasis::toggle_rounds;
use crate::shown::marks_of;
use crate::{emphasis_at, shown_lines, toggle_mark, Hide, Mark, Marks, Spot};

const SHOWN: [Hide; 1] = [Hide::Emphasis];
const SOURCE: [Hide; 0] = [];

fn spot(line: usize, col: usize) -> Spot {
    Spot { line, col }
}

/// The source after `mark` toggled over `from` to `to`.
fn toggled(source: &str, from: Spot, to: Spot, mark: Mark) -> String {
    toggle_mark(source, &SHOWN, (from, to), mark).source
}

/// The same for a selection inside line 0.
fn on_line(source: &str, from: usize, to: usize, mark: Mark) -> String {
    toggled(source, spot(0, from), spot(0, to), mark)
}

#[test]
fn a_plain_word_becomes_bold() {
    assert_eq!(
        on_line("Das ist fett.", 8, 12, Mark::Bold),
        "Das ist **fett**."
    );
    assert_eq!(
        on_line("Das ist kursiv.", 8, 14, Mark::Italic),
        "Das ist *kursiv*."
    );
    assert_eq!(
        on_line("Das ist weg.", 8, 11, Mark::Underline),
        "Das ist _weg_."
    );
}

#[test]
fn a_bold_word_becomes_plain() {
    // The selection stands in the source: inside the marks, or around them.
    assert_eq!(
        on_line("Das ist **fett**.", 10, 14, Mark::Bold),
        "Das ist fett."
    );
    assert_eq!(
        on_line("Das ist **fett**.", 8, 16, Mark::Bold),
        "Das ist fett."
    );
}

#[test]
fn a_selection_that_is_partly_bold_becomes_all_bold() {
    assert_eq!(
        on_line("Das ist **fett** und viel.", 10, 20, Mark::Bold),
        "Das ist **fett und** viel."
    );
}

#[test]
fn toggling_over_the_edge_of_a_run_keeps_the_other_marks() {
    // Bold over `a *b* c`: the italic word stays italic inside.
    assert_eq!(on_line("a *b* c", 0, 7, Mark::Bold), "**a *b* c**");
    assert_eq!(on_line("**a *b* c**", 0, 11, Mark::Bold), "a *b* c");

    // Italic over bold text that goes on behind it.
    assert_eq!(
        on_line("**fett und viel**", 7, 10, Mark::Italic),
        "**fett *und* viel**"
    );
    assert_eq!(
        on_line("**fett *und* viel**", 7, 12, Mark::Italic),
        "**fett und viel**"
    );
}

#[test]
fn toggling_twice_gives_the_line_back() {
    let line = "Ein **fetter** Mann, *kursiv* und weiter.";
    let on = on_line(line, 0, 41, Mark::Underline);
    let off = on_line(&on, 0, 42, Mark::Underline);

    assert_ne!(on, line);
    assert_eq!(off, line);
}

#[test]
fn a_selection_over_lines_is_toggled_as_a_whole() {
    let source = "eins\n\nzwei **drei**";
    let all = toggled(source, spot(0, 0), spot(2, 14), Mark::Bold);

    assert_eq!(all, "**eins**\n\n**zwei drei**");
    assert_eq!(
        toggled(&all, spot(0, 0), spot(2, 14), Mark::Bold),
        "eins\n\nzwei drei"
    );
}

#[test]
fn a_selection_without_characters_changes_nothing() {
    let source = "Das ist **fett**.\n\nZwei";

    assert_eq!(on_line(source, 5, 5, Mark::Bold), source);
    assert_eq!(
        toggled(source, spot(1, 0), spot(1, 0), Mark::Italic),
        source
    );
}

#[test]
fn backslash_note_and_number_stay() {
    let line = "Ein \\*Stern\\* [[Notiz]] und mehr.";
    let new = on_line(line, 0, 13, Mark::Bold);
    let shown = shown_lines(&new, &SHOWN).remove(0);

    assert!(new.contains("[[Notiz]]"), "{new}");
    assert!(new.contains("\\*Stern\\*"), "{new}");
    assert_eq!(shown.text, shown_lines(line, &SHOWN).remove(0).text);
    assert!(shown.runs.iter().any(|r| r.marks.bold), "{new}");
}

#[test]
fn umlauts_and_emoji_count_in_utf16() {
    // The emoji is two units.
    assert_eq!(on_line("Ä🎬 ö", 1, 3, Mark::Bold), "Ä**🎬** ö");
    assert_eq!(on_line("Ä🎬 ö", 0, 5, Mark::Italic), "*Ä🎬 ö*");
}

#[test]
fn a_heading_is_bold_already() {
    let source = "INT. KÜCHE - TAG\n\nEr kommt.";

    assert_eq!(on_line(source, 0, 8, Mark::Bold), source);
    let both = toggled(source, spot(0, 0), spot(2, 9), Mark::Bold);
    assert_eq!(both, "INT. KÜCHE - TAG\n\n**Er kommt.**");
}

#[test]
fn italic_in_a_heading_keeps_it_a_heading() {
    let new = on_line("INT. KÜCHE - TAG", 5, 11, Mark::Italic);
    let shown = shown_lines(&new, &SHOWN).remove(0);

    assert_eq!(shown.text, "INT. KÜCHE - TAG");
    assert_eq!(shown.kind, crate::TextKind::Heading);
    assert!(shown.runs.iter().any(|r| r.marks.italic), "{new}");
}

#[test]
fn lines_that_show_their_marks_are_left_alone() {
    let source = "# Akt *1*\n= Sie kommt\n[[Notiz]]";

    assert_eq!(toggled(source, spot(0, 0), spot(2, 8), Mark::Bold), source);
}

#[test]
fn the_source_view_shows_the_marks_that_were_written() {
    let done = toggle_mark(
        "Das ist fett.",
        &SOURCE,
        (spot(0, 8), spot(0, 12)),
        Mark::Bold,
    );

    assert_eq!(done.source, "Das ist **fett**.");
    assert_eq!(done.shown.len(), 1);
    assert_eq!(done.shown[0].text, "Das ist **fett**.");
}

#[test]
fn a_selection_in_the_source_view_names_the_same_words() {
    // The marks stand in the selection of the source view: `**fett**` is the bold word.
    assert_eq!(
        on_line("Das ist **fett**.", 8, 16, Mark::Bold),
        "Das ist fett."
    );
    assert_eq!(
        on_line("Das ist **fett**.", 8, 16, Mark::Italic),
        "Das ist ***fett***."
    );
}

#[test]
fn the_edit_names_the_lines_to_show_again() {
    let done = toggle_mark("a\nb\nc", &SHOWN, (spot(1, 0), spot(1, 1)), Mark::Bold);

    assert_eq!((done.from, done.to), (1, 2));
    assert_eq!(done.shown.len(), 1);
    assert_eq!(done.source, "a\n**b**\nc");
}

#[test]
fn the_state_is_what_the_whole_selection_has() {
    let source = "Das ist **fett** und *kursiv*.";
    let marks = |from, to| emphasis_at(source, spot(0, from), spot(0, to)).marks;

    assert!(marks(10, 14).bold);
    assert!(!marks(10, 14).italic);
    assert!(!marks(8, 20).bold);
    assert!(marks(22, 28).italic);
    assert!(!marks(0, 3).bold);
}

#[test]
fn the_state_at_a_cursor_is_the_character_before() {
    let source = "Das ist **fett** und";
    let at = |col| emphasis_at(source, spot(0, col), spot(0, col)).marks.bold;

    // Behind the word, inside it, in front of it and at the start of the line.
    assert!(at(16));
    assert!(at(12));
    assert!(!at(7));
    assert!(!at(0));
    // At the start of a bold line the character behind counts.
    assert!(emphasis_at("**a** b", spot(0, 0), spot(0, 0)).marks.bold);
}

#[test]
fn the_state_of_a_heading_is_bold_and_fixed() {
    let state = emphasis_at("INT. KÜCHE - TAG\n\nText", spot(0, 2), spot(0, 6));

    assert!(state.marks.bold);
    assert!(state.heading);
    assert!(!emphasis_at("INT. KÜCHE - TAG\n\nText", spot(0, 2), spot(2, 3)).heading);
}

/// The emphasis of each character of each line of `source`.
fn marks_by_line(source: &str) -> Vec<Vec<Marks>> {
    shown_lines(source, &SHOWN)
        .iter()
        .map(|l| marks_of(&l.runs, l.text.chars().count()))
        .collect()
}

fn has(mark: Mark, marks: &Marks) -> bool {
    match mark {
        Mark::Bold => marks.bold,
        Mark::Italic => marks.italic,
        Mark::Underline => marks.underline,
    }
}

/// After a toggle of `mark` over the whole of line 0 of `before`: the line is as it was, or it
/// has the mark switched as asked on every character and every other emphasis as it was.
fn assert_all_or_nothing(before: &str, mark: Mark) {
    let after = on_line(before, 0, before.len(), mark);
    if after == before {
        return;
    }

    let old = marks_by_line(before);
    let new = marks_by_line(&after);
    let on = !old[0].iter().all(|m| has(mark, m));
    let mut want = old[0].clone();
    for m in &mut want {
        match mark {
            Mark::Bold => m.bold = on,
            Mark::Italic => m.italic = on,
            Mark::Underline => m.underline = on,
        }
    }

    assert_eq!(new[0], want, "{mark:?} over {before:?} gave {after:?}");
}

/// The kinds of the lines of `source` after `mark` over line `line`, whole.
fn kinds_after(
    source: &str,
    line: usize,
    mark: Mark,
) -> (Vec<crate::TextKind>, Vec<crate::TextKind>) {
    let len = source
        .split('\n')
        .nth(line)
        .map_or(0, |l| l.encode_utf16().count());
    let done = toggled(source, spot(line, 0), spot(line, len), mark);

    (crate::text_kinds(source), crate::text_kinds(&done))
}

/// `source` with `mark` toggled over all of it.
fn over_all(source: &str, mark: Mark) -> String {
    let lines: Vec<&str> = source.split('\n').collect();
    let last = lines.len() - 1;
    let end = spot(last, lines[last].encode_utf16().count());

    toggled(source, spot(0, 0), end, mark)
}

/// Whole-book toggles of `mark` leave nothing to do: no line of `source` that did not get the
/// mark can be toggled alone, on the result, without spoiling the book.
fn assert_maximal(source: &str, mark: Mark) {
    let done = over_all(source, mark);
    let lines: Vec<&str> = done.split('\n').collect();
    let old: Vec<&str> = source.split('\n').collect();

    for (at, line) in lines.iter().enumerate() {
        let marks = &marks_by_line(&done)[at];
        let full = !marks.is_empty() && marks.iter().all(|m| has(mark, m));
        if *line != old[at] || full {
            continue;
        }

        let len = line.encode_utf16().count();
        let alone = toggled(&done, spot(at, 0), spot(at, len), mark);
        assert_eq!(
            alone, done,
            "{mark:?} line {at} of {source:?} could take it"
        );
    }
}

const AFTER_A_CUE: &str =
    "INT. HAUS - TAG\n\n!ANNA\nkommt herein.\nSie setzt sich.\n\nANNA\nHallo.";
const ODD_LINES: &str = "INT. HAUS - TAG\nEin _u_ hier.\n\n  \n\nINT. KÜCHE - TAG #12A#\nANNA\nZwei.\nCUT TO:\nzu]] ok\nx_y_z";

/// A forced cue takes bold behind its `!`, and so do the lines under it, which the bold cue
/// turns into dialogue and which are bold as action again once the cue is as it was.
#[test]
fn a_forced_cue_and_the_lines_under_it_take_bold() {
    let done = over_all(AFTER_A_CUE, Mark::Bold);

    assert!(done.contains("\n!**ANNA**\n"), "{done:?}");
    assert!(done.contains("\n**kommt herein.**\n"), "{done:?}");
    assert!(done.contains("\n**Sie setzt sich.**\n"), "{done:?}");
    assert_eq!(crate::text_kinds(&done), crate::text_kinds(AFTER_A_CUE));
}

#[test]
fn a_selection_from_column_0_over_lyrics_marks_every_line() {
    let source = "INT. HAUS - TAG\n\n~eins\n~zwei";
    let done = toggled(source, spot(2, 1), spot(3, 5), Mark::Italic);

    assert_eq!(done, "INT. HAUS - TAG\n\n~*eins*\n~*zwei*");
}

#[test]
fn centered_lines_take_underline_inside_their_marks() {
    let source = "INT. HAUS - TAG\n\n> eins <\n> zwei <\n\nANNA\nHallo.";
    let done = over_all(source, Mark::Underline);

    assert!(
        done.contains(">_ eins _<") && done.contains(">_ zwei _<"),
        "{done:?}"
    );
    assert_eq!(crate::text_kinds(&done), crate::text_kinds(source));
}

/// A song of 100 lines and 100 parentheticals under a cue.
fn long_blocks() -> String {
    let song: Vec<String> = (0..100).map(|n| format!("~zeile {n}")).collect();
    let aside: Vec<String> = (0..100).map(|n| format!("(still {n})")).collect();

    format!(
        "INT. HAUS - TAG\n\n{}\n\nANNA\n{}",
        song.join("\n"),
        aside.join("\n")
    )
}

#[test]
fn a_song_takes_its_mark_on_every_line() {
    let source = long_blocks();
    let done = over_all(&source, Mark::Bold);

    assert!(done.contains("\n~**zeile 0**\n") && done.contains("\n~**zeile 99**\n"));
    assert_eq!(crate::text_kinds(&done), crate::text_kinds(&source));
}

#[test]
fn the_rounds_do_not_grow_with_the_lines_that_cannot_take_it() {
    // Cues and parentheticals in turn: the italic cues are fine, the parentheticals are not.
    let block: Vec<String> = (0..100)
        .map(|n| match n % 2 {
            0 => "ANNA".to_owned(),
            _ => format!("(still {n})"),
        })
        .collect();
    let source = format!("INT. HAUS - TAG\n\n{}\n\nEnde.", block.join("\n"));
    let lines: Vec<&str> = source.split('\n').collect();
    let end = spot(lines.len() - 1, 5);
    let (done, rounds) = toggle_rounds(&source, &SHOWN, (spot(0, 0), end), Mark::Italic);

    assert_ne!(done.source, source);
    assert!(rounds <= 3, "{rounds} rounds");
}

#[test]
fn lines_under_a_cue_that_cannot_be_italic_are_italic() {
    let done = over_all(AFTER_A_CUE, Mark::Italic);

    assert!(done.contains("\n*kommt herein.*\n"), "{done:?}");
    assert!(done.contains("\n*Sie setzt sich.*\n"), "{done:?}");
}

#[test]
fn odd_lines_take_the_italic_they_can() {
    let done = over_all(ODD_LINES, Mark::Italic);
    let changed = done
        .split('\n')
        .zip(ODD_LINES.split('\n'))
        .filter(|(new, old)| new != old)
        .count();

    assert!(changed >= 6, "{changed} lines changed: {done:?}");
    assert_eq!(crate::text_kinds(&done), crate::text_kinds(ODD_LINES));
}

#[test]
fn a_toggle_over_all_leaves_nothing_to_toggle() {
    for book in [AFTER_A_CUE, ODD_LINES, BOOK, EXAMPLE] {
        for mark in MARKS {
            assert_maximal(book, mark);
        }
    }
}

/// Lines that keep their kind in the book are marked, whatever a window around them would say.
const IN_A_BLOCK: &str = "INT. HAUS - TAG\n\nEXT. GARTEN - NACHT\nANNA (V.O.)\nIch bin weg.\n(lacht)\nWirklich.\n\nEnde.";
const A_SONG: &str = "INT. HAUS - TAG\n\nINT. HAUS - TAG\n~eins\n~zwei\n~drei\n\nEnde.";

#[test]
fn a_parenthetical_in_a_block_without_blank_lines_is_judged_in_its_block() {
    for mark in MARKS {
        assert_maximal(IN_A_BLOCK, mark);
    }
    let done = over_all(IN_A_BLOCK, Mark::Bold);

    assert_eq!(crate::text_kinds(&done), crate::text_kinds(IN_A_BLOCK));
    assert!(done.contains("**Wirklich.**"), "{done:?}");
}

#[test]
fn the_third_line_of_a_song_behind_a_heading_is_marked() {
    for mark in MARKS {
        assert_maximal(A_SONG, mark);
    }
    let done = over_all(A_SONG, Mark::Bold);

    assert!(done.contains("**~drei**"), "{done:?}");
}

#[test]
fn an_indented_lyric_and_a_padded_centered_line_are_marked_inside_their_marks() {
    let source = "INT. HAUS - TAG\n\n> eins < \n\n  ~eins";
    let marks = marks_by_line(&over_all(source, Mark::Bold));

    assert!(!marks[2][0].bold && marks[2][2].bold, "{:?}", marks[2]);
    assert!(!marks[4][2].bold && marks[4][3].bold, "{:?}", marks[4]);
}

#[test]
fn a_boneyard_over_lines_ends_far_above_the_line() {
    let source = "/*\neins\nzwei\ndrei\n*/\n!ANNA\nText.";
    let (old, new) = kinds_after(source, 5, Mark::Italic);

    assert_eq!(old, new);
}

#[test]
fn a_boneyard_under_the_line_that_ends_in_text_keeps_the_kinds() {
    let source = "!ANNA\n/*\na\nb\nc\nd\n*/Text.";
    let (old, new) = kinds_after(source, 0, Mark::Italic);

    assert_eq!(old, new);
}

#[test]
fn a_heading_above_a_cue_behind_a_boneyard_keeps_the_kinds() {
    let source = "/*\neins\nzwei\ndrei\n*/\nINT. HAUS - TAG\n!ANNA\nText.";
    let (old, new) = kinds_after(source, 6, Mark::Bold);

    assert_eq!(old, new);
}

#[test]
fn bold_over_a_line_ending_in_italic_loses_nothing() {
    assert_all_or_nothing("Sie sagt *nein*", Mark::Bold);
}

#[test]
fn bold_over_a_short_line_ending_in_italic_loses_nothing() {
    assert_all_or_nothing("Er *kommt*", Mark::Bold);
}

#[test]
fn italic_over_a_line_ending_in_bold_loses_nothing() {
    assert_all_or_nothing("Er **kommt**", Mark::Italic);
}

const EXAMPLE: &str = include_str!("../../../testdata/beispiel.klappe/drehbuch.fountain");
/// Every this many columns of the example's lines start or end a selection.
const SAMPLE_STEP: usize = 13;
const BOOK_STEP: usize = 2;
const MARKS: [Mark; 3] = [Mark::Bold, Mark::Italic, Mark::Underline];

/// A book with every sort of line and emphasis that sits close together.
const BOOK: &str = "!ANNA\n/*\na\nb\nc\nd\n*/Text.\n\n/*\neins\nzwei\ndrei\n*/\n!ANNA\nText.\n\n/*\na\nb\nc\n*/\nINT. HAUS - TAG\n!ANNA\nText.\n\nTitle: Nachtwache\n\n.INT. KÜCHE - TAG #12A#\n\n\
Anna *tritt* ein, **laut** und ***sehr laut***, _still_ [[Licht?]] und _**beides**_.\n\n\
@mcClane\n(leise)\nIhr kommt *spät* [[Pause]].\n\n>ENDE<\n\n# Akt 1\n\n\
Und \\*kein Stern\\* und a_b und 1 * 2, */ oder /* nicht.\n\nCUT TO:\n";

/// No emphasis but the one toggled is lost or made.
fn assert_rest_kept(source: &str, old: &[Vec<Marks>], mark: Mark, said: &str) {
    let rest = |m: &Marks| match mark {
        Mark::Bold => (m.italic, m.underline),
        Mark::Italic => (m.bold, m.underline),
        Mark::Underline => (m.bold, m.italic),
    };

    for (new, was) in marks_by_line(source).iter().zip(old) {
        let (new, was): (Vec<_>, Vec<_>) = (
            new.iter().map(rest).collect(),
            was.iter().map(rest).collect(),
        );

        assert_eq!(new, was, "{said} gave {source:?}");
    }
}

/// Each character of each line of `book` as a selection: toggling changes no text and no kind of
/// any line, and what the writer could mark is marked.
fn each_selection(book: &str, step: usize) {
    let lines: Vec<&str> = book.split('\n').collect();
    let kinds = crate::text_kinds(book);
    let texts: Vec<String> = shown_lines(book, &SHOWN)
        .into_iter()
        .map(|l| l.text)
        .collect();
    let old_marks = marks_by_line(book);

    for (at, line) in lines.iter().enumerate() {
        let len = line.encode_utf16().count();

        for mark in MARKS {
            for (from, to) in (0..=len)
                .step_by(step)
                .flat_map(|f| (f..=len).step_by(step).map(move |t| (f, t)))
            {
                let done = toggle_mark(book, &SHOWN, (spot(at, from), spot(at, to)), mark);
                let said = format!("{mark:?} {from}..{to} of {line:?}");

                assert_eq!(crate::text_kinds(&done.source), kinds, "{said}");
                let after: Vec<String> = shown_lines(&done.source, &SHOWN)
                    .into_iter()
                    .map(|l| l.text)
                    .collect();
                assert_eq!(after, texts, "{said} gave {:?}", done.source);

                assert_rest_kept(&done.source, &old_marks, mark, &said);
            }
        }
    }
}

#[test]
fn toggling_over_any_selection_keeps_text_and_kinds() {
    each_selection(BOOK, BOOK_STEP);
    each_selection(EXAMPLE, SAMPLE_STEP);
}

const LCG_MUL: u64 = 6_364_136_223_846_793_005;
const LCG_ADD: u64 = 1_442_695_040_888_963_407;
const BOOKS: u64 = 400;
const BOOK_LINES: u64 = 12;
const POOL: [&str; 16] = [
    "",
    "",
    "INT. HAUS - TAG",
    "ANNA",
    "ANNA (V.O.)",
    "(lacht)",
    "Ich bin weg.",
    "~eins",
    "!LAUT",
    "> eins <",
    "CUT TO:",
    "Sie *sagt* nein.",
    "@anna",
    "  ~zwei",
    "/* x */",
    "Text.",
];

/// Books of lines in blocks with and without blank lines between them: a toggle over all of a
/// book keeps every kind and leaves nothing that one line alone could still take.
#[test]
fn books_of_mixed_blocks_toggle_exactly_and_fully() {
    let mut state: u64 = 1;
    let mut next = |below: u64| {
        state = state.wrapping_mul(LCG_MUL).wrapping_add(LCG_ADD);
        ((state >> 33) % below) as usize
    };

    for _ in 0..BOOKS {
        let count = 2 + next(BOOK_LINES - 1);
        let lines: Vec<&str> = (0..count).map(|_| POOL[next(POOL.len() as u64)]).collect();
        let book = lines.join("\n");

        for mark in MARKS {
            let done = over_all(&book, mark);

            assert_eq!(
                crate::text_kinds(&done),
                crate::text_kinds(&book),
                "{mark:?} {book:?}"
            );
            assert_maximal(&book, mark);
        }
    }
}
