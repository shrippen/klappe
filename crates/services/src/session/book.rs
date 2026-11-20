//! The register against the book (DATENMODELL §4): aligns on open, save and import, and takes the
//! user's answer for an entry that could not be placed.

use klappe_model::{BookState, Kind, Project, Scene};

use super::Session;
use crate::book::{self, BookReport, Choice, ImportReport, PendingScene};
use crate::error::ServiceError;

/// Undo step of `resolve_scene`.
const RESOLVE_STEP: &str = "ResolveScene";

/// Undo step of `import_script`.
const IMPORT_STEP: &str = "ImportScript";

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

    /// Whether undo would take back an answer to a question about the book: a surface that
    /// keeps its own text history beside the session's asks this before it takes the session's.
    pub fn undo_is_answer(&self) -> bool {
        self.undo_step() == Some(RESOLVE_STEP)
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
            twin_numbers: self.twin_numbers,
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

    /// Takes `text` as the screenplay and aligns the register with it in one undo step (undo
    /// restores project and text). An empty book leaves the register alone, as `reconcile` does.
    pub(super) fn import_book(&mut self, text: &str) -> ImportReport {
        let book = klappe_script::parse(text);
        let scenes = book::heading_count(&book);
        if book.scenes().is_empty() {
            self.set_script_text(text);

            return ImportReport::default();
        }

        let mut next = self.project.clone();
        book::plan(&self.project, &book).apply(&mut next);
        let created = next
            .scenes
            .iter()
            .filter(|s| self.project.scene(&s.id).is_none())
            .count();

        // The book the project has already: no edit, so no unsaved state and no empty undo step.
        let same = next == self.project && self.script_text.as_deref() == Some(text);
        if !same {
            let (scene, shown) = self.ids_on_view();
            self.commit_script(next, text, IMPORT_STEP);
            self.follow_scene(scene, shown);
        }
        let report = self.book_report();

        ImportReport {
            scenes,
            created,
            pending: report.pending,
            not_in_book: report.not_in_book,
        }
    }

    /// Aligns the register with the book as it is now and writes the result into the
    /// project, without an undo step: the register follows the text, it is not an edit.
    /// Returns whether the register changed.
    pub(super) fn reconcile(&mut self) -> bool {
        self.refresh_places();
        if self.script.scenes().is_empty() {
            self.pending.clear();
            self.twin_numbers = 0;
            self.refresh_scene_map(None);
            return false;
        }

        let plan = book::plan(&self.project, &self.script);
        let (scene, shown) = self.ids_on_view();
        self.pending = plan.pending(&self.project);
        self.count_twin_numbers(&plan);
        if !plan.apply(&mut self.project) {
            self.refresh_scene_map(Some(&plan));
            return false;
        }

        // The entries moved: the plan's positions are the old ones.
        self.refresh_scene_map(None);
        self.follow_scene(scene, shown);

        true
    }

    /// Asks the alignment which entries are open, changing nothing. After undo, redo and
    /// a new screenplay text, which move the register or the book under the list.
    pub(super) fn refresh_pending(&mut self) {
        self.refresh_places();
        if self.script.scenes().is_empty() {
            self.pending = Vec::new();
            self.twin_numbers = 0;
            self.refresh_scene_map(None);
            return;
        }

        let plan = book::plan(&self.project, &self.script);
        self.pending = plan.pending(&self.project);
        self.count_twin_numbers(&plan);
        self.refresh_scene_map(Some(&plan));
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
    before.scenes.len() != after.scenes.len()
        || before
            .scenes
            .iter()
            .zip(&after.scenes)
            .any(|(a, b)| aligned(a) != aligned(b))
}

/// What the alignment reads of an entry. Every field is named, so a new one does not compile
/// until someone decides whether the alignment reads it (audit 16, finding 1).
fn aligned(scene: &Scene) -> impl PartialEq + '_ {
    let Scene {
        id,
        kind,
        number,
        heading,
        state,
        prints,
        refused,
        // Not read by the alignment: where it is shot and what is staged.
        location: _,
        sun: _,
        plans: _,
    } = scene;

    (id, kind, number, heading, state, prints, refused)
}
