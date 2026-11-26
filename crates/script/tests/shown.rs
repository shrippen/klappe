//! The projection of the editor (D-36): a round trip of every line of real and generated
//! screenplays, and the single cases of the rules.
#![allow(clippy::unwrap_used)]

use klappe_script::{
    edit_line, shown_lines, splice_lines, text_kinds, Hidden, Marks, Run, Shape, ShownLine,
    TextKind,
};

const EXAMPLE: &str = include_str!("../../../testdata/beispiel.klappe/drehbuch.fountain");

/// Every sort of line the projection knows, canonical as the editor writes it.
const CANON: &str = "Title: Nachtwache\nAuthor: Anna\n\n.INT. KÜCHE - TAG #12A#\n\n\
Anna *tritt* ein, **laut** und ***sehr laut***, _still_ [[Licht?]] und _**beides**_.\n\n\
@mcClane\n(leise)\nIhr kommt *spät* [[Pause]].\n\n\
ANNA ^\nEin Wort.\n\n\
>ENDE<\n\n>ÜBERBLENDUNG:\n\n~Ein Lied\n\n!#Kein Abschnitt\n\n\
# Akt 1\n= Sie kommt an.\n===\n[[Eine Notiz]]\n\n\
Und \\*kein Stern\\*, \\_kein\\_ Strich und 1 * 2 und a_b.\n\n\
/* aus\nkommentiert *x* \\* */\nDanach.\n\nCUT TO:\n\nINT. SAAL - NACHT\n";

fn mark(bold: bool, italic: bool, underline: bool) -> Marks {
    Marks {
        bold,
        italic,
        underline,
        note: false,
        boneyard: false,
    }
}

fn run(start: usize, end: usize, marks: Marks) -> Run {
    Run { start, end, marks }
}

/// What the editor holds of a line: shown and then edited again with the same text.
fn same_again(line: &str, kind: TextKind, shown: &ShownLine) -> String {
    edit_line(line, kind, shown.shape, &shown.text, &shown.runs)
}

fn shown_one(line: &str) -> ShownLine {
    shown_lines(line).remove(0)
}

/// Edits the line `source` (one line) to the text `text` without emphasis.
fn typed(source: &str, text: &str) -> String {
    let shown = shown_one(source);

    edit_line(source, shown.kind, shown.shape, text, &[])
}

fn round_trip(source: &str) -> usize {
    let lines: Vec<&str> = source.split('\n').collect();
    let mut changed = 0;

    for (at, shown) in shown_lines(source).iter().enumerate() {
        let again = same_again(lines[at], shown.kind, shown);
        if again == lines[at] {
            continue;
        }

        // A line that is not canonical comes back as another way to write what it said.
        changed += 1;
        let mut edited = lines.clone();
        edited[at] = &again;
        let after = shown_lines(&edited.join("\n")).remove(at);
        let said = format!("{:?} -> {again:?} in {source:?}", lines[at]);
        assert_eq!(after.kind, shown.kind, "{said}");
        assert_eq!(after.text, shown.text, "{said}");
        assert_eq!(after.runs, shown.runs, "{said}");
        assert_eq!(after.notes, shown.notes, "{said}");
        assert_eq!(after.number, shown.number, "{said}");
    }

    changed
}

#[test]
fn every_line_of_the_example_comes_back() {
    assert_eq!(round_trip(EXAMPLE), 0);
}

#[test]
fn a_canonical_screenplay_comes_back_unchanged() {
    assert_eq!(round_trip(CANON), 0);
}

#[test]
fn a_line_is_one_line_and_a_title_page_is_hidden() {
    let shown = shown_lines(CANON);

    assert_eq!(shown.len(), CANON.split('\n').count());
    assert_eq!(shown[0].hidden, Hidden::TitlePage);
    assert_eq!(shown[3].hidden, Hidden::No);
    assert_eq!(shown[2].kind, TextKind::Blank);
    assert_eq!(shown[2].text, "");
}

