//! What the controller shows: the properties filled from the session after each call, and the
//! JSON they carry.

use super::qobject;
use super::{q, ControllerRust};
use crate::camera;
use crate::manage;
use crate::notice::backup_view;
use crate::plan;
use crate::plan::Measure;
use crate::selection::{Pick, Selection};
use crate::tools;
use crate::tools::MEASURE_SEPARATOR;
use crate::tools::{AreaDraft, Backdrop, MeasureStep, NoteDraft, Preview, ScaleStep};
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use klappe_services::{
    Arrival, Dash, Departure, DisplayList, FigureColor, Fill, Grid, Layer, LightKind, Lock,
    Movement, Point, Pose, Rig, Session, Snapping, Source, Visibility,
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
        self.script_json = v.script_json;
        self.layers_json = v.layers_json;
        self.locks_json = v.locks_json;
        self.time = v.time;
        self.duration = v.duration;
        self.selected_shot = v.selected_shot;
        self.can_undo = v.can_undo;
        self.can_redo = v.can_redo;
        self.dirty = v.dirty;
        self.selected_key = v.selected_key;
        self.selected_count = v.selected_count;
        self.has_background = v.has_background;
        self.scale_points = v.scale_points;
        self.area_points = v.area_points;
        self.asks_note = v.asks_note;
        self.measuring = v.measuring;
        self.choices_json = v.choices_json;
        self.project_json = v.project_json;
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
                &s.display_list_for(self.selected.only()),
                &self.selected,
                (&self.picked(), self.marquee, measure.as_ref()),
            )),
            tracks_json: q(&to_json(&s.tracks())),
            shots_json: q(&to_json(&s.shots())),
            script_json: q(&to_json(&s.script_lines())),
            layers_json: q(&layers_json(s)),
            locks_json: q(&locks_json(s)),
            time: s.time_s(),
            duration: s.duration_s(),
            selected_shot: q(s.selected_shot().unwrap_or_default()),
            can_undo: s.can_undo(),
            can_redo: s.can_redo(),
            dirty: s.is_dirty(),
            selected_key: q(&self.selected.only().map(plan::key).unwrap_or_default()),
            selected_count: i32::try_from(self.selected.len()).unwrap_or_default(),
            has_background: s.has_background(),
            scale_points: i32::try_from(self.scale.points().len()).unwrap_or_default(),
            area_points: i32::try_from(self.area.points().len()).unwrap_or_default(),
            asks_note: self.note != NoteDraft::Idle,
            measuring: self.measure != MeasureStep::Idle,
            choices_json: q(&choices_json(s)),
            grid_name: q(s.grid().name()),
            project_json: q(&manage::project_json(s)),
            scene_id: q(s.scene().map(|sc| sc.id.as_str()).unwrap_or_default()),
        })
    }
}

/// What the window shows, one field per property that follows the session; a new one needs
/// its property in `bridge.rs`, a field here and a line in `collect`, `fill` and `show`.
struct Shown {
    title: QString,
    plan_title: QString,
    plan_json: QString,
    tracks_json: QString,
    shots_json: QString,
    script_json: QString,
    layers_json: QString,
    locks_json: QString,
    time: f64,
    duration: f64,
    selected_shot: QString,
    can_undo: bool,
    can_redo: bool,
    dirty: bool,
    selected_key: QString,
    selected_count: i32,
    has_background: bool,
    scale_points: i32,
    area_points: i32,
    asks_note: bool,
    measuring: bool,
    choices_json: QString,
    grid_name: QString,
    project_json: QString,
    scene_id: QString,
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
    let lights: Vec<_> = LightKind::ALL.iter().map(|k| k.name()).collect();

    let colors: Vec<_> = FigureColor::ALL.iter().map(|c| c.name()).collect();
    let dashes: Vec<_> = Dash::ALL.iter().map(|d| d.name()).collect();
    let fills: Vec<_> = Fill::ALL.iter().map(|f| f.name()).collect();
    let poses: Vec<_> = Pose::ALL.iter().map(|p| p.name()).collect();
    let arrivals: Vec<_> = Arrival::ALL.iter().map(|a| a.name()).collect();
    let departures: Vec<_> = Departure::ALL.iter().map(|d| d.name()).collect();
    let rigs: Vec<_> = Rig::ALL.iter().map(|r| r.name()).collect();
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
        "light": lights,
        "color": colors,
        "dash": dashes,
        "fill": fills,
        "pose": poses,
        "arrival": arrivals,
        "departure": departures,
        "rig": rigs,
        "movement": movements,
        "layer": layers,
        "grid": grids,
    })
    .to_string()
}

fn layers_json(s: &Session) -> String {
    layer_flags(|l| s.layer(l) == Visibility::Shown)
}

fn locks_json(s: &Session) -> String {
    layer_flags(|l| s.lock(l) == Lock::Locked)
}

