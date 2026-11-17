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
    pub key: u64,
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

use klappe_model::{random_id, BookState, Kind, Print, Project, Scene};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use klappe_script::{
    align, heading_print, normal, scene_states, Fingerprint, Outcome, SceneState, Script,
};

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
            let fate = match outcome {
                Outcome::Matched { scene } if has_heading(&book[scene]) => Fate::Matched(scene),
                Outcome::Matched { .. } | Outcome::NotFound => Fate::Missing,
                Outcome::Ambiguous(found) => Fate::Pending(found),
            };
            (entry, fate)
        })
        .collect();

    let mut plan = Plan { fates, book };
    plan.settle();
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

/// Names the content of a book scene, to tell it from another at the same place.
fn key_of(state: &SceneState) -> u64 {
    let mut hasher = DefaultHasher::new();
    (&state.number, &state.heading, &state.prints).hash(&mut hasher);

    hasher.finish()
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
            let exact: Vec<usize> = open.iter().copied().filter(both).collect();
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
            let mut found: Vec<usize> = open.iter().filter(agree).copied().collect();
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
            Choice::NotInBook => self.fates[k].1 = Fate::Missing,
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
                id: free_id(project, &booked),
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

    /// The entry becomes the book's scene `n`.
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
    }
}

/// A random id that no entry has, kept or yet to be placed (ids are file names, which may
/// not differ by case only).
fn free_id(project: &Project, placed: &[(usize, Scene)]) -> String {
    loop {
        let id = random_id();
        let taken = project
            .scenes
            .iter()
            .chain(placed.iter().map(|(_, s)| s))
            .any(|s| s.id.eq_ignore_ascii_case(&id));
        if !taken {
            return id;
        }
    }
}