#[test]
fn the_shown_text_has_no_marks() {
    let shown = shown_lines(CANON);

    assert_eq!(shown[3].text, "INT. KÜCHE - TAG");
    assert_eq!(shown[3].number.as_deref(), Some("12A"));
    assert_eq!(
        shown[5].text,
        "Anna tritt ein, laut und sehr laut, still  und beides."
    );
    assert_eq!(shown[7].text, "mcClane");
    assert_eq!(shown[9].text, "Ihr kommt spät .");
    assert_eq!(shown[11].text, "ANNA");
    assert_eq!(shown[14].text, "ENDE");
    assert_eq!(shown[16].text, "ÜBERBLENDUNG:");
    assert_eq!(shown[18].text, "Ein Lied");
    assert_eq!(shown[20].text, "#Kein Abschnitt");
    assert_eq!(
        shown[27].text,
        "Und *kein Stern*, _kein_ Strich und 1 * 2 und a_b."
    );
}

#[test]
fn sections_notes_and_boneyards_show_their_marks() {
    let shown = shown_lines(CANON);

    for at in [22, 23, 24, 25, 29, 30] {
        assert_eq!(shown[at].shape, Shape::Typed, "{}", shown[at].text);
    }
    assert_eq!(shown[22].text, "# Akt 1");
    assert_eq!(shown[25].text, "[[Eine Notiz]]");
    assert_eq!(shown[30].text, "kommentiert *x* \\* */");
}

#[test]
fn emphasis_is_set_and_removed() {
    let line = "Anna schaut nach oben.";
    let shown = shown_one(line);
    let italic = [
        run(0, 5, mark(false, false, false)),
        run(5, 11, mark(false, true, false)),
        run(11, 22, mark(false, false, false)),
    ];
    let set = edit_line(line, shown.kind, shown.shape, &shown.text, &italic);
    assert_eq!(set, "Anna *schaut* nach oben.");

    let all = [
        run(0, 5, mark(false, false, false)),
        run(5, 11, mark(true, true, true)),
        run(11, 22, mark(false, false, false)),
    ];
    let more = edit_line(&set, shown.kind, shown.shape, &shown.text, &all);
    assert_eq!(more, "Anna _***schaut***_ nach oben.");

    let again = shown_one(&more);
    assert_eq!(again.runs, all);

    let plain = edit_line(&more, shown.kind, shown.shape, &shown.text, &[]);
    assert_eq!(plain, line);
}

#[test]
fn a_typed_mark_becomes_emphasis() {
    let shown = shown_one(&typed("Anna schaut.", "Anna *schaut*."));

    assert_eq!(shown.text, "Anna schaut.");
    assert_eq!(shown.runs[1].marks, mark(false, true, false));
}

#[test]
fn a_typed_heading_is_a_heading() {
    let line = "Es ist spät.";
    let source = format!("\n{}\n", typed(line, "INT. SAAL - TAG"));

    assert_eq!(text_kinds(&source)[1], TextKind::Heading);
}

#[test]
fn the_number_stays_at_the_heading() {
    let source = ".INT. KÜCHE - TAG #12A#";

    assert_eq!(
        typed(source, "INT. KÜCHE 2 - TAG"),
        ".INT. KÜCHE 2 - TAG #12A#"
    );
    // One blank leads up to the number; more are text.
    assert_eq!(
        typed("INT. SAAL - TAG  #3#  ", "INT. HOF"),
        "INT. HOF #3#  "
    );
}

#[test]
fn a_note_stays_where_it_was() {
    let source = "Hallo [[Regie]] Welt";
    let shown = shown_one(source);
    assert_eq!(shown.text, "Hallo  Welt");
    assert_eq!(shown.notes[0].at, 6);

    assert_eq!(
        typed(source, "Gut: Hallo  Welt"),
        "Gut: Hallo [[Regie]] Welt"
    );
    assert_eq!(typed(source, "Hallo  Welt!"), "Hallo [[Regie]] Welt!");
    assert_eq!(typed(source, "Hallo  du Welt"), "Hallo [[Regie]] du Welt");
    // The note of a text that was deleted goes to where the change starts.
    assert_eq!(typed(source, "Hallo!"), "Hallo[[Regie]]!");
}

