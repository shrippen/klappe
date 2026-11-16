//! The drawing tools: picking a tool, dragging, placing and committing what they make.

use super::q;
use super::qobject;
use super::view::plan_json;
use crate::object;
use crate::plan::{Measure, Sweep};
use crate::selection;
use crate::selection::{Pick, Reach};
use crate::tools;
use crate::tools::{
    AreaDraft, Closing, DragPhase, DragStep, DrawStyle, Grip, MeasureStep, NoteDraft, Preview,
    ScaleStep, Tool, Typed,
};
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use klappe_services::error_text;
use klappe_services::{Grid, PlanEdit, Point, Session, Snapping, Source};

/// How many bends the way of the grip `grip` has.
fn bends(s: &Session, grip: &Source) -> usize {
    let Some(plan) = s.plan() else {
        return 0;
    };
    let via = match grip {
        Source::FigureVia {
            character, index, ..
        } => plan
            .figures
            .iter()
            .find(|f| f.character == *character)
            .and_then(|f| f.marks.get(*index))
            .map(|m| &m.via),
        Source::CameraVia { camera, index, .. } => plan
            .camera(camera)
            .and_then(|c| c.marks.get(*index))
            .map(|m| &m.via),
        _ => None,
    };

    via.map_or(0, Vec::len)
}

/// The edit of a drag of the height tool: `grabbed` made `lift_m` taller than it stands now.
fn lift_edit(s: &Session, tool: &Tool, grabbed: Option<&Source>, lift_m: f64) -> Option<PlanEdit> {
    let grabbed = grabbed.filter(|_| *tool == Tool::Height)?;

    tools::height_edit(grabbed, s.height_of(grabbed)?, lift_m)
}

impl super::ControllerRust {
    /// The height a height drag shows at `at` ("1,25 m"), also kept for the 3D view's caption;
    /// empty once the drag is let go or edits nothing.
    fn lift_caption(
        &mut self,
        edit: Option<&PlanEdit>,
        phase: DragPhase,
        at: Point,
    ) -> Option<Measure> {
        let height_m = match (edit, phase) {
            (
                Some(
                    PlanEdit::SetObjectHeight {
                        height_m: Some(height_m),
                        ..
                    }
                    | PlanEdit::SetWallHeight {
                        height_m: Some(height_m),
                        ..
                    },
                ),
                DragPhase::Moving,
            ) => *height_m,
            _ => {
                self.free.caption.clear();
                return None;
            }
        };

        let decimal = self.language.text("unit.decimal");
        let text = tools::length_text(height_m, &decimal, &self.language.text("unit.m"));
        self.free.caption = text.clone();

        Some(Measure {
            points: vec![at],
            caption: text,
        })
    }
}

impl qobject::Controller {
    pub(super) fn set_tool(mut self: Pin<&mut Self>, tool: &QString, choice: &QString) -> bool {
        let Some(tool) = Tool::parse(&tool.to_string(), &choice.to_string()) else {
            return false;
        };

        // Points picked for a scale do not outlast the tool.
        let leaves_scale = tool != Tool::Scale && self.rust().scale != ScaleStep::Idle;
        let leaves_measure = tool != Tool::Measure && self.rust().measure != MeasureStep::Idle;
        let leaves_note = tool != Tool::Note && self.rust().note != NoteDraft::Idle;
        let leaves_area = tool != self.rust().tool && !self.rust().area.is_empty();
        self.as_mut().rust_mut().get_mut().tool = tool;
        if leaves_scale {
            self.as_mut().cancel_scale();
        }
        if leaves_area {
            self.as_mut().cancel_area();
        }
        if leaves_measure {
            self.as_mut().cancel_measure();
        }
        if leaves_note {
            self.cancel_note();
        }
        true
    }

