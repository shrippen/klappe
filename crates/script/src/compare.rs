//! Compares two states of a book, scene by scene and line by line (DB-41, DB-42, DB-44).
//!
//! ```text
//! old text ─► scenes ─┐
//!                     ├─► align pairs them ─► moved? (not in the longest rising run)
//! new text ─► scenes ─┘                    └► lines of paired scenes (LCS)
//! ```
//!
//! Scenes are paired by `align`, so a renumbered scene is the same scene. A scene that
//! `align` finds only ambiguously counts as removed and new: never guess. Scene numbers
//! are not part of the comparison; a scene whose number alone changed is unchanged.

use crate::lcs::{in_rising, unpaired};
use crate::numbering::is_omitted;
use crate::plain_text;
use crate::text::normal;
use crate::{align, parse_located, scene_states, Alignment, Element, Outcome, Script};

/// What became of a scene between the two states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneStatus {
    /// Only in the new state.
    New,
    /// Only in the old state.
    Removed,
    /// In both, with other lines.
    Changed,
    /// In both, with the same lines.
    Same,
}

/// Whether a scene that is in both states kept its place among them (DB-44).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScenePlace {
    Kept,
    /// It stands elsewhere now; its text counts as changed only where the lines differ.
    Moved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edit {
    Added,
    Removed,
}

/// One line that is only in one of the states.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineChange {
    pub change: Edit,
    /// From 0, in the text of its state: the new one for `Added`, the old one for `Removed`.
    pub line: usize,
    pub text: String,
    /// From 0 in the scene: lines with the same number lie in the same gap between common lines
    /// (a removed and an added line replace each other), others are apart.
    pub group: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneDiff {
    /// The new scene's number and heading, the old one's for a removed scene.
    pub number: Option<String>,
    pub heading: Option<String>,
    /// The old state's number and heading: those of `number` and `heading` unless the heading
    /// or number changed; none for a new scene.
    pub old_number: Option<String>,
    pub old_heading: Option<String>,
    pub status: SceneStatus,
    pub place: ScenePlace,
    /// In text order, a removed line above the added line that replaces it. A new scene has all its
    /// lines added, a removed one has all its lines removed.
    pub lines: Vec<LineChange>,
}

/// The scenes of the new state in order, a removed scene after the scene that stood before
/// it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Comparison {
    pub scenes: Vec<SceneDiff>,
}

impl Comparison {
    /// The lines (from 0) of the new text that are added or changed: where a revision mark
    /// goes.
    pub fn marked_lines(&self) -> Vec<usize> {
        let mut lines: Vec<usize> = self
            .scenes
            .iter()
            .flat_map(|s| &s.lines)
            .filter(|l| l.change == Edit::Added)
            .map(|l| l.line)
            .collect();
        lines.sort_unstable();

        lines
    }
}

/// A line that carries text.
struct Row {
    line: usize,
    /// What is compared of it.
    key: String,
    /// As typed.
    shown: String,
}

impl Scene {
    /// Whether the heading is a struck scene's (`.ENTFÄLLT #12#`).
    fn is_struck(&self) -> bool {
        self.heading.as_deref().is_some_and(is_omitted)
    }
}

impl Row {
    fn change(&self, change: Edit, group: usize) -> LineChange {
        LineChange {
            change,
            line: self.line,
            text: self.shown.clone(),
            group,
        }
    }
}

/// A scene's number and heading, and its rows.
struct Scene {
    number: Option<String>,
    heading: Option<String>,
    lines: Vec<Row>,
}

/// The book as `align` and the line diff need it.
struct Book {
    script: Script,
    scenes: Vec<Scene>,
}

/// Compares `old` with `new`.
pub fn compare(old: &str, new: &str) -> Comparison {
    let (old, new) = (read(old), read(new));
    let al = align(&scene_states(&old.script), &new.script);
    let pairs = pairs(&al);

    // Pairs in the new order, with the old positions: those off the longest rising run moved.
    let mut by_new = pairs.clone();
    by_new.sort_by_key(|&(_, n)| n);
    let rising = in_rising(&by_new.iter().map(|&(o, _)| o).collect::<Vec<_>>());
    let moved = |n: usize| {
        let at = by_new.iter().position(|&(_, pn)| pn == n);
        at.is_some_and(|i| !rising[i])
    };
    let old_of = |n: usize| pairs.iter().find(|&&(_, pn)| pn == n).map(|&(o, _)| o);

    // Slot 0 holds the removed scenes before the first new one, slot k+1 those after new
    // scene k.
    let mut slots: Vec<Vec<SceneDiff>> = vec![Vec::new(); new.scenes.len() + 1];
    let mut anchor = 0;
    for (o, scene) in old.scenes.iter().enumerate() {
        if let Some(&(_, n)) = pairs.iter().find(|&&(po, _)| po == o) {
            anchor = n + 1;
            continue;
        }
        slots[anchor].push(lone(scene, SceneStatus::Removed));
    }

    let mut out = std::mem::take(&mut slots[0]);
    for (n, scene) in new.scenes.iter().enumerate() {
        match old_of(n) {
            Some(o) => out.push(both(&old.scenes[o], scene, moved(n))),
            // A struck heading marks the removed scene, it is no new scene.
            None if scene.is_struck() => {}
            None => out.push(lone(scene, SceneStatus::New)),
        }

        out.append(&mut slots[n + 1]);
    }

    Comparison { scenes: out }
}

