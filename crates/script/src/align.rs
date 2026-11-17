//! Matches the scenes of an old script state to the scenes of a new one (DB-65).
//!
//! The old state is what was saved: number, heading and a fingerprint per element
//! ([`SceneState`]). The new state is a parsed [`Script`].
//!
//! Both scripts become one token sequence each: per scene a heading token (the heading
//! without its number), then the fingerprints of its elements. [`pair`] pairs the tokens
//! that are certainly the same (see `anchor.rs`): a token that is unique in both
//! scripts, wherever it stands, and equal stretches between two such tokens.
//!
//! A scene's weight is its elements without cue lines: a cue (`ANNA`) is in nearly every
//! dialogue scene and says nothing about which one it is. Old scene O is **matched** to
//! new scene N if the paired weighted tokens from O to N are more than half of O's and
//! more than half of N's weight; that holds for at most one N. A scene without weight on
//! both sides is compared by its heading token. Anything else is **ambiguous**: first
//! the new scenes that hold paired tokens of O (also taken ones), most first, then the
//! untaken ones with the same heading or the same number; without candidates it is
//! **not found**.
//!
//! Numbers and headings never confirm a match. They only name candidates: a number says
//! what a program believed, and renumbering, copying and inserting scenes with the same
//! heading made every "bonus" for them decide against the content in the first designs.
//! There are no thresholds to tune: a scene is matched by the lines it certainly shares,
//! and twins (identical scenes, a copy) have no unique tokens, so they stay ambiguous
//! unless the scripts are equal around them (never guess; DATENMODELL principle 6).

use std::collections::{HashMap, HashSet};

use crate::anchor::pair;
use crate::quote::kind_text;
use crate::text::{hash, normal};
use crate::{Element, SceneView, Script};

/// Tag of a heading token and of a cue line's fingerprint.
const HEADING_TAG: u8 = 0;
pub(crate) const CUE_TAG: u8 = 2;

/// Fingerprint of one element: kind tag and hash of the normalised text (FNV-1a, stable,
/// can be saved).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Fingerprint {
    pub tag: u8,
    pub hash: u32,
}

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
            prints: prints(scene.elements).into_iter().flatten().collect(),
        }
    }

    /// Elements that tell scenes apart: all but cue lines.
    fn weight(&self) -> usize {
        self.prints.iter().filter(|p| p.tag != CUE_TAG).count()
    }
}

/// The fingerprint of a scene heading (without its number): what `align` takes as the first
/// token of a scene, for a register to keep next to the prints of the elements.
pub fn heading_print(heading: &str) -> Fingerprint {
    Fingerprint {
        tag: HEADING_TAG,
        hash: hash(HEADING_TAG, heading),
    }
}

/// The state of every scene of the script, in text order.
pub fn scene_states(script: &Script) -> Vec<SceneState> {
    script.scenes().iter().map(SceneState::of).collect()
}

/// What became of one old scene. Indices are into `new.scenes()`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Matched {
        scene: usize,
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

/// The tokens of all scenes in order, and the scene each belongs to. A scene's heading
/// token is the first of its tokens.
struct Tokens {
    tokens: Vec<u32>,
    scene: Vec<usize>,
    cue: Vec<bool>,
    /// Index of the heading token of each scene.
    heads: Vec<usize>,
}

fn tokens(states: &[SceneState]) -> Tokens {
    let mut t = Tokens {
        tokens: Vec::new(),
        scene: Vec::new(),
        cue: Vec::new(),
        heads: Vec::new(),
    };

    for (i, s) in states.iter().enumerate() {
        t.heads.push(t.tokens.len());
        t.tokens
            .push(hash(HEADING_TAG, s.heading.as_deref().unwrap_or("")));
        t.scene.push(i);
        t.cue.push(true);

        for p in &s.prints {
            t.tokens.push(p.hash);
            t.scene.push(i);
            t.cue.push(p.tag == CUE_TAG);
        }
    }
    t
}