    pub(super) fn drag(
        mut self: Pin<&mut Self>,
        from_x: f64,
        from_y: f64,
        x: f64,
        y: f64,
        done: bool,
    ) -> bool {
        if self.rust().grip == Grip::Spent {
            return self.skip_spent(done);
        }
        let step = (!done).then_some([from_x, from_y, x, y]);
        self.as_mut().rust_mut().get_mut().drag_step = step;

        let tool = self.rust().tool.clone();
        let style = self.rust().draw;
        let selected = self.rust().selected.clone();
        let how = self.rust().pick_mode();

        // The canvas' key can name a view cone or a path that lies over the thing meant;
        // the session picks what a user can grab, once per drag.
        let from = Point::new(from_x, from_y);
        let to = Point::new(x, y);
        let grip = std::mem::take(&mut self.as_mut().rust_mut().get_mut().grip);
        let Some(s) = self.as_mut().session() else {
            return false;
        };
        let (grip, grabbed) = grip.take(|| s.pick_for(from, selected.only()));

        // Select and Scale move by a distance (the session snaps where an element lands, a
        // background moves freely), Height by the pointer's rise; the other tools draw between
        // two points, which snap here.
        let (from, to) = match tool {
            Tool::Select | Tool::Scale | Tool::Height => (from, to),
            _ => (s.snap(from), s.snap(to)),
        };
        let group = grabbed
            .as_ref()
            .filter(|_| tool == Tool::Select)
            .and_then(|g| selection::group_drag_edit(&selected, g, from, to));
        // Plan y runs down: the pointer's rise is `from.y_m - to.y_m` (3D hands over its own).
        let lifted = lift_edit(s, &tool, grabbed.as_ref(), from.y_m - to.y_m);
        let mut edit = group
            .or_else(|| tools::drag_edit(&tool, grabbed.as_ref(), from, to))
            .or_else(|| tools::draw_edit(&tool, style, from, to))
            .or(lifted);
        if tool == Tool::Scale && !s.has_background() {
            edit = None;
        }
        let preview = self.rust().preview;
        let phase = if done {
            DragPhase::Done
        } else {
            DragPhase::Moving
        };
        self.as_mut().rust_mut().get_mut().grip = grip.after(phase);
        let caption = self
            .as_mut()
            .rust_mut()
            .get_mut()
            .lift_caption(edit.as_ref(), phase, to);

        // Empty ground with the select tool pulls up a box to take what lies in it.
        if tool == Tool::Select && grabbed.is_none() {
            return self.drag_box((from, to), phase, how);
        }

        if tool == Tool::Select && edit.is_some() {
            self.as_mut().rust_mut().get_mut().selected = selected.grab(grabbed);
        }
        match tools::drag_step(edit, phase, preview) {
            DragStep::Preview(edit) => self.preview_edit(&edit, caption.as_ref()),
            DragStep::Commit(edit) => self.commit_pulled(edit),
            DragStep::Restore => {
                self.refresh();
                false
            }
            DragStep::Nothing => false,
        }
    }

    /// A typed measure (LH-95): with a wall drag on, the wall's length from its start toward
    /// the pointer; else the height of the one selected object or wall.
    pub(super) fn type_measure(self: Pin<&mut Self>, text: &QString) -> bool {
        let text = text.to_string();

        // An empty text would take the height off, which is not a measure.
        if text.trim().is_empty() {
            return false;
        }

        let tool = self.rust().tool.clone();
        let step = self
            .rust()
            .drag_step
            .filter(|_| tool == Tool::Wall || tool == Tool::Room);
        if let Some(step) = step {
            return self.finish_wall(step, &text);
        }

        let selected = self.rust().selected.only();
        let typed = object::set_height_edit(selected, object::HeightField::Wall, &text)
            .or_else(|| object::object_height_edit(selected, &text));

        self.commit_typed(typed)
    }

    /// A typed length finished the wall: the rest of the drag adds nothing, the release frees it.
    fn skip_spent(mut self: Pin<&mut Self>, done: bool) -> bool {
        if done {
            self.as_mut().rust_mut().get_mut().grip = Grip::Free;
        }

        false
    }

