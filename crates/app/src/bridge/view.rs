//! What the controller shows: the properties filled from the session after each call, and the
//! JSON they carry.

use super::drehbuch::outline_json;
use super::layers::{fades_json, layers_json, locks_json};
use super::qobject;
use super::{q, ControllerRust};
use crate::arm::Armed;
use crate::camera;
use crate::manage;
use crate::notice::backup_view;
use crate::plan;
use crate::plan::{Measure, Sweep};
use crate::selection::{Pick, Reach, Selection};
use crate::tools;
use crate::tools::MEASURE_SEPARATOR;
use crate::tools::{AreaDraft, Backdrop, MeasureStep, NoteDraft, Preview, ScaleStep};
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use klappe_services::{
    Angle, Arrival, Curve, Dash, Departure, DisplayList, Fade, FigureColor, Fill, Grid, Language,
    Layer, LightKind, Movement, Point, Pose, Rig, Session, ShotOrder, ShotSize, Snapping, Source,
};
use serde_json::json;

/// What of `Session::edits` the `editRevision` property takes: the bits of a positive `i32`.
const EDIT_MASK: u64 = i32::MAX as u64;

/// Whether `refresh` found another scene or plan on view than before.
#[derive(Debug, PartialEq, Eq)]
enum View {
    Same,
    Switched,
}

impl ControllerRust {
    /// Forgets what was picked and half drawn: it belongs to the plan on screen, not to the
    /// next scene or project.
    pub(super) fn drop_drafts(&mut self) {
        self.selected = Selection::default();
        self.marquee = None;
        self.scale = ScaleStep::Idle;
        self.measure = MeasureStep::Idle;
        self.note = NoteDraft::Idle;
        self.arm = Armed::Idle;
        self.area = AreaDraft::default();
    }

    /// Notes the scene and plan on screen; when they changed, the drafts made on the old
    /// one go (undo, redo and project edits can change the view, not only a click).
    fn follow_view(&mut self, shown: String) -> View {
        if shown == self.shown {
            return View::Same;
        }

        self.drop_drafts();
        self.shown = shown;
        View::Switched
    }

    /// What a pick does to the selection: Shift held adds to it.
    pub(super) fn pick_mode(&self) -> Pick {
        if self.extend_selection {
            return Pick::Extend;
        }

        Pick::Replace
    }

    /// What a drag on empty ground pulls up: Ctrl held draws a lasso.
    pub(super) fn reach(&self) -> Reach {
        if self.lasso_selection {
            return Reach::Lasso;
        }

        Reach::Box
    }

    /// What the plan on screen is made of, to tell what an edit added.
    pub(super) fn shown_sources(&self) -> Vec<Source> {
        let Some(s) = &self.session else {
            return Vec::new();
        };

        s.display_list()
            .items
            .into_iter()
            .filter_map(|item| item.source)
            .collect()
    }

    /// The points picked for a scale or an area: a preview on top of the plan.
    pub(super) fn picked(&self) -> Vec<Point> {
        let mut points = self.scale.points();
        points.extend_from_slice(self.area.points());

        points
    }

    /// The measurement to draw: its points, and the length once there are two.
    pub(super) fn measure_view(&self) -> Option<Measure> {
        let points = self.measure.points();
        if points.is_empty() {
            return None;
        }

        let caption = self.measure.length_m().map(|length_m| {
            let decimal = self.language.text("unit.decimal");
            let length = tools::length_text(length_m, &decimal, &self.language.text("unit.m"));
            let Some(angle_deg) = self.measure.angle_deg() else {
                return length;
            };

            let angle = tools::angle_text(angle_deg, &decimal, &self.language.text("unit.deg"));
            format!("{length}{MEASURE_SEPARATOR}{angle}")
        });

        Some(Measure {
            points,
            caption: caption.unwrap_or_default(),
        })
    }

