//! `figuren.toml` against an answer (DB-50): the one entry an answer to a cue changed, kept to
//! do it again or take it back. The file can change under the session (the window beside the
//! TUI), so an answer, its undo and its redo change only their entry, in the file as it is
//! on disk.

use klappe_model::{same_character_name, Character, Project};

use crate::answer_way::Way;
use crate::figures::FigureChoice;

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

/// The content of `figuren.toml`: the characters and the not-figures.
pub(crate) type Figures = (Vec<Character>, Vec<String>);

/// What `disk` (the figures read from `figuren.toml`) has added since the session last took
/// them (`file`), as answers would: a character whose id neither `file` nor `state` has (whole,
/// with its aliases), an alias `state`'s character of that id lacks, a not-figure `state` lacks.
/// Whatever else differs (renamed, deleted, recoloured) is the session's own and not adopted.
pub(crate) fn added(file: &Figures, state: &Project, disk: &Figures) -> Vec<FigureDelta> {
    let known = |id: &str| file.0.iter().any(|c| c.id == id);
    let (characters, not_figures) = disk;
    let mut deltas = Vec::new();

    for on_disk in characters {
        let Some(own) = state.character(&on_disk.id) else {
            if !known(&on_disk.id) {
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

/// `file` plus what answers added to it in `disk`; what else `disk` changed (a deletion
/// outside) is left out, so it still shows as a change of the file.
pub(crate) fn with_answers(file: &Figures, disk: &Figures) -> Figures {
    let mut base = Project::new("");
    base.characters.clone_from(&file.0);
    base.not_figures.clone_from(&file.1);

    let mut after = base.clone();
    for delta in added(file, &base, disk) {
        delta.run(&mut after, Way::Apply);
    }

    (after.characters, after.not_figures)
}

/// `file` with the one entry `delta` wrote.
pub(crate) fn with_delta(file: &Figures, delta: &FigureDelta) -> Figures {
    let mut after = Project::new("");
    after.characters.clone_from(&file.0);
    after.not_figures.clone_from(&file.1);
    delta.run(&mut after, Way::Apply);

    (after.characters, after.not_figures)
}

/// Whether `disk` is `file` plus what answers add, and nothing else: no rename, deletion or
/// change of a character. Then taking it over loses nobody's work (the figure rule).
pub(crate) fn answers_only(file: &Figures, disk: &Figures) -> bool {
    with_answers(file, disk) == *disk
}