    /// Ends the wall or room drag at `step` with a wall of the typed length or a room of the
    /// typed size.
    fn finish_wall(mut self: Pin<&mut Self>, step: [f64; 4], text: &str) -> bool {
        let [from_x, from_y, x, y] = step;
        let Some(s) = self.as_mut().session() else {
            return false;
        };
        let (from, toward) = (s.snap(Point::new(from_x, from_y)), s.snap(Point::new(x, y)));
        let typed = match self.rust().tool {
            Tool::Room => tools::room_of_size(from, toward, text),
            _ => tools::wall_of_length(from, toward, text),
        };
        if !self.as_mut().commit_typed(typed) {
            return false;
        }

        let state = self.rust_mut().get_mut();
        state.grip = Grip::Spent;
        state.drag_step = None;
        true
    }

    pub(super) fn place(mut self: Pin<&mut Self>, x: f64, y: f64) -> bool {
        let tool = self.rust().tool.clone();
        let selected = self.rust().selected.clone();
        let how = self.rust().pick_mode();
        let Some(s) = self.as_mut().session() else {
            return false;
        };

        if tool == Tool::Select {
            let picked = s.pick_click(Point::new(x, y), selected.only());
            let found = picked.is_some();
            self.as_mut().rust_mut().get_mut().selected = selected.click(picked, how);
            self.refresh();
            return found;
        }

        let at = tools::lands_at(&tool, Point::new(x, y), |p| s.snap(p));
        if tool == Tool::Scale {
            return self.pick_scale_point(at);
        }
        if tool == Tool::Measure {
            return self.click_measure(at);
        }
        if tool == Tool::Note {
            let picked = s.pick(Point::new(x, y));
            return self.ask_note(NoteDraft::click(at, picked.as_ref()));
        }
        if tool == Tool::Area || tool == Tool::FreeForm {
            return self.click_area(at);
        }

        let Some(edit) = tools::click_edit(&tool, at) else {
            return false;
        };

        self.commit_edit(edit)
    }

    pub(super) fn set_draw_style(
        mut self: Pin<&mut Self>,
        color: &QString,
        dash: &QString,
        fill: &QString,
    ) -> bool {
        let names = (color.to_string(), dash.to_string(), fill.to_string());
        let Some(style) = DrawStyle::parse(&names.0, &names.1, &names.2) else {
            return false;
        };

        self.as_mut().rust_mut().get_mut().draw = style;
        true
    }

    pub(super) fn finish_area(mut self: Pin<&mut Self>) -> bool {
        let style = self.rust().draw;
        let edit = match self.rust().tool {
            Tool::FreeForm => tools::form_edit(&self.rust().area),
            _ => tools::area_edit(&self.rust().area, style),
        };
        let Some(edit) = edit else {
            return false;
        };

        // The points stay when the edit is refused, so the work is not lost.
        if !self.as_mut().commit_edit(edit) {
            return false;
        }

        self.as_mut().rust_mut().get_mut().area = AreaDraft::default();
        self.refresh();
        true
    }

    pub(super) fn cancel_area(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().get_mut().area = AreaDraft::default();
        self.refresh();
    }

    pub(super) fn set_snapping(mut self: Pin<&mut Self>, on: bool) {
        let snapping = if on { Snapping::On } else { Snapping::Off };
        if let Some(s) = self.as_mut().session() {
            s.set_snapping(snapping);
        }
        self.set_snapping_on(on);
    }

    pub(super) fn set_grid(mut self: Pin<&mut Self>, name: &QString) {
        let Some(grid) = Grid::from_name(&name.to_string()) else {
            return;
        };
        if let Some(s) = self.as_mut().session() {
            s.set_grid(grid);
        }
        self.set_grid_name(q(grid.name()));
    }

