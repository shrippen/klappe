//! One change of a line (D-36): the rules of the kind, the notes and the typed text.

use std::collections::BTreeMap;

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

/// `line` as the editor reads it: a cue has a line of speech below it, speech a cue above it,
/// the rest stands alone.
fn read_as(kind: TextKind, line: &str, hide: &[Hide]) -> ShownLine {
    match kind {
        TextKind::Character => shown_lines(&format!("{line}\nx"), hide).remove(0),
        TextKind::Parenthetical | TextKind::Dialogue => {
            shown_lines(&format!("ANNA\n{line}"), hide).remove(1)
        }
        _ => shown_lines(line, hide).remove(0),
    }
}

/// A shown line of its own and a random change of it, with the emphasis of the new text.
fn one_splice(gen: &mut Gen, hide: &'static [Hide]) -> Option<Case> {
    let line = gen.line();
    // A cue is a cue only with speech below it, speech only below a cue; any other line is
    // read by itself.
    let shown = match gen.next(3) {
        0 => read_as(TextKind::Character, &line, hide),
        1 => read_as(TextKind::Dialogue, &line, hide),
        _ => read_as(TextKind::Action, &line, hide),
    };
    let shown = match shown.kind {
        TextKind::Character | TextKind::Dialogue | TextKind::Parenthetical => shown,
        _ => read_as(TextKind::Action, &line, hide),
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
        hide,
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
    hide: &'static [Hide],
    line: String,
    shown: ShownLine,
    change: LineEdit,
    /// The emphasis of the new text.
    marks: Vec<Marks>,
}

/// What Fountain cannot write: the cases where the rules ask for more than a line can say.
/// Each has a test of its own below that pins what the writer does instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Limit {
    /// A heading cannot start with a dot: `..` is no forcing character.
    DotHeading,
    /// A transition cannot end with `<`: `>Text<` is centered.
    TransitionEnd,
    /// A cue cannot end with `^`: it marks dual dialogue.
    CueEnd,
    /// A note behind `(leise)` keeps the line from being a parenthetical.
    NoteEndsParenthetical,
    /// A blank with a note is a note.
    BlankWithNote,
    /// Without `Hide::Forcing` the forcing character put in front shows as text.
    VisibleLead,
    /// A `/*` that is text could open a boneyard; the writer protects it, the text alone reads
    /// as a boneyard.
    BoneyardText,
    /// Stars after a `/` or `*`, or after the stars of another run, would join them, and stars
    /// after a `/` open a boneyard: the run stays plain.
    TouchingStars,
}

const LIMITS: [Limit; 7] = [
    Limit::DotHeading,
    Limit::TransitionEnd,
    Limit::CueEnd,
    Limit::NoteEndsParenthetical,
    Limit::BlankWithNote,
    Limit::BoneyardText,
    Limit::TouchingStars,
];

/// `read` without its first character, as the forcing character put in front shows.
fn without_lead(read: &ShownLine) -> ShownLine {
    let count = read.text.chars().count();
    let marks = marks_of(&read.runs, count);

    ShownLine {
        text: read.text.chars().skip(1).collect(),
        runs: runs(&marks[count.min(1)..]),
        notes: read
            .notes
            .iter()
            .map(|n| ShownNote {
                at: n.at.saturating_sub(1),
                text: n.text.clone(),
            })
            .collect(),
        ..read.clone()
    }
}

