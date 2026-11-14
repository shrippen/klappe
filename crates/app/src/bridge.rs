//! The QML-facing controller. Thin: every call goes to `klappe_services::Session`, every result
//! comes back as a property (JSON for lists). Property names are camelCase in QML.
//!
//! cxx-qt generates C++ glue with `unsafe` blocks; the workspace lint is relaxed for this file
//! only (agent.md allows `unsafe` in the cxx-qt bridge).
#![allow(unsafe_code)]

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, texts)]
        #[qproperty(bool, loaded)]
        #[qproperty(QString, title)]
        /// Scene number, heading and plan name.
        #[qproperty(QString, plan_title, cxx_name = "planTitle")]
        /// `{ width, depth, shapes }` for KanteShapeCanvas.
        #[qproperty(QString, plan_json, cxx_name = "planJson")]
        #[qproperty(QString, tracks_json, cxx_name = "tracksJson")]
        #[qproperty(QString, shots_json, cxx_name = "shotsJson")]
        #[qproperty(QString, script_json, cxx_name = "scriptJson")]
        /// `{ set, figures, cameras, lights }`: true when shown.
        #[qproperty(QString, layers_json, cxx_name = "layersJson")]
        #[qproperty(f64, time)]
        #[qproperty(f64, duration)]
        #[qproperty(QString, selected_shot, cxx_name = "selectedShot")]
        #[qproperty(bool, can_undo, cxx_name = "canUndo")]
        #[qproperty(bool, can_redo, cxx_name = "canRedo")]
        #[qproperty(bool, dirty)]
        /// Last message for the status line, already translated.
        #[qproperty(QString, status)]
        #[qproperty(QString, screenshot_path, cxx_name = "screenshotPath")]
        #[qproperty(i32, screenshot_after, cxx_name = "screenshotAfter")]
        type Controller = super::ControllerRust;

        /// Moves the playhead (seconds).
        #[qinvokable]
        fn seek(self: Pin<&mut Self>, time: f64);

        #[qinvokable]
        #[cxx_name = "selectShot"]
        fn select_shot(self: Pin<&mut Self>, id: &QString);

        /// Sets one field (`lens`, `size` …) of a shot; false if the value is refused.
        #[qinvokable]
        #[cxx_name = "editShot"]
        fn edit_shot(self: Pin<&mut Self>, id: &QString, field: &QString, value: &QString) -> bool;

        #[qinvokable]
        fn undo(self: Pin<&mut Self>);

        #[qinvokable]
        fn redo(self: Pin<&mut Self>);

        #[qinvokable]
        fn save(self: Pin<&mut Self>) -> bool;

        /// Shows or hides one figure, camera or light (`shown`: "shown" or "hidden").
        #[qinvokable]
        #[cxx_name = "setTrack"]
        fn set_track(self: Pin<&mut Self>, id: &QString, shown: &QString);

        /// `layer`: set, figures, cameras, lights; `shown`: "shown" or "hidden".
        #[qinvokable]
        #[cxx_name = "setLayer"]
        fn set_layer(self: Pin<&mut Self>, layer: &QString, shown: &QString);

        /// Writes the plan as PDF next to the project folder; the path goes to `status`.
        #[qinvokable]
        #[cxx_name = "exportPdf"]
        fn export_pdf(self: Pin<&mut Self>) -> bool;

        /// Writes the shot list as CSV next to the project folder; the path goes to `status`.
        #[qinvokable]
        #[cxx_name = "exportCsv"]
        fn export_csv(self: Pin<&mut Self>) -> bool;

        /// What a tooltip says about a plan element (`camera:a` …), as JSON.
        #[qinvokable]
        fn describe(self: &Self, key: &QString) -> QString;
    }
}

use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use klappe_services::{
    horizontal_fov_deg, Layer, Movement, Paper, Session, ShotColumns, ShotField, ShotSize,
    ShotTexts, Source, Texts, Visibility,
};
use serde_json::json;

use crate::args::AppArgs;
use crate::catalog::Language;
use crate::notice::{error_text, Status};
use crate::plan;

pub struct ControllerRust {
    session: Option<Session>,
    language: Language,
    notice: Status,
    texts: QString,
    loaded: bool,
    title: QString,
    plan_title: QString,
    plan_json: QString,
    tracks_json: QString,
    shots_json: QString,
    script_json: QString,
    layers_json: QString,
    time: f64,
    duration: f64,
    selected_shot: QString,
    can_undo: bool,
    can_redo: bool,
    dirty: bool,
    status: QString,
    screenshot_path: QString,
    screenshot_after: i32,
}

