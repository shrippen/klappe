use std::path::{Path, PathBuf};

use klappe_export::{Page, ShotView, Texts, Theme};
use klappe_model::{FloorPlan, PaperSetup};
use klappe_stellprobe::{render, vector_view, DisplayList, Eye, Lamp, Shape, ShotOrder};

use super::{Session, Size};
use crate::error::{write_failed, ServiceError};
use crate::library;
use crate::ExportFormat;

const SECONDS_PER_MINUTE: u64 = 60;
const MIN_NUMBER_WIDTH: usize = 2;
/// Width of the PNG of a view (LH-86).
const VIEW_WIDTH_PX: f64 = 1920.0;

/// What an export needs besides its path; each format takes what it uses.
#[derive(Debug, Clone)]
pub struct ExportOptions {
    /// The sheet of the formats that print on one (`ExportFormat::needs_paper`).
    pub paper: PaperSetup,
    /// The order of the shot list in PDF and the lists.
    pub order: ShotOrder,
    pub texts: Texts,
    /// The size the 3D view has on screen, for its PNG.
    pub shown: Size,
    /// The camera of the camera view when no shot is selected.
    pub camera: Option<String>,
}

impl Session {
    /// Writes `format` to `path`. A plan printed on a sheet makes that sheet the project's, as
    /// `set_paper` says; an export that fails printed nothing.
    pub fn export(
        &mut self,
        format: ExportFormat,
        path: &Path,
        options: &ExportOptions,
    ) -> Result<(), ServiceError> {
        let (paper, order, texts) = (options.paper, options.order, &options.texts);

        match format {
            ExportFormat::Pdf => self.export_pdf(path, paper, texts.clone(), order),
            ExportFormat::Png => self.export_png(path, paper, texts.clone()),
            ExportFormat::Svg => self.export_svg(path, paper, texts.clone()),
            ExportFormat::Csv => self.export_csv(path, texts, order),
            ExportFormat::Ods => self.export_ods(path, texts, order),
            ExportFormat::Xlsx => self.export_xlsx(path, texts, order),
            ExportFormat::Frames => self.export_frames(path, paper, texts).map(|_| ()),
            ExportFormat::View3d => self.export_view_png(path, options.shown),
            ExportFormat::Camera => self.export_camera_png(path, options.camera.as_deref()),
        }?;

        if format.needs_paper() {
            self.set_paper(paper);
        }

        Ok(())
    }

    /// Prints the current plan at the playhead as PDF (LH-80), followed by its shot list;
    /// `title` names scene and plan.
    pub fn export_pdf(
        &self,
        path: &Path,
        paper: impl Into<PaperSetup>,
        texts: Texts,
        order: ShotOrder,
    ) -> Result<(), ServiceError> {
        let (list, page) = self.printout(paper.into(), texts)?;
        let bytes = match self.plan() {
            Some(plan) => {
                let views = self.shot_views();

                klappe_export::pdf_with_views(&list, &page, Theme::Light, plan, order, &views)?
            }
            None => klappe_export::pdf(&list, &page, Theme::Light)?,
        };

        write_file(path, bytes)
    }

    /// Writes the 3D view as PNG (LH-86), as it is on screen: orbit or through a camera, with
    /// the cut, `shown` the size it has there; the PNG is as wide as `VIEW_WIDTH_PX` and keeps
    /// its aspect.
    pub fn export_view_png(&self, path: &Path, shown: Size) -> Result<(), ServiceError> {
        if shown.0 <= 0.0 || shown.1 <= 0.0 {
            return Err(ServiceError::NoPicture);
        }

        let size = (VIEW_WIDTH_PX, (VIEW_WIDTH_PX * shown.1 / shown.0).round());
        let picture = self.free_picture(size).ok_or(ServiceError::NoPicture)?;

        write_file(path, klappe_export::picture_png(&picture, Theme::Light)?)
    }

    /// Writes the picture of the selected shot's camera at the playhead as PNG (LH-86), else
    /// that of `fallback`, the camera picked on the plan; as wide as `VIEW_WIDTH_PX`, in the
    /// camera's aspect.
    pub fn export_camera_png(
        &self,
        path: &Path,
        fallback: Option<&str>,
    ) -> Result<(), ServiceError> {
        let picture = self
            .camera_picture(fallback, VIEW_WIDTH_PX as usize)
            .ok_or(ServiceError::NoPicture)?;

        write_file(path, klappe_export::picture_png(&picture, Theme::Light)?)
    }

