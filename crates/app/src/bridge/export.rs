//! Exports of the plan: PDF, PNG, SVG and CSV.

use super::qobject;
use crate::notice::error_text;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use klappe_services::{Paper, ServiceError, Session, ShotOrder, Texts};

impl qobject::Controller {
    /// Writes the plan next to the project folder with `write`; says where it went (`saved`
    /// is the catalogue key of the message) or what failed.
    fn export_plan(
        mut self: Pin<&mut Self>,
        extension: &str,
        saved: &str,
        write: impl FnOnce(&Session, &std::path::Path, Texts) -> Result<(), ServiceError>,
    ) -> bool {
        let language = self.rust().language;
        let texts = language.print_texts();
        let Some(s) = self.as_mut().session() else {
            return false;
        };

        let path = plan_file(s, extension);
        let message = match write(s, &path, texts) {
            Ok(()) => language
                .text(saved)
                .replace("{path}", &path.to_string_lossy()),
            Err(e) => error_text(language, &e),
        };
        let ok = path.exists();
        self.as_mut().say(message);
        ok
    }

    pub(super) fn export_pdf(self: Pin<&mut Self>, order: &QString) -> bool {
        let order = shot_order(&order.to_string());

        self.export_plan("pdf", "menu.pdfSaved", |s, path, texts| {
            s.export_pdf(path, Paper::A4, texts, order)
        })
    }

    pub(super) fn export_png(self: Pin<&mut Self>) -> bool {
        self.export_plan("png", "menu.pngSaved", |s, path, texts| {
            s.export_png(path, Paper::A4, texts)
        })
    }

    pub(super) fn export_svg(self: Pin<&mut Self>) -> bool {
        self.export_plan("svg", "menu.svgSaved", |s, path, texts| {
            s.export_svg(path, Paper::A4, texts)
        })
    }

    pub(super) fn export_csv(mut self: Pin<&mut Self>, order: &QString) -> bool {
        let language = self.rust().language;
        let texts = language.print_texts();
        let Some(s) = self.as_mut().session() else {
            return false;
        };

        let path = plan_file(s, "csv");
        let message = match s.export_csv(&path, &texts, shot_order(&order.to_string())) {
            Ok(()) => language
                .text("menu.csvSaved")
                .replace("{path}", &path.to_string_lossy()),
            Err(e) => error_text(language, &e),
        };
        let ok = path.exists();
        self.as_mut().say(message);
        ok
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