fn q(s: &str) -> QString {
    QString::from(s)
}

const SHOWN: &str = "shown";

impl Default for ControllerRust {
    fn default() -> Self {
        let args = AppArgs::from_env();
        let language = Language::pick(args.language.as_deref());
        let mut notice = Status::default();

        let session = args.open.as_ref().and_then(|dir| match Session::open(dir) {
            Ok(mut s) => {
                if let Some(shot) = &args.shot {
                    let _ = s.select_shot(shot);
                }
                s.set_time(args.time_s.max(s.time_s()));
                Some(s)
            }
            Err(e) => {
                notice.say(error_text(language, &e));
                None
            }
        });

        let mut this = Self {
            session,
            language,
            status: q(notice.text()),
            notice,
            texts: q(language.json()),
            loaded: false,
            title: QString::default(),
            plan_title: QString::default(),
            plan_json: QString::default(),
            tracks_json: QString::default(),
            shots_json: QString::default(),
            script_json: QString::default(),
            layers_json: QString::default(),
            time: 0.0,
            duration: 0.0,
            selected_shot: QString::default(),
            can_undo: false,
            can_redo: false,
            dirty: false,
            screenshot_path: args
                .screenshot
                .map(|p| q(&p.to_string_lossy()))
                .unwrap_or_default(),
            screenshot_after: args.screenshot_after_ms,
        };
        this.fill();
        this
    }
}

impl ControllerRust {
    /// Every property from the session, at start (no signals needed yet).
    fn fill(&mut self) {
        let Some(s) = &self.session else {
            return;
        };
        self.loaded = true;
        self.title = q(&s.project().title);
        self.plan_title = q(&s.plan_title());
        self.plan_json = q(&plan_json(s));
        self.tracks_json = q(&to_json(&s.tracks()));
        self.shots_json = q(&to_json(&s.shots()));
        self.script_json = q(&to_json(&s.script_lines()));
        self.layers_json = q(&layers_json(s));
        self.time = s.time_s();
        self.duration = s.duration_s();
        self.selected_shot = q(s.selected_shot().unwrap_or_default());
    }
}

fn to_json<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string(value).unwrap_or_default()
}

/// The plan with the selected shot's camera marked.
fn plan_json(s: &Session) -> String {
    let camera = s
        .selected_shot()
        .and_then(|id| s.plan()?.shots.iter().find(|shot| shot.id == id))
        .map(|shot| shot.camera.clone());
    let selected =
        |source: &Source| matches!(source, Source::Camera(id) if Some(id) == camera.as_ref());

    plan::shapes_json(&s.display_list(), &selected)
}

fn layers_json(s: &Session) -> String {
    let shown = |l| s.layer(l) == Visibility::Shown;

    json!({
        "set": shown(Layer::Set),
        "figures": shown(Layer::Figures),
        "cameras": shown(Layer::Cameras),
        "lights": shown(Layer::Lights),
    })
    .to_string()
}

fn visibility(text: &QString) -> Visibility {
    if text.to_string() == SHOWN {
        return Visibility::Shown;
    }
    Visibility::Hidden
}

fn layer_of(name: &str) -> Option<Layer> {
    match name {
        "set" => Some(Layer::Set),
        "figures" => Some(Layer::Figures),
        "cameras" => Some(Layer::Cameras),
        "lights" => Some(Layer::Lights),
        _ => None,
    }
}

impl qobject::Controller {
    fn session(self: Pin<&mut Self>) -> Option<&mut Session> {
        self.rust_mut().get_mut().session.as_mut()
    }

    /// Pushes what may have changed after a call to the properties (and their signals).
    fn refresh(mut self: Pin<&mut Self>) {
        let values = {
            let this = self.as_ref();
            let Some(s) = this.rust().session.as_ref() else {
                return;
            };
            (
                plan_json(s),
                to_json(&s.tracks()),
                to_json(&s.shots()),
                to_json(&s.script_lines()),
                layers_json(s),
                s.time_s(),
                s.selected_shot().unwrap_or_default().to_owned(),
                s.can_undo(),
                s.can_redo(),
                s.is_dirty(),
            )
        };
        let (plan, tracks, shots, script, layers, time, selected, undo, redo, dirty) = values;

        self.as_mut().set_plan_json(q(&plan));
        self.as_mut().set_tracks_json(q(&tracks));
        self.as_mut().set_shots_json(q(&shots));
        self.as_mut().set_script_json(q(&script));
        self.as_mut().set_layers_json(q(&layers));
        self.as_mut().set_time(time);
        self.as_mut().set_selected_shot(q(&selected));
        self.as_mut().set_can_undo(undo);
        self.as_mut().set_can_redo(redo);
        self.as_mut().set_dirty(dirty);
    }

