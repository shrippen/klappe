//! Marks inside a line of Fountain text: emphasis, notes and boneyards, so a surface can
//! show them without reading the rules again.

use crate::parse::{BONEYARD_CLOSE, BONEYARD_OPEN, ESCAPE, NOTE_CLOSE, NOTE_OPEN, STAR};

const UNDERLINE: char = '_';
/// Longest run of stars that means emphasis: `***bold italic***`.
const MAX_STARS: usize = 3;

/// What applies to a character.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Marks {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub note: bool,
    pub boneyard: bool,
}

/// Characters `start..end` of the line share `marks`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Run {
    pub start: usize,
    pub end: usize,
    pub marks: Marks,
}

fn starts(chars: &[char], at: usize, token: &str) -> bool {
    token
        .chars()
        .enumerate()
        .all(|(i, t)| chars.get(at + i) == Some(&t))
}

fn find(chars: &[char], from: usize, token: &str) -> Option<usize> {
    (from..chars.len()).find(|&i| starts(chars, i, token))
}

/// Length of the run of `c` at `at`.
fn run_of(chars: &[char], at: usize, c: char) -> usize {
    chars[at..].iter().take_while(|&&x| x == c).count()
}

/// Whether the same star run comes again later on the line.
fn closes_later(chars: &[char], from: usize, run: usize) -> bool {
    let mut at = from;

    while at < chars.len() {
        if chars[at] == ESCAPE {
            at += 2;
            continue;
        }
        if chars[at] != STAR {
            at += 1;
            continue;
        }
        let here = run_of(chars, at, STAR);
        if here == run {
            return true;
        }
        at += here;
    }

    false
}

/// Reads one line. `in_boneyard` says whether the line starts inside a `/* … */` that an
/// earlier line opened; the second value says whether the next line does.
///
/// Emphasis counts only when its closing mark is on the same line, so a lone `*` is text.
/// Example: `a *b* [[c]]` gives italic for `*b*` and note for `[[c]]`.
pub fn inline_runs(line: &str, in_boneyard: bool) -> (Vec<Run>, bool) {
    let scan = scan(line, in_boneyard);

    (runs(&scan.marks), scan.boneyard)
}

/// What `scan` found: the marks of each character, which characters open or close an
/// emphasis (a lone `*` or `_` does neither), and whether the next line is in a boneyard.
struct Scan {
    marks: Vec<Marks>,
    toggles: Vec<bool>,
    boneyard: bool,
}

fn scan(line: &str, in_boneyard: bool) -> Scan {
    let chars: Vec<char> = line.chars().collect();
    let mut marks = vec![Marks::default(); chars.len()];
    let mut toggles = vec![false; chars.len()];
    let mut now = Marks::default();
    let mut boneyard = in_boneyard;
    let mut at = 0;

    while at < chars.len() {
        if boneyard {
            let end =
                find(&chars, at, BONEYARD_CLOSE).map_or(chars.len(), |i| i + BONEYARD_CLOSE.len());
            marks[at..end].iter_mut().for_each(|m| m.boneyard = true);
            boneyard = end == chars.len() && !starts(&chars, end.saturating_sub(2), BONEYARD_CLOSE);
            at = end;
            continue;
        }

        let c = chars[at];
        if starts(&chars, at, BONEYARD_OPEN) {
            boneyard = true;
            marks[at..at + 2].iter_mut().for_each(|m| m.boneyard = true);
            at += 2;
            continue;
        }
        if starts(&chars, at, NOTE_OPEN) {
            if let Some(end) = find(&chars, at + 2, NOTE_CLOSE) {
                let end = end + NOTE_CLOSE.len();
                marks[at..end].iter_mut().for_each(|m| m.note = true);
                at = end;
                continue;
            }
        }
        if c == ESCAPE {
            // An escaped character is text inside the emphasis it stands in.
            paint(&mut marks, at, 2, now, now);
            at += 2;
            continue;
        }

        let before = now;
        let step = match c {
            STAR => toggle_stars(&chars, at, &mut now),
            UNDERLINE => toggle_underline(&chars, at, &mut now),
            _ => 1,
        };
        // The marks of an emphasis count to it, the opening and the closing one.
        paint(&mut marks, at, step, before, now);
        // A mark that changed the emphasis is a mark; any other is text.
        if now != before {
            toggles[at..at + step].iter_mut().for_each(|t| *t = true);
        }
        at += step;
    }

    Scan {
        marks,
        toggles,
        boneyard,
    }
}

