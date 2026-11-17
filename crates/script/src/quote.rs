//! Quote anchors (DB-64): a line of a scene named by its content, so a link survives
//! lines inserted before it.
//!
//! A [`Quote`] holds the line, how many equal lines the scene had and the fingerprints of
//! the elements next to it. Lookup: the exact line; if the scene still has as many equal
//! lines, the occurrence decides; if the number changed, the one equal line with both
//! neighbours (not the scene edge, not another equal line) does; else the clearly most
//! similar line of the same kind, neighbours
//! breaking ties; else the candidates. Case, spacing and dash variants (`–`, `—`, `-`)
//! do not matter.

use crate::text::{hash, normal, words};
use crate::Element;
use crate::Fingerprint;

/// Words in common over words in all (Jaccard) from which a line counts as similar:
/// 0.6 survives a fixed typo or one changed word in a sentence of five.
const SIMILAR_MIN: f64 = 0.6;
/// A best candidate needs this lead over the next to be taken without asking.
const LEAD_MIN: f64 = 0.15;
/// The elements before and after a line.
const NEIGHBOURS: usize = 2;

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
    /// The elements before and after (`None` at the scene's edge or at a page break).
    pub before: Option<Fingerprint>,
    pub after: Option<Fingerprint>,
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

    Some(Quote {
        kind: line.kind,
        character: line.character,
        text: line.text,
        occurrence,
        equal: equal_at.len(),
        before: index.checked_sub(1).and_then(|i| print(scene, i)),
        after: print(scene, index + 1),
    })
}

/// Finds the quote in the (matched) scene: the exact line, else the one clearly most
/// similar line, else the candidates.
pub fn find_quote(scene: &[Element], quote: &Quote) -> Found {
    let lines: Vec<(usize, Line)> = (0..scene.len())
        .filter_map(|i| line(scene, i).map(|l| (i, l)))
        .collect();

    let wanted = Line::of(quote);
    let equal: Vec<usize> = lines
        .iter()
        .filter(|(_, l)| l.same(&wanted))
        .map(|&(i, _)| i)
        .collect();
    if equal.is_empty() {
        return similar(scene, &lines, quote);
    }
    if equal.len() == quote.equal {
        return Found::At(equal[quote.occurrence.min(equal.len() - 1)]);
    }

    // The number of equal lines changed: only the neighbours tell which one it was.
    match by_both(scene, &equal, quote) {
        Some(i) => Found::At(i),
        None => Found::Ambiguous(equal),
    }
}

/// How many of the stored neighbours the element at `i` has.
fn near(scene: &[Element], i: usize, quote: &Quote) -> usize {
    let before = i.checked_sub(1).and_then(|b| print(scene, b));
    usize::from(before == quote.before) + usize::from(print(scene, i + 1) == quote.after)
}

/// The only one of the equal lines with both stored neighbours. The scene's edge and a
/// neighbour that is an equal line itself are no evidence: lines added before or after
/// the anchor look the same there.
fn by_both(scene: &[Element], equal: &[usize], quote: &Quote) -> Option<usize> {
    let proves = |at: Option<usize>, stored: Option<Fingerprint>| {
        let Some(i) = at.filter(|i| !equal.contains(i)) else {
            return false;
        };
        stored.is_some() && print(scene, i) == stored
    };

    let mut both = equal
        .iter()
        .filter(|&&i| proves(i.checked_sub(1), quote.before) && proves(Some(i + 1), quote.after));
    let first = both.next()?;
    both.next().is_none().then_some(*first)
}

/// The one of `candidates` with more matching neighbours than any other (at least one);
/// a lone candidate needs both.
fn by_neighbours(scene: &[Element], candidates: &[usize], quote: &Quote) -> Option<usize> {
    let mut rated: Vec<(usize, usize)> = candidates
        .iter()
        .map(|&i| (i, near(scene, i, quote)))
        .collect();
    rated.sort_by_key(|r| std::cmp::Reverse(r.1));

    match rated.as_slice() {
        [(i, n)] if *n == NEIGHBOURS => Some(*i),
        [(i, n), (_, next), ..] if n > next && *n > 0 => Some(*i),
        _ => None,
    }
}

/// Most similar line of the same kind, preferring the same speaker.
fn similar(scene: &[Element], lines: &[(usize, Line)], quote: &Quote) -> Found {
    let wanted = Line::of(quote);
    let mut pool: Vec<&(usize, Line)> = lines
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

    // Everything within the lead of the best is a rival; one rival is the answer.
    let rivals: Vec<usize> = scored
        .iter()
        .filter(|&&(_, s)| best - s < LEAD_MIN)
        .map(|&(i, _)| i)
        .collect();
    if rivals.len() == 1 {
        return Found::At(best_at);
    }
    match by_neighbours(scene, &rivals, quote) {
        Some(i) => Found::At(i),
        None => Found::Ambiguous(scored.iter().map(|a| a.0).collect()),
    }
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

fn line(scene: &[Element], index: usize) -> Option<Line> {
    let (kind, text) = match scene.get(index)? {
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
    };

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

/// Fingerprint of the element at `index`, if it is a line.
fn print(scene: &[Element], index: usize) -> Option<Fingerprint> {
    let l = line(scene, index)?;
    Some(Fingerprint(hash(l.kind as u8, &l.text)))
}

fn jaccard(a: &[String], b: &[String]) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    let shared = a.iter().filter(|w| b.contains(w)).count();
    shared as f64 / (a.len() + b.len() - shared) as f64
}