    /// Shows `text` in the header until the project changes.
    fn say(mut self: Pin<&mut Self>, text: String) {
        self.as_mut().rust_mut().get_mut().notice.say(text.clone());
        self.set_status(q(&text));
    }

    /// The project changed: a "saved" or error message no longer fits.
    fn changed(mut self: Pin<&mut Self>) {
        if self.as_mut().rust_mut().get_mut().notice.project_changed() {
            self.set_status(QString::default());
        }
    }

    fn seek(mut self: Pin<&mut Self>, time: f64) {
        let Some(s) = self.as_mut().session() else {
            return;
        };
        s.set_time(time);
        self.refresh();
    }

    fn select_shot(mut self: Pin<&mut Self>, id: &QString) {
        let Some(s) = self.as_mut().session() else {
            return;
        };
        if s.select_shot(&id.to_string()).is_err() {
            return;
        }
        self.refresh();
    }

    fn edit_shot(mut self: Pin<&mut Self>, id: &QString, field: &QString, value: &QString) -> bool {
        let language = self.rust().language;
        let Some(field) = ShotField::from_name(&field.to_string()) else {
            return false;
        };
        let Some(s) = self.as_mut().session() else {
            return false;
        };
        if let Err(e) = s.edit_shot(&id.to_string(), field, &value.to_string()) {
            self.say(error_text(language, &e));
            return false;
        }
        self.as_mut().changed();
        self.refresh();
        true
    }

    fn undo(mut self: Pin<&mut Self>) {
        if let Some(s) = self.as_mut().session() {
            s.undo();
        }
        self.as_mut().changed();
        self.refresh();
    }

    fn redo(mut self: Pin<&mut Self>) {
        if let Some(s) = self.as_mut().session() {
            s.redo();
        }
        self.as_mut().changed();
        self.refresh();
    }

    fn save(mut self: Pin<&mut Self>) -> bool {
        let language = self.rust().language;
        let result = match self.as_mut().session() {
            Some(s) => s.save().map_err(|e| error_text(language, &e)),
            None => return false,
        };

        let message = match &result {
            Ok(()) => language.text("menu.saved"),
            Err(e) => e.clone(),
        };
        self.as_mut().say(message);
        self.refresh();
        result.is_ok()
    }

    fn set_track(mut self: Pin<&mut Self>, id: &QString, shown: &QString) {
        if let Some(s) = self.as_mut().session() {
            s.set_track(&id.to_string(), visibility(shown));
        }
        self.refresh();
    }

    fn set_layer(mut self: Pin<&mut Self>, layer: &QString, shown: &QString) {
        let Some(layer) = layer_of(&layer.to_string()) else {
            return;
        };
        if let Some(s) = self.as_mut().session() {
            s.set_layer(layer, visibility(shown));
        }
        self.refresh();
    }

    fn export_pdf(mut self: Pin<&mut Self>) -> bool {
        let language = self.rust().language;
        let texts = print_texts(language);
        let Some(s) = self.as_mut().session() else {
            return false;
        };

        let path = plan_file(s, "pdf");
        let message = match s.export_pdf(&path, Paper::A4, texts) {
            Ok(()) => language
                .text("menu.pdfSaved")
                .replace("{path}", &path.to_string_lossy()),
            Err(e) => error_text(language, &e),
        };
        let ok = path.exists();
        self.as_mut().say(message);
        ok
    }

    fn export_csv(mut self: Pin<&mut Self>) -> bool {
        let language = self.rust().language;
        let texts = print_texts(language);
        let Some(s) = self.as_mut().session() else {
            return false;
        };

        let path = plan_file(s, "csv");
        let message = match s.export_csv(&path, &texts) {
            Ok(()) => language
                .text("menu.csvSaved")
                .replace("{path}", &path.to_string_lossy()),
            Err(e) => error_text(language, &e),
        };
        let ok = path.exists();
        self.as_mut().say(message);
        ok
    }

    fn describe(&self, key: &QString) -> QString {
        let Some(s) = self.rust().session.as_ref() else {
            return QString::default();
        };
        q(&describe(s, &key.to_string()).to_string())
    }
}

/// Where an export of the current plan goes: next to the project folder, named by the plan.
fn plan_file(s: &Session, extension: &str) -> std::path::PathBuf {
    let name = format!(
        "{}.{extension}",
        s.plan_title().replace([' ', '·', '/', '–'], "-")
    );

    s.dir()
        .parent()
        .map(|p| p.join(&name))
        .unwrap_or_else(|| name.into())
}

