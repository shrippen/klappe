//! Matches the scenes of an old script state to the scenes of a new one (DB-65).
//!
//! The old state is what was saved: number, heading and a short fingerprint per element
//! ([`SceneState`]). The new state is a parsed [`Script`].
//!
//! Every old × new pair gets a score: the content similarity (Dice coefficient of the
//! fingerprint multisets, `2·shared / (a + b)`: 1 for equal content, independent of
//! order) plus small bonuses for the same number ([`NUMBER_BONUS`]) and the same heading
//! ([`HEADING_BONUS`]). A pair is accepted only if
//!
//! - its score is at least [`ACCEPT_MIN`],
//! - it is the best of its row and of its column, and
//! - it leads the second best of its row and of its column by [`LEAD_MIN`].
//!
//! So a neighbour that was deleted, inserted or renumbered cannot take a number, and a
//! near-twin makes the answer ambiguous: never guess (DATENMODELL principle 6). Every
//! other old scene gets the new scenes with a score of at least [`CANDIDATE_MIN`],
//! best first, as [`Outcome::Ambiguous`], except scenes another old scene was accepted
//! for; with none left it is [`Outcome::NotFound`].
//!
//! Numbers and headings never suffice alone: the bonuses together (0.4) stay below
//! `ACCEPT_MIN`; with number and heading equal and the content gone, the new scene is
//! offered as the single candidate. A scene without elements has no content to compare;
//! two of them count as equal if their headings are, so unchanged heading-only scenes
//! still match.
//!
//! Thresholds, chosen on the tests' cases:
//!
//! - `ACCEPT_MIN` 0.8: unnumbered scenes need nearly unchanged content; with number
//!   (0.3) and heading (0.1) half the content may have changed.
//! - `LEAD_MIN` 0.25: 0.8 against 0.6 is ambiguous; it is above `NUMBER_BONUS`, which
//!   alone separates two identical scenes with different numbers (1.4 against 1.1).
//! - `CANDIDATE_MIN` 0.4: a scene cut in half scores 0.67 against each half; below
//!   0.4 a "similar" scene is a coincidence of common lines ("Ja.").

use crate::text::{hash, normal};
use crate::{Element, SceneView, Script};

const ACCEPT_MIN: f64 = 0.8;
const LEAD_MIN: f64 = 0.25;
const NUMBER_BONUS: f64 = 0.3;
const HEADING_BONUS: f64 = 0.1;
const CANDIDATE_MIN: f64 = NUMBER_BONUS + HEADING_BONUS;

/// Hash of one element's kind and normalised text (FNV-1a, stable, can be saved).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Fingerprint(pub u32);

/// What is remembered of a scene to find it again after the script changed.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SceneState {
    pub number: Option<String>,
    pub heading: Option<String>,
    /// One per element that carries script text, in text order.
    pub prints: Vec<Fingerprint>,
}

impl SceneState {
    pub fn of(scene: &SceneView) -> SceneState {
        SceneState {
            number: scene.number.map(str::to_string),
            heading: scene.heading.map(str::to_string),
            prints: scene.elements.iter().filter_map(print).collect(),
        }
    }
}

/// The state of every scene of the script, in text order.
pub fn scene_states(script: &Script) -> Vec<SceneState> {
    script.scenes().iter().map(SceneState::of).collect()
}

/// How an old scene was found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum How {
    /// Both have the same number.
    Number,
    /// The numbers differ or are missing.
    Content,
}

/// What became of one old scene. Indices are into `new.scenes()`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Matched {
        scene: usize,
        how: How,
    },
    /// Candidates, best first; one candidate means "probably, please confirm".
    Ambiguous(Vec<usize>),
    NotFound,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Alignment {
    /// One per old scene.
    pub old: Vec<Outcome>,
    /// New scenes that are no old scene's match (ambiguous candidates included).
    pub unmatched: Vec<usize>,
}

/// Matches the old scenes to the scenes of `new`.
pub fn align(old: &[SceneState], new: &Script) -> Alignment {
    let new = scene_states(new);
    let scores = scores(old, &new);
    let rows: Vec<Lead> = scores.iter().map(|r| Lead::of(r)).collect();
    let cols: Vec<Lead> = (0..new.len())
        .map(|n| Lead::of(&scores.iter().map(|r| r[n]).collect::<Vec<_>>()))
        .collect();

    // Accepted pairs: best and clear leader in row and column, score high enough.
    let accepted: Vec<Option<usize>> = (0..old.len())
        .map(|o| {
            let n = rows[o].best?;
            let sure = scores[o][n] >= ACCEPT_MIN
                && rows[o].lead >= LEAD_MIN
                && cols[n].best == Some(o)
                && cols[n].lead >= LEAD_MIN;
            sure.then_some(n)
        })
        .collect();
    let taken: Vec<usize> = accepted.iter().flatten().copied().collect();

    let old_out = (0..old.len())
        .map(|o| match accepted[o] {
            Some(n) => Outcome::Matched {
                scene: n,
                how: how(&old[o], &new[n]),
            },
            None => offer(&scores[o], &taken),
        })
        .collect();
    let unmatched = (0..new.len()).filter(|n| !taken.contains(n)).collect();

    Alignment {
        old: old_out,
        unmatched,
    }
}

