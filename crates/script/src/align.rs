//! Matches the scenes of an old script state to the scenes of a new one (DB-65).
//!
//! The old state is what was saved: number, heading and a short fingerprint per element
//! ([`SceneState`]). The new state is a parsed [`Script`]. Per old scene:
//!
//! 1. The number is a candidate. A new scene with that number is the match if its content
//!    is similar ([`CONFIRM_MIN`]) or, failing that, its heading is equal and the content
//!    did not turn up in another scene.
//! 2. Otherwise the content is searched among the new scenes no number claimed. One
//!    scene at or above [`MATCH_MIN`] is the match; several, or only weak ones above
//!    [`CANDIDATE_MIN`], are ambiguous.
//! 3. Nothing similar: not found.
//!
//! Anything that is not unique is ambiguous, never guessed (DATENMODELL principle 6).
//!
//! Similarity is the Dice coefficient of the fingerprint multisets, `2·shared / (a + b)`:
//! 1 for equal content, deterministic, independent of order (a moved paragraph still
//! counts) and cheap. Thresholds, chosen on the tests' cases:
//!
//! - `CONFIRM_MIN` 0.6: a scene cut in half keeps 0.67 against each half, a scene with a
//!   third of its lines rewritten stays above; unrelated scenes sit near 0.
//! - `MATCH_MIN` 0.8: without the number's help, only a scene that is nearly unchanged
//!   counts as a unique match; a copy or twin then shows up as a second candidate.
//! - `CANDIDATE_MIN` 0.3: below, a "similar" scene is a coincidence of common lines
//!   ("Ja.") and not worth the user's question.

use std::collections::HashMap;

use crate::text::normal;
use crate::{Element, SceneView, Script};

const CONFIRM_MIN: f64 = 0.6;
const MATCH_MIN: f64 = 0.8;
const CANDIDATE_MIN: f64 = 0.3;

const FNV_OFFSET: u32 = 0x811c_9dc5;
const FNV_PRIME: u32 = 0x0100_0193;

/// Hash of one element's kind and normalised text. FNV-1a, so the value is the same on
/// every machine and release and can be saved.
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
    /// Same number, confirmed by heading or content.
    Number,
    /// Found by content alone.
    Content,
}

/// What became of one old scene. Indices are into `new.scenes()`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Matched {
        scene: usize,
        how: How,
    },
    /// Several or only weak candidates, best first.
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
    let ctx = Ctx::new(old, &new);

    let mut outcomes: Vec<Option<Outcome>> = (0..old.len()).map(|i| ctx.by_number(i)).collect();
    let claimed = claimed(&outcomes);
    for (i, outcome) in outcomes.iter_mut().enumerate() {
        if outcome.is_none() {
            *outcome = Some(ctx.by_content(i, &claimed));
        }
    }

    let mut old_out: Vec<Outcome> = outcomes.into_iter().flatten().collect();
    split_shared(&mut old_out);

    let taken: Vec<usize> = old_out
        .iter()
        .filter_map(|o| match o {
            Outcome::Matched { scene, .. } => Some(*scene),
            _ => None,
        })
        .collect();
    let unmatched = (0..new.len()).filter(|i| !taken.contains(i)).collect();

    Alignment {
        old: old_out,
        unmatched,
    }
}

struct Ctx<'a> {
    old: &'a [SceneState],
    new: &'a [SceneState],
    /// `sims[o][n]`: similarity of old scene `o` and new scene `n`.
    sims: Vec<Vec<f64>>,
}

impl<'a> Ctx<'a> {
    fn new(old: &'a [SceneState], new: &'a [SceneState]) -> Ctx<'a> {
        let old_sorted: Vec<Vec<Fingerprint>> = old.iter().map(sorted).collect();
        let new_sorted: Vec<Vec<Fingerprint>> = new.iter().map(sorted).collect();
        let sims = old_sorted
            .iter()
            .map(|o| new_sorted.iter().map(|n| dice(o, n)).collect())
            .collect();
        Ctx { old, new, sims }
    }

    /// Step 1: the scene with the old number, if heading or content confirm it.
    fn by_number(&self, o: usize) -> Option<Outcome> {
        let number = self.old[o].number.as_ref()?;
        let same: Vec<usize> = (0..self.new.len())
            .filter(|&n| self.new[n].number.as_ref() == Some(number))
            .collect();

        let by_content: Vec<usize> = same
            .iter()
            .copied()
            .filter(|&n| self.sims[o][n] >= CONFIRM_MIN)
            .collect();
        if !by_content.is_empty() {
            return Some(number_outcome(by_content));
        }

        // The content may have moved to another scene; then the heading proves nothing.
        let moved = self.sims[o].iter().any(|&s| s >= MATCH_MIN);
        if moved {
            return None;
        }
        let by_heading: Vec<usize> = same
            .into_iter()
            .filter(|&n| same_heading(&self.old[o], &self.new[n]))
            .collect();
        if by_heading.is_empty() {
            return None;
        }
        Some(number_outcome(by_heading))
    }

    /// Step 2 and 3: search by content among the scenes no number claimed.
    fn by_content(&self, o: usize, claimed: &[usize]) -> Outcome {
        let mut found: Vec<(usize, f64)> = (0..self.new.len())
            .filter(|n| !claimed.contains(n))
            .map(|n| (n, self.sims[o][n]))
            .filter(|&(_, s)| s >= CANDIDATE_MIN)
            .collect();
        found.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));

        let strong: Vec<usize> = found
            .iter()
            .filter(|&&(_, s)| s >= MATCH_MIN)
            .map(|&(n, _)| n)
            .collect();
        match (strong.len(), found.len()) {
            (_, 0) => Outcome::NotFound,
            (1, _) => Outcome::Matched {
                scene: strong[0],
                how: How::Content,
            },
            (0, _) => Outcome::Ambiguous(found.iter().map(|f| f.0).collect()),
            _ => Outcome::Ambiguous(strong),
        }
    }
}

fn number_outcome(mut scenes: Vec<usize>) -> Outcome {
    if scenes.len() == 1 {
        return Outcome::Matched {
            scene: scenes[0],
            how: How::Number,
        };
    }
    scenes.sort_unstable();
    Outcome::Ambiguous(scenes)
}

fn same_heading(a: &SceneState, b: &SceneState) -> bool {
    match (&a.heading, &b.heading) {
        (Some(x), Some(y)) => normal(x) == normal(y),
        _ => false,
    }
}

fn claimed(outcomes: &[Option<Outcome>]) -> Vec<usize> {
    outcomes
        .iter()
        .filter_map(|o| match o {
            Some(Outcome::Matched { scene, .. }) => Some(*scene),
            _ => None,
        })
        .collect()
}

/// Two old scenes that found the same new scene (an old twin, a split) are both unsure.
fn split_shared(outcomes: &mut [Outcome]) {
    let mut count: HashMap<usize, usize> = HashMap::new();
    for o in outcomes.iter() {
        if let Outcome::Matched { scene, .. } = o {
            *count.entry(*scene).or_default() += 1;
        }
    }

    for o in outcomes.iter_mut() {
        let Outcome::Matched { scene, .. } = o else {
            continue;
        };
        if count[scene] > 1 {
            *o = Outcome::Ambiguous(vec![*scene]);
        }
    }
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

    let mut hash = FNV_OFFSET;
    for byte in std::iter::once(tag).chain(normal(text).bytes()) {
        hash = (hash ^ u32::from(byte)).wrapping_mul(FNV_PRIME);
    }
    Some(Fingerprint(hash))
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