    /// A drag on empty ground: the box or the lasso line on screen while the button is down,
    /// the elements it takes selected on release.
    fn drag_box(
        mut self: Pin<&mut Self>,
        (from, to): (Point, Point),
        phase: DragPhase,
        how: Pick,
    ) -> bool {
        let reach = self.rust().reach();
        let sweep = match (reach, self.rust().marquee.clone()) {
            (Reach::Box, _) => Sweep::Box(from, to),
            (Reach::Lasso, Some(Sweep::Lasso(mut path))) => {
                path.push(to);
                Sweep::Lasso(path)
            }
            (Reach::Lasso, _) => Sweep::Lasso(vec![from, to]),
        };

        if phase == DragPhase::Moving {
            self.as_mut().rust_mut().get_mut().marquee = Some(sweep.clone());
            let json = {
                let this = self.as_ref();
                this.rust().session.as_ref().map(|s| {
                    let selected = &this.rust().selected;
                    let list = s.display_list_for(selected.only());

                    plan_json(
                        s,
                        &list,
                        selected,
                        this.rust().language,
                        (&this.rust().picked(), Some(&sweep), None),
                    )
                })
            };
            if let Some(json) = json {
                self.set_plan_json(q(&json));
            }
            return false;
        }

        let found = {
            let this = self.as_ref();
            this.rust()
                .session
                .as_ref()
                .map(|s| match &sweep {
                    Sweep::Box(a, b) => s.pick_in(*a, *b),
                    Sweep::Lasso(path) => s.pick_lasso(path),
                })
                .unwrap_or_default()
        };
        let any = !found.is_empty();
        let selected = self.rust().selected.clone().boxed(found, how);
        self.as_mut().rust_mut().get_mut().marquee = None;
        self.as_mut().rust_mut().get_mut().selected = selected;
        self.refresh();
        any
    }

    pub(super) fn cancel_drag(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().get_mut().grip = Grip::Free;
        // No drag runs any more: a typed measure must not end it as a wall.
        self.as_mut().rust_mut().get_mut().drag_step = None;
        self.as_mut().rust_mut().get_mut().free.caption.clear();
        self.as_mut().rust_mut().get_mut().marquee = None;
        self.refresh();
    }

    /// A click of the area or free form tool: the next point, or the closing click on the first
    /// one.
    fn click_area(mut self: Pin<&mut Self>, at: Point) -> bool {
        let language = self.rust().language;
        let layer = tools::draft_layer(&self.rust().tool);
        let lock = self.rust().session.as_ref().map(|s| s.lock(layer));
        if let Some(Err(e)) = lock.map(|lock| tools::layer_open(layer, lock)) {
            self.as_mut().say(error_text(language, &e));
            return false;
        }

        let draft = self.rust().area.clone();
        let closing = self
            .rust()
            .session
            .as_ref()
            .filter(|s| s.snapping() == Snapping::On)
            .map_or(Closing::Free, |s| Closing::OnGrid(s.grid().step_m()));
        if draft.closes(at, closing) {
            return self.finish_area();
        }

        self.as_mut().rust_mut().get_mut().area = draft.click(at);
        self.refresh();
        true
    }

    /// Sets the plan on screen again, nothing else: for what changes with the pointer.
    pub(super) fn show_plan(self: Pin<&mut Self>) {
        let json = {
            let this = self.as_ref();
            this.rust().session.as_ref().map(|s| {
                let measure = this.rust().measure_view();
                plan_json(
                    s,
                    &s.display_list_for(this.rust().selected.only()),
                    &this.rust().selected,
                    this.rust().language,
                    (
                        &this.rust().picked(),
                        this.rust().marquee.as_ref(),
                        measure.as_ref(),
                    ),
                )
            })
        };

        let Some(json) = json else {
            return;
        };
        self.set_plan_json(q(&json));
    }