/// `(old, new)` scene indices that `align` is sure of, in old order.
fn pairs(al: &Alignment) -> Vec<(usize, usize)> {
    al.old
        .iter()
        .enumerate()
        .filter_map(|(o, out)| match out {
            Outcome::Matched { scene } => Some((o, *scene)),
            _ => None,
        })
        .collect()
}

/// A scene that has no partner: all its lines are added or removed.
fn lone(scene: &Scene, status: SceneStatus) -> SceneDiff {
    let change = match status {
        SceneStatus::Removed => Edit::Removed,
        _ => Edit::Added,
    };

    let old = status == SceneStatus::Removed;

    SceneDiff {
        number: scene.number.clone(),
        heading: scene.heading.clone(),
        old_number: scene.number.clone().filter(|_| old),
        old_heading: scene.heading.clone().filter(|_| old),
        status,
        place: ScenePlace::Kept,
        lines: scene
            .lines
            .iter()
            .map(|row| row.change(change, 0))
            .collect(),
    }
}

/// Two scenes that are the same scene: what differs between their lines.
fn both(old: &Scene, new: &Scene, moved: bool) -> SceneDiff {
    let keys = |s: &Scene| s.lines.iter().map(|r| r.key.clone()).collect::<Vec<_>>();
    let (removed, added) = unpaired(&keys(old), &keys(new));

    // Each change by the common lines before it, so a hunk's removed lines come right above
    // its added ones: `Removed < Added` breaks the tie. (Line `i` has `i - k` common lines
    // before it when `k` lines before it are unpaired.)
    let common = |at: usize, unpaired: &[usize]| at - unpaired.iter().filter(|&&u| u < at).count();
    let mut changes: Vec<(usize, Edit, &Row)> = removed
        .iter()
        .map(|&i| (common(i, &removed), Edit::Removed, &old.lines[i]))
        .chain(
            added
                .iter()
                .map(|&j| (common(j, &added), Edit::Added, &new.lines[j])),
        )
        .collect();
    changes.sort_by_key(|&(before, change, _)| (before, change == Edit::Added));

    // Changes in the same gap share a number, counted up from 0 gap by gap.
    let mut group = 0;
    let mut last = None;
    let mut lines: Vec<LineChange> = Vec::new();
    for (before, change, row) in changes {
        if last.is_some_and(|b| b != before) {
            group += 1;
        }
        last = Some(before);
        lines.push(row.change(change, group));
    }

    SceneDiff {
        number: new.number.clone(),
        heading: new.heading.clone(),
        old_number: old.number.clone(),
        old_heading: old.heading.clone(),
        status: if lines.is_empty() {
            SceneStatus::Same
        } else {
            SceneStatus::Changed
        },
        place: if moved {
            ScenePlace::Moved
        } else {
            ScenePlace::Kept
        },
        lines,
    }
}

/// Parses `text` and cuts it into scenes the way `Script::scenes` does, keeping the text lines
/// each element stands on.
fn read(text: &str) -> Book {
    let located = parse_located(text);
    let mut scenes: Vec<Scene> = Vec::new();

    for l in &located {
        if let Element::SceneHeading { text, number } = &l.element {
            scenes.push(Scene {
                number: number.clone(),
                heading: Some(text.clone()),
                lines: Vec::new(),
            });
        } else if scenes.is_empty() {
            // Text before the first heading is a scene without one.
            scenes.push(Scene {
                number: None,
                heading: None,
                lines: Vec::new(),
            });
        }

        let Some(scene) = scenes.last_mut() else {
            continue;
        };
        for (at, text) in printed(&l.element).iter().enumerate() {
            let shown = plain_text(text);
            if shown.is_empty() {
                continue;
            }

            let line = l
                .lines
                .get(at)
                .or(l.lines.last())
                .copied()
                .unwrap_or_default();
            let key = match &l.element {
                // The number is not part of what is compared.
                Element::SceneHeading { text, .. } => normal(text),
                _ => shown.clone(),
            };
            scene.lines.push(Row { line, key, shown });
        }
    }

    // Shown as typed, not as compared.
    let script = Script {
        title_page: Vec::new(),
        elements: located.into_iter().map(|l| l.element).collect(),
    };

    Book { script, scenes }
}

/// The lines the page prints for an element: none for what stays off the page (notes,
/// sections, synopses). Boneyards are gone by the parser, inline notes by `plain_text`.
fn printed(element: &Element) -> Vec<String> {
    match element {
        Element::SceneHeading { text, number } => {
            let mark = number.as_ref().map(|n| format!(" #{n}#"));

            vec![format!("{text}{}", mark.unwrap_or_default())]
        }
        Element::Character {
            name, extension, ..
        } => {
            let extension = extension.as_ref().map(|e| format!(" ({e})"));

            vec![format!("{name}{}", extension.unwrap_or_default())]
        }
        Element::Action(t)
        | Element::Dialogue(t)
        | Element::Lyrics(t)
        | Element::Parenthetical(t)
        | Element::Transition(t)
        | Element::Centered(t) => t.split('\n').map(str::to_owned).collect(),
        Element::Section { .. } | Element::Synopsis(_) | Element::Note(_) | Element::PageBreak => {
            Vec::new()
        }
    }
}