fn has_stars(marks: Marks) -> bool {
    (marks.bold || marks.italic) && !marks.underline
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

    /// Whether the new text itself has a `/*`: it may open a boneyard.
    fn has_boneyard_text(&self) -> bool {
        self.text().iter().collect::<String>().contains("/*")
    }

    /// The limit of Fountain this case runs into, if any.
    fn limit(&self) -> Option<Limit> {
        let text = self.text();
        let expected = self.kind();
        let trimmed: String = text.iter().collect::<String>().trim_end().to_owned();
        let at_end = moved(&self.shown.notes, &self.change)
            .iter()
            .any(|n| n.at >= text.len());

        let kind_limits = [
            (
                TextKind::Heading,
                trimmed.starts_with('.'),
                Limit::DotHeading,
            ),
            (
                TextKind::Transition,
                trimmed.ends_with('<'),
                Limit::TransitionEnd,
            ),
            (TextKind::Character, trimmed.ends_with('^'), Limit::CueEnd),
            (
                TextKind::Parenthetical,
                at_end,
                Limit::NoteEndsParenthetical,
            ),
            (
                TextKind::Blank,
                !self.shown.notes.is_empty(),
                Limit::BlankWithNote,
            ),
        ];

        if trimmed.contains("/*") {
            return Some(Limit::BoneyardText);
        }
        if let Some((_, _, limit)) = kind_limits
            .iter()
            .find(|(k, met, _)| *k == expected && *met)
        {
            return Some(*limit);
        }

        self.touching().then_some(Limit::TouchingStars)
    }

    /// Whether a run of stars starts behind a `/`, a `*` or the stars of another run, or ends
    /// behind a `/`: the stars would join or open a boneyard.
    fn touching(&self) -> bool {
        let text = self.text();

        (1..=text.len()).any(|i| {
            let before = self.marks[i - 1];
            let now = self.marks.get(i).copied().unwrap_or_default();
            let opens = has_stars(now) && (matches!(text[i - 1], '/' | '*') || has_stars(before));

            before != now && (opens || has_stars(before) && text[i - 1] == '/')
        })
    }

    /// The kind: the old one while the visible text reads as before, else the new one.
    fn kind(&self) -> TextKind {
        let (old, text) = (self.old(), self.text());
        let reads = |chars: &[char]| {
            read_as(
                self.shown.kind,
                &chars.iter().collect::<String>(),
                self.hide,
            )
            .kind
        };
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

/// The lines below a changed one: the scene stays a scene and nothing is commented out.
const BELOW: &str = "\n\nINT. HOF\n\nAnna geht.";

/// The kinds of the lines of `BELOW` in a text that has `line` above it.
fn kinds_below(line: &str, hide: &[Hide]) -> Vec<TextKind> {
    let all = format!("{line}{BELOW}");

    shown_lines(&all, hide)
        .iter()
        .skip(1)
        .map(|s| s.kind)
        .collect()
}

/// What the change did: whether a forcing character put in front shows as text, and a miss:
/// the limit it ran into, or what went wrong outside every limit.
fn judge(case: &Case, below: &[TextKind]) -> (bool, Option<Result<Limit, String>>) {
    let (line, shown, hide) = (&case.line, &case.shown, case.hide);
    let new = edit_line(line, shown.kind, shown.shape, hide, &case.change);
    let read = read_as(shown.kind, &new, hide);

    // A line that became another kind shows its marks: nothing to compare.
    let moved = read.shape != Shape::Shown || read.kind != shown.kind;
    // Without `Hide::Forcing` a forcing character put in front shows as text.
    let exact = case.kept_by(&read);
    let kept = exact || case.kept_by(&without_lead(&read));
    let visible_lead = !moved && !exact && kept && !hide.contains(&Hide::Forcing);
    let wrong = read.kind != case.kind() || (!moved && !kept);
    let opened = kinds_below(&new, hide) != below && !case.has_boneyard_text();
    if !wrong && !opened {
        return (visible_lead, None);
    }

    let said = format!(
        "{line:?} {:?} {:?} -> {new:?} (was {:?}, {hide:?})",
        case.change.removed, case.change.typed, shown.kind
    );

    (visible_lead, Some(case.limit().ok_or(said)))
}

/// Random changes of a line, shown with `hide`: none panics, the kind follows the rules, what
/// the change leaves alone keeps its text, emphasis and notes, and the lines below stay what
/// they were, except where Fountain cannot say it (`Limit`). Returns how often each limit hit.
fn fuzz_changes(hide: &'static [Hide], seed: u64) -> BTreeMap<Limit, usize> {
    let mut gen = Gen(seed);
    let mut tried = 0;
    let mut limited: BTreeMap<Limit, usize> = BTreeMap::new();
    let mut outside = Vec::new();
    let below = kinds_below("", hide);

    for _ in 0..SEEDS * 4 {
        let Some(case) = one_splice(&mut gen, hide) else {
            continue;
        };
        if case.text().is_empty() {
            continue;
        }

        tried += 1;
        let (visible_lead, miss) = judge(&case, &below);
        if visible_lead {
            *limited.entry(Limit::VisibleLead).or_default() += 1;
        }
        match miss {
            Some(Ok(limit)) => *limited.entry(limit).or_default() += 1,
            Some(Err(said)) => outside.push(said),
            None => {}
        }
    }

    assert!(tried > 2000, "{tried} changes");
    assert!(
        outside.is_empty(),
        "{} outside: {outside:#?}",
        outside.len()
    );

    limited
}

#[test]
fn fuzz_a_change_leaves_the_rest_alone() {
    let limited = fuzz_changes(&ALL, 4);

    // Every limit is met by the fuzz, so none of the predicates is dead.
    for limit in LIMITS {
        assert!(
            limited.contains_key(&limit),
            "{limit:?} never met: {limited:?}"
        );
    }
}

/// With only the emphasis hidden the forcing characters are text: the lines below must still
/// stay what they were (a `!` put in front must not hide a `/*`).
#[test]
fn fuzz_a_change_with_only_emphasis_hidden() {
    let limited = fuzz_changes(&[Hide::Emphasis], 5);

    assert!(limited.contains_key(&Limit::VisibleLead), "{limited:?}");
}

#[test]
fn a_forcing_character_put_in_front_shows_without_hide_forcing() {
    let hide = [Hide::Emphasis];
    let new = edited("*INT.* KÜCHE", &hide, &format_only(&[]));

    assert_eq!(new, "!INT. KÜCHE");
    assert_eq!(shown_lines(&new, &hide)[0].text, "!INT. KÜCHE");
}

// --- a put-in-front `!` must not hide a boneyard ---

/// The kinds of the lines of `BELOW` after the first line `source` is changed, with only the
/// emphasis hidden.
fn below_after(source: &str, change: &LineEdit) -> Vec<TextKind> {
    let hide = [Hide::Emphasis];
    let new = edited(&format!("{source}{BELOW}"), &hide, change);

    kinds_below(new.split('\n').next().unwrap_or_default(), &hide)
}

fn scene_stays(source: &str, change: &LineEdit) {
    let hide = [Hide::Emphasis];

    assert_eq!(
        below_after(source, change),
        kinds_below(source, &hide),
        "{source:?}"
    );
}

#[test]
fn italic_that_would_close_behind_a_slash_opens_no_boneyard() {
    let (italic, plain) = (mark(false, true, false), mark(false, false, false));
    let change = format_only(&[run(0, 4, italic), run(4, 8, plain)]);

    scene_stays("und/oder", &change);
    scene_stays("*und/oder*", &change);
}

#[test]
fn text_typed_behind_a_slash_in_italic_opens_no_boneyard() {
    let (italic, plain) = (mark(false, true, false), mark(false, false, false));
    let change = LineEdit {
        removed: 3..3,
        typed: "x".to_owned(),
        runs: vec![run(0, 3, italic), run(3, 4, plain), run(4, 8, italic)],
    };

    scene_stays("*Er/geht*", &change);
}

#[test]
fn a_typed_word_in_emphasis_around_a_slash_opens_no_boneyard() {
    let (italic, plain) = (mark(false, true, false), mark(false, false, false));
    let change = LineEdit {
        removed: 1..1,
        typed: "KÜCHE".to_owned(),
        runs: vec![
            run(0, 1, italic),
            run(1, 6, plain),
            run(6, 7, italic),
            run(7, 8, plain),
        ],
    };

    scene_stays("*/<_*_.", &change);
}

// --- the limits, one by one ---

#[test]
fn a_heading_cannot_start_with_a_dot() {
    // `.` alone is the forcing character: the text of the line is empty.
    let new = splice("INT.", 0..3, "");

    assert_eq!(new, ".");
    assert_eq!(shown_one(&new).kind, TextKind::Heading);
    assert_eq!(shown_one(&new).text, "");
}

#[test]
fn a_transition_cannot_end_with_a_chevron() {
    let new = splice(">Text", 4..4, "<");

    assert_eq!(new, ">Text<");
    assert_eq!(shown_one(&new).kind, TextKind::Centered);
}

#[test]
fn a_cue_cannot_end_with_a_caret() {
    let new = splice("@Anna", 4..4, "^");

    assert_eq!(new, "@Anna^");
    assert_eq!(read_as(TextKind::Character, &new, &ALL).text, "Anna");
}

#[test]
fn a_note_behind_a_parenthetical_makes_it_speech() {
    let source = "Hallo[[n]]";
    let shown = read_as(TextKind::Dialogue, source, &ALL);
    let new = edit_line(
        source,
        shown.kind,
        shown.shape,
        &ALL,
        &replace_text(0..5, "(leise)"),
    );
    let read = read_as(TextKind::Dialogue, &new, &ALL);

    assert_eq!(new, "(leise)[[n]]");
    assert_eq!(
        (read.kind, read.text.as_str()),
        (TextKind::Dialogue, "(leise)")
    );
}

#[test]
fn a_blank_with_a_note_is_a_note() {
    let new = splice("Hallo[[n]]", 0..5, " ");

    assert_eq!(new, " [[n]]");
    assert_eq!(shown_one(&new).kind, TextKind::Note);
}

#[test]
fn a_typed_boneyard_opener_is_protected() {
    let new = splice("a\\*", 0..1, "/");

    assert_eq!(new, "/\\*");
    assert_eq!(shown_one(&new).text, "/*");
    assert_eq!(shown_one(&new).kind, TextKind::Action);
}

#[test]
fn a_run_that_ends_behind_a_slash_stays_plain() {
    let source = edited(
        "a/",
        &ALL,
        &format_only(&[run(0, 2, mark(false, true, false))]),
    );

    assert_eq!(source, "a/");
}

#[test]
fn a_run_that_would_touch_the_stars_before_it_stays_plain() {
    let change = format_only(&[
        run(0, 1, mark(false, true, false)),
        run(1, 2, mark(true, true, false)),
    ]);
    let source = edited("ab", &ALL, &change);

    assert_eq!(
        shown_one(&source).runs,
        [
            run(0, 1, mark(false, true, false)),
            run(1, 2, mark(false, false, false)),
        ]
    );
}

// --- what a round trip may change ---

/// Whether the line has an emphasis mark or a note: the only things a rewrite moves.
pub(super) fn has_marks(line: &str) -> bool {
    line.contains("[[") || unmarked(line) != line
}

/// The line without its unescaped marks of emphasis and without its notes.
fn unmarked(line: &str) -> String {
    let mut out = String::new();
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '\\' => {
                out.push(ch);
                out.extend(chars.next());
            }
            '*' | '_' => {}
            '[' if chars.peek() == Some(&'[') => {
                let rest: String = chars.clone().collect();
                let Some(end) = rest.find("]]") else {
                    out.push(ch);
                    continue;
                };
                chars.nth(end + 1);
            }
            _ => out.push(ch),
        }
    }

    out
}

/// Whether `again` is `line` with other marks and notes in other places, or with the forcing
/// character that keeps its kind in front.
pub(super) fn moves_only_marks(line: &str, again: &str) -> bool {
    let (line, again) = (unmarked(line), unmarked(again));
    let forced = again
        .strip_prefix(['!', '@', '.', '~', '>'])
        .is_some_and(|rest| rest == line);

    again == line || forced
}
