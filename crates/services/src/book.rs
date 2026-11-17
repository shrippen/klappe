//! Joins the register to the book (DATENMODELL §4).

/// What the user decides for an entry the alignment could not place by itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Choice {
    /// The entry is this scene of the book, as a candidate of `PendingScene` named it.
    Scene(Candidate),
    /// The entry is not in the book; the candidates are new scenes.
    NotInBook,
}

/// A scene of the book an open entry may be.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    /// Position among the scenes of the book, as `Choice::Scene` takes it.
    pub index: usize,
    pub number: String,
    pub heading: String,
    /// Names the scene's content, so a choice made on an older book is recognised.
    pub key: SceneKey,
}

/// An entry of the register the alignment could not place without asking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingScene {
    pub id: String,
    pub number: String,
    pub heading: String,
    /// Best first.
    pub candidates: Vec<Candidate>,
}

/// What the status line says about the register and the book.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BookReport {
    /// Entries waiting for a decision.
    pub pending: usize,
    /// Entries whose scene is no longer in the book.
    pub not_in_book: usize,
}

use std::collections::HashSet;

use klappe_model::{BookState, Kind, Print, Project, Scene, SceneKey};

use klappe_script::{
    align, heading_print, normal, scene_states, Fingerprint, Outcome, SceneState, Script,
};

use crate::project_edit::free_scene_id;

/// What the alignment made of one entry of the register.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Fate {
    /// The entry is this scene of the book.
    Matched(usize),
    /// Nothing in the book is this entry.
    Missing,
    /// Candidates, best first: the user decides.
    Pending(Vec<usize>),
}

/// The register against the book: what becomes of each entry and which scenes are new.
#[derive(Debug)]
pub(crate) struct Plan {
    /// One per entry of kind scene: its index in `Project::scenes` and its fate.
    fates: Vec<(usize, Fate)>,
    /// The scenes of the book, in text order.
    book: Vec<SceneState>,
    /// Answers "not in the book" of this run: the entry and the scenes it was asked about.
    refusals: Vec<(usize, Vec<SceneKey>)>,
}

pub(crate) fn print_of(fingerprint: Fingerprint) -> Print {
    Print {
        tag: fingerprint.tag,
        hash: fingerprint.hash,
    }
}

fn fingerprint_of(print: &Print) -> Fingerprint {
    Fingerprint {
        tag: print.tag,
        hash: print.hash,
    }
}

/// What the register remembers of an entry, as the alignment takes it. The heading print (the
/// first of `Scene::prints`) is dropped: every reader of register prints must do the same, or
/// positions among the prints (`Quote::at`) are off by one.
fn state_of(scene: &Scene) -> SceneState {
    let some = |text: &str| (!text.is_empty()).then(|| text.to_owned());
    // The heading is the alignment's own first token: not one of the lines.
    let heading_tag = heading_print("").tag;

    SceneState {
        number: some(&scene.number),
        heading: some(&scene.heading),
        prints: scene
            .prints
            .iter()
            .map(fingerprint_of)
            .filter(|p| p.tag != heading_tag)
            .collect(),
    }
}

/// Whether a scene of the book is one to keep an entry for: text before the first heading
/// is no scene.
fn has_heading(state: &SceneState) -> bool {
    state.heading.is_some()
}

/// Aligns the entries of kind scene with the scenes of `script`. Without a scene in the
/// book there is nothing to align with: the caller does not ask.
///
/// An entry with prints has been aligned with a book before (a book scene's prints always
/// hold its heading's): `align` places it. One without has not: it is placed by `first_run`.
pub(crate) fn plan(project: &Project, script: &Script) -> Plan {
    let book = scene_states(script);
    let kept: Vec<usize> = (0..project.scenes.len())
        .filter(|&i| project.scenes[i].kind == Kind::Scene)
        .collect();
    let (aligned, first): (Vec<usize>, Vec<usize>) = kept
        .into_iter()
        .partition(|&i| !project.scenes[i].prints.is_empty());
    let old: Vec<SceneState> = aligned
        .iter()
        .map(|&i| state_of(&project.scenes[i]))
        .collect();
    let alignment = align(&old, script);

    let fates = aligned
        .iter()
        .zip(alignment.old)
        .map(|(&entry, outcome)| {
            let allowed = |n: &usize| !refuses(&project.scenes[entry], &book[*n]);
            let fate = match outcome {
                Outcome::Matched { scene } if has_heading(&book[scene]) && allowed(&scene) => {
                    Fate::Matched(scene)
                }
                Outcome::Matched { .. } | Outcome::NotFound => Fate::Missing,
                Outcome::Ambiguous(found) => {
                    Fate::Pending(found.into_iter().filter(allowed).collect())
                }
            };
            (entry, fate)
        })
        .collect();

    let mut plan = Plan {
        fates,
        book,
        refusals: Vec::new(),
    };
    plan.settle();
    plan.pair_twins(project, &first);
    plan.first_run(project, &first);
    plan.settle();

    plan
}

