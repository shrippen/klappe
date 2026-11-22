//! `figuren.toml` against an answer (DB-50): the one entry an answer to a cue changed, kept to
//! do it again or take it back. The file can change under the session (the window beside the
//! TUI), so an answer, its undo and its redo change only their entry, in the file as it is
//! on disk.

use klappe_model::{same_character_name, Character, Project};

use crate::figures::FigureChoice;

/// Which way a delta goes on the file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Way {
    /// The answer, done (again).
    Apply,
    /// The answer, taken back.
    Revert,
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

    /// Changes `file` by this delta, in `way`; an entry already (not) there stays as it is.
    pub(crate) fn run(&self, file: &mut Project, way: Way) {
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
                if !c.aliases.iter().any(|a| same_character_name(a, alias)) {
                    c.aliases.push(alias.clone());
                }
            }
            (Self::Alias { id, alias }, Way::Revert) => {
                if let Some(c) = file.characters.iter_mut().find(|c| c.id == *id) {
                    c.aliases.retain(|a| !same_character_name(a, alias));
                }
            }
            (Self::NotFigure(cue), Way::Apply) => {
                if !file.not_figures.iter().any(|n| same_character_name(n, cue)) {
                    file.not_figures.push(cue.clone());
                }
            }
            (Self::NotFigure(cue), Way::Revert) => {
                file.not_figures.retain(|n| !same_character_name(n, cue));
            }
        }
    }
}
