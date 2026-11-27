//! One change of a line (D-36): the rules of the kind, the notes and the typed text.

use super::*;

/// The first line of `source` after `change`, shown with `hide`.
fn edited(source: &str, hide: &[Hide], change: &LineEdit) -> String {
    let shown = shown_lines(source, hide).remove(0);

    edit_line(source, shown.kind, shown.shape, hide, change)
}

fn kind_in(source: &str, hide: &[Hide]) -> TextKind {
    shown_lines(source, hide).remove(0).kind
}

fn replace_text(removed: Range<usize>, typed: &str) -> LineEdit {
    LineEdit {
        removed,
        typed: typed.to_owned(),
        runs: Vec::new(),
    }
}

#[test]
fn replacing_all_text_reads_as_the_new_text() {
    let source = "*E*s ist spät.";
    let all = replace_text(0..12, "EXT. HOF - TAG");
    let new = edited(source, &ALL, &all);

    assert_eq!(new, "EXT. HOF - TAG");
    assert_eq!(kind_in(&new, &ALL), TextKind::Heading);

    // The forcing character of the old line goes when it would hold the old kind.
    let new = edited("!Hallo", &ALL, &replace_text(0..5, "INT. HOF"));

    assert_eq!(new, "INT. HOF");
    assert_eq!(kind_in(&new, &ALL), TextKind::Heading);
}

#[test]
fn a_line_stays_what_it_was_when_only_its_emphasis_goes() {
    let only_emphasis = [Hide::Emphasis];

    for hide in [&ALL[..], &only_emphasis[..]] {
        let new = edited("*INT.* KÜCHE", hide, &format_only(&[]));

        assert_eq!(new, "!INT. KÜCHE", "{hide:?}");
        assert_eq!(kind_in(&new, hide), TextKind::Action, "{hide:?}");
    }
    for source in ["*#1* Hit", "*>* Hallo", "*~*Lied", "*=*Syn"] {
        let new = edited(source, &ALL, &format_only(&[]));

        assert_eq!(
            kind_in(&new, &ALL),
            TextKind::Action,
            "{source:?} -> {new:?}"
        );
        assert_eq!(shown_one(&new).text, shown_one(source).text, "{source:?}");
    }

    // A `!` typed behind `>Text` does not make it a transition.
    let new = edited("__>Text", &ALL, &replace_text(5..5, "!"));

    assert_eq!(kind_in(&new, &ALL), TextKind::Action);
    assert_eq!(shown_one(&new).text, ">Text!");
}

#[test]
fn deleting_a_visible_forcing_character_changes_the_kind() {
    let hide = [Hide::Emphasis];
    let source = "!INT. KÜCHE";

    assert_eq!(kind_in(source, &hide), TextKind::Action);
    let new = edited(source, &hide, &replace_text(0..1, ""));

    assert_eq!(new, "INT. KÜCHE");
    assert_eq!(kind_in(&new, &hide), TextKind::Heading);
}

#[test]
fn a_note_moves_behind_what_is_typed_at_it() {
    let source = "Hallo [[Regie]] Welt";

    // Typed exactly at the note: the note moves along.
    assert_eq!(splice(source, 6..6, "x"), "Hallo x[[Regie]] Welt");
    // Typed in front of it.
    assert_eq!(splice(source, 5..5, "x"), "Hallox [[Regie]] Welt");
    // Deleted over the note: it stands behind what replaces the text.
    assert_eq!(splice(source, 5..8, ""), "Hallo[[Regie]]elt");
    assert_eq!(splice(source, 5..8, "-"), "Hallo-[[Regie]]elt");
    // A note at the end stays at the end.
    assert_eq!(splice("Hallo[[Regie]]", 5..5, "!"), "Hallo![[Regie]]");
}

#[test]
fn typed_emphasis_that_would_hurt_its_neighbours_is_literal() {
    let source = "*a*";
    let change = LineEdit {
        removed: 1..1,
        typed: "*".to_owned(),
        runs: vec![
            run(0, 1, mark(false, true, false)),
            run(1, 2, mark(false, false, false)),
        ],
    };
    let new = edited(source, &ALL, &change);
    let shown = shown_one(&new);

    assert_eq!(new, "*a*\\*");
    assert_eq!(shown.text, "a*");
    assert_eq!(shown.runs[0].marks, mark(false, true, false));
}

#[test]
fn a_typed_heading_in_a_cue_line_is_not_a_cue() {
    assert_eq!(splice("@mcClane", 7..7, " Jr"), "@mcClane Jr");
    assert_eq!(
        kind_in(&splice("@mcClane", 7..7, " Jr"), &ALL),
        TextKind::Character
    );
}

/// `line` as the editor reads it: a cue has a line of speech below it, the rest stands alone.
fn read_as(kind: TextKind, line: &str) -> ShownLine {
    match kind {
        TextKind::Character => shown_lines(&format!("{line}\nx"), &ALL).remove(0),
        _ => shown_one(line),
    }
}

