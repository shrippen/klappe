//! Exports of the plan: PDF, PNG, SVG, the shot list (CSV, ODS, XLSX) and the frames.

use super::qobject;
use crate::fileurl;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use klappe_services::error_text;
use klappe_services::{PaperSetup, ServiceError, Session, ShotOrder, Source, Texts};

impl qobject::Controller {
    /// Writes the plan to `path` (a `file://` URL or a path) with `write`; says where it went
    /// (`saved` is the catalogue key of the message) or what failed. A file name without
    /// `extension` gets it; a folder has none.
    fn export_plan(
        mut self: Pin<&mut Self>,
        path: &QString,
        extension: Option<&str>,
        saved: &str,
        write: impl FnOnce(&mut Session, &std::path::Path, Texts) -> Result<(), ServiceError>,
    ) -> bool {
        let language = self.rust().language;
        let texts = language.print_texts();
        let mut path = fileurl::to_path(&path.to_string());
        if let Some(extension) = extension {
            let Some(named) = self.as_mut().named_file(path, extension) else {
                return false;
            };
            path = named;
        }
        let Some(s) = self.as_mut().session() else {
            return false;
        };

        let result = write(s, &path, texts);
        let ok = result.is_ok();
        let message = match result {
            Ok(()) => language
                .text(saved)
                .replace("{path}", &path.to_string_lossy()),
            Err(e) => error_text(language, &e),
        };
        self.as_mut().say(message);
        ok
    }

    /// `path` with `extension` added. The dialog asked about the name as typed; where the
    /// added extension lands on a file that exists, nothing is replaced unasked: says so and
    /// returns `None`.
    pub(super) fn named_file(
        mut self: Pin<&mut Self>,
        path: std::path::PathBuf,
        extension: &str,
    ) -> Option<std::path::PathBuf> {
        let named = fileurl::with_extension(path.clone(), extension);
        if named == path || !named.exists() {
            return Some(named);
        }

        let language = self.rust().language;
        let message = language
            .text("error.nameTaken")
            .replace("{path}", &named.to_string_lossy());
        self.as_mut().say(message);

        None
    }

    pub(super) fn export_to(
        mut self: Pin<&mut Self>,
        kind: &QString,
        order: &QString,
        paper: &QString,
        path: &QString,
    ) -> bool {
        let order = shot_order(&order.to_string());
        let paper = PaperSetup::from_name(&paper.to_string());
        let kind = kind.to_string();

        let ok = self.as_mut().export_kind(&kind, order, paper, path);

        // The sheet a plan was printed on is the project's from now on (no undo step, see
        // `Session::set_paper`); a failed export printed nothing.
        if ok && PAPER_KINDS.contains(&kind.as_str()) {
            if let Some(s) = self.as_mut().session() {
                s.set_paper(paper);
            }
            self.as_mut().refresh();
        }

        ok
    }

    /// Writes `path` as `kind` says; false and a message when it fails.
    fn export_kind(
        mut self: Pin<&mut Self>,
        kind: &str,
        order: ShotOrder,
        paper: PaperSetup,
        path: &QString,
    ) -> bool {
        match kind {
            "pdf" => self.export_plan(path, Some("pdf"), "menu.pdfSaved", |s, path, texts| {
                s.export_pdf(path, paper, texts, order)
            }),
            "png" => self.export_plan(path, Some("png"), "menu.pngSaved", |s, path, texts| {
                s.export_png(path, paper, texts)
            }),
            "svg" => self.export_plan(path, Some("svg"), "menu.svgSaved", |s, path, texts| {
                s.export_svg(path, paper, texts)
            }),
            "csv" => self.export_plan(path, Some("csv"), "menu.csvSaved", |s, path, texts| {
                s.export_csv(path, &texts, order)
            }),
            "ods" => self.export_plan(path, Some("ods"), "menu.odsSaved", |s, path, texts| {
                s.export_ods(path, &texts, order)
            }),
            "xlsx" => self.export_plan(path, Some("xlsx"), "menu.xlsxSaved", |s, path, texts| {
                s.export_xlsx(path, &texts, order)
            }),
            VIEW3D => {
                let shown = self.rust().free_shown();

                self.export_plan(path, Some("png"), "menu.view3dSaved", move |s, path, _| {
                    s.export_view_png(path, shown)
                })
            }
            CAMERA => {
                let picked = match self.rust().selected.only() {
                    Some(Source::Camera(id)) => Some(id.clone()),
                    _ => None,
                };

                self.export_plan(path, Some("png"), "menu.cameraSaved", move |s, path, _| {
                    s.export_camera_png(path, picked.as_deref())
                })
            }
            FRAMES => self.export_plan(path, None, "menu.framesSaved", |s, dir, texts| {
                s.export_frames(dir, paper, &texts).map(|_| ())
            }),
            other => {
                let language = self.rust().language;
                let message = language.text("error.export").replace("{detail}", other);
                self.as_mut().say(message);
                false
            }
        }
    }

    /// The file the export dialog offers: next to the project folder, named by the plan.
    pub(super) fn export_file(mut self: Pin<&mut Self>, extension: &QString) -> QString {
        let Some(s) = self.as_mut().session() else {
            return QString::default();
        };

        QString::from(fileurl::from_path(&plan_file(s, &extension.to_string())))
    }
}

/// The kind that writes a folder of PNGs, one per mark, not a file.
const FRAMES: &str = "frames";
/// The kinds that write the 3D view and the camera's view as PNG (LH-86).
const VIEW3D: &str = "view3d";
const CAMERA: &str = "camera";

/// The kinds that print on a sheet.
const PAPER_KINDS: [&str; 4] = ["pdf", "png", "svg", FRAMES];

/// `shooting` is the shooting order; anything else is the cut order.
fn shot_order(name: &str) -> ShotOrder {
    match name {
        "shooting" => ShotOrder::Shooting,
        _ => ShotOrder::Cut,
    }
}

/// Where an export of the current plan goes: next to the project folder, named by the plan.
fn plan_file(s: &Session, extension: &str) -> std::path::PathBuf {
    let title = s.plan_title().replace([' ', '·', '/', '–'], "-");

    // A folder has no extension; the PNGs of a view sit beside the plan's under their own name.
    let name = match extension {
        FRAMES => title,
        VIEW3D => format!("{title}-3d.png"),
        CAMERA => format!("{title}-camera.png"),
        _ => format!("{title}.{extension}"),
    };

    s.dir()
        .parent()
        .map(|p| p.join(&name))
        .unwrap_or_else(|| name.into())
}
