//! The playback as an MP4 (LH-84): the plan every 1/`VIDEO_FPS` s from 0 to the end of the plan,
//! drawn like the PNG on a landscape A4 and handed to `ffmpeg`. The job copies what a frame
//! needs from the session, so a surface can run it on a thread of its own.
//!
//! ```text
//! Session ─► video_job ─► VideoJob { project, PlanView }
//!                              (thread) run: frame i ─► display list ─► pixels ─► ffmpeg ─► .mp4
//! ```
//!
//! A frame is made when it is written, one at a time: what the job holds does not grow with
//! the length of the plan.

use std::ops::ControlFlow;
use std::path::Path;

use klappe_export::{
    video_program_found, Frame, Page, Texts, Theme, VideoSpec, VideoWriter, FFMPEG,
};
use klappe_model::{PaperSetup, Project};
use klappe_stellprobe::Shape;

use super::plan_view::PlanView;
use super::Session;
use crate::error::ServiceError;

/// Frames per second of the video.
const VIDEO_FPS: u32 = 25;
/// Full HD; the plan sheet is scaled to fit and centred.
const VIDEO_WIDTH_PX: u32 = 1920;
const VIDEO_HEIGHT_PX: u32 = 1080;

/// The plan as the session shows it, copied: its project and view settings, and the sheet. A frame is drawn from it on demand.
#[derive(Debug)]
pub struct VideoJob {
    program: String,
    fps: u32,
    count: usize,
    project: Project,
    scene: usize,
    plan: usize,
    view: PlanView,
    page: Page,
}

impl VideoJob {
    /// How many frames the video has.
    pub fn frame_count(&self) -> usize {
        self.count
    }

    /// Frame `i` (0-based), drawn as the PNG of the plan at `i / fps` s.
    pub fn frame(&mut self, i: usize) -> Result<Frame, ServiceError> {
        let t_s = i as f64 / f64::from(self.fps);
        let at = (self.scene, self.plan);
        let list = self.view.list(&self.project, at, t_s, None);

        // The images of the list are read once; a later frame may name another.
        for item in &list.items {
            let Shape::Image { file, .. } = &item.shape else {
                continue;
            };
            if self.page.images.contains_key(file) {
                continue;
            }
            let bytes = klappe_store::read_image(&self.view.dir, file)?;
            self.page.images.insert(file.clone(), bytes);
        }
        self.page.fade = self.view.fades(&list);

        Ok(klappe_export::frame(&list, &self.page, Theme::Light)?)
    }

    /// Draws every frame and writes the video to `path`; `progress` gets the number of
    /// frames done and the total after each, and stops the video by returning `Break`
    /// (`Cancelled`). A failure or a stop leaves no file.
    pub fn run(
        mut self,
        path: &Path,
        progress: &mut dyn FnMut(usize, usize) -> ControlFlow<()>,
    ) -> Result<(), ServiceError> {
        let mut writer: Option<VideoWriter> = None;

        for i in 0..self.count {
            let frame = self.frame(i)?;
            let video = match writer.as_mut() {
                Some(video) => video,
                None => writer.insert(self.start(path, &frame)?),
            };

            video.push(&frame.pixels)?;
            if progress(i + 1, self.count).is_break() {
                // The writer is dropped: ffmpeg stops and the part file goes.
                return Err(ServiceError::Cancelled);
            }
        }

        match writer {
            Some(video) => Ok(video.finish()?),
            None => Err(ServiceError::NoDuration),
        }
    }

    /// Starts the encoder for frames of the size of `first`; the colour of its first pixel is
    /// the paper, which fills what the scaling leaves around the sheet.
    fn start(&self, path: &Path, first: &Frame) -> Result<VideoWriter, ServiceError> {
        let spec = VideoSpec {
            width: VIDEO_WIDTH_PX,
            height: VIDEO_HEIGHT_PX,
            fps: self.fps,
        };
        let paper = [first.pixels[0], first.pixels[1], first.pixels[2]];

        Ok(VideoWriter::start(
            &self.program,
            path,
            spec,
            (first.width, first.height),
            paper,
        )?)
    }
}

impl Session {
    /// The job for the plan on view and `program` (`FFMPEG` in use); a plan without
    /// duration has none (`NoDuration`), a missing program is `Export(NoVideoProgram)`. Cheap:
    /// no frame is drawn yet.
    pub fn video_job(&self, program: &str, texts: &Texts) -> Result<VideoJob, ServiceError> {
        self.video_job_at(program, texts, VIDEO_FPS)
    }

    /// `video_job` with `fps` frames per second, for a test that need not wait for 25.
    pub fn video_job_at(
        &self,
        program: &str,
        texts: &Texts,
        fps: u32,
    ) -> Result<VideoJob, ServiceError> {
        if !video_program_found(program) {
            return Err(klappe_export::ExportError::NoVideoProgram(program.to_owned()).into());
        }

        let duration_s = self.duration_s();
        if duration_s <= 0.0 {
            return Err(ServiceError::NoDuration);
        }

        let setup = PaperSetup::default();

        Ok(VideoJob {
            program: program.to_owned(),
            fps,
            count: frame_count(duration_s, fps),
            project: self.project.clone(),
            scene: self.scene,
            plan: self.plan,
            view: self.plan_view.clone(),
            page: Page::new(
                setup.size,
                setup.orientation,
                self.plan_title(),
                texts.clone(),
            ),
        })
    }

    /// Writes the playback as MP4 with the installed `ffmpeg`, without progress and in
    /// the caller's thread.
    pub fn export_video(&self, path: &Path, texts: &Texts) -> Result<(), ServiceError> {
        let mut go_on = |_, _| ControlFlow::Continue(());

        self.video_job(FFMPEG, texts)?.run(path, &mut go_on)
    }
}

/// The frames of `duration_s` at `fps`: one every 1/`fps` s, the first at 0 and the last at the end.
pub fn frame_count(duration_s: f64, fps: u32) -> usize {
    (duration_s * f64::from(fps)).floor() as usize + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_frame_per_tick_with_the_end_included() {
        assert_eq!(frame_count(20.0, VIDEO_FPS), 501);
        assert_eq!(frame_count(0.5, VIDEO_FPS), 13);
    }
}