#[test]
fn a_forced_cue_stays_forced() {
    assert_eq!(typed("@mcClane", "mcClane Jr"), "@mcClane Jr");
    assert_eq!(typed("@ mcClane ^", " mcClane Jr"), "@ mcClane Jr ^");
}

#[test]
fn a_forced_element_without_text_is_cleared() {
    assert_eq!(typed(".INT. KÜCHE #1#", ""), "");
    assert_eq!(typed(">ENDE<", ""), "");
}

#[test]
fn an_untouched_backslash_stays() {
    let line = "Ein \\*Stern\\* hier";

    assert_eq!(typed(line, "Ein *Stern* hier!"), "Ein \\*Stern\\* hier!");
    // Typed in front of it, it is emphasis.
    assert_eq!(typed(line, "Ein *x* *Stern* hier").matches('\\').count(), 2);
}

#[test]
fn a_slash_before_emphasis_does_not_open_a_boneyard() {
    let line = "Und/oder schaut";
    let shown = shown_one(line);
    let italic = [
        run(0, 4, mark(false, false, false)),
        run(4, 15, mark(false, true, false)),
    ];
    let source = edit_line(line, shown.kind, shown.shape, &shown.text, &italic);

    assert!(!source.contains("/*"), "{source}");
    assert_eq!(shown_one(&source).text, shown.text);
}

#[test]
fn touching_marks_do_not_become_stars() {
    let line = "ab";
    let shown = shown_one(line);
    let runs = [
        run(0, 1, mark(false, true, false)),
        run(1, 2, mark(true, true, false)),
    ];
    let source = edit_line(line, shown.kind, shown.shape, &shown.text, &runs);

    assert_eq!(shown_one(&source).text, "ab");
}

#[test]
fn columns_hold_umlauts_and_emoji() {
    let line = ".EXT. ÄÖ 🎬 *Ü* #1#";
    let shown = shown_one(line);

    assert_eq!(shown.text, "EXT. ÄÖ 🎬 Ü");
    assert_eq!(shown.cols.len(), shown.text.chars().count() + 1);
    for (i, ch) in shown.text.chars().enumerate() {
        let at = shown.cols[i];
        assert_eq!(line[at..].chars().next(), Some(ch), "column {i}");
    }
    assert_eq!(
        line[..shown.cols[shown.text.chars().count()]]
            .chars()
            .last(),
        Some('*')
    );
    assert_eq!(shown.cols[0], 1);
}

#[test]
fn a_typed_line_is_a_blank_or_typed_text() {
    assert_eq!(typed("  ", "x"), "x");
    assert_eq!(typed("# Akt", "# Akt 2"), "# Akt 2");
}

#[test]
fn a_line_is_deleted_and_inserted() {
    let source = "a\nb\nc\nd";
    let apply = |range: std::ops::Range<usize>, new: &[&str]| {
        let new: Vec<String> = new.iter().map(|s| s.to_string()).collect();
        let splice = splice_lines(source, range, &new);
        let mut lines: Vec<&str> = source.split('\n').collect();
        lines.splice(
            splice.from..splice.to,
            splice.lines.iter().map(String::as_str),
        );

        (splice.from..splice.to, lines.join("\n"))
    };

    assert_eq!(apply(1..2, &[]), (1..2, "a\nc\nd".to_owned()));
    assert_eq!(
        apply(2..2, &["x", "y"]),
        (2..2, "a\nb\nx\ny\nc\nd".to_owned())
    );
    assert_eq!(apply(1..3, &["b", "z"]), (2..3, "a\nb\nz\nd".to_owned()));
    // Nothing that stays is part of the splice.
    assert_eq!(
        apply(0..4, &["a", "b", "c", "d"]),
        (4..4, source.to_owned())
    );
    assert_eq!(apply(3..9, &["D"]), (3..4, "a\nb\nc\nD".to_owned()));
}

