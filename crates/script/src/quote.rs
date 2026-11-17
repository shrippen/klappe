//! Quote anchors (DB-64): a line of a scene named by its content, so a link survives
//! lines inserted before it.

use crate::text::{normal, words};
use crate::Element;

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

/// One line of a scene: kind, speaker, text and which of the equal lines it is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Quote {
    pub kind: LineKind,
    /// The speaker of a dialogue or parenthetical.
    pub character: Option<String>,
    pub text: String,
    /// Among the lines of the scene equal in kind, speaker and text, from 0.
    pub occurrence: usize,
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
    let occurrence = (0..index)
        .filter_map(|i| self::line(scene, i))
        .filter(|l| l.same(&line))
        .count();

    Some(Quote {
        kind: line.kind,
        character: line.character,
        text: line.text,
        occurrence,
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
    match equal.len() {
        0 => {}
        n if quote.occurrence < n => return Found::At(equal[quote.occurrence]),
        _ => return Found::Ambiguous(equal),
    }

    similar(&lines, &wanted)
}

/// Most similar line of the same kind, preferring the same speaker.
fn similar(lines: &[(usize, Line)], wanted: &Line) -> Found {
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

    match scored.as_slice() {
        [] => Found::Missing,
        [(i, _)] => Found::At(*i),
        [(i, best), (_, next), ..] if best - next >= LEAD_MIN => Found::At(*i),
        all => Found::Ambiguous(all.iter().map(|a| a.0).collect()),
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

fn jaccard(a: &[String], b: &[String]) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    let shared = a.iter().filter(|w| b.contains(w)).count();
    shared as f64 / (a.len() + b.len() - shared) as f64
}