/// Whether the entry and the book scene have the same number.
fn same_number(scene: &Scene, book: &SceneState) -> bool {
    !scene.number.is_empty() && book.number.as_deref() == Some(scene.number.as_str())
}

/// Whether the entry and the book scene have the same heading.
fn same_heading(scene: &Scene, book: &SceneState) -> bool {
    let heading = book.heading.as_deref().map(normal);

    !scene.heading.is_empty() && heading == Some(normal(&scene.heading))
}

/// FNV-1a, 64 bits: the same on every machine and release, so a key can be saved.
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0100_0000_01b3;

/// Names the content of a book scene, to tell it from another at the same place. Each part
/// ends in a byte no text holds, so `("1", "2")` and `("12", "")` differ.
fn key_of(state: &SceneState) -> SceneKey {
    let mut bytes = Vec::new();
    for text in [&state.number, &state.heading] {
        bytes.extend(text.as_deref().unwrap_or_default().bytes());
        bytes.push(0xff);
    }
    for print in &state.prints {
        bytes.push(print.tag);
        bytes.extend(print.hash.to_le_bytes());
    }

    let hash = bytes.iter().fold(FNV_OFFSET, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(FNV_PRIME)
    });

    SceneKey(hash)
}

/// Whether the user said the entry is not this scene of the book.
fn refuses(scene: &Scene, book: &SceneState) -> bool {
    !scene.refused.is_empty() && scene.refused.contains(&key_of(book))
}

impl Plan {
    fn matched(&self) -> HashSet<usize> {
        let found = self.fates.iter().filter_map(|(_, fate)| match fate {
            Fate::Matched(scene) => Some(*scene),
            _ => None,
        });

        found.collect()
    }

    /// An entry that never met a book has nothing but number and heading to go by, and they
    /// never confirm alone (DATENMODELL §4): it is matched only if both agree with one scene
    /// of the book and with no other entry. Otherwise it is asked about, with the scenes that
    /// agree in either, or missing if none does. Content ranks first: a scene that is a
    /// candidate of an entry with prints is never taken silently, the entry is asked about
    /// too.
    fn first_run(&mut self, project: &Project, first: &[usize]) {
        let taken = self.matched();
        let open: Vec<usize> = (0..self.book.len())
            .filter(|n| has_heading(&self.book[*n]) && !taken.contains(n))
            .collect();
        let claimed: HashSet<usize> = self
            .fates
            .iter()
            .flat_map(|(_, fate)| match fate {
                Fate::Pending(found) => found.clone(),
                _ => Vec::new(),
            })
            .collect();

        for &entry in first {
            let scene = &project.scenes[entry];
            let both = |n: &usize| {
                same_number(scene, &self.book[*n]) && same_heading(scene, &self.book[*n])
            };
            let exact: Vec<usize> = open
                .iter()
                .copied()
                .filter(both)
                .filter(|n| !refuses(scene, &self.book[*n]))
                .collect();
            let twins = first
                .iter()
                .map(|&e| &project.scenes[e])
                .filter(|o| {
                    o.number == scene.number && normal(&o.heading) == normal(&scene.heading)
                })
                .count();
            if let ([only], 1) = (&exact[..], twins) {
                if !claimed.contains(only) {
                    self.fates.push((entry, Fate::Matched(*only)));
                    continue;
                }
            }

            let agree = |n: &&usize| {
                same_number(scene, &self.book[**n]) || same_heading(scene, &self.book[**n])
            };
            let mut found: Vec<usize> = open
                .iter()
                .filter(agree)
                .filter(|n| !refuses(scene, &self.book[**n]))
                .copied()
                .collect();
            found.sort_by_key(|n| !exact.contains(n));
            self.fates.push((entry, Fate::Pending(found)));
        }
    }

