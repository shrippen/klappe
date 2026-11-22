use std::sync::atomic::{AtomicU64, Ordering};

use klappe_model::Project;

use super::book::register_moved;
use super::Session;
use crate::figure_file::{FigureDelta, Way};

/// Undo keeps this many steps (LH-06 asks for no fixed limit within a session; this only
/// guards memory on very long sessions).
const UNDO_LIMIT: usize = 500;

static EDITS: AtomicU64 = AtomicU64::new(0);

/// A number no state has had before: one count for all sessions, so opening another project
/// never repeats one.
pub(super) fn next_edit() -> u64 {
    EDITS.fetch_add(1, Ordering::Relaxed)
}

/// A state an undo goes back to: the project, and the screenplay text when the step changed
/// it too (a rename in the project also renames in the text, as one step).
#[derive(Debug)]
pub(super) struct Snapshot {
    project: Project,
    script: Option<String>,
    /// What the step that made this state's successor did (`AddObject`); empty if unnamed.
    step: String,
    /// The entry of `figuren.toml` that step wrote, if it answered a cue (DB-50).
    figure: Option<FigureDelta>,
}

/// The name of an edit for the undo tooltip: its variant (`Move { .. }` is `Move`). The app
/// turns it into words (`step.<name>`).
pub(super) fn step_name(edit: &impl std::fmt::Debug) -> String {
    format!("{edit:?}")
        .chars()
        .take_while(|c| c.is_alphanumeric())
        .collect()
}

impl Session {
    pub(super) fn commit(&mut self, next: Project) {
        self.commit_step(next, "");
    }

    /// Like `commit`; undo and redo name the step `step`.
    pub(super) fn commit_step(&mut self, next: Project, step: &str) {
        self.commit_with(next, None, step);
    }

    /// Like `commit`, and the screenplay becomes `script`; undo restores both.
    pub(super) fn commit_script(&mut self, next: Project, script: &str, step: &str) {
        let before = self.script_text().to_owned();

        self.commit_with(next, Some(before), step);
        self.set_script_text(script);
    }

    /// Like `commit_step`, for the answer to a cue: undo and redo write `figure` again or
    /// take it back in `figuren.toml`.
    pub(super) fn commit_answer(&mut self, next: Project, step: &str, figure: FigureDelta) {
        self.commit_step(next, step);
        if let Some(last) = self.undo.last_mut() {
            last.figure = Some(figure);
        }
    }

    fn commit_with(&mut self, next: Project, script: Option<String>, step: &str) {
        let project = std::mem::replace(&mut self.project, next);
        self.undo.push(Snapshot {
            project,
            script,
            step: step.to_owned(),
            figure: None,
        });
        if self.undo.len() > UNDO_LIMIT {
            self.undo.remove(0);
        }
        self.redo.clear();
        self.project_changed = true;
        self.edits = next_edit();
        self.touch();
        if self
            .undo
            .last()
            .is_some_and(|s| register_moved(&s.project, &self.project))
        {
            self.refresh_pending();
        }
    }

    pub(super) fn touch(&mut self) {
        self.dirty = true;
        self.unbacked = true;
    }

    /// The view stays with the scene `id` if the (new) project has it, else it starts over.
    pub(super) fn follow_scene(&mut self, id: Option<String>, plan: Option<String>) {
        let index = id.and_then(|id| self.project.scenes.iter().position(|s| s.id == id));
        let Some(index) = index else {
            self.scene = self.scene.min(self.project.scenes.len().saturating_sub(1));
            self.plan = 0;
            self.time_s = 0.0;
            self.selected_shot = None;
            return;
        };

        if index != self.scene {
            self.scene = index;
            self.plan = 0;
        }

        // Plans come and go too (add, snapshot, delete, undo): stay with the one on view.
        let plan_index = plan.and_then(|id| {
            self.scene()
                .and_then(|s| s.plans.iter().position(|p| p.id == id))
        });
        match plan_index {
            Some(index) => self.plan = index,
            None => {
                self.plan = 0;
                self.time_s = 0.0;
                self.selected_shot = None;
            }
        }
        self.keep_view_valid();
    }

    /// Scene and plan indices of the view fit the (new) project again.
    pub(super) fn keep_view_valid(&mut self) {
        if self.scene >= self.project.scenes.len() {
            self.scene = 0;
        }
        if self.scene().is_none_or(|s| self.plan >= s.plans.len()) {
            self.plan = 0;
        }

        // Undo and redo can take the selected shot away.
        let selected_is_gone = self.selected_shot.as_deref().is_some_and(|id| {
            self.plan()
                .is_none_or(|p| !p.shots.iter().any(|s| s.id == id))
        });
        if selected_is_gone {
            self.selected_shot = None;
        }
    }

    /// Names the state of the plan on view: it changes with every edit, undo and redo and with
    /// the scene or plan shown, not with the selection or the playhead, so what is drawn from
    /// the plan is out of date when it does.
    pub fn edits(&self) -> u64 {
        self.edits
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// What undo takes back: the name of the step; `None` for no step or an unnamed one.
    pub fn undo_step(&self) -> Option<&str> {
        named(self.undo.last())
    }

    /// What redo does again, named as `undo_step`.
    pub fn redo_step(&self) -> Option<&str> {
        named(self.redo.last())
    }

    pub fn undo(&mut self) {
        let Some(previous) = self.undo.pop() else {
            return;
        };
        let step = previous.step.clone();
        let figure = previous.figure.clone();
        let on_view = self.scene().map(|s| s.id.clone());
        let plan_on_view = self.plan().map(|p| p.id.clone());
        let mut current = self.swap(previous);
        current.step = step;
        current.figure.clone_from(&figure);
        self.disk_step = figure.map(|f| (f, Way::Revert));
        let moved = register_moved(&current.project, &self.project);
        self.redo.push(current);
        self.project_changed = true;
        self.edits = next_edit();
        self.touch();
        self.follow_scene(on_view, plan_on_view);
        self.end_stale_look();
        if moved {
            self.refresh_pending();
        }
    }

    pub fn redo(&mut self) {
        let Some(next) = self.redo.pop() else {
            return;
        };
        let step = next.step.clone();
        let figure = next.figure.clone();
        let on_view = self.scene().map(|s| s.id.clone());
        let plan_on_view = self.plan().map(|p| p.id.clone());
        let mut current = self.swap(next);
        current.step = step;
        current.figure.clone_from(&figure);
        self.disk_step = figure.map(|f| (f, Way::Apply));
        let moved = register_moved(&current.project, &self.project);
        self.undo.push(current);
        self.project_changed = true;
        self.edits = next_edit();
        self.touch();
        self.follow_scene(on_view, plan_on_view);
        self.end_stale_look();
        if moved {
            self.refresh_pending();
        }
    }

    /// Makes `step` the state and returns the one it replaces, for the other stack.
    fn swap(&mut self, step: Snapshot) -> Snapshot {
        // The export paper is a setting, not an edit: undo and redo keep the current one.
        let paper = self.project.paper;
        let project = std::mem::replace(&mut self.project, step.project);
        self.project.paper = paper;
        let script = step.script.map(|text| {
            let now = self.script_text().to_owned();
            self.set_script_text(&text);
            now
        });

        Snapshot {
            project,
            script,
            step: String::new(),
            figure: None,
        }
    }
}

fn named(step: Option<&Snapshot>) -> Option<&str> {
    step.map(|s| s.step.as_str()).filter(|s| !s.is_empty())
}
