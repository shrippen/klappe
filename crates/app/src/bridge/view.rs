//! What the controller shows: the properties filled from the session after each call, and the
//! JSON they carry.

use super::qobject;
use super::{q, ControllerRust};
use crate::camera;
use crate::manage;
use crate::notice::backup_view;
use crate::plan;
use crate::plan::Measure;
use crate::tools;
use crate::tools::{
    AreaDraft, Backdrop, MeasureStep, NoteDraft, Pick, Preview, ScaleStep, Selection,
};
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use klappe_services::{
    DisplayList, FigureColor, Layer, Lock, Point, Session, Snapping, Source, Visibility,
};
use serde_json::json;

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
            tools::length_text(length_m, &decimal, &self.language.text("unit.m"))
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
        self.loaded = true;
        self.shown = shown_view(s);
        self.title = q(&s.project().title);
        self.plan_title = q(&s.plan_title());
        self.plan_json = q(&plan_json(
            s,
            &s.display_list(),
            &Selection::default(),
            (&[], None, None),
        ));
        self.has_background = s.has_background();
        self.tracks_json = q(&to_json(&s.tracks()));
        self.shots_json = q(&to_json(&s.shots()));
        self.script_json = q(&to_json(&s.script_lines()));
        self.layers_json = q(&layers_json(s));
        self.locks_json = q(&locks_json(s));
        self.time = s.time_s();
        self.duration = s.duration_s();
        self.selected_shot = q(s.selected_shot().unwrap_or_default());
        self.snapping_on = s.snapping() == Snapping::On;
        self.choices_json = q(&choices_json(s));
        self.project_json = q(&manage::project_json(s));
        self.scene_id = q(s.scene().map(|sc| sc.id.as_str()).unwrap_or_default());

        let (state, message) = backup_view(s.backup());
        self.backup_state = q(state);
        self.backup_message = q(&message);
    }
}

fn to_json<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string(value).unwrap_or_default()
}

/// `list` (the session's, or a preview of an edit) with the selected shot's camera and the
/// selected elements marked; `picked` are the points of a scale in progress, the box the
/// one a drag pulls up.
pub(super) fn plan_json(
    s: &Session,
    list: &DisplayList,
    selected: &Selection,
    (picked, marquee, measure): (&[Point], Option<(Point, Point)>, Option<&Measure>),
) -> String {
    let camera = s
        .selected_shot()
        .and_then(|id| s.plan()?.shots.iter().find(|shot| shot.id == id))
        .map(|shot| shot.camera.clone());
    let marked = |source: &Source| {
        selected.contains(source)
            || matches!(source, Source::Camera(id) if Some(id) == camera.as_ref())
    };

    plan::shapes_json(list, s.dir(), &marked, picked, marquee, measure)
}

/// What the tool lists of the tool bar offer: the project's characters, the object library
/// and the lamp kinds.
pub(super) fn choices_json(s: &Session) -> String {
    let figures: Vec<_> = s
        .project()
        .characters
        .iter()
        .map(|c| json!({ "id": c.id, "name": c.name }))
        .collect();
    let lights: Vec<_> = tools::LIGHT_KINDS.iter().map(|(name, _)| name).collect();

    let colors: Vec<_> = FigureColor::ALL.iter().map(|c| c.name()).collect();
    let dashes: Vec<_> = tools::DRAW_DASHES.iter().map(|(n, _)| n).collect();
    let fills: Vec<_> = tools::DRAW_FILLS.iter().map(|(n, _)| n).collect();

    json!({
        "figure": figures,
        "object": s.library(),
        "camera": camera::presets_json(s.camera_presets()),
        "light": lights,
        "color": colors,
        "dash": dashes,
        "fill": fills,
    })
    .to_string()
}

pub(super) fn layers_json(s: &Session) -> String {
    let shown = |l| s.layer(l) == Visibility::Shown;

    json!({
        "set": shown(Layer::Set),
        "figures": shown(Layer::Figures),
        "cameras": shown(Layer::Cameras),
        "lights": shown(Layer::Lights),
        "notes": shown(Layer::Notes),
        "drawings": shown(Layer::Drawings),
    })
    .to_string()
}