    /// Drops candidates that are held by a match; an entry left without one has nothing to
    /// ask about and is missing.
    fn settle(&mut self) {
        let taken = self.matched();

        for (_, fate) in &mut self.fates {
            let Fate::Pending(found) = fate else {
                continue;
            };
            found.retain(|n| !taken.contains(n) && has_heading(&self.book[*n]));
            if found.is_empty() {
                *fate = Fate::Missing;
            }
        }
    }

    /// Entries that are word-identical to the scenes they ask about (twins: the same number,
    /// heading and lines, so the same key) cannot be told apart by content. Among twins the
    /// order decides, without a question: the entries in register order take the twins in
    /// book order. Twins left over become new entries (a scene copied and pasted), entries
    /// left over are missing (a copy deleted). The register follows the book's order, so
    /// this is the same after saving and reopening.
    fn pair_twins(&mut self, project: &Project, first: &[usize]) {
        let mut groups: Vec<(SceneKey, Vec<usize>)> = Vec::new();

        for (entry, fate) in &self.fates {
            let Fate::Pending(found) = fate else {
                continue;
            };
            let key = key_of(&self.book[found[0]]);
            let alike = found.iter().all(|&n| key_of(&self.book[n]) == key);
            if !alike || key_of(&state_of(&project.scenes[*entry])) != key {
                continue;
            }

            match groups.iter_mut().find(|(k, _)| *k == key) {
                Some((_, entries)) => entries.push(*entry),
                None => groups.push((key, vec![*entry])),
            }
        }

        for (key, entries) in groups {
            let mut twins: Vec<usize> = (0..self.book.len())
                .filter(|&n| key_of(&self.book[n]) == key && has_heading(&self.book[n]))
                .collect();
            twins.retain(|n| !self.matched().contains(n));
            // An entry outside the group that may be one of the twins (its scene was
            // rewritten into their text, or it never met the book and agrees in number and
            // heading) leaves the order no say: all of them are asked.
            if self.claimed_outside(&entries, &twins) || self.first_may_be(project, first, &twins) {
                continue;
            }
            let mut twins = twins.into_iter();

            for entry in entries {
                let fate = match twins.next() {
                    Some(n) => Fate::Matched(n),
                    None => Fate::Missing,
                };
                if let Some(f) = self.fates.iter_mut().find(|f| f.0 == entry) {
                    f.1 = fate;
                }
            }
        }
        self.settle();
    }

    /// Whether an entry that never met a book (`first_run` comes later) could take one of
    /// `twins` silently there: same number and heading, and not refused.
    fn first_may_be(&self, project: &Project, first: &[usize], twins: &[usize]) -> bool {
        first.iter().any(|&e| {
            let scene = &project.scenes[e];
            twins.iter().any(|&n| {
                let twin = &self.book[n];
                same_number(scene, twin) && same_heading(scene, twin) && !refuses(scene, twin)
            })
        })
    }

    /// Whether an open entry not in `group` has one of `twins` among its candidates.
    fn claimed_outside(&self, group: &[usize], twins: &[usize]) -> bool {
        self.fates.iter().any(|(entry, fate)| match fate {
            Fate::Pending(found) => {
                !group.contains(entry) && found.iter().any(|n| twins.contains(n))
            }
            _ => false,
        })
    }

    /// The entries to ask about, with the scenes they may be.
    pub(crate) fn pending(&self, project: &Project) -> Vec<PendingScene> {
        let mut out = Vec::new();

        for (entry, fate) in &self.fates {
            let Fate::Pending(found) = fate else {
                continue;
            };
            let scene = &project.scenes[*entry];
            out.push(PendingScene {
                id: scene.id.clone(),
                number: scene.number.clone(),
                heading: scene.heading.clone(),
                candidates: found
                    .iter()
                    .map(|&index| Candidate {
                        index,
                        number: self.book[index].number.clone().unwrap_or_default(),
                        heading: self.book[index].heading.clone().unwrap_or_default(),
                        key: key_of(&self.book[index]),
                    })
                    .collect(),
            });
        }

        out
    }

