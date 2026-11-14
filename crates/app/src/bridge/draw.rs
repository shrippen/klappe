//! The drawing tools: picking a tool, dragging, placing and committing what they make.

use super::q;
use super::qobject;
use super::view::plan_json;
use crate::notice::error_text;
use crate::tools;
use crate::tools::{
    AreaDraft, DragPhase, DragStep, DrawStyle, Grip, MeasureStep, NoteDraft, Pick, Preview,
    ScaleStep, Tool, Typed,
};
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use klappe_services::{Layer, PlanEdit, Point, Snapping};

impl qobject::Controller {
    pub(super) fn set_tool(mut self: Pin<&mut Self>, tool: &QString, choice: &QString) -> bool {
        let Some(tool) = Tool::parse(&tool.to_string(), &choice.to_string()) else {
            return false;
        };

        // Points picked for a scale do not outlast the tool.
        let leaves_scale = tool != Tool::Scale && self.rust().scale != ScaleStep::Idle;
        let leaves_measure = tool != Tool::Measure && self.rust().measure != MeasureStep::Idle;
        let leaves_note = tool != Tool::Note && self.rust().note != NoteDraft::Idle;
        let leaves_area = tool != Tool::Area && !self.rust().area.is_empty();
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
        // background moves freely); the other tools draw between two points, which snap here.
        let (from, to) = match tool {
            Tool::Select | Tool::Scale => (from, to),
            _ => (s.snap(from), s.snap(to)),
        };
        let group = grabbed
            .as_ref()
            .filter(|_| tool == Tool::Select)
            .and_then(|g| tools::group_drag_edit(&selected, g, from, to));
        let mut edit = group
            .or_else(|| tools::drag_edit(&tool, grabbed.as_ref(), from, to))
            .or_else(|| tools::draw_edit(&tool, style, from, to));
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

        // Empty ground with the select tool pulls up a box to take what lies in it.
        if tool == Tool::Select && grabbed.is_none() {
            return self.drag_box((from, to), phase, how);
        }

        if tool == Tool::Select && edit.is_some() {
            self.as_mut().rust_mut().get_mut().selected = selected.grab(grabbed);
        }
        match tools::drag_step(edit, phase, preview) {
            DragStep::Preview(edit) => self.preview_edit(&edit),
            DragStep::Commit(edit) => self.commit_edit(edit),
            DragStep::Restore => {
                self.refresh();
                false
            }
            DragStep::Nothing => false,
        }
    }

    pub(super) fn place(mut self: Pin<&mut Self>, x: f64, y: f64) -> bool {
        let tool = self.rust().tool.clone();
        let selected = self.rust().selected.clone();
        let how = self.rust().pick_mode();
        let Some(s) = self.as_mut().session() else {
            return false;
        };

        if tool == Tool::Select {
            let picked = s.pick_for(Point::new(x, y), selected.only());
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
        if tool == Tool::Area {
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
        let Some(edit) = tools::area_edit(&self.rust().area, style) else {
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

    /// A drag on empty ground: the box on screen while the button is down, the elements wholly
    /// in it selected on release.
    fn drag_box(
        mut self: Pin<&mut Self>,
        corners: (Point, Point),
        phase: DragPhase,
        how: Pick,
    ) -> bool {
        if phase == DragPhase::Moving {
            self.as_mut().rust_mut().get_mut().marquee = Some(corners);
            let json = {
                let this = self.as_ref();
                this.rust().session.as_ref().map(|s| {
                    let selected = &this.rust().selected;
                    let list = s.display_list_for(selected.only());

                    plan_json(
                        s,
                        &list,
                        selected,
                        (&this.rust().picked(), Some(corners), None),
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
                .map(|s| s.pick_in(corners.0, corners.1))
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
        self.as_mut().rust_mut().get_mut().marquee = None;
        self.refresh();
    }

    /// A click of the area tool: the next point, or the closing click on the first one.
    fn click_area(mut self: Pin<&mut Self>, at: Point) -> bool {
        let language = self.rust().language;
        let lock = self
            .rust()
            .session
            .as_ref()
            .map(|s| s.lock(Layer::Drawings));
        if let Some(Err(e)) = lock.map(tools::drawing_open) {
            self.as_mut().say(error_text(language, &e));
            return false;
        }

        let draft = self.rust().area.clone();
        if draft.closes(at) {
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
                    (&this.rust().picked(), this.rust().marquee, measure.as_ref()),
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
    fn preview_edit(mut self: Pin<&mut Self>, edit: &PlanEdit) -> bool {
        let json = {
            let this = self.as_ref();
            this.rust().session.as_ref().and_then(|s| {
                let list = s.preview_for(edit, this.rust().selected.only()).ok()?;
                let picked = this.rust().picked();
                Some(plan_json(
                    s,
                    &list,
                    &this.rust().selected,
                    (&picked, None, None),
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
