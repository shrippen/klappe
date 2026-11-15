//! Exports of the plan: PDF, PNG, SVG and CSV.

use super::qobject;
use crate::fileurl;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use klappe_services::error_text;
use klappe_services::{Paper, ServiceError, Session, ShotOrder, Texts};

impl qobject::Controller {
    /// Writes the plan to `path` (a `file://` URL or a path) with `write`; says where it went
    /// (`saved` is the catalogue key of the message) or what failed.
    fn export_plan(
        mut self: Pin<&mut Self>,
        path: &QString,
        saved: &str,
        write: impl FnOnce(&Session, &std::path::Path, Texts) -> Result<(), ServiceError>,
    ) -> bool {
        let language = self.rust().language;
        let texts = language.print_texts();
        let path = fileurl::to_path(&path.to_string());
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

    pub(super) fn export_to(
        mut self: Pin<&mut Self>,
        kind: &QString,
        order: &QString,
        paper: &QString,
        path: &QString,
    ) -> bool {
        let order = shot_order(&order.to_string());
        let paper = paper_of(&paper.to_string());

        match kind.to_string().as_str() {
            "pdf" => self.export_plan(path, "menu.pdfSaved", |s, path, texts| {
                s.export_pdf(path, paper, texts, order)
            }),
            "png" => self.export_plan(path, "menu.pngSaved", |s, path, texts| {
                s.export_png(path, paper, texts)
            }),
            "svg" => self.export_plan(path, "menu.svgSaved", |s, path, texts| {
                s.export_svg(path, paper, texts)
            }),
            "csv" => self.export_plan(path, "menu.csvSaved", |s, path, texts| {
                s.export_csv(path, &texts, order)
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

/// `a3` is A3; anything else is A4.
fn paper_of(name: &str) -> Paper {
    match name {
        "a3" => Paper::A3,
        _ => Paper::A4,
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