    /// Every property from the session, at start (no signals needed yet).
    pub(super) fn fill(&mut self) {
        let Some(s) = &self.session else {
            return;
        };
        let view = shown_view(s);
        let snapping_on = s.snapping() == Snapping::On;
        let (backup_state, backup_message) = backup_view(s.backup());
        let Some(v) = self.collect() else {
            return;
        };

        self.loaded = true;
        self.shown = view;
        self.snapping_on = snapping_on;
        self.grid_name = v.grid_name;
        self.backup_state = q(backup_state);
        self.backup_message = q(&backup_message);
        self.title = v.title;
        self.plan_title = v.plan_title;
        self.plan_json = v.plan_json;
        self.tracks_json = v.tracks_json;
        self.shots_json = v.shots_json;
        self.shots_shooting_json = v.shots_shooting_json;
        self.script_json = v.script_json;
        self.script_text = v.script_text;
        self.outline_json = v.outline_json;
        self.layers_json = v.layers_json;
        self.locks_json = v.locks_json;
        self.fades_json = v.fades_json;
        self.time = v.time;
        self.duration = v.duration;
        self.selected_shot = v.selected_shot;
        self.can_undo = v.can_undo;
        self.can_redo = v.can_redo;
        self.undo_step = v.undo_step;
        self.redo_step = v.redo_step;
        self.dirty = v.dirty;
        self.selected_key = v.selected_key;
        self.selected_count = v.selected_count;
        self.has_background = v.has_background;
        self.missing_heights = v.missing_heights;
        self.scale_points = v.scale_points;
        self.area_points = v.area_points;
        self.asks_note = v.asks_note;
        self.armed = v.armed;
        self.measuring = v.measuring;
        self.view_free = v.view_free;
        self.choices_json = v.choices_json;
        self.project_json = v.project_json;
        self.edit_revision = v.edit_revision;
        self.scene_id = v.scene_id;
    }

    /// Every value the window shows, from the session and the tool state; `None` without a
    /// session. Start and refresh both set the properties from it.
    fn collect(&self) -> Option<Shown> {
        let s = self.session.as_ref()?;
        let measure = self.measure_view();

        Some(Shown {
            title: q(&s.project().title),
            plan_title: q(&s.plan_title()),
            plan_json: q(&plan_json(
                s,
                &s.display_list_for(self.selected.handled()),
                &self.selected,
                self.language,
                (&self.picked(), self.marquee.as_ref(), measure.as_ref()),
            )),
            tracks_json: q(&to_json(&s.tracks())),
            shots_json: q(&to_json(&s.shots())),
            shots_shooting_json: q(&to_json(&s.shots_in(ShotOrder::Shooting))),
            script_json: q(&to_json(&s.script_lines())),
            script_text: q(s.script_text()),
            outline_json: q(&outline_json(s.script_text())),
            layers_json: q(&layers_json(s)),
            locks_json: q(&locks_json(s)),
            fades_json: q(&fades_json(s)),
            time: s.time_s(),
            duration: s.duration_s(),
            selected_shot: q(s.selected_shot().unwrap_or_default()),
            can_undo: s.can_undo(),
            can_redo: s.can_redo(),
            undo_step: q(s.undo_step().unwrap_or_default()),
            redo_step: q(s.redo_step().unwrap_or_default()),
            dirty: s.is_dirty(),
            selected_key: q(&self.selected.only().map(plan::key).unwrap_or_default()),
            selected_count: i32::try_from(self.selected.len()).unwrap_or_default(),
            has_background: s.has_background(),
            missing_heights: i32::try_from(s.missing_heights()).unwrap_or_default(),
            scale_points: i32::try_from(self.scale.points().len()).unwrap_or_default(),
            area_points: i32::try_from(self.area.points().len()).unwrap_or_default(),
            asks_note: self.note != NoteDraft::Idle,
            armed: q(self.arm.name()),
            measuring: self.measure != MeasureStep::Idle,
            view_free: s.view() == klappe_services::View::Free,
            choices_json: q(&choices_json(s)),
            grid_name: q(s.grid().name()),
            project_json: q(&manage::project_json(s)),
            // A name of the state, not a number to compare: it wraps.
            edit_revision: i32::try_from(s.edits() & EDIT_MASK).unwrap_or_default(),
            scene_id: q(s.scene().map(|sc| sc.id.as_str()).unwrap_or_default()),
        })
    }
}

/// What the window shows, one field per property that follows the session; a new one needs
/// its property in `bridge.rs`, a field here and a line in `collect`, `fill` and `show`.
/// The default is what the window shows without a project (closing feeds it to `show`).
#[derive(Default)]
pub(super) struct Shown {
    title: QString,
    plan_title: QString,
    plan_json: QString,
    tracks_json: QString,
    shots_json: QString,
    shots_shooting_json: QString,
    script_json: QString,
    script_text: QString,
    outline_json: QString,
    layers_json: QString,
    locks_json: QString,
    fades_json: QString,
    time: f64,
    duration: f64,
    selected_shot: QString,
    can_undo: bool,
    can_redo: bool,
    undo_step: QString,
    redo_step: QString,
    dirty: bool,
    selected_key: QString,
    selected_count: i32,
    has_background: bool,
    missing_heights: i32,
    scale_points: i32,
    area_points: i32,
    asks_note: bool,
    armed: QString,
    measuring: bool,
    view_free: bool,
    choices_json: QString,
    grid_name: QString,
    project_json: QString,
    edit_revision: i32,
    scene_id: QString,
}

