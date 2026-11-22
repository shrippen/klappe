//! `figuren.toml` against an answer (DB-50): the one entry an answer to a cue changed, kept to
//! do it again or take it back. The file can change under the session (the window beside the
//! TUI), so an answer, its undo and its redo change only their entry, in the file as it is
//! on disk.

use std::collections::HashSet;

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

/// What `disk` (the figures read from `figuren.toml`) has added since the session last took
/// them, as answers would: a character whose id neither `known` nor `state` has (whole, with
/// its aliases), an alias `state`'s character of that id lacks, a not-figure `state` lacks.
/// Whatever else differs (renamed, deleted, recoloured) is the session's own and not adopted.
pub(crate) fn added(
    known: &HashSet<String>,
    state: &Project,
    disk: &(Vec<Character>, Vec<String>),
) -> Vec<FigureDelta> {
    let (characters, not_figures) = disk;
    let mut deltas = Vec::new();

    for on_disk in characters {
        let Some(own) = state.character(&on_disk.id) else {
            if !known.contains(&on_disk.id) {
                deltas.push(FigureDelta::Created(on_disk.clone()));
            }
            continue;
        };
        for alias in &on_disk.aliases {
            if !own.aliases.iter().any(|a| same_character_name(a, alias)) {
                deltas.push(FigureDelta::Alias {
                    id: own.id.clone(),
                    alias: alias.clone(),
                });
            }
        }
    }
    for cue in not_figures {
        if !state
            .not_figures
            .iter()
            .any(|n| same_character_name(n, cue))
        {
            deltas.push(FigureDelta::NotFigure(cue.clone()));
        }
    }

    deltas
}

/// Whether `delta` can be done in `project` without two characters answering to one name.
pub(crate) fn fits(project: &Project, delta: &FigureDelta) -> bool {
    match delta {
        FigureDelta::Created(c) => {
            let taken = |name: &str| project.characters.iter().any(|o| o.is_called(name));

            project.character(&c.id).is_none()
                && !taken(&c.name)
                && !c.aliases.iter().any(|a| taken(a))
        }
        FigureDelta::Alias { id, alias } => {
            project.character(id).is_some()
                && !project
                    .characters
                    .iter()
                    .any(|o| o.id != *id && o.is_called(alias))
        }
        FigureDelta::NotFigure(_) => true,
    }
}
