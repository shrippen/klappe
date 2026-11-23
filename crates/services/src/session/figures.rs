//! The characters against the book (DB-50): the cues no character answers to, and the user's
//! answer for each.

use klappe_model::Project;

use super::outside::FIGURES_FILE;
use super::questions::Answered;
use super::undo::AnswerDelta;
use super::Session;
use crate::answer_way::Way;
use crate::error::ServiceError;
use crate::figure_file::{self, FigureDelta};
use crate::figures::{self, FigureChoice, PendingFigure};

/// Undo step of `resolve_figure`.
pub(super) const RESOLVE_STEP: &str = "ResolveFigure";

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
    pub fn answer_figure(&mut self, name: &str, index: usize) -> Result<Answered, ServiceError> {
        let answers = self.figure_answers(name);
        let Some(answer) = answers.into_iter().nth(index) else {
            return Err(ServiceError::NoDecision(name.to_owned()));
        };

        self.resolve_figure(name, answer.choice)
    }

    /// Settles the cue `name`: a new character, an alias of one, or no character. One undo
    /// step, written to `figuren.toml` at once, from the file as it is on disk: another
    /// program (the window beside the TUI) may have changed it, and the answer changes only
    /// its entry, never the session's list (the window's unsaved edits stay unsaved). What
    /// the file gained since is adopted first (`adopt_figures`). A question that is gone
    /// there (answered elsewhere) is no error: it disappears. A cue never asked is refused.
    pub fn resolve_figure(
        &mut self,
        name: &str,
        choice: FigureChoice,
    ) -> Result<Answered, ServiceError> {
        if !self.pending_figures.iter().any(|p| p.name == name) {
            return Err(ServiceError::NoDecision(name.to_owned()));
        }

        let before = self.before(FIGURES_FILE)?;
        self.adopt_figures()?;
        // The character may be gone from the file (deleted elsewhere), and with it from here.
        if let FigureChoice::Assign(id) = &choice {
            if self.project.character(id).is_none() {
                return Err(ServiceError::NoCharacter(id.clone()));
            }
        }
        let open = self.pending_figures.iter().any(|p| p.name == name);
        let mut next = self.project.clone();
        if !open || !figures::decide(&mut next, name, &choice) {
            return Ok(Answered::Gone);
        }
        let Some(delta) = FigureDelta::of(name, &choice, &next) else {
            return Err(ServiceError::NoDecision(name.to_owned()));
        };

        let mut disk = self.project_with_disk_figures()?;
        // An alias for a character the file had and lost (deleted elsewhere) would be lost
        // without a trace. One the session added and not yet saved gets it at the save.
        if let FigureDelta::Alias { id, .. } = &delta {
            if self.file_figures.0.iter().any(|c| c.id == *id) && disk.character(id).is_none() {
                return Err(ServiceError::NoCharacter(id.clone()));
            }
        }
        delta.run(&mut disk, Way::Apply);
        klappe_store::save_characters(&self.dir, &disk)?;

        let (scene, shown) = self.ids_on_view();
        self.commit_answer(next, RESOLVE_STEP, AnswerDelta::Figure(delta));
        self.follow_scene(scene, shown);
        self.refresh_figures();
        self.rebase(FIGURES_FILE, before)?;

        Ok(Answered::Step)
    }

    /// Undo or redo of a figure answer: its entry in `figuren.toml`, in the file as it is on
    /// disk (another program may have changed the rest).
    pub(super) fn save_figure_delta(
        &self,
        delta: &FigureDelta,
        way: Way,
    ) -> Result<(), ServiceError> {
        let mut disk = self.project_with_disk_figures()?;
        delta.run(&mut disk, way);
        klappe_store::save_characters(&self.dir, &disk)?;

        Ok(())
    }

    /// Takes what answers gave `figuren.toml` since the session last read it: new characters,
    /// aliases, not-figures, as long as no name would belong to two characters. They go into
    /// the project and into every undo and redo step, so neither undo nor a save drops them.
    /// The session's own changes (rename, delete) stay: the file is never taken whole. A file
    /// that does not parse adopts nothing; the next save rewrites it.
    pub(super) fn adopt_figures(&mut self) -> Result<(), ServiceError> {
        let disk = match klappe_store::load_figures(&self.dir) {
            Ok(disk) => disk,
            Err(klappe_store::StoreError::Parse { .. }) => return Ok(()),
            Err(other) => return Err(other.into()),
        };

        for delta in figure_file::added(&self.file_figures, &self.project, &disk) {
            self.in_every_state(|project| {
                if figure_file::fits(project, &delta) {
                    delta.run(project, Way::Apply);
                }
            });
        }
        self.file_figures = disk;
        self.refresh_figures();

        Ok(())
    }

    /// Sets the ids of the project's characters as just read from or written to `figuren.toml`.
    pub(super) fn know_figures(&mut self) {
        self.file_figures = (
            self.project.characters.clone(),
            self.project.not_figures.clone(),
        );
    }

    /// The project with the figures of `figuren.toml` as it is now, not the session's copy:
    /// it may be older.
    fn project_with_disk_figures(&self) -> Result<Project, ServiceError> {
        let (characters, not_figures) = klappe_store::load_figures(&self.dir)?;
        let mut project = self.project.clone();
        project.characters = characters;
        project.not_figures = not_figures;

        Ok(project)
    }

    /// Asks the book which cues are open, changing nothing.
    pub(super) fn refresh_figures(&mut self) {
        self.pending_figures = figures::pending(&self.project, &self.script);
    }
}
