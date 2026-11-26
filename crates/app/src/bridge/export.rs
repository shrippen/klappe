//! Exports of the plan: PDF, PNG, SVG, the shot list (CSV, ODS, XLSX) and the frames; the
//! reports of DB-61.

use super::qobject;
use crate::fileurl;
use core::pin::Pin;
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;
use klappe_services::error_text;
use klappe_services::{
    ExportFormat, ExportOptions, Laid, PaperSetup, Report, ReportFormat, Session, ShotOrder,
    Source, Texts, FFMPEG,
};
use std::ops::ControlFlow;
use std::sync::atomic::{AtomicBool, Ordering};

/// A video is being written: one at a time, the next export has to wait for it.
static VIDEO_RUNNING: AtomicBool = AtomicBool::new(false);
/// Set when the window closes: the video stops and its part file goes.
static VIDEO_STOP: AtomicBool = AtomicBool::new(false);
const PERCENT: usize = 100;
/// How long closing the window waits for ffmpeg to be stopped and its file removed.
const STOP_WAIT: std::time::Duration = std::time::Duration::from_secs(5);
const STOP_POLL: std::time::Duration = std::time::Duration::from_millis(20);

/// The window closes while a video is written: stops it and waits until its part file is gone,
/// else ffmpeg would see the end of its input and leave a playable, cut-off `.mp4.part`.
pub(super) struct VideoStop;

impl Drop for VideoStop {
    fn drop(&mut self) {
        VIDEO_STOP.store(true, Ordering::SeqCst);
        let waited_from = std::time::Instant::now();
        while VIDEO_RUNNING.load(Ordering::SeqCst) && waited_from.elapsed() < STOP_WAIT {
            std::thread::sleep(STOP_POLL);
        }
    }
}

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
        if format == ExportFormat::Video {
            return self.write_video(path, &options.texts);
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

    /// Writes the playback as MP4 (LH-84) on a thread of its own, so the window stays alive
    /// while ffmpeg works: here the job copies the project, there each frame is drawn when it
    /// is written and encoded. The status
    /// line shows the progress and, last, where the video went or what failed. `true` if it
    /// started.
    fn write_video(mut self: Pin<&mut Self>, path: std::path::PathBuf, texts: &Texts) -> bool {
        let language = self.rust().language;
        if VIDEO_RUNNING.swap(true, Ordering::SeqCst) {
            let message = language.text("menu.mp4Started");
            self.as_mut().say(message);
            return false;
        }

        let job = match self.as_mut().session() {
            Some(s) => s.video_job(FFMPEG, texts),
            None => Err(klappe_services::ServiceError::Gone),
        };
        let job = match job {
            Ok(job) => job,
            Err(e) => {
                VIDEO_RUNNING.store(false, Ordering::SeqCst);
                self.as_mut().say(error_text(language, &e));
                return false;
            }
        };

        VIDEO_STOP.store(false, Ordering::SeqCst);
        let started = language.text("menu.mp4Started");
        self.as_mut().say(started);
        let qt = self.qt_thread();
        std::thread::spawn(move || {
            let mut shown = 0;
            let result = job.run(&path, &mut |done, total| {
                if VIDEO_STOP.load(Ordering::SeqCst) {
                    return ControlFlow::Break(());
                }
                let percent = done * PERCENT / total;
                if percent == shown {
                    return ControlFlow::Continue(());
                }
                shown = percent;
                let message = language
                    .text("menu.mp4Progress")
                    .replace("{percent}", &percent.to_string());
                // Fails only when the window is gone; then nobody reads the progress.
                let _ = qt.queue(move |this| this.say(message));

                ControlFlow::Continue(())
            });
            let message = match result {
                Ok(()) => language
                    .text("menu.mp4Saved")
                    .replace("{path}", &path.to_string_lossy()),
                Err(e) => error_text(language, &e),
            };

            VIDEO_RUNNING.store(false, Ordering::SeqCst);
            let _ = qt.queue(move |this| this.say(message));
        });

        true
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

    pub(super) fn export_report(
        self: Pin<&mut Self>,
        report: &QString,
        format: &QString,
        path: &QString,
    ) -> bool {
        self.write_report(report, "", format, path)
    }

    /// The dialogue report (DB-52) of the character `figure`, as `export_report`.
    pub(super) fn export_dialogue(
        self: Pin<&mut Self>,
        figure: &QString,
        format: &QString,
        path: &QString,
    ) -> bool {
        let report = QString::from(Report::Dialogue.name());

        self.write_report(&report, &figure.to_string(), format, path)
    }

    /// The characters with a reply in the book, most replies first, as
    /// `[{ id, name, replies }]`; the choice before the dialogue report. A book without one
    /// says so on the status line.
    pub(super) fn dialogue_figures(mut self: Pin<&mut Self>) -> QString {
        let figures = self
            .rust()
            .session
            .as_ref()
            .map(Session::dialogue_figures)
            .unwrap_or_default();
        if figures.is_empty() {
            let message = self.rust().language.text("error.noReplies");
            self.as_mut().say(message);
        }

        QString::from(serde_json::json!(figures).to_string())
    }

    fn write_report(
        mut self: Pin<&mut Self>,
        report: &QString,
        figure: &str,
        format: &QString,
        path: &QString,
    ) -> bool {
        let language = self.rust().language;
        let (report, format) = (report.to_string(), format.to_string());
        let (Some(kind), Some(format)) = (
            Report::from_name(&report),
            ReportFormat::from_extension(&format),
        ) else {
            let message = language
                .text("error.export")
                .replace("{detail}", &format!("{report} {format}"));
            self.as_mut().say(message);
            return false;
        };

        let path = fileurl::to_path(&path.to_string());
        let Some(path) = self.as_mut().named_file(path, format.name()) else {
            return false;
        };
        let Some(s) = self.as_mut().session() else {
            return false;
        };

        // The report lays the text out if need be; the scene list then shows its lengths.
        let laid = s.paginate_if_changed(language);
        let result = match kind {
            Report::Dialogue => s.export_dialogue(figure, format, &path, language),
            _ => s.export_report(kind, format, &path, language),
        };
        let ok = result.is_ok();
        let message = match result {
            Ok(()) => language
                .text("menu.reportSaved")
                .replace("{path}", &path.to_string_lossy()),
            Err(e) => error_text(language, &e),
        };
        if laid == Laid::Anew {
            self.as_mut().refresh();
        }
        self.as_mut().say(message);

        ok
    }

    /// The file the export dialog offers for a report: next to the project folder, named by
    /// the report.
    pub(super) fn report_file(mut self: Pin<&mut Self>, report: &QString) -> QString {
        let language = self.rust().language;
        let Some(s) = self.as_mut().session() else {
            return QString::default();
        };

        let title = language
            .text(&format!("report.{report}"))
            .replace([' ', '/'], "-");
        let name = format!("{title}.{}", ReportFormat::Pdf.name());
        let path = s
            .dir()
            .parent()
            .map(|p| p.join(&name))
            .unwrap_or_else(|| name.into());

        QString::from(fileurl::from_path(&path))
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
    let name = format.file_name(&title);

    s.dir()
        .parent()
        .map(|p| p.join(&name))
        .unwrap_or_else(|| name.into())
}
