//! The playback as an MP4 (LH-84): the plan every 1/`VIDEO_FPS` s from 0 to the end of the plan,
//! drawn like the PNG on a landscape A4 and handed to `ffmpeg`. Collecting what each frame
//! shows takes the session; drawing and encoding do not, so a surface can run them on a
//! thread of its own.
//!
//! ```text
//! Session ─► video_job ─► VideoJob { display list per frame }  ─► (thread) run ─► ffmpeg ─► .mp4
//! ```

use std::path::Path;

use klappe_export::{
    video_program_found, Frame, Page, Texts, Theme, VideoSpec, VideoWriter, FFMPEG,
};
use klappe_model::PaperSetup;
use klappe_stellprobe::DisplayList;

use super::Session;
use crate::error::ServiceError;

/// Frames per second of the video.
const VIDEO_FPS: u32 = 25;
/// Full HD; the plan sheet is scaled to fit and centred.
const VIDEO_WIDTH_PX: u32 = 1920;
const VIDEO_HEIGHT_PX: u32 = 1080;

/// What the video shows, ready to draw: one display list (and the fade of its items) per
/// frame, and the sheet they are drawn on.
#[derive(Debug)]
pub struct VideoJob {
    program: String,
    fps: u32,
    page: Page,
    frames: Vec<(DisplayList, Vec<f64>)>,
}

impl VideoJob {
    /// How many frames the video has.
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    /// Draws every frame and writes the video to `path`; `progress` gets the number of
    /// frames done and the total after each. A failure leaves no file.
    pub fn run(
        mut self,
        path: &Path,
        progress: &mut dyn FnMut(usize, usize),
    ) -> Result<(), ServiceError> {
        let total = self.frames.len();
        let mut writer: Option<VideoWriter> = None;

        for (i, (list, fade)) in std::mem::take(&mut self.frames).into_iter().enumerate() {
            self.page.fade = fade;
            let frame = klappe_export::frame(&list, &self.page, Theme::Light)?;
            let video = match writer.as_mut() {
                Some(video) => video,
                None => writer.insert(self.start(path, &frame)?),
            };

            video.push(&frame.pixels)?;
            progress(i + 1, total);
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
    /// Collects the frames of the plan on view for `program` (`FFMPEG` in use); a plan
    /// without duration has none (`NoDuration`), a missing program is `Export(NoVideoProgram)`
    /// before anything is collected. The playhead stays where it was.
    pub fn video_job(&mut self, program: &str, texts: &Texts) -> Result<VideoJob, ServiceError> {
        self.video_job_at(program, texts, VIDEO_FPS)
    }

    /// `video_job` with `fps` frames per second, for a test that need not wait for 25.
    pub fn video_job_at(
        &mut self,
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

        let (_, mut page) = self.printout(PaperSetup::default(), texts.clone())?;
        let playhead_s = self.time_s;
        let count = frame_count(duration_s, fps);
        let mut frames = Vec::with_capacity(count);

        // Frame i shows the plan at i / fps; the last one is the end of the plan.
        for i in 0..count {
            self.set_time(i as f64 / f64::from(fps));
            let list = self.display_list();
            let fade = self.item_fades(&list);
            frames.push((list, fade));
        }
        self.set_time(playhead_s);

        // The images were read once with the page; every frame draws with them.
        page.fade = Vec::new();

        Ok(VideoJob {
            program: program.to_owned(),
            fps,
            page,
            frames,
        })
    }

    /// Writes the playback as MP4 with the installed `ffmpeg`, without progress and in
    /// the caller's thread.
    pub fn export_video(&mut self, path: &Path, texts: &Texts) -> Result<(), ServiceError> {
        self.video_job(FFMPEG, texts)?.run(path, &mut |_, _| {})
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