    /// Puts the user's decision for entry `id` in place of the open question. `false` if it
    /// is no open question or the scene no candidate.
    pub(crate) fn decide(&mut self, project: &Project, id: &str, choice: Choice) -> bool {
        let Some(k) = self.fates.iter().position(|(entry, fate)| {
            project.scenes[*entry].id == id && matches!(fate, Fate::Pending(_))
        }) else {
            return false;
        };
        let Fate::Pending(found) = &self.fates[k].1 else {
            return false;
        };

        match choice {
            Choice::Scene(c)
                if found.contains(&c.index) && key_of(&self.book[c.index]) == c.key =>
            {
                self.fates[k].1 = Fate::Matched(c.index);
            }
            Choice::Scene(_) => return false,
            Choice::NotInBook => {
                let keys = found.iter().map(|&n| key_of(&self.book[n])).collect();
                self.refusals.push((self.fates[k].0, keys));
                self.fates[k].1 = Fate::Missing;
            }
        }
        self.settle();

        true
    }

    /// Writes the plan into the register: matched entries take number, heading and prints
    /// from the book, missing ones leave it, scenes nobody holds nor may be become entries,
    /// and the entries come in the order of the text (the others after them, in their old
    /// order). Open questions change nothing. Returns whether anything changed.
    pub(crate) fn apply(&self, project: &mut Project) -> bool {
        let before = project.scenes.clone();
        let mut booked: Vec<(usize, Scene)> = Vec::new();
        let mut rest: Vec<Scene> = Vec::new();
        let mut held = HashSet::new();

        for (i, mut scene) in std::mem::take(&mut project.scenes).into_iter().enumerate() {
            self.remember(i, &mut scene);
            let fate = self
                .fates
                .iter()
                .find(|(entry, _)| *entry == i)
                .map(|f| &f.1);
            match fate {
                Some(Fate::Matched(n)) => {
                    held.insert(*n);
                    self.take(&mut scene, *n);
                    booked.push((*n, scene));
                }
                Some(Fate::Missing) => {
                    scene.state = BookState::NotInBook;
                    rest.push(scene);
                }
                Some(Fate::Pending(found)) => {
                    held.extend(found.iter().copied());
                    rest.push(scene);
                }
                None => rest.push(scene),
            }
        }

        project.scenes = rest;
        for n in (0..self.book.len()).filter(|n| !held.contains(n) && has_heading(&self.book[*n])) {
            let mut scene = Scene {
                id: free_scene_id(project, &booked),
                kind: Kind::Scene,
                ..Scene::default()
            };
            self.take(&mut scene, n);
            booked.push((n, scene));
        }

        booked.sort_by_key(|(n, _)| *n);
        let mut scenes: Vec<Scene> = booked.into_iter().map(|(_, scene)| scene).collect();
        scenes.append(&mut project.scenes);
        project.scenes = scenes;

        project.scenes != before
    }

    /// Keeps the answers "not in the book" given for entry `i` in this run.
    fn remember(&self, i: usize, scene: &mut Scene) {
        let keys = self.refusals.iter().filter(|(entry, _)| *entry == i);

        for key in keys.flat_map(|(_, keys)| keys) {
            if !scene.refused.contains(key) {
                scene.refused.push(*key);
            }
        }
    }

    /// The entry becomes the book's scene `n`; what it was not no longer matters.
    fn take(&self, scene: &mut Scene, n: usize) {
        let state = &self.book[n];

        scene.number = state.number.clone().unwrap_or_default();
        scene.heading = state.heading.clone().unwrap_or_default();
        let heading = heading_print(state.heading.as_deref().unwrap_or_default());
        scene.prints = std::iter::once(heading)
            .chain(state.prints.iter().copied())
            .map(print_of)
            .collect();
        scene.state = BookState::InBook;
        scene.refused.clear();
    }
}
