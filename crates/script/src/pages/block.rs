//! The elements as blocks of rows, and where a block may be cut.

use super::wrap::wrap;
use super::{ContdAfterAction, Line, PageKind, PageOptions, Words, TEXT_COLUMNS};
use crate::Element;

/// A block cut in two keeps at least this many rows (not counting the name) on each page.
pub(super) const MIN_SPLIT_ROWS: usize = 2;

const DIALOGUE_X: usize = 10;
const DIALOGUE_WIDTH: usize = 35;
const PAREN_X: usize = 16;
const PAREN_WIDTH: usize = 20;
const CHARACTER_X: usize = 22;
const CHARACTER_WIDTH: usize = 38;
/// Dual dialogue: two columns of this width, the second starting at `DUAL_RIGHT_X`.
const DUAL_WIDTH: usize = 29;
const DUAL_RIGHT_X: usize = 31;
const DUAL_CHARACTER_X: usize = 6;
const DUAL_CHARACTER_WIDTH: usize = 23;
const DUAL_PAREN_X: usize = 4;
const DUAL_PAREN_WIDTH: usize = 21;
/// Blank rows before a scene heading and before the other blocks.
const HEADING_SPACE: usize = 2;
const BLOCK_SPACE: usize = 1;
const TERMINATORS: [char; 4] = ['.', '!', '?', '…'];
/// Words that end in a full stop without ending the sentence, lowercase; a single letter
/// (`z. B.`, `St.` is listed) does the same.
const ABBREVIATIONS: [&str; 16] = [
    "dr.", "hr.", "fr.", "st.", "nr.", "mr.", "mrs.", "ms.", "mt.", "prof.", "bzw.", "usw.", "ca.",
    "vgl.", "sr.", "jr.",
];
/// Month names, lowercase, German and English: `3. Mai` is a date, no sentence end.
const MONTHS: [&str; 24] = [
    "januar",
    "jänner",
    "februar",
    "märz",
    "april",
    "mai",
    "juni",
    "juli",
    "august",
    "september",
    "oktober",
    "november",
    "dezember",
    "january",
    "february",
    "march",
    "may",
    "june",
    "july",
    "october",
    "december",
    "sept",
    "okt",
    "dez",
];
/// Quotes and brackets that may stand before a word.
const OPENERS: [char; 8] = ['"', '\'', '(', '[', '„', '“', '«', '‚'];
const CLOSERS: [char; 6] = ['"', '\'', ')', '”', '»', '’'];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Align {
    Left,
    Right,
    Center,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cut {
    Whole,
    BetweenSentences,
}

/// Text of one element, in the column it is set in.
#[derive(Debug, Clone)]
struct Part {
    kind: PageKind,
    element: usize,
    x: usize,
    width: usize,
    align: Align,
    text: String,
    /// Where `text` starts in the element's text.
    base: usize,
    cut: Cut,
    number: Option<String>,
    /// Not in the script: the name repeated with `(CONT'D)`.
    added: bool,
}

impl Part {
    fn new(kind: PageKind, element: usize, text: String) -> Self {
        Self {
            kind,
            element,
            x: 0,
            width: TEXT_COLUMNS,
            align: Align::Left,
            text,
            base: 0,
            cut: Cut::Whole,
            number: None,
            added: false,
        }
    }

    fn at(mut self, x: usize, width: usize) -> Self {
        self.x = x;
        self.width = width;
        self
    }

    fn rows(&self) -> Vec<Vec<Line>> {
        let mut rows = Vec::new();
        let mut start = 0;

        for hard in self.text.split('\n') {
            for range in wrap(hard, self.width) {
                let text = hard[range.clone()].to_string();
                rows.push(vec![self.line(text, start + range.start)]);
            }
            start += hard.len() + 1;
        }

        if let Some(first) = rows.first_mut().and_then(|r| r.first_mut()) {
            first.number.clone_from(&self.number);
        }

        rows
    }

    fn line(&self, text: String, offset: usize) -> Line {
        let room = self.width.saturating_sub(text.chars().count());
        let shift = match self.align {
            Align::Left => 0,
            Align::Right => room,
            Align::Center => room / 2,
        };

        Line {
            row: 0,
            x: self.x + shift,
            kind: self.kind,
            text,
            element: self.element,
            offset: self.base + offset,
            number: None,
            added: self.added,
        }
    }

    /// The sentences before `at`, and the ones from `at` on.
    fn halves(&self, at: usize) -> (Part, Part) {
        let tail = &self.text[at..];
        let skipped = tail.len() - tail.trim_start().len();
        let first = Part {
            text: self.text[..at].trim_end().to_string(),
            ..self.clone()
        };
        let rest = Part {
            text: tail[skipped..].to_string(),
            base: self.base + at + skipped,
            number: None,
            ..self.clone()
        };

        (first, rest)
    }
}

/// Byte offsets where a sentence starts, but the first: after `. ! ? …` and closing quotes
/// when a capital or a digit follows (not `z. B. etwas`); a line break is none.
pub(super) fn sentence_starts(text: &str) -> Vec<usize> {
    let mut starts = Vec::new();
    let mut ended = false;
    let mut spaced = false;

    for (pos, c) in text.char_indices() {
        if c.is_whitespace() {
            spaced |= ended;
            continue;
        }

        if ended
            && spaced
            && (!c.is_lowercase()
                && !abbreviation(&text[..pos])
                && !ordinal_date(&text[..pos], &text[pos..]))
        {
            starts.push(pos);
        }
        if ended && !spaced && CLOSERS.contains(&c) {
            continue;
        }

        ended = TERMINATORS.contains(&c);
        spaced = false;
    }

    starts
}

/// Whether the last word of `before` is an abbreviation: `Dr.`, `Nr.`, `z.`.
fn abbreviation(before: &str) -> bool {
    let word = before
        .trim_end()
        .rsplit(char::is_whitespace)
        .next()
        .unwrap_or("");
    let word = word
        .trim_start_matches(OPENERS)
        .trim_end_matches(CLOSERS)
        .to_lowercase();
    let single = word
        .strip_suffix('.')
        .is_some_and(|w| w.chars().count() == 1 && w.chars().all(char::is_alphabetic));

    single || ABBREVIATIONS.contains(&word.as_str())
}

/// Whether `before` ends in a number with a full stop and a month follows: `3. Mai`.
fn ordinal_date(before: &str, after: &str) -> bool {
    let word = before
        .trim_end()
        .rsplit(char::is_whitespace)
        .next()
        .unwrap_or("");
    let number = word.strip_suffix('.').filter(|n| !n.is_empty());
    if !number.is_some_and(|n| n.chars().all(|c| c.is_ascii_digit())) {
        return false;
    }

    let next: String = after.chars().take_while(|c| c.is_alphabetic()).collect();

    MONTHS.contains(&next.to_lowercase().as_str())
}

#[derive(Debug, Clone)]
enum Body {
    Single(Vec<Part>),
    /// Two dialogues side by side.
    Dual(Vec<Part>, Vec<Part>),
    /// Rows already set, from a block that had to be cut anywhere.
    Rows(Vec<Vec<Line>>),
}

#[derive(Debug, Clone)]
pub(super) struct Block {
    /// Blank rows above, not at the top of a page.
    pub before: usize,
    body: Body,
    /// A scene heading: stays with the lines after it.
    pub heading: bool,
    /// A forced page break and nothing to print.
    pub page_break: bool,
}

impl Block {
    fn new(before: usize, parts: Vec<Part>) -> Self {
        Self {
            before,
            body: Body::Single(parts),
            heading: false,
            page_break: false,
        }
    }

    pub(super) fn rows(&self) -> Vec<Vec<Line>> {
        match &self.body {
            Body::Single(parts) => parts.iter().flat_map(Part::rows).collect(),
            Body::Dual(left, right) => zip(
                left.iter().flat_map(Part::rows).collect(),
                right.iter().flat_map(Part::rows).collect(),
            ),
            Body::Rows(rows) => rows.clone(),
        }
    }

    /// The first element of the block.
    pub(super) fn element(&self) -> Option<usize> {
        match &self.body {
            Body::Single(parts) | Body::Dual(parts, _) => parts.first().map(|p| p.element),
            Body::Rows(rows) => rows.first().and_then(|r| r.first()).map(|l| l.element),
        }
    }

    fn is_dialogue(&self) -> bool {
        matches!(&self.body, Body::Single(p) if p.first().is_some_and(|p| p.kind == PageKind::Character))
    }

    /// Rows without the name above a dialogue.
    fn body_rows(&self) -> usize {
        let rows = self.rows();

        rows.iter()
            .filter(|r| r.first().is_some_and(|l| l.kind != PageKind::Character))
            .count()
    }

    /// Cuts the block in two at the last place where the first half, with `(MORE)` for a
    /// dialogue, takes at most `room` rows and both halves keep `MIN_SPLIT_ROWS`; `None`
    /// when there is no such place.
    pub(super) fn split(&self, room: usize, words: Words) -> Option<(Block, Block)> {
        let Body::Single(parts) = &self.body else {
            return None;
        };
        let more = usize::from(self.is_dialogue());
        let mut fitting = Vec::new();

        for (i, at) in cuts(parts) {
            let (first, _) = self.cut(parts, i, at, words);
            if first.rows().len() + more > room {
                break;
            }
            fitting.push((i, at));
        }

        fitting.into_iter().rev().find_map(|(i, at)| {
            let (first, rest) = self.cut(parts, i, at, words);
            let both = first.body_rows() >= MIN_SPLIT_ROWS && rest.body_rows() >= MIN_SPLIT_ROWS;

            both.then_some((first, rest))
        })
    }

    fn cut(&self, parts: &[Part], i: usize, at: usize, words: Words) -> (Block, Block) {
        let mut first: Vec<Part> = parts[..i].to_vec();
        let mut rest = Vec::new();

        if self.is_dialogue() {
            rest.push(continued(&parts[0], words));
        }
        match at {
            0 => rest.push(parts[i].clone()),
            at => {
                let (head, tail) = parts[i].halves(at);
                first.push(head);
                rest.push(tail);
            }
        }
        rest.extend_from_slice(&parts[i + 1..]);

        let first = Block::new(self.before, first);
        let rest = Block::new(0, rest);

        (first, rest)
    }

    /// `rows` as a block of its own, for a cut at any row.
    pub(super) fn from_rows(rows: Vec<Vec<Line>>) -> Self {
        Block {
            before: 0,
            body: Body::Rows(rows),
            heading: false,
            page_break: false,
        }
    }
}

/// The column of a line: 1 from where a dual dialogue's right column starts, else 0.
pub(super) fn column(line: &Line) -> usize {
    usize::from(line.x >= DUAL_RIGHT_X)
}

/// The name with `(CONT'D)` after it, once.
pub(super) fn with_contd(name: &str, contd: &str) -> String {
    let mark = format!(" ({contd})");
    if name.ends_with(&mark) {
        return name.to_owned();
    }

    format!("{name}{mark}")
}

/// The name part of the next page's dialogue: the name with `(CONT'D)`.
fn continued(cue: &Part, words: Words) -> Part {
    Part {
        text: with_contd(&cue.text, words.contd),
        number: None,
        added: true,
        ..cue.clone()
    }
}

/// Rows side by side; the shorter column leaves the rest of the rows to the longer.
fn zip(left: Vec<Vec<Line>>, right: Vec<Vec<Line>>) -> Vec<Vec<Line>> {
    let count = left.len().max(right.len());
    let mut left = left.into_iter();
    let mut right = right.into_iter();

    (0..count)
        .map(|_| {
            let mut row = left.next().unwrap_or_default();
            row.extend(right.next().unwrap_or_default());
            row
        })
        .collect()
}

/// Places a block may be cut at, in order: `(part, byte in the part)`; byte 0 is between
/// parts. A dialogue breaks before a parenthetical or between sentences; never right
/// after the name or between a parenthetical and the words it belongs to.
fn cuts(parts: &[Part]) -> Vec<(usize, usize)> {
    let mut out = Vec::new();

    for (i, part) in parts.iter().enumerate() {
        let before = i.checked_sub(1).map(|p| parts[p].kind);
        let between =
            i > 0 && before != Some(PageKind::Character) && part.kind == PageKind::Parenthetical;
        if between {
            out.push((i, 0));
        }
        if part.cut == Cut::BetweenSentences {
            out.extend(sentence_starts(&part.text).into_iter().map(|at| (i, at)));
        }
    }

    out
}

/// What the cue of the next dialogue is called, and whether action came between.
#[derive(Default)]
struct Speaker {
    name: Option<String>,
    after_action: bool,
}

/// The printable blocks of the script, in order; notes, sections and synopses are not
/// printed.
pub(super) fn blocks(elements: &[Element], options: &PageOptions) -> Vec<Block> {
    let mut out: Vec<Block> = Vec::new();
    let mut speaker = Speaker::default();
    let mut i = 0;

    while i < elements.len() {
        let used = one(elements, i, options, &mut speaker, &mut out);
        i += used;
    }

    out
}

/// Turns the element at `i` (a name takes its dialogue with it) into a block; returns the
/// elements used.
fn one(
    elements: &[Element],
    i: usize,
    options: &PageOptions,
    speaker: &mut Speaker,
    out: &mut Vec<Block>,
) -> usize {
    match &elements[i] {
        Element::SceneHeading { text, number } => {
            *speaker = Speaker::default();
            let mut part = Part::new(PageKind::Heading, i, text.to_uppercase());
            part.number.clone_from(number);
            let mut block = Block::new(HEADING_SPACE, vec![part]);
            block.heading = true;
            out.push(block);
        }
        // Words without a name above them print as action.
        Element::Action(text) | Element::Dialogue(text) | Element::Parenthetical(text) => {
            speaker.after_action = true;
            let mut part = Part::new(PageKind::Action, i, text.clone());
            part.cut = Cut::BetweenSentences;
            out.push(Block::new(BLOCK_SPACE, vec![part]));
        }
        Element::Character { .. } => return dialogue(elements, i, options, speaker, out),
        Element::Transition(text) => {
            *speaker = Speaker::default();
            let mut part = Part::new(PageKind::Transition, i, text.to_uppercase());
            part.align = Align::Right;
            out.push(Block::new(BLOCK_SPACE, vec![part]));
        }
        Element::Centered(text) => {
            let mut part = Part::new(PageKind::Centered, i, text.clone());
            part.align = Align::Center;
            out.push(Block::new(BLOCK_SPACE, vec![part]));
        }
        Element::Lyrics(text) => {
            let part = Part::new(PageKind::Lyrics, i, text.clone()).at(DIALOGUE_X, DIALOGUE_WIDTH);
            // Lines of one song stay together.
            let joined = matches!(
                i.checked_sub(1).map(|p| &elements[p]),
                Some(Element::Lyrics(_))
            );
            out.push(Block::new(if joined { 0 } else { BLOCK_SPACE }, vec![part]));
        }
        Element::PageBreak => {
            let mut block = Block::new(0, Vec::new());
            block.page_break = true;
            out.push(block);
        }
        Element::Section { .. } | Element::Synopsis(_) | Element::Note(_) => {}
    }

    1
}

/// A name and what it says; a second voice (`^`) joins the block before it.
fn dialogue(
    elements: &[Element],
    i: usize,
    options: &PageOptions,
    speaker: &mut Speaker,
    out: &mut Vec<Block>,
) -> usize {
    let Element::Character {
        name,
        extension,
        dual,
    } = &elements[i]
    else {
        return 1;
    };

    let mut cue = name.to_uppercase();
    if let Some(ext) = extension {
        cue.push_str(&format!(" ({ext})"));
    }
    let again = speaker.name.as_deref() == Some(name.as_str()) && speaker.after_action;
    if again && options.contd_after_action == ContdAfterAction::On && !*dual {
        cue.push_str(&format!(" ({})", options.words.contd));
    }
    *speaker = Speaker {
        name: Some(name.clone()),
        after_action: false,
    };

    let side = *dual;
    let mut parts = vec![if side {
        Part::new(PageKind::Character, i, cue).at(DUAL_CHARACTER_X, DUAL_CHARACTER_WIDTH)
    } else {
        Part::new(PageKind::Character, i, cue).at(CHARACTER_X, CHARACTER_WIDTH)
    }];
    let mut used = 1;

    while let Some(element) = elements.get(i + used) {
        match element {
            Element::Parenthetical(text) => {
                let part = Part::new(PageKind::Parenthetical, i + used, format!("({text})"));
                parts.push(if side {
                    part.at(DUAL_PAREN_X, DUAL_PAREN_WIDTH)
                } else {
                    part.at(PAREN_X, PAREN_WIDTH)
                });
            }
            Element::Dialogue(text) => {
                let mut part = Part::new(PageKind::Dialogue, i + used, text.clone());
                part.cut = Cut::BetweenSentences;
                parts.push(if side {
                    part.at(0, DUAL_WIDTH)
                } else {
                    part.at(DIALOGUE_X, DIALOGUE_WIDTH)
                });
            }
            _ => break,
        }
        used += 1;
    }

    if *dual {
        join_dual(parts, out);
        return used;
    }
    out.push(Block::new(BLOCK_SPACE, parts));

    used
}

/// Puts the second voice beside the dialogue block before it; alone, it stays a block of
/// its own in the left column.
fn join_dual(right: Vec<Part>, out: &mut Vec<Block>) {
    let left = match out.pop() {
        Some(Block {
            body: Body::Single(parts),
            before,
            ..
        }) if parts.first().is_some_and(|p| p.kind == PageKind::Character) => Some((parts, before)),
        Some(other) => {
            out.push(other);
            None
        }
        None => None,
    };
    let Some((left, before)) = left else {
        out.push(Block::new(BLOCK_SPACE, right));
        return;
    };

    // The first voice was set at full width; narrow it to its column.
    let left = left.into_iter().map(narrow).collect();
    let right = right.into_iter().map(|mut p| {
        p.x += DUAL_RIGHT_X;
        p
    });
    let mut block = Block::new(before, Vec::new());
    block.body = Body::Dual(left, right.collect());
    out.push(block);
}

fn narrow(mut part: Part) -> Part {
    let (x, width) = match part.kind {
        PageKind::Character => (DUAL_CHARACTER_X, DUAL_CHARACTER_WIDTH),
        PageKind::Parenthetical => (DUAL_PAREN_X, DUAL_PAREN_WIDTH),
        _ => (0, DUAL_WIDTH),
    };
    part.x = x;
    part.width = width;

    part
}