    /// What each shot's camera sees at the shot's start, as faces for print; a shot whose
    /// camera sees nothing has none.
    fn shot_views(&self) -> Vec<ShotView> {
        let (Some(plan), Some(location)) = (
            self.plan(),
            self.scene()
                .and_then(|s| self.project.location(&s.location)),
        ) else {
            return Vec::new();
        };

        let forms = library::forms_of(&self.project.library, &location.set);
        let mut views = Vec::new();
        for shot in &plan.shots {
            let eye = Eye::Shot {
                camera: shot.camera.clone(),
                t_s: shot.from_s,
                aspect: None,
            };
            let set = &location.set;
            let Some(view) = vector_view(&self.project, set, plan, &forms, &eye) else {
                continue;
            };

            views.push(ShotView {
                shot: shot.id.clone(),
                view,
            });
        }

        views
    }

    /// Writes the current plan at the playhead as PNG (LH-81): RGB, 150 dpi, no shot list.
    pub fn export_png(
        &self,
        path: &Path,
        paper: impl Into<PaperSetup>,
        texts: Texts,
    ) -> Result<(), ServiceError> {
        let (list, page) = self.printout(paper.into(), texts)?;
        let bytes = klappe_export::png(&list, &page, Theme::Light)?;

        write_file(path, bytes)
    }

    /// Writes the current plan as a PNG without the sheet title, for the start page's tile.
    pub fn export_preview(&self, path: &Path, texts: Texts) -> Result<(), ServiceError> {
        let (list, mut page) = self.printout(PaperSetup::default(), texts)?;
        page.title.clear();
        let bytes = klappe_export::png(&list, &page, Theme::Light)?;

        write_file(path, bytes)
    }

    /// Writes the current plan at the playhead as SVG (LH-81), without the shot list.
    pub fn export_svg(
        &self,
        path: &Path,
        paper: impl Into<PaperSetup>,
        texts: Texts,
    ) -> Result<(), ServiceError> {
        let (list, page) = self.printout(paper.into(), texts)?;
        let svg = klappe_export::svg(&list, &page, Theme::Light)?;

        write_file(path, svg.into_bytes())
    }

    /// The display list at the playhead and the sheet around it, with the images the list
    /// names read from the project; what every plan export starts from.
    fn printout(
        &self,
        paper: PaperSetup,
        texts: Texts,
    ) -> Result<(DisplayList, Page), ServiceError> {
        let mut page = Page::new(paper.size, paper.orientation, self.plan_title(), texts);
        let list = self.display_list();
        for item in &list.items {
            let Shape::Image { file, .. } = &item.shape else {
                continue;
            };
            page.images
                .insert(file.clone(), klappe_store::read_image(&self.dir, file)?);
        }

        Ok((list, page))
    }

    /// Writes the plan's shot list as CSV (LH-82); `texts` carries the column heads and the
    /// names of sizes and movements.
    pub fn export_csv(
        &self,
        path: &Path,
        texts: &Texts,
        order: ShotOrder,
    ) -> Result<(), ServiceError> {
        let empty = FloorPlan::new("", "");
        let csv = klappe_export::shots_csv(self.plan().unwrap_or(&empty), &texts.shots, order);

        write_file(path, csv.into_bytes())
    }

    /// Every lamp of the plan on view, hidden ones too: a table lists what the plan holds, as
    /// its shot sheet lists every shot. The printout leaves hidden lamps out (`display_list`).
    fn plan_lamps(&self) -> Vec<Lamp> {
        let (Some(scene), Some(plan)) = (self.scene(), self.plan()) else {
            return Vec::new();
        };
        let Some(location) = self.project.location(&scene.location) else {
            return Vec::new();
        };

        render(&self.project, &location.set, plan, self.time_s, None).lamps
    }

    /// Writes the shot list as ODS (LH-85), the CSV's columns; numbers are numbers. A second sheet
    /// lists the plan's lamps (LH-50); the CSV holds the shot list only.
    pub fn export_ods(
        &self,
        path: &Path,
        texts: &Texts,
        order: ShotOrder,
    ) -> Result<(), ServiceError> {
        let empty = FloorPlan::new("", "");
        let lamps = self.plan_lamps();
        let bytes = klappe_export::shots_ods(
            self.plan().unwrap_or(&empty),
            &texts.shots,
            order,
            &lamps,
            &texts.lamps,
        );

        write_file(path, bytes)
    }

