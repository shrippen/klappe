//! The characters against the book (DB-50): the cues no character answers to, and the user's
//! answer for each.

use super::Session;
use crate::error::ServiceError;
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
    /// step.
    pub fn resolve_figure(&mut self, name: &str, choice: FigureChoice) -> Result<(), ServiceError> {
        let none = || ServiceError::NoDecision(name.to_owned());
        if !self.pending_figures.iter().any(|p| p.name == name) {
            return Err(none());
        }

        let mut next = self.project.clone();
        if !figures::decide(&mut next, name, &choice) {
            return Err(none());
        }

        let (scene, shown) = self.ids_on_view();
        self.commit_step(next, RESOLVE_STEP);
        self.follow_scene(scene, shown);
        self.refresh_figures();

        Ok(())
    }

    /// Asks the book which cues are open, changing nothing.
    pub(super) fn refresh_figures(&mut self) {
        self.pending_figures = figures::pending(&self.project, &self.script);
    }
}
