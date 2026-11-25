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
use crate::text::normal;
use crate::{align, parse_located, scene_states, Alignment, Element, Outcome, Script};

/// What became of a scene between the two states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
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
pub enum Place {
    Kept,
    /// It stands elsewhere now; its text counts as changed only where the lines differ.
    Moved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
    Added,
    Removed,
}

/// One line that is only in one of the states.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineChange {
    pub change: Change,
    /// From 0, in the text of its state: the new one for `Added`, the old one for `Removed`.
    pub line: usize,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneDiff {
    /// The new scene's number and heading, the old one's for a removed scene.
    pub number: Option<String>,
    pub heading: Option<String>,
    pub status: Status,
    pub place: Place,
    /// Removed lines first, then added ones, each in text order. A new scene has all its
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
            .filter(|l| l.change == Change::Added)
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

impl Row {
    fn change(&self, change: Change) -> LineChange {
        LineChange {
            change,
            line: self.line,
            text: self.shown.clone(),
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
        slots[anchor].push(lone(scene, Status::Removed));
    }

    let mut out = std::mem::take(&mut slots[0]);
    for (n, scene) in new.scenes.iter().enumerate() {
        out.push(match old_of(n) {
            Some(o) => both(&old.scenes[o], scene, moved(n)),
            None => lone(scene, Status::New),
        });
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
fn lone(scene: &Scene, status: Status) -> SceneDiff {
    let change = match status {
        Status::Removed => Change::Removed,
        _ => Change::Added,
    };

    SceneDiff {
        number: scene.number.clone(),
        heading: scene.heading.clone(),
        status,
        place: Place::Kept,
        lines: scene.lines.iter().map(|row| row.change(change)).collect(),
    }
}

/// Two scenes that are the same scene: what differs between their lines.
fn both(old: &Scene, new: &Scene, moved: bool) -> SceneDiff {
    let keys = |s: &Scene| s.lines.iter().map(|r| r.key.clone()).collect::<Vec<_>>();
    let (removed, added) = unpaired(&keys(old), &keys(new));

    let mut lines: Vec<LineChange> = removed
        .iter()
        .map(|&i| (Change::Removed, &old.lines[i]))
        .chain(added.iter().map(|&i| (Change::Added, &new.lines[i])))
        .map(|(change, row)| row.change(change))
        .collect();
    lines.sort_by_key(|l| l.change == Change::Added);

    SceneDiff {
        number: new.number.clone(),
        heading: new.heading.clone(),
        status: if lines.is_empty() {
            Status::Same
        } else {
            Status::Changed
        },
        place: if moved { Place::Moved } else { Place::Kept },
        lines,
    }
}

/// Parses `text` and cuts it into scenes the way `Script::scenes` does, keeping the text lines
/// each element stands on.
fn read(text: &str) -> Book {
    let located = parse_located(text);
    let source = text.replace("\r\n", "\n").replace('\r', "\n");
    let source: Vec<&str> = source.split('\n').collect();
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
        for &line in &l.lines {
            let shown = source.get(line).map(|s| s.trim()).unwrap_or_default();
            let key = match &l.element {
                // The number is not part of what is compared.
                Element::SceneHeading { text, .. } => normal(text),
                _ => shown.to_owned(),
            };
            if !key.is_empty() {
                scene.lines.push(Row {
                    line,
                    key,
                    shown: shown.to_owned(),
                });
            }
        }
    }

    // Shown as typed, not as compared.
    let script = Script {
        title_page: Vec::new(),
        elements: located.into_iter().map(|l| l.element).collect(),
    };

    Book { script, scenes }
}
