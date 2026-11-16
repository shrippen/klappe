//! Exports of the plan: PDF, PNG, SVG, the shot list (CSV, ODS, XLSX) and the frames.

use super::qobject;
use crate::fileurl;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use klappe_services::error_text;
use klappe_services::{ExportFormat, ExportOptions, PaperSetup, Session, ShotOrder, Source};

impl qobject::Controller {
    /// Writes the plan to `path` (a `file://` URL or a path) as `format` says; says where it
    /// went or what failed. A file name without the format's extension gets it; a folder has
    /// none.
    fn export_plan(
        mut self: Pin<&mut Self>,
        path: &QString,
        format: ExportFormat,
        options: &ExportOptions,
    ) -> bool {
        let language = self.rust().language;
        let mut path = fileurl::to_path(&path.to_string());
        if let Some(extension) = format.extension() {
            let Some(named) = self.as_mut().named_file(path, extension) else {
                return false;
            };
            path = named;
        }
        let Some(s) = self.as_mut().session() else {
            return false;
        };

        let result = s.export(format, &path, options);
        let ok = result.is_ok();
        let message = match result {
            // The catalogue key of the message is `menu.<format>Saved`.
            Ok(()) => language
                .text(&format!("menu.{}Saved", format.name()))
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
        let language = self.rust().language;
        let kind = kind.to_string();
        let Some(format) = ExportFormat::from_name(&kind) else {
            let message = language.text("error.export").replace("{detail}", &kind);
            self.as_mut().say(message);
            return false;
        };

        let camera = match self.rust().selected.only() {
            Some(Source::Camera(id)) => Some(id.clone()),
            _ => None,
        };
        let options = ExportOptions {
            paper: PaperSetup::from_name(&paper.to_string()),
            order: shot_order(&order.to_string()),
            texts: language.print_texts(),
            shown: self.rust().free_shown(),
            camera,
        };

        let ok = self.as_mut().export_plan(path, format, &options);

        // The session took the sheet over; the project view shows it.
        if ok && format.needs_paper() {
            self.as_mut().refresh();
        }

        ok
    }

    /// The file the export dialog offers: next to the project folder, named by the plan.
    pub(super) fn export_file(mut self: Pin<&mut Self>, kind: &QString) -> QString {
        let Some(format) = ExportFormat::from_name(&kind.to_string()) else {
            return QString::default();
        };
        let Some(s) = self.as_mut().session() else {
            return QString::default();
        };

        QString::from(fileurl::from_path(&plan_file(s, format)))
    }
}

/// `shooting` is the shooting order; anything else is the cut order.
fn shot_order(name: &str) -> ShotOrder {
    match name {
        "shooting" => ShotOrder::Shooting,
        _ => ShotOrder::Cut,
    }
}

/// Where an export of the current plan goes: next to the project folder, named by the plan.
fn plan_file(s: &Session, format: ExportFormat) -> std::path::PathBuf {
    let title = s.plan_title().replace([' ', '·', '/', '–'], "-");

    // A folder has no extension; the PNGs of a view sit beside the plan's under their own name.
    let name = match format {
        ExportFormat::Frames => title,
        ExportFormat::View3d => format!("{title}-3d.png"),
        ExportFormat::Camera => format!("{title}-camera.png"),
        _ => format!("{title}.{}", format.name()),
    };

    s.dir()
        .parent()
        .map(|p| p.join(&name))
        .unwrap_or_else(|| name.into())
}
