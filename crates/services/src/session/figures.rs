//! The characters against the book (DB-50): the cues no character answers to, and the user's
//! answer for each.

use super::Session;
use crate::disk_step::DiskStep;
use crate::error::ServiceError;
use crate::figure_file::{self, FigureDelta, FigureSet};
use crate::figures::{self, FigureChoice, PendingFigure};

/// Undo step of `resolve_figure`.
const RESOLVE_STEP: &str = "ResolveFigure";

/// One line of the dialog for an unknown cue: make it a character, take it for a character
/// of the project, or none. Surfaces show the lines and hand back the position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FigureAnswer {
    /// Name of the character for `FigureChoice::Assign`; empty for the other lines.
    pub name: String,
    pub choice: FigureChoice,
}

impl Session {
    /// The names of the book no character answers to (DATENMODELL principle 6): asked, never
    /// made characters on their own.
    pub fn pending_figures(&self) -> &[PendingFigure] {
        &self.pending_figures
    }

    /// The answers to the question about the cue `name`: new character, each character of
    /// the project (those the book no longer cues first), not a character. Empty when there
    /// is no such question.
    pub fn figure_answers(&self, name: &str) -> Vec<FigureAnswer> {
        if !self.pending_figures.iter().any(|p| p.name == name) {
            return Vec::new();
        }

        let create = FigureAnswer {
            name: String::new(),
            choice: FigureChoice::Create,
        };
        let none = FigureAnswer {
            name: String::new(),
            choice: FigureChoice::NotFigure,
        };
        let assigns = figures::candidates(&self.project, &self.script)
            .into_iter()
            .filter_map(|id| {
                let name = self.project.character(&id)?.name.clone();

                Some(FigureAnswer {
                    name,
                    choice: FigureChoice::Assign(id),
                })
            });

        std::iter::once(create)
            .chain(assigns)
            .chain(std::iter::once(none))
            .collect()
    }

    /// Answers the question about the cue `name` with line `index` of `figure_answers`.
    pub fn answer_figure(&mut self, name: &str, index: usize) -> Result<(), ServiceError> {
        let answers = self.figure_answers(name);
        let Some(answer) = answers.into_iter().nth(index) else {
            return Err(ServiceError::NoDecision(name.to_owned()));
        };

        self.resolve_figure(name, answer.choice)
    }

    /// Settles the cue `name`: a new character, an alias of one, or no character. One undo
    /// step, written to `figuren.toml` at once, from the file as it is on disk: another
    /// program (the window beside the TUI) may have changed it, and the answer changes only
    /// what it touches. A question that is gone there (answered elsewhere) is no error: the
    /// disk is taken and the question disappears. A cue never asked is refused.
    pub fn resolve_figure(&mut self, name: &str, choice: FigureChoice) -> Result<(), ServiceError> {
        if !self.pending_figures.iter().any(|p| p.name == name) {
            return Err(ServiceError::NoDecision(name.to_owned()));
        }

        let mut next = self.project_with_disk_figures()?;
        let open = figures::pending(&next, &self.script)
            .iter()
            .any(|p| p.name == name);
        if !open || !figures::decide(&mut next, name, &choice) {
            self.take_figures(next);

            return Ok(());
        }
        klappe_store::save_characters(&self.dir, &next)?;

        let Some(delta) = FigureDelta::of(name, &choice, &next) else {
            return Err(ServiceError::NoDecision(name.to_owned()));
        };
        let (scene, shown) = self.ids_on_view();
        self.figures_base = FigureSet::of(&next);
        self.commit_answer(next, RESOLVE_STEP, delta);
        self.follow_scene(scene, shown);
        self.refresh_figures();

        Ok(())
    }

    /// Writes what undo or redo left: the register, and for the answer to a cue only its
    /// entry in `figuren.toml`, taken back or done again in the file as it is on disk (the
    /// window beside the TUI may have changed the rest). The session then has the file's
    /// figures. Undo of anything else leaves `figuren.toml` alone.
    pub fn save_answer(&mut self) -> Result<(), ServiceError> {
        klappe_store::save_register(&self.dir, &self.project.scenes)?;

        let mut disk = self.project_with_disk_figures()?;
        if let Some((delta, way)) = self.disk_step.take() {
            delta.run(&mut disk, way);
            klappe_store::save_characters(&self.dir, &disk)?;
        }
        self.take_figures(disk);

        Ok(())
    }

    /// Before a save: the figures of the file and of the session, merged (see `figure_file`).
    pub(super) fn merge_figures(&mut self) -> Result<(), ServiceError> {
        let (characters, not_figures) = klappe_store::load_figures(&self.dir)?;
        let disk = FigureSet {
            characters,
            not_figures,
        };
        let merged = figure_file::merge(&self.figures_base, &self.project, disk);

        merged.clone().put(&mut self.project);
        self.figures_base = merged;
        self.refresh_figures();

        Ok(())
    }

    /// The project with the figures of `figuren.toml` as it is now, not the session's copy:
    /// it may be older.
    fn project_with_disk_figures(&self) -> Result<klappe_model::Project, ServiceError> {
        let (characters, not_figures) = klappe_store::load_figures(&self.dir)?;
        let mut project = self.project.clone();
        project.characters = characters;
        project.not_figures = not_figures;

        Ok(project)
    }

    /// The figures of `from` (read from the file) become the session's.
    fn take_figures(&mut self, from: klappe_model::Project) {
        let figures = FigureSet::of(&from);

        self.figures_base = figures.clone();
        figures.put(&mut self.project);
        self.refresh_figures();
    }

    /// Asks the book which cues are open, changing nothing.
    pub(super) fn refresh_figures(&mut self) {
        self.pending_figures = figures::pending(&self.project, &self.script);
    }
}