// --- fuzz ---

const LCG_MUL: u64 = 6_364_136_223_846_793_005;
const LCG_ADD: u64 = 1_442_695_040_888_963_407;
const SEEDS: u64 = 4000;
const PARTS: [&str; 24] = [
    "Anna", "geht", "ÄÖÜ", "🎬", "ANNA", "INT.", "KÜCHE", " ", " ", "*", "**", "_", "\\*", "\\",
    "#1#", "[[n]]", ".", "!", "@", "^", "/", "(V.O.)", ">", "<",
];
const LINE_PARTS: u64 = 7;

struct Gen(u64);

impl Gen {
    fn next(&mut self, below: u64) -> u64 {
        self.0 = self.0.wrapping_mul(LCG_MUL).wrapping_add(LCG_ADD);

        (self.0 >> 33) % below
    }

    fn line(&mut self) -> String {
        if self.next(5) == 0 {
            return String::new();
        }
        let count = 1 + self.next(LINE_PARTS);

        (0..count)
            .map(|_| PARTS[self.next(PARTS.len() as u64) as usize])
            .collect()
    }

    fn text(&mut self) -> String {
        let lines = 1 + self.next(8);

        (0..lines)
            .map(|_| self.line())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[test]
fn fuzz_a_line_without_change_reads_the_same() {
    let mut gen = Gen(1);
    let mut changed = 0;
    let mut lines = 0;

    for _ in 0..SEEDS {
        let source = gen.text();
        lines += source.split('\n').count();
        changed += round_trip(&source);
    }

    // Only a line with two marks that touch is written differently.
    assert!(changed * 50 < lines, "{changed} of {lines} lines changed");
}

const WORDS: [&str; 4] = ["Anna", "ÄÖÜ", "🎬", " "];

/// A word typed into a line shows as that word, and no other line changes.
#[test]
fn fuzz_an_edit_changes_only_its_line() {
    let mut gen = Gen(2);
    let mut moved = 0;

    for _ in 0..SEEDS {
        let source = gen.text();
        let lines: Vec<String> = source.split('\n').map(str::to_owned).collect();
        let shown = shown_lines(&source);
        let at = gen.next(lines.len() as u64) as usize;
        let one = &shown[at];

        let mut chars: Vec<char> = one.text.chars().collect();
        let place = gen.next(chars.len() as u64 + 1) as usize;
        let word = WORDS[gen.next(WORDS.len() as u64) as usize];
        chars.splice(place..place, word.chars());
        let text: String = chars.iter().collect();
        let new = edit_line(&lines[at], one.kind, one.shape, &text, &[]);

        let splice = splice_lines(&source, at..at + 1, std::slice::from_ref(&new));
        let mut edited = lines.clone();
        edited.splice(splice.from..splice.to, splice.lines);
        assert_eq!(edited.len(), lines.len(), "{source:?}");
        for (i, line) in edited.iter().enumerate().filter(|(i, _)| *i != at) {
            assert_eq!(line, &lines[i], "line {i} of {source:?}");
        }

        let after = shown_lines(&edited.join("\n")).remove(at);
        if after.kind != one.kind || after.shape != one.shape {
            moved += 1;
            continue;
        }
        // Marks the user left alone and a word that splits them are the user's: `**` becomes
        // italic between two stars once something stands between them.
        if one.text.contains(['*', '_', '\\', '[', '/']) {
            continue;
        }
        assert_eq!(after.text, text, "{:?} -> {new:?} in {source:?}", lines[at]);
    }

    // A word makes a blank line action or a cue not capitals; most lines stay what they are.
    assert!(
        moved * 2 < SEEDS as usize,
        "{moved} lines changed their kind"
    );
}