/// Best entry of a row or column and its lead over the second best (the whole value if
/// it is the only one; 0 on a tie).
struct Lead {
    best: Option<usize>,
    lead: f64,
}

impl Lead {
    fn of(values: &[f64]) -> Lead {
        let mut best: Option<usize> = None;
        for (i, v) in values.iter().enumerate() {
            if best.is_none_or(|b| *v > values[b]) {
                best = Some(i);
            }
        }
        let Some(b) = best else {
            return Lead { best, lead: 0.0 };
        };

        let second = values
            .iter()
            .enumerate()
            .filter(|&(i, _)| i != b)
            .map(|(_, v)| *v)
            .fold(0.0, f64::max);
        Lead {
            best,
            lead: values[b] - second,
        }
    }
}

/// Not accepted: the new scenes worth asking about, unless another old scene has them.
fn offer(row: &[f64], taken: &[usize]) -> Outcome {
    let mut found: Vec<usize> = (0..row.len())
        .filter(|n| row[*n] >= CANDIDATE_MIN && !taken.contains(n))
        .collect();
    if found.is_empty() {
        return Outcome::NotFound;
    }

    found.sort_by(|a, b| row[*b].total_cmp(&row[*a]).then(a.cmp(b)));
    Outcome::Ambiguous(found)
}

fn how(old: &SceneState, new: &SceneState) -> How {
    match (&old.number, &new.number) {
        (Some(a), Some(b)) if a == b => How::Number,
        _ => How::Content,
    }
}

/// `scores[o][n]`: content similarity plus the bonuses.
fn scores(old: &[SceneState], new: &[SceneState]) -> Vec<Vec<f64>> {
    let old_sorted: Vec<Vec<Fingerprint>> = old.iter().map(sorted).collect();
    let new_sorted: Vec<Vec<Fingerprint>> = new.iter().map(sorted).collect();
    let old_heads: Vec<Option<String>> = old.iter().map(heading).collect();
    let new_heads: Vec<Option<String>> = new.iter().map(heading).collect();

    old.iter()
        .enumerate()
        .map(|(o, a)| {
            new.iter()
                .enumerate()
                .map(|(n, b)| {
                    let same_head = old_heads[o].is_some() && old_heads[o] == new_heads[n];
                    score(a, b, same_head, dice(&old_sorted[o], &new_sorted[n]))
                })
                .collect()
        })
        .collect()
}

fn heading(state: &SceneState) -> Option<String> {
    state.heading.as_deref().map(normal)
}

fn score(old: &SceneState, new: &SceneState, same_head: bool, content: f64) -> f64 {
    let same_number = old.number.is_some() && old.number == new.number;

    // Two scenes without elements have no content to differ in.
    let empty = old.prints.is_empty() && new.prints.is_empty();
    let base = if empty && same_head { 1.0 } else { content };

    base + if same_number { NUMBER_BONUS } else { 0.0 }
        + if same_head { HEADING_BONUS } else { 0.0 }
}

/// Fingerprint of an element, `None` for those that are not script text (notes, page
/// breaks, sections, synopses, headings): editing them does not change the scene.
fn print(el: &Element) -> Option<Fingerprint> {
    let (tag, text): (u8, &str) = match el {
        Element::Action(t) => (1, t),
        Element::Character { name, .. } => (2, name),
        Element::Parenthetical(t) => (3, t),
        Element::Dialogue(t) => (4, t),
        Element::Transition(t) => (5, t),
        Element::Centered(t) => (6, t),
        Element::Lyrics(t) => (7, t),
        _ => return None,
    };
    Some(Fingerprint(hash(tag, text)))
}

fn sorted(state: &SceneState) -> Vec<Fingerprint> {
    let mut prints = state.prints.clone();
    prints.sort_unstable();
    prints
}

/// Dice coefficient of two sorted multisets; 0 if either is empty (nothing to compare).
fn dice(a: &[Fingerprint], b: &[Fingerprint]) -> f64 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }

    let (mut i, mut j, mut shared) = (0, 0, 0);
    while i < a.len() && j < b.len() {
        match a[i].cmp(&b[j]) {
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
            std::cmp::Ordering::Equal => {
                shared += 1;
                i += 1;
                j += 1;
            }
        }
    }
    (2 * shared) as f64 / (a.len() + b.len()) as f64
}
