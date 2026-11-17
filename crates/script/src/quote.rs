//! Quote anchors (DB-64): a line of a scene named by its content, so a link survives
//! lines inserted before it.
//!
//! A [`Quote`] holds the line (kind, speaker, text), how many equal lines the scene had
//! and where the line stood among the scene's fingerprints (`at`). To find it again the
//! old scene's fingerprints and the new scene's lines are paired (`anchor.rs`):
//!
//! - The line's old fingerprint is paired and the new line is the same (kind, speaker,
//!   text): that is the line, wherever it moved.
//! - It is not paired (edited, or one of several equal lines): look at its nearest paired
//!   neighbours before and after (the scene's edge counts). The line is among the unpaired
//!   new lines between their partners, if the partners are directly consecutive paired
//!   lines (else the line may have moved: ambiguous). One clearly most similar line there
//!   is the answer, several are ambiguous, none is missing.
//! - Nothing to go by (a note, a changed scene, a speaker that differs): the line only if
//!   it was the only equal line and is the only one now; else the equal lines as
//!   candidates, or missing.
//!
//! Case, spacing and dash variants (`–`, `—`, `-`) do not matter.

use std::collections::{HashMap, HashSet};

use crate::align::{print, CUE_TAG};
use crate::anchor::pair;
use crate::text::{hash, normal, words};
use crate::{Element, Fingerprint, SceneState};

/// Words in common over words in all (Jaccard) from which a line counts as similar:
/// 0.6 survives a fixed typo or one changed word in a sentence of five.
const SIMILAR_MIN: f64 = 0.6;
/// A best candidate needs this lead over the next to be taken without asking.
const LEAD_MIN: f64 = 0.15;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineKind {
    Action,
    Character,
    Parenthetical,
    Dialogue,
    Transition,
    Centered,
    Lyrics,
    Section,
    Synopsis,
    Note,
}

impl LineKind {
    /// The tag of the kind's fingerprints; `None` for lines that have none.
    pub(crate) fn tag(self) -> Option<u8> {
        match self {
            LineKind::Action => Some(1),
            LineKind::Character => Some(CUE_TAG),
            LineKind::Parenthetical => Some(3),
            LineKind::Dialogue => Some(4),
            LineKind::Transition => Some(5),
            LineKind::Centered => Some(6),
            LineKind::Lyrics => Some(7),
            LineKind::Section | LineKind::Synopsis | LineKind::Note => None,
        }
    }
}

/// One line of a scene: kind, speaker, text and which of the equal lines it is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Quote {
    pub kind: LineKind,
    /// The speaker of a dialogue or parenthetical.
    pub character: Option<String>,
    pub text: String,
    /// Among the lines of the scene equal in kind, speaker and text, from 0.
    pub occurrence: usize,
    /// How many equal lines the scene had when the quote was made.
    pub equal: usize,
    /// Index of the line among the scene's fingerprints (`SceneState::prints`); `None`
    /// for lines without one (note, section, synopsis).
    pub at: Option<usize>,
}

/// Where a quote is in a scene (indices into the scene's elements).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Found {
    At(usize),
    /// Candidates, best first.
    Ambiguous(Vec<usize>),
    Missing,
}

/// The quote of `scene[index]`; `None` for a page break or a scene heading.
pub fn quote_at(scene: &[Element], index: usize) -> Option<Quote> {
    let line = line(scene, index)?;
    let equal_at: Vec<usize> = (0..scene.len())
        .filter(|&i| self::line(scene, i).is_some_and(|l| l.same(&line)))
        .collect();
    let occurrence = equal_at.iter().filter(|&&i| i < index).count();
    let at = print(&scene[index]).map(|_| scene[..index].iter().filter_map(print).count());

    Some(Quote {
        kind: line.kind,
        character: line.character,
        text: line.text,
        occurrence,
        equal: equal_at.len(),
        at,
    })
}