/// Gives `len` characters from `at` the emphasis that holds before or after them.
fn paint(marks: &mut [Marks], at: usize, len: usize, before: Marks, now: Marks) {
    for m in marks.iter_mut().skip(at).take(len) {
        m.bold = before.bold || now.bold;
        m.italic = before.italic || now.italic;
        m.underline = before.underline || now.underline;
    }
}

/// Handles a run of stars at `at`; returns how many characters it took.
fn toggle_stars(chars: &[char], at: usize, now: &mut Marks) -> usize {
    let run = run_of(chars, at, STAR);
    if run > MAX_STARS {
        return run;
    }

    let open = match run {
        1 => now.italic,
        2 => now.bold,
        _ => now.bold && now.italic,
    };
    if !open && !closes_later(chars, at + run, run) {
        return run;
    }

    match run {
        1 => now.italic = !now.italic,
        2 => now.bold = !now.bold,
        _ => {
            now.bold = !open;
            now.italic = !open;
        }
    }

    run
}

fn toggle_underline(chars: &[char], at: usize, now: &mut Marks) -> usize {
    let closes = chars[at + 1..].contains(&UNDERLINE);
    if now.underline || closes {
        now.underline = !now.underline;
    }

    1
}

/// The text of `text` without what a reader does not say: notes, the marks of emphasis
/// (`*`, `_`) and the backslash of `\*`; lines are joined by a space. Example:
/// `Hallo [[Regie]] *laut*` gives `Hallo laut`.
/// Boneyards are gone before (the parser cuts them); a `/*` here is text, as in `Und/* oder`.
pub fn plain_text(text: &str) -> String {
    let mut out = String::new();

    for line in text.lines() {
        for (marks, part) in visible_runs(line) {
            if !marks.note {
                out.push_str(&part);
            }
        }
        out.push(' ');
    }

    // Left-out parts leave double spaces.
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The runs of a line with the text a reader sees: the marks that open or close an emphasis
/// are gone, and so is the backslash of `\*`, `\_`, `\[` and `\\`; a `*` or `_` that opens
/// nothing is text. Notes come with their marks, for the caller to leave out.
pub(crate) fn visible_runs(line: &str) -> Vec<(Marks, String)> {
    let chars: Vec<char> = line.chars().collect();
    let scan = scan(line, false);
    let escapes = escape_marks(&chars);

    runs(&scan.marks)
        .iter()
        .map(|run| {
            let mut part = String::new();
            for i in run.start..run.end {
                if !scan.toggles[i] && !escapes[i] {
                    part.push(chars[i]);
                }
            }

            (run.marks, part)
        })
        .collect()
}

/// Which characters are a backslash that protects what follows (a mark, a bracket or another
/// backslash). The protected character is text, so `\\*` is a backslash and
/// then a mark.
fn escape_marks(chars: &[char]) -> Vec<bool> {
    let mut escapes = vec![false; chars.len()];
    let mut i = 0;

    while i < chars.len() {
        let protects = chars[i] == ESCAPE
            && chars
                .get(i + 1)
                .is_some_and(|c| is_mark(*c) || *c == ESCAPE || *c == NOTE_BRACKET);
        escapes[i] = protects;
        i += if protects { 2 } else { 1 };
    }

    escapes
}

/// The character a note opens with.
const NOTE_BRACKET: char = '[';

fn is_mark(c: char) -> bool {
    c == STAR || c == UNDERLINE
}

/// Joins neighbours with the same marks; plain text gets runs too, so they cover the line.
fn runs(marks: &[Marks]) -> Vec<Run> {
    let mut out: Vec<Run> = Vec::new();

    for (i, m) in marks.iter().enumerate() {
        match out.last_mut() {
            Some(last) if last.marks == *m => last.end = i + 1,
            _ => out.push(Run {
                start: i,
                end: i + 1,
                marks: *m,
            }),
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn marked(line: &str) -> Vec<(String, Marks)> {
        let chars: Vec<char> = line.chars().collect();
        let (runs, _) = inline_runs(line, false);

        runs.iter()
            .map(|r| (chars[r.start..r.end].iter().collect(), r.marks))
            .collect()
    }

    fn only(f: impl Fn(&mut Marks)) -> Marks {
        let mut m = Marks::default();
        f(&mut m);
        m
    }

    #[test]
    fn emphasis_covers_its_marks() {
        let got = marked("a *b* **c** ***d*** _e_");

        let italic = only(|m| m.italic = true);
        let bold = only(|m| m.bold = true);
        let both = only(|m| {
            m.bold = true;
            m.italic = true;
        });
        let under = only(|m| m.underline = true);
        let plain = Marks::default();
        assert_eq!(
            got,
            [
                ("a ".to_owned(), plain),
                ("*b*".to_owned(), italic),
                (" ".to_owned(), plain),
                ("**c**".to_owned(), bold),
                (" ".to_owned(), plain),
                ("***d***".to_owned(), both),
                (" ".to_owned(), plain),
                ("_e_".to_owned(), under),
            ]
        );
    }

    #[test]
    fn a_lone_mark_is_text() {
        for line in ["2 * 3", "a_b", "**auf", "x \\*y\\* z"] {
            assert_eq!(
                marked(line),
                [(line.to_owned(), Marks::default())],
                "{line}"
            );
        }
    }

    #[test]
    fn notes_mark_their_brackets() {
        let got = marked("Hallo [[prüfen]] Welt");

        assert_eq!(got.len(), 3);
        assert_eq!(got[1], ("[[prüfen]]".to_owned(), only(|m| m.note = true)));
        assert_eq!(marked("[[offen")[0].1, Marks::default());
    }

    #[test]
    fn a_boneyard_can_run_over_lines() {
        let (first, open) = inline_runs("a /* b", false);
        assert!(open);
        assert_eq!(first[1].marks, only(|m| m.boneyard = true));

        let (middle, open) = inline_runs("noch drin", true);
        assert!(open);
        assert_eq!(middle.len(), 1);
        assert!(middle[0].marks.boneyard);

        let (last, open) = inline_runs("ende */ danach", true);
        assert!(!open);
        assert_eq!(last.len(), 2);
        assert!(last[0].marks.boneyard && !last[1].marks.boneyard);

        let (whole, open) = inline_runs("x /* y */ z", false);
        assert!(!open);
        assert_eq!(whole.len(), 3);
    }

    #[test]
    fn plain_text_drops_notes_and_marks() {
        assert_eq!(
            plain_text("Hallo [[Notiz an Regie]] *laut* _jetzt_.\nNoch **mehr**."),
            "Hallo laut jetzt. Noch mehr."
        );
        assert_eq!(plain_text("Zwei \\* drei"), "Zwei * drei");
        assert_eq!(
            plain_text("Hallo  *laut* Und/* oder."),
            "Hallo laut Und/* oder."
        );
        assert_eq!(plain_text("[[Nur eine Notiz]]"), "");
    }

    #[test]
    fn runs_cover_the_whole_line() {
        let (runs, _) = inline_runs("ä *b* ü", false);

        assert_eq!(runs.first().map(|r| r.start), Some(0));
        assert_eq!(runs.last().map(|r| r.end), Some(7));
        assert!(inline_runs("", false).0.is_empty());
    }
}
