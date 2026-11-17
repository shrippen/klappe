//! The playback as an MP4 through ffmpeg (LH-84).
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};
use std::process::Command;

use klappe_services::{video_frame_count, Language, ServiceError, Session};

/// Few frames: the drawing is slow in a debug build.
const TEST_FPS: u32 = 1;

fn open(name: &str) -> (Session, PathBuf) {
    let example = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe");
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    copy_dir(&example, &dir);
    (Session::open(&dir).unwrap(), dir)
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
            continue;
        }
        std::fs::copy(entry.path(), target).unwrap();
    }
}

fn installed(program: &str) -> bool {
    Command::new(program)
        .arg("-version")
        .output()
        .is_ok_and(|o| o.status.success())
}

/// The frames of the first video stream, counted by ffprobe.
fn frames_in(video: &Path) -> usize {
    let out = Command::new("ffprobe")
        .args(["-v", "error", "-count_frames", "-select_streams", "v:0"])
        .args([
            "-show_entries",
            "stream=nb_read_frames,width,height,pix_fmt,codec_name",
        ])
        .args(["-of", "default=noprint_wrappers=1"])
        .arg(video)
        .output()
        .unwrap();
    let text = String::from_utf8(out.stdout).unwrap();

    assert!(text.contains("codec_name=h264"), "{text}");
    assert!(
        text.contains("width=1920") && text.contains("height=1080"),
        "{text}"
    );
    assert!(text.contains("pix_fmt=yuv420p"), "{text}");

    text.lines()
        .find_map(|l| l.strip_prefix("nb_read_frames="))
        .unwrap()
        .parse()
        .unwrap()
}

#[test]
fn the_playback_becomes_an_mp4_with_a_frame_per_tick() {
    if !installed("ffmpeg") || !installed("ffprobe") {
        eprintln!("skipped: ffmpeg or ffprobe is not installed");
        return;
    }
    let (mut s, dir) = open("video");
    let out = dir.join("plan.mp4");
    s.set_time(1.5);

    let mut seen = Vec::new();
    let job = s
        .video_job_at("ffmpeg", &Language::English.print_texts(), TEST_FPS)
        .unwrap();
    let count = job.frame_count();
    job.run(&out, &mut |done, total| seen.push((done, total)))
        .unwrap();

    assert_eq!(count, video_frame_count(s.duration_s(), TEST_FPS));
    assert_eq!(seen.last(), Some(&(count, count)));
    assert_eq!(frames_in(&out), count);
    assert_eq!(s.time_s(), 1.5, "the playhead stays");
    assert!(!out.with_extension("mp4.part").exists());
}

#[test]
fn without_ffmpeg_nothing_is_written() {
    let (mut s, dir) = open("video-missing");
    let out = dir.join("plan.mp4");

    let result = s.video_job("klappe-no-such-ffmpeg", &Language::English.print_texts());

    assert!(matches!(
        result,
        Err(ServiceError::Export(
            klappe_export::ExportError::NoVideoProgram(_)
        ))
    ));
    assert!(!out.exists());
    let text = klappe_services::error_text(Language::German, &result.unwrap_err());
    assert!(text.contains("ffmpeg"), "{text}");
}