pub(super) fn locks_json(s: &Session) -> String {
    let locked = |l| s.lock(l) == Lock::Locked;

    json!({
        "set": locked(Layer::Set),
        "figures": locked(Layer::Figures),
        "cameras": locked(Layer::Cameras),
        "lights": locked(Layer::Lights),
        "notes": locked(Layer::Notes),
        "drawings": locked(Layer::Drawings),
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
                let grip = matches!(source, Source::FigureVia { .. } | Source::CameraVia { .. });

                shown(&plain, source) || (grip && shown(&s.display_list_for(Some(source)), source))
            })
        };
        self.as_mut().rust_mut().get_mut().selected = kept;
    }

    /// Pushes what may have changed after a call to the properties (and their signals).
    pub(super) fn refresh(mut self: Pin<&mut Self>) {
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

        let values = {
            let this = self.as_ref();
            let Some(s) = this.rust().session.as_ref() else {
                return;
            };
            let measure = this.rust().measure_view();
            (
                plan_json(
                    s,
                    &s.display_list_for(this.rust().selected.only()),
                    &this.rust().selected,
                    (&this.rust().picked(), this.rust().marquee, measure.as_ref()),
                ),
                to_json(&s.tracks()),
                to_json(&s.shots()),
                to_json(&s.script_lines()),
                layers_json(s),
                locks_json(s),
                s.time_s(),
                s.selected_shot().unwrap_or_default().to_owned(),
                s.can_undo(),
                s.can_redo(),
                s.is_dirty(),
                this.rust().selected.only().map(plan::key),
                this.rust().selected.len(),
            )
        };
        let (
            plan,
            tracks,
            shots,
            script,
            layers,
            locks,
            time,
            shot,
            undo,
            redo,
            dirty,
            selected,
            count,
        ) = values;
        let scale_points = self.rust().scale.points().len();
        let area_points = self.rust().area.points().len();
        let asks_note = self.rust().note != NoteDraft::Idle;
        let measuring = self.rust().measure != MeasureStep::Idle;

        // Restoring a backup or undoing it can change what only the start used to set.
        let header = {
            let this = self.as_ref();
            this.rust().session.as_ref().map(|s| {
                (
                    s.project().title.clone(),
                    s.plan_title(),
                    s.duration_s(),
                    choices_json(s),
                    manage::project_json(s),
                    s.scene().map(|sc| sc.id.clone()).unwrap_or_default(),
                )
            })
        };
        if let Some((title, plan_title, duration, choices, project, scene)) = header {
            self.as_mut().set_project_json(q(&project));
            self.as_mut().set_scene_id(q(&scene));
            self.as_mut().set_title(q(&title));
            self.as_mut().set_plan_title(q(&plan_title));
            self.as_mut().set_duration(duration);
            self.as_mut().set_choices_json(q(&choices));
        }

        // The plan set here is the project's: a preview is gone.
        self.as_mut().rust_mut().get_mut().preview = Preview::Hidden;
        self.as_mut().set_plan_json(q(&plan));
        self.as_mut().set_tracks_json(q(&tracks));
        self.as_mut().set_shots_json(q(&shots));
        self.as_mut().set_script_json(q(&script));
        self.as_mut().set_layers_json(q(&layers));
        self.as_mut().set_locks_json(q(&locks));
        self.as_mut().set_time(time);
        self.as_mut().set_selected_shot(q(&shot));
        self.as_mut()
            .set_selected_key(q(&selected.unwrap_or_default()));
        self.as_mut()
            .set_selected_count(i32::try_from(count).unwrap_or_default());
        self.as_mut().set_can_undo(undo);
        self.as_mut().set_can_redo(redo);
        self.as_mut().set_dirty(dirty);
        self.as_mut().set_has_background(background);
        self.as_mut()
            .set_scale_points(i32::try_from(scale_points).unwrap_or_default());
        self.as_mut()
            .set_area_points(i32::try_from(area_points).unwrap_or_default());
        self.as_mut().set_asks_note(asks_note);
        self.as_mut().set_measuring(measuring);
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
mod tests {
    use super::*;
    use crate::bridge::tests::example;

    /// The object tool lists the library: sizes in metres, built in first.
    #[test]
    fn the_object_list_is_the_library() {
        let s = example();

        let choices: serde_json::Value = serde_json::from_str(&choices_json(&s)).unwrap();
        let objects = choices["object"].as_array().unwrap();

        assert_eq!(objects[0]["kind"], "table");
        assert_eq!(objects[0]["width_m"], 1.6);
        assert_eq!(objects[0]["depth_m"], 0.9);
        assert_eq!(objects[0]["shape"], "rect");
        assert_eq!(objects[0]["custom"], false);
        assert_eq!(objects[0]["height_m"], 0.75);
        assert!(objects
            .iter()
            .any(|o| o["kind"] == "horse" && o["shape"] == "ellipse"));
        assert_eq!(objects.len(), s.library().len());
    }

    #[test]
    fn shift_makes_a_pick_extend_the_selection() {
        let mut c = ControllerRust::default();
        assert_eq!(c.pick_mode(), Pick::Replace);

        c.extend_selection = true;

        assert_eq!(c.pick_mode(), Pick::Extend);
    }

    /// Review of PR 23: area, scale and note drafts outlived a change of scene or project;
    /// Enter then made an area from the old plan's points in the new one.
    #[test]
    fn drafts_go_with_the_plan() {
        let mut c = ControllerRust {
            area: AreaDraft::default().click(Point::new(1.0, 1.0)),
            scale: ScaleStep::Idle.click(Point::new(2.0, 2.0)),
            measure: MeasureStep::Idle.click(Point::new(4.0, 4.0)),
            note: NoteDraft::New(Point::new(3.0, 3.0)),
            selected: Selection::of(Some(Source::Wall(0))),
            ..ControllerRust::default()
        };

        c.drop_drafts();

        assert_eq!(c.measure, MeasureStep::Idle);
        assert!(c.area.is_empty());
        assert_eq!(c.scale, ScaleStep::Idle);
        assert_eq!(c.note, NoteDraft::Idle);
        assert!(c.selected.is_empty());
    }

    /// Review of PR 23, round 4: after a start with `--open` the view on screen was not
    /// noted, so the first refresh dropped the selection or draft it was to show.
    #[test]
    fn the_first_refresh_keeps_its_draft() {
        let dir =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe");
        let mut c = ControllerRust {
            session: Some(Session::open(&dir).unwrap()),
            ..ControllerRust::default()
        };
        c.fill();
        c.area = AreaDraft::default().click(Point::new(1.0, 1.0));

        let shown = shown_view(c.session.as_ref().unwrap());
        c.follow_view(shown);

        assert!(!c.area.is_empty());
    }

    /// LH-04: choosing, adding or copying a plan changes the plan on screen; a draft made on
    /// the old plan must not be finished on the new one.
    #[test]
    fn drafts_go_when_another_plan_is_shown() {
        let dir =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe");
        let mut session = Session::open(&dir).unwrap();
        let mut c = ControllerRust {
            shown: shown_view(&session),
            area: AreaDraft::default().click(Point::new(1.0, 1.0)),
            ..ControllerRust::default()
        };
        c.follow_view(shown_view(&session));
        assert!(!c.area.is_empty());

        session.copy_plan("Stand 1").unwrap();
        c.follow_view(shown_view(&session));

        assert!(c.area.is_empty());
        assert_eq!(c.shown, "szene-012/stand-1");
    }

    /// Review of PR 23, round 3: undo, redo or deleting the scene on view changed the scene
    /// without a click; the drafts stayed and Enter drew in the other scene. Here only the
    /// comparison; `refresh` calls it with `shown_view`.
    #[test]
    fn drafts_go_when_the_view_changes() {
        let mut c = ControllerRust {
            shown: "szene-012/a".into(),
            area: AreaDraft::default().click(Point::new(1.0, 1.0)),
            ..ControllerRust::default()
        };

        c.follow_view("szene-012/a".into());
        assert!(!c.area.is_empty());

        c.follow_view("szene-013/a".into());
        assert!(c.area.is_empty());
        assert_eq!(c.shown, "szene-013/a");
    }

    /// Review of PR 47, round 2: undo across a plan, or removing the plan on view, showed
    /// another plan while play went on. Every change of the view is a switch.
    #[test]
    fn undo_across_a_plan_is_a_switch() {
        let dir =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe");
        let mut session = Session::open(&dir).unwrap();
        session.copy_plan("Stand 1").unwrap();
        let mut c = ControllerRust {
            shown: shown_view(&session),
            ..ControllerRust::default()
        };

        assert_eq!(c.follow_view(shown_view(&session)), View::Same);
        session.undo();
        assert_eq!(c.follow_view(shown_view(&session)), View::Switched);
    }
}
