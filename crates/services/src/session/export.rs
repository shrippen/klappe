use std::path::{Path, PathBuf};

use klappe_export::{Orientation, Page, Paper, Texts, Theme};
use klappe_model::FloorPlan;
use klappe_stellprobe::{DisplayList, Shape, ShotOrder};

use super::Session;
use crate::error::{write_failed, ServiceError};

const SECONDS_PER_MINUTE: u64 = 60;

impl Session {
    /// Prints the current plan at the playhead as PDF (LH-80), followed by its shot list;
    /// `title` names scene and plan.
    pub fn export_pdf(
        &self,
        path: &Path,
        paper: Paper,
        texts: Texts,
        order: ShotOrder,
    ) -> Result<(), ServiceError> {
        let (list, page) = self.printout(paper, texts)?;
        let bytes = match self.plan() {
            Some(plan) => klappe_export::pdf_with_shots(&list, &page, Theme::Light, plan, order)?,
            None => klappe_export::pdf(&list, &page, Theme::Light)?,
        };

        write_file(path, bytes)
    }

    /// Writes the current plan at the playhead as PNG (LH-81): RGB, 150 dpi, no shot list.
    pub fn export_png(&self, path: &Path, paper: Paper, texts: Texts) -> Result<(), ServiceError> {
        let (list, page) = self.printout(paper, texts)?;
        let bytes = klappe_export::png(&list, &page, Theme::Light)?;

        write_file(path, bytes)
    }

    /// Writes the current plan as a PNG without the sheet title, for the start page's tile.
    pub fn export_preview(&self, path: &Path, texts: Texts) -> Result<(), ServiceError> {
        let (list, mut page) = self.printout(Paper::A4, texts)?;
        page.title.clear();
        let bytes = klappe_export::png(&list, &page, Theme::Light)?;

        write_file(path, bytes)
    }

    /// Writes the current plan at the playhead as SVG (LH-81), without the shot list.
    pub fn export_svg(&self, path: &Path, paper: Paper, texts: Texts) -> Result<(), ServiceError> {
        let (list, page) = self.printout(paper, texts)?;
        let svg = klappe_export::svg(&list, &page, Theme::Light)?;

        write_file(path, svg.into_bytes())
    }

    /// The display list at the playhead and the sheet around it, with the images the list
    /// names read from the project; what every plan export starts from.
    fn printout(&self, paper: Paper, texts: Texts) -> Result<(DisplayList, Page), ServiceError> {
        let mut page = Page::new(paper, Orientation::Landscape, self.plan_title(), texts);
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

    /// Writes the shot list as ODS (LH-85), the CSV's columns; numbers are numbers.
    pub fn export_ods(
        &self,
        path: &Path,
        texts: &Texts,
        order: ShotOrder,
    ) -> Result<(), ServiceError> {
        let empty = FloorPlan::new("", "");
        let bytes = klappe_export::shots_ods(self.plan().unwrap_or(&empty), &texts.shots, order);

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
        let bytes = klappe_export::shots_xlsx(self.plan().unwrap_or(&empty), &texts.shots, order);

        write_file(path, bytes)
    }

    /// Writes the plan at every mark of the scene (`marks_s`) as PNG into `dir` (LH-83),
    /// named `<scene>-<nr>-<mm-ss>.png`, to read the blocking at the set. `dir` must be empty or
    /// missing. The playhead stays where it was. Returns the files in mark order.
    pub fn export_frames(
        &mut self,
        dir: &Path,
        paper: Paper,
        texts: &Texts,
    ) -> Result<Vec<PathBuf>, ServiceError> {
        // Pictures of an earlier export would mix with these and show blocking that is gone.
        if !klappe_store::is_empty_dir(dir).map_err(write_failed)? {
            return Err(ServiceError::NotEmpty(dir.to_owned()));
        }

        let playhead_s = self.time_s;
        let scene = self.scene_number();
        let mut files = Vec::new();

        for (i, at_s) in self.marks_s().into_iter().enumerate() {
            self.set_time(at_s);
            let path = dir.join(frame_name(&scene, i + 1, at_s));
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
/// keep their own file.
fn frame_name(scene: &str, nr: usize, at_s: f64) -> String {
    let whole_s = at_s as u64;

    format!(
        "{}-{nr:02}-{:02}-{:02}.png",
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
