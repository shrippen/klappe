//! Joins the register to the book (DATENMODELL §4).

/// What the user decides for an entry the alignment could not place by itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Choice {
    /// The entry is this scene of the book (`Candidate::index`).
    Scene(usize),
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
use klappe_script::{align, normal, scene_states, Fingerprint, Outcome, SceneState, Script};

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

/// What the register remembers of an entry, as the alignment takes it.
fn state_of(scene: &Scene) -> SceneState {
    let some = |text: &str| (!text.is_empty()).then(|| text.to_owned());

    SceneState {
        number: some(&scene.number),
        heading: some(&scene.heading),
        prints: scene.prints.iter().map(fingerprint_of).collect(),
    }
}

/// Whether a scene of the book is one to keep an entry for: text before the first heading
/// is no scene.
fn has_heading(state: &SceneState) -> bool {
    state.heading.is_some()
}

/// Aligns the entries of kind scene with the scenes of `script`. Without a scene in the
/// book there is nothing to align with: the caller does not ask.
pub(crate) fn plan(project: &Project, script: &Script) -> Plan {
    let book = scene_states(script);
    let kept: Vec<usize> = (0..project.scenes.len())
        .filter(|&i| project.scenes[i].kind == Kind::Scene)
        .collect();
    let old: Vec<SceneState> = kept.iter().map(|&i| state_of(&project.scenes[i])).collect();
    let alignment = align(&old, script);

    let fates = kept
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
    plan.claim_first_run(project);
    plan.settle();

    plan
}

impl Plan {
    fn matched(&self) -> HashSet<usize> {
        let found = self.fates.iter().filter_map(|(_, fate)| match fate {
            Fate::Matched(scene) => Some(*scene),
            _ => None,
        });

        found.collect()
    }

    /// An entry without prints has no content to go by (the project had no book before, or
    /// it was made in Klappe): it is matched once by its number, then by its heading, each
    /// only if that names one scene that nobody holds (DATENMODELL §4).
    fn claim_first_run(&mut self, project: &Project) {
        for by_number in [true, false] {
            for k in 0..self.fates.len() {
                let (entry, Fate::Pending(found)) = &self.fates[k] else {
                    continue;
                };
                let scene = &project.scenes[*entry];
                if !scene.prints.is_empty() {
                    continue;
                }

                let taken = self.matched();
                let named: Vec<usize> = found
                    .iter()
                    .copied()
                    .filter(|n| !taken.contains(n))
                    .filter(|&n| names(&self.book[n], scene, by_number))
                    .collect();
                if let [only] = named[..] {
                    self.fates[k].1 = Fate::Matched(only);
                }
            }
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
            Choice::Scene(n) if found.contains(&n) => self.fates[k].1 = Fate::Matched(n),
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
        scene.prints = state.prints.iter().copied().map(print_of).collect();
        scene.state = BookState::InBook;
    }
}

/// Whether the entry's number (or else its heading) is the book scene's.
fn names(book: &SceneState, scene: &Scene, by_number: bool) -> bool {
    if by_number {
        return !scene.number.is_empty() && book.number.as_deref() == Some(scene.number.as_str());
    }

    let heading = book.heading.as_deref().map(normal);
    !scene.heading.is_empty() && heading == Some(normal(&scene.heading))
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