/// Finds the quote in the (matched) new scene; `old` is the state of the old scene the
/// quote was made in.
pub fn find_quote(old: &SceneState, scene: &[Element], quote: &Quote) -> Found {
    let lines: Vec<(usize, Line)> = (0..scene.len())
        .filter_map(|i| line(scene, i).map(|l| (i, l)))
        .collect();
    let wanted = Line::of(quote);

    let Some(at) = quote
        .at
        .filter(|&a| old.prints.get(a) == own_print(quote).as_ref())
    else {
        return by_equal(&lines, &wanted, quote);
    };

    // The scene's fingerprints with the element each one belongs to.
    let printed: Vec<usize> = (0..scene.len())
        .filter(|&i| print(&scene[i]).is_some())
        .collect();
    let new_tokens: Vec<u32> = printed
        .iter()
        .filter_map(|&i| print(&scene[i]))
        .map(|p| p.hash)
        .collect();
    let old_tokens: Vec<u32> = old.prints.iter().map(|p| p.hash).collect();
    let pairs = pair(&old_tokens, &new_tokens);

    if let Some(n) = pairs[at] {
        let same = lines
            .iter()
            .any(|(i, l)| *i == printed[n] && l.same(&wanted));
        if same {
            return Found::At(printed[n]);
        }
        return by_equal(&lines, &wanted, quote);
    }

    // Unpaired: between the partners of the nearest paired lines before and after.
    let before = pairs[..at].iter().rev().find_map(|p| *p);
    let after = pairs[at + 1..].iter().find_map(|p| *p);
    let (from, to) = (before.map_or(0, |n| n + 1), after.unwrap_or(printed.len()));
    if from > to {
        return by_equal(&lines, &wanted, quote);
    }

    let paired: HashSet<usize> = pairs.iter().flatten().copied().collect();
    // Paired lines between the partners: the line may have moved out of its stretch.
    let spans = (from..to).any(|n| paired.contains(&n));
    let gap: HashSet<usize> = (from..to)
        .filter(|n| !paired.contains(n))
        .map(|n| printed[n])
        .collect();
    let between: Vec<&(usize, Line)> = lines.iter().filter(|(i, _)| gap.contains(i)).collect();
    let found = similar(&between, &wanted);

    // Equal twins unpaired in the same old stretch: one new line cannot tell whose it is.
    let twins = (before_old(&pairs, at)..after_old(&pairs, at))
        .filter(|&o| pairs[o].is_none() && old.prints[o] == old.prints[at])
        .count();
    match found {
        Found::At(i) if twins > 1 || spans => Found::Ambiguous(vec![i]),
        other => other,
    }
}

/// First old fingerprint after the nearest paired one before `at`.
fn before_old(pairs: &[Option<usize>], at: usize) -> usize {
    pairs[..at]
        .iter()
        .rposition(|p| p.is_some())
        .map_or(0, |i| i + 1)
}

/// The nearest paired old fingerprint after `at`, or the end.
fn after_old(pairs: &[Option<usize>], at: usize) -> usize {
    pairs[at + 1..]
        .iter()
        .position(|p| p.is_some())
        .map_or(pairs.len(), |i| at + 1 + i)
}

/// Looks up all quotes of one scene at once. Two different quotes that land on the same
/// line cannot both be right: both become ambiguous (a safety net on top of
/// [`find_quote`]; the same quote twice is the same answer twice).
pub fn find_quotes(old: &SceneState, scene: &[Element], quotes: &[Quote]) -> Vec<Found> {
    let mut found: Vec<Found> = quotes.iter().map(|q| find_quote(old, scene, q)).collect();

    let mut owners: HashMap<usize, Vec<&Quote>> = HashMap::new();
    for (f, q) in found.iter().zip(quotes) {
        let Found::At(i) = f else {
            continue;
        };
        let owner = owners.entry(*i).or_default();
        if !owner.contains(&q) {
            owner.push(q);
        }
    }

    for f in found.iter_mut() {
        match f {
            Found::At(i) if owners[i].len() > 1 => *f = Found::Ambiguous(vec![*i]),
            _ => {}
        }
    }
    found
}

/// The fingerprint the quote's line has.
fn own_print(q: &Quote) -> Option<Fingerprint> {
    let tag = q.kind.tag()?;
    Some(Fingerprint {
        tag,
        hash: hash(tag, &q.text),
    })
}

