//! `figuren.toml` against the session (DB-50): the entry an answer changed, and the merge of
//! the session's figures with the file at a save. The file can change under the session
//! (the window beside the TUI), so neither writes the session's whole list blindly.

use klappe_model::{Character, Project};

use crate::disk_step::{DiskStep, Way};
use crate::figures::FigureChoice;

/// The characters and the not-figures: the content of `figuren.toml`.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct FigureSet {
    pub(crate) characters: Vec<Character>,
    pub(crate) not_figures: Vec<String>,
}

impl FigureSet {
    pub(crate) fn of(project: &Project) -> Self {
        Self {
            characters: project.characters.clone(),
            not_figures: project.not_figures.clone(),
        }
    }

    /// Makes these the figures of `project`.
    pub(crate) fn put(self, project: &mut Project) {
        project.characters = self.characters;
        project.not_figures = self.not_figures;
    }
}

/// The one entry of `figuren.toml` an answer to a cue wrote.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum FigureDelta {
    Created(Character),
    Alias { id: String, alias: String },
    NotFigure(String),
}

impl FigureDelta {
    /// The entry that answering `cue` with `choice` wrote: `after` is the project it made.
    pub(crate) fn of(cue: &str, choice: &FigureChoice, after: &Project) -> Option<Self> {
        let cue = cue.trim().to_owned();

        match choice {
            FigureChoice::NotFigure => Some(Self::NotFigure(cue)),
            FigureChoice::Assign(id) => Some(Self::Alias {
                id: id.clone(),
                alias: cue,
            }),
            FigureChoice::Create => after.characters.last().cloned().map(Self::Created),
        }
    }
}

fn same(a: &str, b: &str) -> bool {
    a.trim().to_uppercase() == b.trim().to_uppercase()
}

impl DiskStep for FigureDelta {
    type File = Project;

    fn run(&self, file: &mut Project, way: Way) {
        match (self, way) {
            (Self::Created(c), Way::Apply) => {
                if !file.characters.iter().any(|o| o.id == c.id) {
                    file.characters.push(c.clone());
                }
            }
            (Self::Created(c), Way::Revert) => file.characters.retain(|o| o.id != c.id),
            (Self::Alias { id, alias }, Way::Apply) => {
                let Some(c) = file.characters.iter_mut().find(|c| c.id == *id) else {
                    return;
                };
                if !c.aliases.iter().any(|a| same(a, alias)) {
                    c.aliases.push(alias.clone());
                }
            }
            (Self::Alias { id, alias }, Way::Revert) => {
                if let Some(c) = file.characters.iter_mut().find(|c| c.id == *id) {
                    c.aliases.retain(|a| !same(a, alias));
                }
            }
            (Self::NotFigure(cue), Way::Apply) => {
                if !file.not_figures.iter().any(|n| same(n, cue)) {
                    file.not_figures.push(cue.clone());
                }
            }
            (Self::NotFigure(cue), Way::Revert) => file.not_figures.retain(|n| !same(n, cue)),
        }
    }
}

/// The figures to save: the file's (`disk`) plus what the session did to them since it
/// last took them (`base`). Union by id, in the file's order, the session's own new
/// figures after; an entry the session changed is the session's, with the aliases of both;
/// one it removed stays removed. The not-figures are united.
pub(crate) fn merge(base: &FigureSet, session: &Project, disk: FigureSet) -> FigureSet {
    let mut characters = Vec::new();

    for on_disk in disk.characters {
        let own = session.characters.iter().find(|c| c.id == on_disk.id);
        let before = base.characters.iter().find(|c| c.id == on_disk.id);

        match own {
            None if before.is_some() => {}
            Some(own) if before != Some(own) => characters.push(united(own, on_disk)),
            _ => characters.push(on_disk),
        }
    }
    for own in &session.characters {
        if !characters.iter().any(|c| c.id == own.id) {
            characters.push(own.clone());
        }
    }

    let mut not_figures = disk.not_figures;
    for own in &session.not_figures {
        if !not_figures.iter().any(|n| same(n, own)) {
            not_figures.push(own.clone());
        }
    }

    FigureSet {
        characters,
        not_figures,
    }
}

/// The session's `own` with the aliases the file has besides.
fn united(own: &Character, on_disk: Character) -> Character {
    let mut merged = own.clone();
    for alias in on_disk.aliases {
        if !merged.is_called(&alias) {
            merged.aliases.push(alias);
        }
    }

    merged
}
