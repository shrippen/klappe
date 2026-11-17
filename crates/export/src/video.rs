//! Writes frames as an MP4 (H.264) through an installed `ffmpeg` (LH-84): the frames go to
//! its standard input as raw RGB, so no file but the video is written.
//!
//! ```text
//! Frame ─► ffmpeg stdin (rawvideo rgb24) ─► scale, pad, yuv420p ─► libx264 ─► <path>.part ─► <path>
//! ```

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};

use crate::error::ExportError;

/// The program that writes the video; found on `PATH`.
pub const FFMPEG: &str = "ffmpeg";
const PART_EXTENSION: &str = "part";
const RGB_CHANNELS: usize = 3;

/// The video: its size (both even, as yuv420p needs) and its frame rate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VideoSpec {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
}

/// Whether `program` starts; a missing one is the common case, so asking costs one `-version`.
pub fn video_program_found(program: &str) -> bool {
    Command::new(program)
        .arg("-version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

/// A running `ffmpeg`. The video goes to `<path>.part` and is renamed when it is whole, so a
/// failed export leaves no half video under the name the user chose.
#[derive(Debug)]
pub struct VideoWriter {
    child: Child,
    stdin: Option<ChildStdin>,
    part: PathBuf,
    path: PathBuf,
    frame_bytes: usize,
}

impl VideoWriter {
    /// Starts `program` for frames of `input` pixels, scaled to fit `spec` and centred on
    /// `pad` (RGB, the paper). `NoVideoProgram` if it is not installed.
    pub fn start(
        program: &str,
        path: &Path,
        spec: VideoSpec,
        input: (u32, u32),
        pad: [u8; RGB_CHANNELS],
    ) -> Result<Self, ExportError> {
        let part = path.with_extension(format!(
            "{}.{PART_EXTENSION}",
            path.extension()
                .and_then(|e| e.to_str())
                .unwrap_or_default()
        ));
        let filter = format!(
            "scale={w}:{h}:force_original_aspect_ratio=decrease,\
             pad={w}:{h}:(ow-iw)/2:(oh-ih)/2:color=0x{:02x}{:02x}{:02x},format=yuv420p",
            pad[0],
            pad[1],
            pad[2],
            w = spec.width,
            h = spec.height
        );
        let mut command = Command::new(program);
        command
            .args([
                "-y",
                "-loglevel",
                "error",
                "-f",
                "rawvideo",
                "-pix_fmt",
                "rgb24",
            ])
            .args(["-s", &format!("{}x{}", input.0, input.1)])
            .args(["-framerate", &spec.fps.to_string(), "-i", "-"])
            .args(["-vf", &filter, "-c:v", "libx264", "-pix_fmt", "yuv420p"])
            .args(["-f", "mp4"])
            .arg(&part)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped());

        let mut child = command.spawn().map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => ExportError::NoVideoProgram(program.to_owned()),
            _ => ExportError::Video(e.to_string()),
        })?;
        let stdin = child.stdin.take();

        Ok(Self {
            child,
            stdin,
            part,
            path: path.to_owned(),
            frame_bytes: input.0 as usize * input.1 as usize * RGB_CHANNELS,
        })
    }

    /// Hands on the next frame, `3 * width * height` bytes as given to `start`.
    pub fn push(&mut self, rgb: &[u8]) -> Result<(), ExportError> {
        if rgb.len() != self.frame_bytes {
            self.abort();
            return Err(ExportError::Video("frame of another size".to_owned()));
        }

        let written = self.stdin.as_mut().map(|s| s.write_all(rgb));
        if let Some(Err(_)) = written {
            // ffmpeg stopped reading: its own message says why.
            return Err(self.fail());
        }

        Ok(())
    }

    /// Ends the input and waits until the video is written; then it has its name.
    pub fn finish(mut self) -> Result<(), ExportError> {
        drop(self.stdin.take());
        let status = self
            .child
            .wait()
            .map_err(|e| ExportError::Video(e.to_string()))?;
        if !status.success() {
            return Err(self.fail());
        }

        std::fs::rename(&self.part, &self.path).map_err(|e| ExportError::Video(e.to_string()))
    }

    /// Gives up: stops the program and removes the part file; returns what it said.
    pub fn abort(&mut self) -> String {
        drop(self.stdin.take());
        let _ = self.child.kill();
        let mut said = String::new();
        if let Some(mut err) = self.child.stderr.take() {
            let _ = err.read_to_string(&mut said);
        }
        let _ = self.child.wait();
        let _ = std::fs::remove_file(&self.part);

        said.trim().to_owned()
    }

    fn fail(&mut self) -> ExportError {
        ExportError::Video(self.abort())
    }
}

impl Drop for VideoWriter {
    /// A writer dropped before `finish` (an error on the way) leaves no part file.
    fn drop(&mut self) {
        if self.part.exists() {
            self.abort();
        }
    }
}