/// A shown line of its own and a random change of it, with the emphasis of the new text.
fn one_splice(gen: &mut Gen) -> Option<Case> {
    let line = gen.line();
    // A cue is a cue only with speech below it; any other line is read by itself.
    let cue = read_as(TextKind::Character, &line);
    let shown = match cue.kind == TextKind::Character {
        true => cue,
        false => shown_one(&line),
    };
    let len = shown.text.chars().count();
    if shown.shape != Shape::Shown || len == 0 {
        return None;
    }

    let start = gen.next(len as u64 + 1) as usize;
    let end = start + gen.next((len - start) as u64 + 1) as usize;
    let typed = match gen.next(4) {
        0 => "",
        _ => PARTS[gen.next(PARTS.len() as u64) as usize],
    };
    let mut marks = marks_of(&shown.runs, len);
    marks.splice(start..end, vec![Marks::default(); typed.chars().count()]);
    let change = LineEdit {
        removed: start..end,
        typed: typed.to_owned(),
        runs: runs(&marks),
    };

    Some(Case {
        line,
        shown,
        change,
        marks,
    })
}

/// The notes of the old text where rule 1 puts them.
fn moved(notes: &[ShownNote], change: &LineEdit) -> Vec<ShownNote> {
    let (start, end) = (change.removed.start, change.removed.end);
    let typed = change.typed.chars().count();

    notes
        .iter()
        .map(|n| ShownNote {
            at: match n.at < start {
                true => n.at,
                false => n.at.max(end) - (end - start) + typed,
            },
            text: n.text.clone(),
        })
        .collect()
}

/// What a changed line has to be: the kind the rules ask for, and what the change leaves alone.
struct Case {
    line: String,
    shown: ShownLine,
    change: LineEdit,
    /// The emphasis of the new text.
    marks: Vec<Marks>,
}

impl Case {
    fn old(&self) -> Vec<char> {
        self.shown.text.chars().collect()
    }

    fn typed(&self) -> Vec<char> {
        self.change.typed.chars().collect()
    }

    /// The new shown text.
    fn text(&self) -> Vec<char> {
        let old = self.old();
        let removed = &self.change.removed;

        [
            &old[..removed.start],
            &self.typed()[..],
            &old[removed.end..],
        ]
        .concat()
    }

    /// The kind: the old one while the visible text reads as before, else the new one.
    fn kind(&self) -> TextKind {
        let (old, text) = (self.old(), self.text());
        let reads =
            |chars: &[char]| read_as(self.shown.kind, &chars.iter().collect::<String>()).kind;
        let removed = &self.change.removed;
        let all_gone = removed.start == 0 && removed.end == old.len();

        match all_gone || reads(&text) != reads(&old) {
            true => reads(&text),
            false => self.shown.kind,
        }
    }

    /// Whether `read` keeps the text, emphasis and notes the change leaves alone.
    fn kept_by(&self, read: &ShownLine) -> bool {
        let old = self.old();
        let (start, end) = (self.change.removed.start, self.change.removed.end);
        let behind = start + self.typed().len();
        let now: Vec<char> = read.text.chars().collect();
        let Some(back) = now.len().checked_sub(old.len() - end) else {
            return false;
        };
        if back < start {
            return false;
        }

        let marks = marks_of(&read.runs, now.len());
        let notes = moved(&self.shown.notes, &self.change);
        let head: Vec<ShownNote> = notes.iter().filter(|n| n.at < start).cloned().collect();
        let tail: Vec<ShownNote> = notes
            .iter()
            .filter(|n| n.at >= behind)
            .map(|n| ShownNote {
                at: n.at - behind + back,
                text: n.text.clone(),
            })
            .collect();
        let found = &read.notes;

        now[..start] == old[..start]
            && now[back..] == old[end..]
            && marks[..start] == self.marks[..start]
            && marks[back..] == self.marks[behind..]
            && found.len() >= head.len() + tail.len()
            && found[..head.len()] == head[..]
            && found[found.len() - tail.len()..] == tail[..]
    }
}

/// A random change of a line never panics, the kind follows the rules, and what it leaves
/// alone keeps its text, emphasis and notes. A few lines cannot do both (a centered line
/// whose text turns into action keeps its marks), so the misses are counted.
#[test]
fn fuzz_a_change_leaves_the_rest_alone() {
    let mut gen = Gen(4);
    let (mut tried, mut kind_off, mut rest_off) = (0, 0, 0);
    let mut first = None;

    for _ in 0..SEEDS * 4 {
        let Some(case) = one_splice(&mut gen) else {
            continue;
        };
        if case.text().is_empty() {
            continue;
        }

        let (line, shown) = (&case.line, &case.shown);
        let new = edit_line(line, shown.kind, shown.shape, &ALL, &case.change);
        let read = read_as(shown.kind, &new);
        let said = format!(
            "{line:?} {:?} {:?} -> {new:?}",
            case.change.removed, case.change.typed
        );
        tried += 1;

        if read.kind != case.kind() {
            kind_off += 1;
            first.get_or_insert(format!("kind {said}"));
        }
        // A line that became another kind shows its marks: nothing to compare.
        if read.shape == Shape::Shown && read.kind == shown.kind && !case.kept_by(&read) {
            rest_off += 1;
            first.get_or_insert(said);
        }
    }

    assert!(tried > 2000, "{tried} changes");
    assert!(
        kind_off * 100 < tried,
        "{kind_off} of {tried} read as another kind: {first:?}"
    );
    assert!(
        rest_off * 100 < tried,
        "{rest_off} of {tried} changed what they leave alone: {first:?}"
    );
}