/// One switch per layer, by name: what `layersJson` and `locksJson` hand to QML.
fn layer_flags(flag: impl Fn(Layer) -> bool) -> String {
    let flags: serde_json::Map<_, _> = Layer::ALL
        .iter()
        .map(|l| (l.name().to_owned(), json!(flag(*l))))
        .collect();

    json!(flags).to_string()
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

        let Some(values) = self.rust().collect() else {
            return;
        };

        // The plan set here is the project's: a preview is gone.
        self.as_mut().rust_mut().get_mut().preview = Preview::Hidden;
        self.show(values);
    }

    /// Sets the properties from `v`; each signals only when its value changed.
    fn show(mut self: Pin<&mut Self>, v: Shown) {
        // Restoring a backup or undoing it can change what only the start used to set.
        self.as_mut().set_project_json(v.project_json);
        self.as_mut().set_scene_id(v.scene_id);
        self.as_mut().set_title(v.title);
        self.as_mut().set_plan_title(v.plan_title);
        self.as_mut().set_duration(v.duration);
        self.as_mut().set_choices_json(v.choices_json);
        self.as_mut().set_grid_name(v.grid_name);

        self.as_mut().set_plan_json(v.plan_json);
        self.as_mut().set_tracks_json(v.tracks_json);
        self.as_mut().set_shots_json(v.shots_json);
        self.as_mut().set_script_json(v.script_json);
        self.as_mut().set_layers_json(v.layers_json);
        self.as_mut().set_locks_json(v.locks_json);
        self.as_mut().set_time(v.time);
        self.as_mut().set_selected_shot(v.selected_shot);
        self.as_mut().set_selected_key(v.selected_key);
        self.as_mut().set_selected_count(v.selected_count);
        self.as_mut().set_can_undo(v.can_undo);
        self.as_mut().set_can_redo(v.can_redo);
        self.as_mut().set_dirty(v.dirty);
        self.as_mut().set_has_background(v.has_background);
        self.as_mut().set_scale_points(v.scale_points);
        self.as_mut().set_area_points(v.area_points);
        self.as_mut().set_asks_note(v.asks_note);
        self.as_mut().set_measuring(v.measuring);
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

    /// The tool bar's lists, by the names the file and the catalogue use.
    #[test]
    fn the_tool_lists_are_the_file_names() {
        let s = example();

        let choices: serde_json::Value = serde_json::from_str(&choices_json(&s)).unwrap();

        assert_eq!(
            choices["light"],
            json!([
                "fresnel",
                "led-panel",
                "tube",
                "balloon",
                "reflector",
                "flag"
            ])
        );
        assert_eq!(choices["dash"], json!(["solid", "dashed", "dotted"]));
        assert_eq!(choices["fill"], json!(["outline", "filled"]));
        assert_eq!(
            choices["rig"],
            json!(["tripod", "dolly", "track", "crane", "handheld", "gimbal"])
        );
        assert_eq!(choices["movement"][0], "static");
        assert_eq!(
            choices["movement"].as_array().unwrap().len(),
            Movement::ALL.len()
        );
        assert_eq!(
            choices["color"],
            json!(["red", "orange", "yellow", "green", "cyan", "blue", "purple"])
        );
    }

    /// QML keeps one switch per layer: `layersJson` and `locksJson` map the layer name to it.
    #[test]
    fn layers_and_locks_are_listed_by_name() {
        let mut s = example();
        s.set_layer(Layer::Notes, Visibility::Hidden);
        s.set_lock(Layer::Cameras, Lock::Locked);

        let shown: serde_json::Value = serde_json::from_str(&layers_json(&s)).unwrap();
        let locked: serde_json::Value = serde_json::from_str(&locks_json(&s)).unwrap();

        let names = ["set", "figures", "cameras", "lights", "notes", "drawings"];
        assert_eq!(shown.as_object().unwrap().len(), names.len());
        assert_eq!(locked.as_object().unwrap().len(), names.len());
        for name in names {
            assert_eq!(shown[name], name != "notes", "{name}");
            assert_eq!(locked[name], name == "cameras", "{name}");
        }
    }

    #[test]
    fn shift_makes_a_pick_extend_the_selection() {
        let mut c = ControllerRust::default();
        assert_eq!(c.pick_mode(), Pick::Replace);

        c.extend_selection = true;

        assert_eq!(c.pick_mode(), Pick::Extend);
    }

    /// A measurement shows length and direction; one point alone shows nothing to read.
    #[test]
    fn the_measurement_caption_has_length_and_angle() {
        let mut c = ControllerRust::default();
        let (a, b) = (Point::new(0.0, 0.0), Point::new(3.0, 3.0));

        c.measure = MeasureStep::Start(a);
        assert_eq!(c.measure_view().unwrap().caption, "");

        c.measure = MeasureStep::Held(a, b);
        assert_eq!(c.measure_view().unwrap().caption, "4,24 m · 45°");
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

    /// The start with a project shows what the first refresh would: every value from the
    /// session, none of the tool or draft state.
    #[test]
    fn the_start_fills_every_value() {
        let mut c = ControllerRust {
            session: Some(example()),
            ..ControllerRust::default()
        };
        c.fill();

        assert!(c.loaded);
        assert_eq!(c.title.to_string(), "Die Wartburg-Szene");
        assert_eq!(
            c.plan_title.to_string(),
            "12 · INT. SÄNGERSAAL – TAG · Plan A"
        );
        assert_eq!(c.scene_id.to_string(), "szene-012");
        assert_eq!(c.duration, 20.0);
        assert_eq!(c.time, 0.0);
        assert_eq!(c.selected_shot.to_string(), "");
        assert!(c.snapping_on);
        assert_eq!(c.grid_name.to_string(), "cm25");
        assert!(!c.has_background);
        assert_eq!(c.backup_state.to_string(), "none");
        assert_eq!(c.backup_message.to_string(), "");
        assert!(!c.can_undo && !c.can_redo && !c.dirty);
        assert_eq!(c.selected_key.to_string(), "");
        assert_eq!(c.selected_count, 0);
        assert_eq!((c.scale_points, c.area_points), (0, 0));
        assert!(!c.asks_note && !c.measuring);
        for json in [
            &c.plan_json,
            &c.tracks_json,
            &c.shots_json,
            &c.script_json,
            &c.layers_json,
            &c.locks_json,
            &c.choices_json,
            &c.project_json,
        ] {
            assert!(!json.to_string().is_empty());
        }
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