    /// Shows what `edit` would do, as the plan; the project stays as it is. A failed preview
    /// shows the project, not the last preview.
    fn preview_edit(mut self: Pin<&mut Self>, edit: &PlanEdit, caption: Option<&Measure>) -> bool {
        let json = {
            let this = self.as_ref();
            this.rust().session.as_ref().and_then(|s| {
                let list = s.preview_for(edit, this.rust().selected.only()).ok()?;
                let picked = this.rust().picked();
                Some(plan_json(
                    s,
                    &list,
                    &this.rust().selected,
                    this.rust().language,
                    (&picked, None, caption),
                ))
            })
        };

        let Some(json) = json else {
            self.refresh();
            return false;
        };
        self.as_mut().rust_mut().get_mut().preview = Preview::Shown;
        self.set_plan_json(q(&json));
        true
    }

    /// Applies what a typed field made of the selection; a refused text goes to the status
    /// line.
    pub(super) fn commit_typed(self: Pin<&mut Self>, typed: Typed) -> bool {
        match typed {
            None => false,
            Some(Ok(edit)) => self.commit_edit(edit),
            Some(Err(e)) => self.report(Some(Err(e))),
        }
    }

    /// Applies `edit`; a bend pulled out of a way is the selection, not its old grip.
    fn commit_pulled(mut self: Pin<&mut Self>, edit: PlanEdit) -> bool {
        let grip = self.rust().selected.only().cloned();
        let count = |this: &Self| match (&this.rust().session, &grip) {
            (Some(s), Some(grip)) => bends(s, grip),
            _ => 0,
        };
        let before = count(&self);
        let done = self.as_mut().commit_edit(edit.clone());
        if done {
            let counts = (before, count(&self));
            let pulled = self.rust().selected.clone().pulled(&edit, counts);
            self.as_mut().rust_mut().get_mut().selected = pulled;
            self.refresh();
        }

        done
    }

    /// Applies `edit`; an error goes to the status line.
    pub(super) fn commit_edit(mut self: Pin<&mut Self>, edit: PlanEdit) -> bool {
        let language = self.rust().language;
        let Some(s) = self.as_mut().session() else {
            return false;
        };

        let result = s.edit_plan(edit);
        let ok = result.is_ok();
        match result {
            Ok(()) => self.as_mut().changed(),
            Err(e) => self.as_mut().say(error_text(language, &e)),
        }
        self.refresh();
        ok
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::selection::Selection;
    use klappe_services::Bend;

    fn grip(bend: Bend) -> Source {
        Source::FigureVia {
            character: "burgvogt".into(),
            index: 1,
            bend,
        }
    }

    /// The selection after a pull of `grip` by (0, `dy_m`), as `commit_pulled` makes it.
    fn pulled(s: &mut Session, grip: Source, dy_m: f64) -> Selection {
        let edit = PlanEdit::Shift {
            source: grip.clone(),
            dx_m: 0.0,
            dy_m,
        };
        let before = bends(s, &grip);
        s.edit_plan(edit.clone()).unwrap();
        let after = bends(s, &grip);

        Selection::of(Some(grip)).pulled(&edit, (before, after))
    }

    /// Burgvogt's way to mark 1 has a bend. A pull of 0.1 um makes no second one, so the
    /// selection stays on the grip and does not jump to the old bend.
    #[test]
    fn a_pull_that_makes_no_bend_keeps_the_selection() {
        let mut s = crate::bridge::tests::example();
        s.set_snapping(Snapping::Off);
        pulled(&mut s, grip(Bend::New(0)), -4.0);
        assert_eq!(bends(&s, &grip(Bend::New(0))), 1);

        let kept = pulled(&mut s, grip(Bend::New(0)), 1e-7);

        assert_eq!(bends(&s, &grip(Bend::New(0))), 1);
        assert_eq!(kept, Selection::of(Some(grip(Bend::New(0)))));
    }

    /// A pull that makes a bend selects it (LH-21).
    #[test]
    fn a_pull_that_makes_a_bend_selects_it() {
        let mut s = crate::bridge::tests::example();
        s.set_snapping(Snapping::Off);

        let made = pulled(&mut s, grip(Bend::New(0)), -4.0);

        assert_eq!(made, Selection::of(Some(grip(Bend::At(0)))));
    }
}