pub(super) fn to_json<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string(value).unwrap_or_default()
}

/// `list` (the session's, or a preview of an edit) with the selected shot's camera and the
/// selected elements marked; `picked` are the points of a scale in progress, the box the
/// one a drag pulls up.
pub(super) fn plan_json(
    s: &Session,
    list: &DisplayList,
    selected: &Selection,
    language: Language,
    (picked, marquee, measure): (&[Point], Option<&Sweep>, Option<&Measure>),
) -> String {
    let camera = s
        .selected_shot()
        .and_then(|id| s.plan()?.shots.iter().find(|shot| shot.id == id))
        .map(|shot| shot.camera.clone());
    let marked = |source: &Source| {
        selected.contains(source)
            || matches!(source, Source::Camera(id) if Some(id) == camera.as_ref())
    };

    let heights = plan::HeightTexts::of(language);

    let fades = s.item_fades(list);
    let drawn = plan::Faded::new(list, &fades);

    plan::shapes_json(drawn, s.dir(), &marked, picked, marquee, measure, &heights)
}

/// What the tool lists of the tool bar offer: the project's characters, the object library,
/// the plan's cameras and the built-in lamps.
pub(super) fn choices_json(s: &Session) -> String {
    let figures: Vec<_> = s
        .project()
        .characters
        .iter()
        .map(|c| json!({ "id": c.id, "name": c.name }))
        .collect();
    let cameras: Vec<_> = s
        .plan()
        .iter()
        .flat_map(|p| p.cameras.iter())
        .map(|c| json!({ "id": c.id, "label": c.label }))
        .collect();
    let lights: Vec<_> = s.light_presets().iter().map(|l| l.id.as_str()).collect();

    let colors: Vec<_> = FigureColor::ALL.iter().map(|c| c.name()).collect();
    let dashes: Vec<_> = Dash::ALL.iter().map(|d| d.name()).collect();
    let fills: Vec<_> = Fill::ALL.iter().map(|f| f.name()).collect();
    let poses: Vec<_> = Pose::ALL.iter().map(|p| p.name()).collect();
    let arrivals: Vec<_> = Arrival::ALL.iter().map(|a| a.name()).collect();
    let departures: Vec<_> = Departure::ALL.iter().map(|d| d.name()).collect();
    let rigs: Vec<_> = Rig::ALL.iter().map(|r| r.name()).collect();
    let curves: Vec<_> = Curve::ALL.iter().map(|c| c.name()).collect();
    let sizes: Vec<_> = ShotSize::ALL.iter().map(|z| z.name()).collect();
    let angles: Vec<_> = Angle::ALL.iter().map(|a| a.name()).collect();
    let movements: Vec<_> = Movement::ALL.iter().map(|m| m.name()).collect();
    let layers: Vec<_> = Layer::ALL.iter().map(|l| l.name()).collect();
    let grids: Vec<_> = Grid::ALL
        .iter()
        .map(|g| json!({ "id": g.name(), "step_m": g.step_m() }))
        .collect();

    json!({
        "figure": figures,
        "object": s.library(),
        "camera": camera::presets_json(s.camera_presets()),
        "cameras": cameras,
        "light": lights,
        "lightKind": LightKind::ALL.iter().map(|k| k.name()).collect::<Vec<_>>(),
        "color": colors,
        "dash": dashes,
        "fill": fills,
        "pose": poses,
        "arrival": arrivals,
        "departure": departures,
        "rig": rigs,
        "curve": curves,
        "size": sizes,
        "angle": angles,
        "movement": movements,
        "layer": layers,
        "fade": Fade::ALL.iter().map(|f| f.name()).collect::<Vec<_>>(),
        "grid": grids,
        "axis": s.axis_choice(),
        "template": s.templates(),
    })
    .to_string()
}

impl qobject::Controller {
    pub(super) fn session(self: Pin<&mut Self>) -> Option<&mut Session> {
        self.rust_mut().get_mut().session.as_mut()
    }

    /// Forgets the selected elements that are gone (undo, a layer switched off).
    fn drop_stale(mut self: Pin<&mut Self>) {
        let kept = {
            let this = self.as_ref();
            let Some(s) = &this.rust().session else {
                return;
            };

            // A grip shows only with its own way selected.
            let plain = s.display_list();
            let shown = |list: &DisplayList, source: &Source| {
                list.items.iter().any(|i| i.source.as_ref() == Some(source))
            };
            this.rust().selected.clone().keep(|source| {
                let grip = source.selected_itself();

                shown(&plain, source) || (grip && shown(&s.display_list_for(Some(source)), source))
            })
        };
        self.as_mut().rust_mut().get_mut().selected = kept;
    }