/// Sizes and movements in the order the model lists them, with their catalogue keys.
const SIZES: [(ShotSize, &str); 8] = [
    (ShotSize::ExtremeWide, "extreme-wide"),
    (ShotSize::Wide, "wide"),
    (ShotSize::MediumWide, "medium-wide"),
    (ShotSize::Cowboy, "cowboy"),
    (ShotSize::Medium, "medium"),
    (ShotSize::MediumClose, "medium-close"),
    (ShotSize::CloseUp, "close-up"),
    (ShotSize::ExtremeCloseUp, "extreme-close-up"),
];
const MOVEMENTS: [(Movement, &str); 8] = [
    (Movement::Static, "static"),
    (Movement::Pan, "pan"),
    (Movement::Tilt, "tilt"),
    (Movement::Dolly, "dolly"),
    (Movement::Crane, "crane"),
    (Movement::Handheld, "handheld"),
    (Movement::Gimbal, "gimbal"),
    (Movement::Zoom, "zoom"),
];

/// The shot list's words from the catalogue.
fn shot_texts(language: Language) -> ShotTexts {
    let t = |k: &str| language.text(k);

    ShotTexts {
        title: t("shots.title"),
        columns: ShotColumns {
            number: t("field.number"),
            camera: t("field.camera"),
            lens_mm: t("field.lens"),
            size: t("field.size"),
            movement: t("field.movement"),
            from_s: t("field.from"),
            to_s: t("field.to"),
            duration_s: t("field.duration"),
            description: t("field.description"),
            props: t("field.props"),
            gear: t("field.gear"),
            crew: t("field.crew"),
            note: t("field.note"),
        },
        sizes: SIZES
            .iter()
            .map(|(size, key)| (*size, t(&format!("size.{key}"))))
            .collect(),
        movements: MOVEMENTS
            .iter()
            .map(|(movement, key)| (*movement, t(&format!("movement.{key}"))))
            .collect(),
    }
}

/// The printed texts from the catalogue (E-05: no texts in code).
fn print_texts(language: Language) -> Texts {
    let t = |k: &str| language.text(k);

    Texts {
        figure: t("legend.figure"),
        camera: t("legend.camera"),
        light: t("legend.light"),
        axis: t("legend.axis"),
        path: t("legend.path"),
        mark: t("legend.mark"),
        unit: t("unit.m"),
        decimal_separator: t("unit.decimal").chars().next().unwrap_or('.'),
        shots: shot_texts(language),
    }
}

/// Data for a tooltip; QML puts it into words with the catalogue.
fn describe(s: &Session, key: &str) -> serde_json::Value {
    let mut parts = key.split(':');
    let (Some(kind), Some(id)) = (parts.next(), parts.next()) else {
        return json!({});
    };
    let Some(plan) = s.plan() else {
        return json!({});
    };

    match kind {
        "camera" => {
            let Some(c) = plan.camera(id) else {
                return json!({});
            };
            let shots: Vec<&str> = plan
                .shots
                .iter()
                .filter(|shot| shot.camera == id)
                .map(|shot| shot.number.as_str())
                .collect();
            json!({
                "kind": "camera",
                "label": c.label,
                "lens": c.lens_mm,
                "fov": (horizontal_fov_deg(c.sensor, c.lens_mm) * 10.0).round() / 10.0,
                "height": c.marks.first().map(|m| m.height_m).unwrap_or(0.0),
                "shots": shots.join(", "),
            })
        }
        "figure" | "mark" => {
            let name = s
                .project()
                .character(id)
                .map(|c| c.name.clone())
                .unwrap_or_default();
            let index: Option<usize> = parts.next().and_then(|i| i.parse().ok());
            let at = index
                .and_then(|i| plan.figure(id)?.marks.get(i))
                .map(|m| m.at_s);
            json!({ "kind": kind, "name": name, "index": index.map(|i| i + 1), "time": at })
        }
        "light" => {
            let Some(l) = plan.lights.iter().find(|l| l.id == id) else {
                return json!({});
            };
            json!({ "kind": "light", "label": l.label, "temp": l.color_temp_k, "power": l.power_w })
        }
        "object" => {
            let set = s
                .scene()
                .and_then(|sc| s.project().location(&sc.location))
                .map(|l| &l.set);
            let label = set
                .and_then(|set| set.objects.iter().find(|o| o.id == id))
                .map(|o| o.label.clone())
                .unwrap_or_default();
            json!({ "kind": "object", "label": label })
        }
        _ => json!({}),
    }
}
