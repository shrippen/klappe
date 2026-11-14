use klappe_model::Project;

use super::Session;

/// Undo keeps this many steps (LH-06 asks for no fixed limit within a session; this only
/// guards memory on very long sessions).
const UNDO_LIMIT: usize = 500;

impl Session {
    pub(super) fn commit(&mut self, next: Project) {
        let previous = std::mem::replace(&mut self.project, next);
        self.undo.push(previous);
        if self.undo.len() > UNDO_LIMIT {
            self.undo.remove(0);
        }
        self.redo.clear();
        self.project_changed = true;
        self.touch();
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

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn undo(&mut self) {
        let Some(previous) = self.undo.pop() else {
            return;
        };
        let on_view = self.scene().map(|s| s.id.clone());
        let plan_on_view = self.plan().map(|p| p.id.clone());
        self.redo
            .push(std::mem::replace(&mut self.project, previous));
        self.project_changed = true;
        self.touch();
        self.follow_scene(on_view, plan_on_view);
    }

    pub fn redo(&mut self) {
        let Some(next) = self.redo.pop() else {
            return;
        };
        let on_view = self.scene().map(|s| s.id.clone());
        let plan_on_view = self.plan().map(|p| p.id.clone());
        self.undo.push(std::mem::replace(&mut self.project, next));
        self.project_changed = true;
        self.touch();
        self.follow_scene(on_view, plan_on_view);
    }
}