    /// Pushes what may have changed after a call to the properties (and their signals).
    pub(super) fn refresh(mut self: Pin<&mut Self>) {
        // The scene list shows each scene's length: the layout follows text and paper.
        let language = self.rust().language;
        if let Some(s) = self.as_mut().session() {
            s.paginate_if_changed(language);
        }

        let shown = self
            .rust()
            .session
            .as_ref()
            .map(shown_view)
            .unwrap_or_default();
        // Undo, redo or removing the plan on view can show another one: play stops there.
        if self.as_mut().rust_mut().get_mut().follow_view(shown) == View::Switched {
            self.as_mut().switched();
        }
        self.as_mut().drop_stale();

        // Undo and redo can take the image away; its scale points go with it.
        let background = self
            .rust()
            .session
            .as_ref()
            .is_some_and(Session::has_background);
        let backdrop = if background {
            Backdrop::Present
        } else {
            Backdrop::Missing
        };
        let scale = self.rust().scale.under(backdrop);
        self.as_mut().rust_mut().get_mut().scale = scale;

        let Some(values) = self.rust().collect() else {
            return;
        };

        // The plan set here is the project's: a preview is gone.
        self.as_mut().rust_mut().get_mut().preview = Preview::Hidden;
        self.show(values);
    }

    /// Sets the properties from `v`; each signals only when its value changed.
    pub(super) fn show(mut self: Pin<&mut Self>, v: Shown) {
        // Restoring a backup or undoing it can change what only the start used to set.
        self.as_mut().set_project_json(v.project_json);
        self.as_mut().set_edit_revision(v.edit_revision);
        self.as_mut().set_scene_id(v.scene_id);
        self.as_mut().set_title(v.title);
        self.as_mut().set_plan_title(v.plan_title);
        self.as_mut().set_duration(v.duration);
        self.as_mut().set_choices_json(v.choices_json);
        self.as_mut().set_grid_name(v.grid_name);

        self.as_mut().set_plan_json(v.plan_json);
        self.as_mut().set_tracks_json(v.tracks_json);
        self.as_mut().set_shots_json(v.shots_json);
        self.as_mut().set_shots_shooting_json(v.shots_shooting_json);
        self.as_mut().set_script_json(v.script_json);
        self.as_mut().set_script_text(v.script_text);
        self.as_mut().set_outline_json(v.outline_json);
        self.as_mut().set_layers_json(v.layers_json);
        self.as_mut().set_locks_json(v.locks_json);
        self.as_mut().set_fades_json(v.fades_json);
        self.as_mut().set_time(v.time);
        self.as_mut().set_selected_shot(v.selected_shot);
        self.as_mut().set_selected_key(v.selected_key);
        self.as_mut().set_selected_count(v.selected_count);
        self.as_mut().set_can_undo(v.can_undo);
        self.as_mut().set_can_redo(v.can_redo);
        self.as_mut().set_undo_step(v.undo_step);
        self.as_mut().set_redo_step(v.redo_step);
        self.as_mut().set_dirty(v.dirty);
        self.as_mut().set_has_background(v.has_background);
        self.as_mut().set_missing_heights(v.missing_heights);
        self.as_mut().set_scale_points(v.scale_points);
        self.as_mut().set_area_points(v.area_points);
        self.as_mut().set_asks_note(v.asks_note);
        self.as_mut().set_armed(v.armed);
        self.as_mut().set_measuring(v.measuring);
        self.as_mut().set_view_free(v.view_free);

        // The picture follows anything the session did.
        let revision = self.picture_revision().wrapping_add(1);
        self.as_mut().set_picture_revision(revision);
    }

    /// Shows `text` in the header until the project changes.
    pub(super) fn say(mut self: Pin<&mut Self>, text: String) {
        self.as_mut().rust_mut().get_mut().notice.say(text.clone());
        self.set_status(q(&text));
    }

    /// The project changed: a "saved" or error message no longer fits.
    pub(super) fn changed(mut self: Pin<&mut Self>) {
        if self.as_mut().rust_mut().get_mut().notice.project_changed() {
            self.set_status(QString::default());
        }
    }
}

/// The scene and plan on screen as `scene/plan`, empty parts when there is none.
fn shown_view(s: &Session) -> String {
    let scene = s.scene().map(|sc| sc.id.as_str()).unwrap_or_default();
    let plan = s.plan().map(|p| p.id.as_str()).unwrap_or_default();

    format!("{scene}/{plan}")
}

#[cfg(test)]
mod tests;