/// No pairing to go by: the line if it was and is the only equal one.
fn by_equal(lines: &[(usize, Line)], wanted: &Line, quote: &Quote) -> Found {
    let equal: Vec<usize> = lines
        .iter()
        .filter(|(_, l)| l.same(wanted))
        .map(|&(i, _)| i)
        .collect();

    match equal.as_slice() {
        [] => Found::Missing,
        [i] if quote.equal == 1 => Found::At(*i),
        _ => Found::Ambiguous(equal),
    }
}

/// Most similar line of the same kind, preferring the same speaker.
fn similar(lines: &[&(usize, Line)], wanted: &Line) -> Found {
    let mut pool: Vec<&&(usize, Line)> = lines
        .iter()
        .filter(|(_, l)| l.kind == wanted.kind)
        .collect();
    let same_speaker = pool.iter().any(|(_, l)| l.character == wanted.character);
    if same_speaker {
        pool.retain(|(_, l)| l.character == wanted.character);
    }

    let wanted_words = words(&wanted.text);
    let mut scored: Vec<(usize, f64)> = pool
        .iter()
        .map(|(i, l)| (*i, jaccard(&wanted_words, &words(&l.text))))
        .filter(|&(_, s)| s >= SIMILAR_MIN)
        .collect();
    scored.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));

    let Some(&(best_at, best)) = scored.first() else {
        return Found::Missing;
    };

    // Another speaker may be a rename or another line: only a confirmation will tell.
    let speaker_kept = lines
        .iter()
        .find(|(i, _)| *i == best_at)
        .is_some_and(|(_, l)| {
            l.character.as_deref().map(normal) == wanted.character.as_deref().map(normal)
        });
    let rivals = scored.iter().filter(|&&(_, s)| best - s < LEAD_MIN).count();
    if rivals == 1 && speaker_kept {
        return Found::At(best_at);
    }
    Found::Ambiguous(scored.iter().map(|a| a.0).collect())
}

struct Line {
    kind: LineKind,
    character: Option<String>,
    text: String,
}

impl Line {
    fn of(q: &Quote) -> Line {
        Line {
            kind: q.kind,
            character: q.character.clone(),
            text: q.text.clone(),
        }
    }

    /// Equal ignoring case and spacing of the words.
    fn same(&self, other: &Line) -> bool {
        self.kind == other.kind
            && self.character.as_deref().map(normal) == other.character.as_deref().map(normal)
            && normal(&self.text) == normal(&other.text)
    }
}

/// Kind and text of an element, `None` for a scene heading and a page break.
pub(crate) fn kind_text(el: &Element) -> Option<(LineKind, &str)> {
    Some(match el {
        Element::Action(t) => (LineKind::Action, t.as_str()),
        Element::Character { name, .. } => (LineKind::Character, name.as_str()),
        Element::Parenthetical(t) => (LineKind::Parenthetical, t.as_str()),
        Element::Dialogue(t) => (LineKind::Dialogue, t.as_str()),
        Element::Transition(t) => (LineKind::Transition, t.as_str()),
        Element::Centered(t) => (LineKind::Centered, t.as_str()),
        Element::Lyrics(t) => (LineKind::Lyrics, t.as_str()),
        Element::Section { text, .. } => (LineKind::Section, text.as_str()),
        Element::Synopsis(t) => (LineKind::Synopsis, t.as_str()),
        Element::Note(t) => (LineKind::Note, t.as_str()),
        Element::SceneHeading { .. } | Element::PageBreak => return None,
    })
}

fn line(scene: &[Element], index: usize) -> Option<Line> {
    let (kind, text) = kind_text(scene.get(index)?)?;

    let speaks = matches!(kind, LineKind::Parenthetical | LineKind::Dialogue);
    Some(Line {
        kind,
        character: if speaks { speaker(scene, index) } else { None },
        text: text.to_string(),
    })
}

/// The cue above a parenthetical or dialogue.
fn speaker(scene: &[Element], index: usize) -> Option<String> {
    scene[..index].iter().rev().find_map(|el| match el {
        Element::Character { name, .. } => Some(Some(name.clone())),
        Element::Parenthetical(_) | Element::Dialogue(_) => None,
        _ => Some(None),
    })?
}

fn jaccard(a: &[String], b: &[String]) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    let shared = a.iter().filter(|w| b.contains(w)).count();
    shared as f64 / (a.len() + b.len() - shared) as f64
}
