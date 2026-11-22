//! The characters against the book (DB-50): the cues no character answers to, and the user's
//! answer for each.

use klappe_model::Project;

use super::Session;
use crate::error::ServiceError;
use crate::figure_file::{self, FigureDelta, Way};
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
    /// its entry, never the session's list (the window's unsaved edits stay unsaved). What
    /// the file gained since is adopted first (`adopt_figures`). A question that is gone
    /// there (answered elsewhere) is no error: it disappears. A cue never asked is refused.
    pub fn resolve_figure(&mut self, name: &str, choice: FigureChoice) -> Result<(), ServiceError> {
        if !self.pending_figures.iter().any(|p| p.name == name) {
            return Err(ServiceError::NoDecision(name.to_owned()));
        }

        self.adopt_figures()?;
        let open = self.pending_figures.iter().any(|p| p.name == name);
        let mut next = self.project.clone();
        if !open || !figures::decide(&mut next, name, &choice) {
            return Ok(());
        }
        let Some(delta) = FigureDelta::of(name, &choice, &next) else {
            return Err(ServiceError::NoDecision(name.to_owned()));
        };

        let mut disk = self.project_with_disk_figures()?;
        delta.run(&mut disk, Way::Apply);
        klappe_store::save_characters(&self.dir, &disk)?;

        if let FigureDelta::Created(character) = &delta {
            self.known_figures.insert(character.id.clone());
        }
        let (scene, shown) = self.ids_on_view();
        self.commit_answer(next, RESOLVE_STEP, delta);
        self.follow_scene(scene, shown);
        self.refresh_figures();

        Ok(())
    }

    /// Writes what undo or redo left: the register, and for the answer to a cue only its
    /// entry in `figuren.toml`, taken back or done again in the file as it is on disk (the
    /// window beside the TUI may have changed the rest). Undo of anything else leaves
    /// `figuren.toml` alone.
    pub fn save_answer(&mut self) -> Result<(), ServiceError> {
        klappe_store::save_register(&self.dir, &self.project.scenes)?;

        // The delta stays until the file is written: a failed read or write keeps it for the
        // next try (`run` leaves an entry already (not) there as it is).
        let mut disk = self.project_with_disk_figures()?;
        if let Some((delta, way)) = &self.disk_step {
            delta.run(&mut disk, *way);
            klappe_store::save_characters(&self.dir, &disk)?;
        }
        self.disk_step = None;

        self.adopt_figures()
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

        for delta in figure_file::added(&self.known_figures, &self.project, &disk) {
            self.in_every_state(|project| {
                if figure_file::fits(project, &delta) {
                    delta.run(project, Way::Apply);
                }
            });
        }
        self.known_figures
            .extend(disk.0.iter().map(|c| c.id.clone()));
        self.refresh_figures();

        Ok(())
    }

    /// Sets the ids of the project's characters as just read from or written to `figuren.toml`.
    pub(super) fn know_figures(&mut self) {
        self.known_figures = self
            .project
            .characters
            .iter()
            .map(|c| c.id.clone())
            .collect();
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
