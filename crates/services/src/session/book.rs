//! The register against the book (DATENMODELL §4): aligns on open and save, and takes the
//! user's answer for an entry that could not be placed.

use klappe_model::{BookState, Kind, Project, Scene};

use super::Session;
use crate::book::{self, BookReport, Choice, PendingScene};
use crate::error::ServiceError;

/// Undo step of `resolve_scene`.
const RESOLVE_STEP: &str = "ResolveScene";

/// One line of the dialog for an open question: a scene of the book the entry may be, or
/// "not in the book" as the last line. Surfaces show the lines and hand back the position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Answer {
    /// Empty for `Choice::NotInBook`; the surface names that line.
    pub number: String,
    pub heading: String,
    pub choice: Choice,
}

impl Session {
    /// The entries of the register the book could not place without asking, with the scenes
    /// each may be (DATENMODELL principle 6). Their scenes get no entry before an answer.
    pub fn pending_scenes(&self) -> &[PendingScene] {
        &self.pending
    }

    /// The answers of the open question about entry `id`: its candidates best first, then
    /// "not in the book". Empty when there is no such question.
    pub fn book_answers(&self, id: &str) -> Vec<Answer> {
        let Some(pending) = self.pending.iter().find(|p| p.id == id) else {
            return Vec::new();
        };
        let scenes = pending.candidates.iter().map(|c| Answer {
            number: c.number.clone(),
            heading: c.heading.clone(),
            choice: Choice::Scene(c.clone()),
        });
        let none = Answer {
            number: String::new(),
            heading: String::new(),
            choice: Choice::NotInBook,
        };

        scenes.chain(std::iter::once(none)).collect()
    }

    /// Answers the question about entry `id` with line `index` of `book_answers`.
    pub fn answer_book(&mut self, id: &str, index: usize) -> Result<(), ServiceError> {
        let answers = self.book_answers(id);
        let Some(answer) = answers.into_iter().nth(index) else {
            return Err(ServiceError::NoDecision(id.to_owned()));
        };

        self.resolve_scene(id, answer.choice)
    }

    /// What there is to say about the book and the register: for the status line.
    pub fn book_report(&self) -> BookReport {
        let not_in_book = self
            .project
            .scenes
            .iter()
            .filter(|s| s.kind == Kind::Scene && s.state == BookState::NotInBook)
            .count();

        BookReport {
            pending: self.pending.len(),
            not_in_book,
            places: self.pending_places.len(),
        }
    }

    /// Answers the open question about entry `old_id`: it is one of its candidates, or no
    /// scene of the book (the candidates then become new entries). One undo step. The entry
    /// takes number, heading and prints from the scene, as if it had been matched.
    pub fn resolve_scene(&mut self, old_id: &str, choice: Choice) -> Result<(), ServiceError> {
        let none = || ServiceError::NoDecision(old_id.to_owned());
        if self.script.scenes().is_empty() {
            return Err(none());
        }

        let mut plan = book::plan(&self.project, &self.script);
        if !plan.decide(&self.project, old_id, choice) {
            return Err(none());
        }

        let mut next = self.project.clone();
        plan.apply(&mut next);
        // The answer may settle others: an entry whose rival is out now agrees alone.
        let settled = book::plan(&next, &self.script);
        settled.apply(&mut next);

        let (scene, shown) = self.ids_on_view();
        self.commit_step(next, RESOLVE_STEP);
        self.follow_scene(scene, shown);

        Ok(())
    }

    /// Aligns the register with the book as it is now and writes the result into the
    /// project, without an undo step: the register follows the text, it is not an edit.
    /// Returns whether the register changed.
    pub(super) fn reconcile(&mut self) -> bool {
        self.refresh_places();
        if self.script.scenes().is_empty() {
            self.pending.clear();
            return false;
        }

        let plan = book::plan(&self.project, &self.script);
        let (scene, shown) = self.ids_on_view();
        self.pending = plan.pending(&self.project);
        if !plan.apply(&mut self.project) {
            return false;
        }

        self.follow_scene(scene, shown);

        true
    }

    /// Asks the alignment which entries are open, changing nothing. After undo, redo and
    /// a new screenplay text, which move the register or the book under the list.
    pub(super) fn refresh_pending(&mut self) {
        self.refresh_places();
        self.pending = match self.script.scenes().is_empty() {
            true => Vec::new(),
            false => book::plan(&self.project, &self.script).pending(&self.project),
        };
    }

    /// The scene and plan on view, to find them again after the entries moved.
    fn ids_on_view(&self) -> (Option<String>, Option<String>) {
        let scene = self.scene().map(|s| s.id.clone());
        let plan = self.plan().map(|p| p.id.clone());

        (scene, plan)
    }
}

/// Whether two states hold different registers (what the alignment reads of the entries), so
/// that the open questions may have moved. An edit of the plans alone does not move them, and
/// must not pay for aligning the register with the book again.
pub(super) fn register_moved(before: &Project, after: &Project) -> bool {
    let differ = |(a, b): (&Scene, &Scene)| {
        a.id != b.id
            || a.kind != b.kind
            || a.number != b.number
            || a.heading != b.heading
            || a.state != b.state
            || a.prints != b.prints
            || a.refused != b.refused
    };

    before.scenes.len() != after.scenes.len() || before.scenes.iter().zip(&after.scenes).any(differ)
}