/// Matches the old scenes to the scenes of `new`.
pub fn align(old: &[SceneState], new: &Script) -> Alignment {
    let new = scene_states(new);
    let (ot, nt) = (tokens(old), tokens(&new));
    let pairs = pair(&ot.tokens, &nt.tokens);

    // Paired weighted tokens from each old scene to each new scene.
    let mut shared: HashMap<(usize, usize), usize> = HashMap::new();
    for (o, n) in pairs.iter().enumerate() {
        let Some(n) = n else {
            continue;
        };
        if !ot.cue[o] && !nt.cue[*n] {
            *shared.entry((ot.scene[o], nt.scene[*n])).or_default() += 1;
        }
    }

    let matched: Vec<Option<usize>> = (0..old.len())
        .map(|o| matched(o, old, &new, &shared, &ot, &nt, &pairs))
        .collect();
    let taken: HashSet<usize> = matched.iter().flatten().copied().collect();

    let old_out = (0..old.len())
        .map(|o| match matched[o] {
            Some(scene) => Outcome::Matched { scene },
            None => offer(o, &old[o], &new, &shared, &taken),
        })
        .collect();
    let unmatched = (0..new.len()).filter(|n| !taken.contains(n)).collect();

    Alignment {
        old: old_out,
        unmatched,
    }
}

/// The new scene that holds more than half of the weight of old scene `o` and of which
/// `o` holds more than half, if there is one.
fn matched(
    o: usize,
    old: &[SceneState],
    new: &[SceneState],
    shared: &HashMap<(usize, usize), usize>,
    ot: &Tokens,
    nt: &Tokens,
    pairs: &[Option<usize>],
) -> Option<usize> {
    let weight = old[o].weight();
    if weight == 0 {
        // Nothing but the heading to go by, and only against a scene that is the same.
        let n = nt.scene[pairs[ot.heads[o]]?];
        return (new[n].weight() == 0).then_some(n);
    }

    shared
        .iter()
        .filter(|((so, _), _)| *so == o)
        .find(|((_, n), c)| 2 * **c > weight && 2 * **c > new[*n].weight())
        .map(|((_, n), _)| *n)
}

/// Not matched: the new scenes worth asking about.
fn offer(
    o: usize,
    old: &SceneState,
    new: &[SceneState],
    shared: &HashMap<(usize, usize), usize>,
    taken: &HashSet<usize>,
) -> Outcome {
    let mut holders: Vec<(usize, usize)> = shared
        .iter()
        .filter(|((so, _), _)| *so == o)
        .map(|((_, n), c)| (*n, *c))
        .collect();
    holders.sort_by_key(|&(n, c)| (std::cmp::Reverse(c), n));
    let mut found: Vec<usize> = holders.iter().map(|h| h.0).collect();

    let heading = old.heading.as_deref().map(normal);
    for (n, s) in new.iter().enumerate() {
        let same_head = heading.is_some() && heading == s.heading.as_deref().map(normal);
        let same_number = old.number.is_some() && old.number == s.number;
        if (same_head || same_number) && !taken.contains(&n) && !found.contains(&n) {
            found.push(n);
        }
    }

    if found.is_empty() {
        return Outcome::NotFound;
    }
    Outcome::Ambiguous(found)
}

/// Separates a cue's name from the lines of its speech in the text that is hashed.
const SPEECH_SEPARATOR: char = '\u{1}';

/// The fingerprint of every element, `None` for those that are not script text (notes,
/// page breaks, sections, synopses, headings): editing them does not change the scene.
///
/// A cue line is only a name, which says nothing about which cue it is: its fingerprint
/// covers its speech too (the parentheticals and dialogue up to the next other element).
pub(crate) fn prints(elements: &[Element]) -> Vec<Option<Fingerprint>> {
    (0..elements.len())
        .map(|i| match &elements[i] {
            Element::Character { name, .. } => Some(cue_print(name, &elements[i + 1..])),
            el => print(el),
        })
        .collect()
}

fn cue_print(name: &str, speech: &[Element]) -> Fingerprint {
    let mut text = name.to_string();
    for el in speech {
        let (Element::Parenthetical(t) | Element::Dialogue(t)) = el else {
            break;
        };
        text.push(SPEECH_SEPARATOR);
        text.push_str(t);
    }
    Fingerprint {
        tag: CUE_TAG,
        hash: hash(CUE_TAG, &text),
    }
}

fn print(el: &Element) -> Option<Fingerprint> {
    let (kind, text) = kind_text(el)?;
    let tag = kind.tag()?;
    Some(Fingerprint {
        tag,
        hash: hash(tag, text),
    })
}
