//! Output with one page per shot or per camera (LH-87): the plan at the shot's start, that
//! shot's camera strong (body, cone, way), everything else faded (`Fade::Faint`, LH-15). What
//! the layers and tracks hide on screen stays hidden.
//!
//! ```text
//! shot 12.1 ─► camera A ─► page: plan at 12.1's start, A strong, the rest pale   ┐ PDF: pages
//! shot 12.2 ─► camera B ─► page: plan at 12.2's start, B strong, the rest pale   ┘ PNG: files
//! ```

use std::path::{Path, PathBuf};

use klappe_export::{Page, Texts, Theme};
use klappe_model::PaperSetup;
use klappe_stellprobe::{DisplayList, Source};

use super::export::{file_part, write_file, MIN_NUMBER_WIDTH};
use super::{Fade, Session};
use crate::error::{write_failed, ServiceError};

/// What a page of the output is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pages {
    /// One page per shot, in cut order, at the shot's start.
    PerShot,
    /// One page per camera of the plan, at its first shot's start (else at 0).
    PerCamera,
}

/// A page: the camera it is about, the moment, and what to call it.
struct Focus {
    camera: String,
    at_s: f64,
    name: String,
}

impl Session {
    /// Writes one PDF page per shot or camera (LH-87), each titled with the plan and the
    /// shot's number or the camera's label. The playhead stays where it was.
    pub fn export_pages(
        &mut self,
        path: &Path,
        paper: impl Into<PaperSetup>,
        texts: &Texts,
        pages: Pages,
    ) -> Result<(), ServiceError> {
        let paper = paper.into();
        let focuses = self.focuses(pages);
        if focuses.is_empty() {
            return Err(ServiceError::NoPages);
        }

        let playhead_s = self.time_s;
        let made: Result<Vec<_>, _> = focuses
            .iter()
            .map(|focus| self.focus_page(focus, paper, texts))
            .collect();
        self.set_time(playhead_s);

        write_file(path, klappe_export::pdf_pages(&made?, Theme::Light)?)
    }

    /// Writes one PNG per shot or camera (LH-87) into `dir`, named `<scene>-<nr>-<shot or
    /// camera>.png`; `dir` must be empty or missing. The playhead stays where it was.
    /// Returns the files in page order.
    pub fn export_page_files(
        &mut self,
        dir: &Path,
        paper: impl Into<PaperSetup>,
        texts: &Texts,
        pages: Pages,
    ) -> Result<Vec<PathBuf>, ServiceError> {
        let paper = paper.into();
        let focuses = self.focuses(pages);
        if focuses.is_empty() {
            return Err(ServiceError::NoPages);
        }
        if !klappe_store::is_empty_dir(dir).map_err(write_failed)? {
            return Err(ServiceError::NotEmpty(dir.to_owned()));
        }

        let playhead_s = self.time_s;
        let scene = file_part(&self.scene_number());
        let width = focuses.len().to_string().len().max(MIN_NUMBER_WIDTH);
        let mut files = Vec::new();
        let mut result = Ok(());

        for (i, focus) in focuses.iter().enumerate() {
            let name = format!("{scene}-{:0width$}-{}.png", i + 1, file_part(&focus.name));
            let path = dir.join(name);
            result = self
                .focus_page(focus, paper, texts)
                .and_then(|(list, page)| Ok(klappe_export::png(&list, &page, Theme::Light)?))
                .and_then(|bytes| write_file(&path, bytes));
            if result.is_err() {
                break;
            }
            files.push(path);
        }
        self.set_time(playhead_s);

        result.map(|()| files)
    }

    /// The pages of `pages` in the plan on view.
    fn focuses(&self, pages: Pages) -> Vec<Focus> {
        let Some(plan) = self.plan() else {
            return Vec::new();
        };

        match pages {
            Pages::PerShot => plan
                .shots
                .iter()
                .map(|s| Focus {
                    camera: s.camera.clone(),
                    at_s: s.from_s,
                    name: s.number.clone(),
                })
                .collect(),
            Pages::PerCamera => plan
                .cameras
                .iter()
                .map(|c| Focus {
                    camera: c.id.clone(),
                    at_s: plan
                        .shots
                        .iter()
                        .find(|s| s.camera == c.id)
                        .map_or(0.0, |s| s.from_s),
                    name: c.label.clone(),
                })
                .collect(),
        }
    }

    /// The plan at the focus's moment with its camera strong and the rest faded.
    fn focus_page(
        &mut self,
        focus: &Focus,
        paper: PaperSetup,
        texts: &Texts,
    ) -> Result<(DisplayList, Page), ServiceError> {
        self.set_time(focus.at_s);
        let (list, mut page) = self.printout(paper, texts.clone())?;

        page.title = format!("{} · {}", page.title, focus.name);
        page.fade = list
            .items
            .iter()
            .zip(self.item_fades(&list))
            .map(|(item, own)| match &item.source {
                Some(Source::Camera(id)) if *id == focus.camera => 1.0,
                _ => own.min(Fade::Faint.strength()),
            })
            .collect();

        Ok((list, page))
    }
}