    /// Writes the shot list as XLSX (LH-85), the same as `export_ods`.
    pub fn export_xlsx(
        &self,
        path: &Path,
        texts: &Texts,
        order: ShotOrder,
    ) -> Result<(), ServiceError> {
        let empty = FloorPlan::new("", "");
        let lamps = self.plan_lamps();
        let bytes = klappe_export::shots_xlsx(
            self.plan().unwrap_or(&empty),
            &texts.shots,
            order,
            &lamps,
            &texts.lamps,
        );

        write_file(path, bytes)
    }

    /// Remembers the sheet the plan is exported on in the project, written with the next save.
    /// Not an undo step: it is a setting, and undo leaves it as it is.
    pub fn set_paper(&mut self, paper: PaperSetup) {
        if self.project.paper == paper {
            return;
        }

        self.project.paper = paper;
        self.project_changed = true;
        self.touch();
    }

    /// Writes the plan at every mark of the scene (`marks_s`) as PNG into `dir` (LH-83),
    /// named `<scene>-<nr>-<mm-ss>.png`, to read the blocking at the set. `dir` must be empty or
    /// missing. The playhead stays where it was. Returns the files in mark order.
    pub fn export_frames(
        &mut self,
        dir: &Path,
        paper: impl Into<PaperSetup>,
        texts: &Texts,
    ) -> Result<Vec<PathBuf>, ServiceError> {
        // No marks, no frames: an empty success would name a folder that is not there.
        let marks = self.marks_s();
        if marks.is_empty() {
            return Err(ServiceError::NoMarks);
        }

        // Pictures of an earlier export would mix with these and show blocking that is gone.
        if !klappe_store::is_empty_dir(dir).map_err(write_failed)? {
            return Err(ServiceError::NotEmpty(dir.to_owned()));
        }

        let paper = paper.into();
        let playhead_s = self.time_s;
        let scene = self.scene_number();
        let mut files = Vec::new();

        for (i, at_s) in marks.iter().copied().enumerate() {
            self.set_time(at_s);
            let path = dir.join(frame_name(&scene, i + 1, marks.len(), at_s));
            let written = self.export_png(&path, paper, texts.clone());
            if let Err(e) = written {
                self.set_time(playhead_s);
                return Err(e);
            }
            files.push(path);
        }

        self.set_time(playhead_s);

        Ok(files)
    }
}

/// The scene's number as part of a file name: letters and digits, the rest an underscore.
fn file_part(number: &str) -> String {
    number
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect()
}

/// `<scene>-<nr>-<mm-ss>.png`; `nr` counts the marks from 1, so two marks within one second
/// keep their own file. `nr` is as wide as `count` needs, so names sort like the marks.
fn frame_name(scene: &str, nr: usize, count: usize, at_s: f64) -> String {
    let whole_s = at_s as u64;
    let width = count.to_string().len().max(MIN_NUMBER_WIDTH);

    format!(
        "{}-{nr:0width$}-{:02}-{:02}.png",
        file_part(scene),
        whole_s / SECONDS_PER_MINUTE,
        whole_s % SECONDS_PER_MINUTE
    )
}

/// A failed write names `path`, the file the user chose, not the temp file beside it.
fn write_file(path: &Path, bytes: Vec<u8>) -> Result<(), ServiceError> {
    klappe_store::write_atomic(path, &bytes).map_err(|e| match e {
        klappe_store::StoreError::Io { source, .. } => ServiceError::Write {
            path: path.to_owned(),
            source,
        },
        other => write_failed(other),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Names sort like the marks: the number is as wide as the count needs.
    #[test]
    fn frame_names_sort_with_over_99_marks() {
        let names: Vec<String> = [9, 10, 99, 100, 101]
            .into_iter()
            .map(|nr| frame_name("14", nr, 120, 0.0))
            .collect();
        let mut sorted = names.clone();
        sorted.sort();

        assert_eq!(names, sorted);
        assert_eq!(names[0], "14-009-00-00.png");
        assert_eq!(frame_name("14", 9, 12, 0.0), "14-09-00-00.png");
    }
}
