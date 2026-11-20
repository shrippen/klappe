//! `klappe-cli export` on the example project (`testdata/beispiel.klappe`). The files go to
//! cargo's per-crate test directory, not `/tmp`.
#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const PNG_SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n";
/// Where the width sits in a PNG file; the height follows.
const PNG_WIDTH_AT: usize = 16;
/// The width of the PNG of a view (LH-86).
const CAMERA_VIEW_WIDTH_PX: u32 = 1920;
const EXAMPLE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../testdata/beispiel.klappe"
);

fn out_file(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"));
    let path = dir.join(name);
    let _ = std::fs::remove_file(&path);
    path
}

/// `klappe-cli export` on the example, scene 12, with `extra` arguments.
fn export(out: &Path, extra: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_klappe-cli"))
        .args(["export", "--project", EXAMPLE, "--scene", "12", "--out"])
        .arg(out)
        .args(extra)
        .output()
        .unwrap()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn exports_a_png_by_extension() {
    let out = out_file("cli-plan.png");

    let output = export(&out, &["--at", "6", "--language", "de"]);

    assert!(output.status.success(), "{}", stderr(&output));
    let bytes = std::fs::read(&out).unwrap();
    assert!(bytes.starts_with(PNG_SIGNATURE));
}

#[test]
fn the_format_option_beats_the_extension() {
    let out = out_file("cli-plan.dat");

    let output = export(&out, &["--format", "svg", "--language", "en"]);

    assert!(output.status.success(), "{}", stderr(&output));
    let text = std::fs::read_to_string(&out).unwrap();
    assert!(text.contains("<svg"));
    assert!(text.contains("Axis"));
}

#[test]
fn exports_a_pdf_on_letter_paper() {
    let out = out_file("cli-plan.pdf");

    let output = export(&out, &["--paper", "letter"]);

    assert!(output.status.success(), "{}", stderr(&output));
    let text = String::from_utf8_lossy(&std::fs::read(&out).unwrap()).into_owned();
    assert!(text.starts_with("%PDF-"));
    assert!(text.contains("/MediaBox [0 0 792 612]"));
}

#[test]
fn exports_a_portrait_pdf() {
    let out = out_file("cli-portrait.pdf");

    let output = export(&out, &["--paper", "a4", "--orientation", "portrait"]);

    assert!(output.status.success(), "{}", stderr(&output));
    let text = String::from_utf8_lossy(&std::fs::read(&out).unwrap()).into_owned();
    assert!(text.contains("/MediaBox [0 0 595.28 841.89]"));
    assert!(
        !text.contains("/MediaBox [0 0 841.89 595.28]"),
        "the shot list is upright too"
    );
}

#[test]
fn an_unknown_paper_is_refused() {
    let out = out_file("cli-tabloid.pdf");

    let output = export(&out, &["--paper", "tabloid"]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("use one of a4, a3, letter"));
    assert!(!out.exists());
}

#[test]
fn an_unknown_scene_fails_without_a_file() {
    let out = out_file("cli-none.png");
    let output = Command::new(env!("CARGO_BIN_EXE_klappe-cli"))
        .args([
            "export",
            "--project",
            EXAMPLE,
            "--scene",
            "99",
            "--language",
            "en",
            "--out",
        ])
        .arg(&out)
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(stderr(&output).contains("klappe-cli: no scene 99"));
    assert!(!out.exists());
}

#[test]
fn an_unknown_plan_fails() {
    let out = out_file("cli-noplan.png");

    let output = export(&out, &["--plan", "nope", "--language", "en"]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("klappe-cli: Plan not found: nope"));
}

/// Audit 3, finding 1: the CLI says what the app says, in its `--language`.
#[test]
fn a_service_error_uses_the_catalogue_text() {
    let out = out_file("cli-noplan-de.png");

    let output = export(&out, &["--plan", "nope", "--language", "de"]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("klappe-cli: Plan nicht gefunden: nope"));
}

#[test]
fn an_unknown_extension_asks_for_a_format() {
    let out = out_file("cli-plan.jpg");

    let output = export(&out, &["--language", "en"]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("use --format"));
}

#[test]
fn a_negative_time_is_refused() {
    let out = out_file("cli-neg.png");

    let output = export(&out, &["--at=-1", "--language", "en"]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("time must be"));
}

#[test]
fn exports_the_shot_list_as_ods_and_xlsx_by_extension() {
    for name in ["cli-list.ods", "cli-list.xlsx"] {
        let out = out_file(name);

        let output = export(&out, &["--language", "de"]);

        assert!(output.status.success(), "{}", stderr(&output));
        let bytes = std::fs::read(&out).unwrap();
        assert_eq!(&bytes[..2], b"PK", "{name}");
        assert!(
            String::from_utf8_lossy(&bytes).contains("Shotliste"),
            "{name}"
        );
    }
}

#[test]
fn exports_the_shot_list_as_csv_by_extension() {
    let out = out_file("cli-list.csv");

    let output = export(&out, &["--language", "de"]);

    assert!(output.status.success(), "{}", stderr(&output));
    let text = std::fs::read_to_string(&out).unwrap();
    assert!(text.starts_with("Nummer,Kamera,"));
}

#[test]
fn exports_the_shot_list_as_csv_by_format() {
    let out = out_file("cli-list-format.dat");

    let output = export(&out, &["--format", "csv", "--language", "de"]);

    assert!(output.status.success(), "{}", stderr(&output));
    let text = std::fs::read_to_string(&out).unwrap();
    assert!(text.starts_with("Nummer,Kamera,"));
}

#[test]
fn the_3d_view_is_no_format() {
    let out = out_file("cli-view.png");

    let output = export(&out, &["--format", "view3d"]);

    assert!(!output.status.success());
    let said = String::from_utf8_lossy(&output.stderr);
    assert!(said.contains("invalid value 'view3d'"), "{said}");
    assert!(!out.exists());
}

#[test]
fn exports_one_picture_per_mark_into_a_folder() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("cli-frames");
    let _ = std::fs::remove_dir_all(&dir);

    let output = export(&dir, &["--format", "frames"]);

    assert!(output.status.success(), "{}", stderr(&output));
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert_eq!(names.len(), 5, "{names:?}");
    assert_eq!(names[0], "12-01-00-00.png");
    assert_eq!(names[4], "12-05-00-20.png");
    let first = std::fs::read(dir.join(&names[0])).unwrap();
    assert!(first.starts_with(PNG_SIGNATURE));
}

/// LH-86: what camera `a` sees at a time, as the PNG the window's "Kamerablick als PNG" writes.
#[test]
fn exports_the_view_of_a_camera() {
    let out = out_file("cli-camera.png");

    let output = export(&out, &["--format", "camera", "--camera", "a", "--at", "6"]);

    assert!(output.status.success(), "{}", stderr(&output));
    let bytes = std::fs::read(&out).unwrap();
    assert!(bytes.starts_with(PNG_SIGNATURE));
    let width = u32::from_be_bytes(bytes[PNG_WIDTH_AT..PNG_WIDTH_AT + 4].try_into().unwrap());
    assert_eq!(width, CAMERA_VIEW_WIDTH_PX);
}

#[test]
fn the_view_of_a_camera_needs_the_camera() {
    let out = out_file("cli-nocamera.png");

    let output = export(&out, &["--format", "camera", "--language", "en"]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("--camera"), "{}", stderr(&output));
    assert!(!out.exists());
}

#[test]
fn an_unknown_camera_fails_without_a_file() {
    let out = out_file("cli-badcamera.png");

    let output = export(&out, &["--format", "camera", "--camera", "zz"]);

    assert!(!output.status.success());
    assert!(!out.exists());
}

#[test]
fn a_camera_goes_with_the_camera_format_only() {
    let out = out_file("cli-stray-camera.png");

    let output = export(&out, &["--camera", "a"]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("--camera"), "{}", stderr(&output));
    assert!(!out.exists());
}

#[test]
fn an_orphan_scene_file_is_told_on_stderr_and_the_export_goes_on() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("cli-orphan.klappe");
    let _ = std::fs::remove_dir_all(&dir);
    copy_dir(Path::new(EXAMPLE), &dir);
    std::fs::write(dir.join("szenen.toml"), "").unwrap();
    let out = out_file("cli-orphan.svg");

    let output = Command::new(env!("CARGO_BIN_EXE_klappe-cli"))
        .args(["export", "--project"])
        .arg(&dir)
        .args(["--scene", ""])
        .arg("--out")
        .arg(&out)
        .args(["--language", "en"])
        .output()
        .unwrap();

    assert!(
        stderr(&output).contains("without a register entry"),
        "{}",
        stderr(&output)
    );
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        match entry.file_type().unwrap().is_dir() {
            true => copy_dir(&entry.path(), &target),
            false => drop(std::fs::copy(entry.path(), target).unwrap()),
        }
    }
}

/// LH-84: without ffmpeg the command says so and writes nothing. With it, the whole example
/// is 500 frames, too slow for a debug build; `klappe-services` tests the writing.
#[test]
fn the_video_needs_ffmpeg() {
    if Command::new("ffmpeg").arg("-version").output().is_ok() {
        eprintln!("skipped: ffmpeg is installed; the video itself is tested in klappe-services");
        return;
    }
    let out = out_file("cli-video.mp4");

    let output = export(&out, &["--format", "mp4"]);

    assert!(!output.status.success());
    assert!(stderr(&output).contains("ffmpeg"), "{}", stderr(&output));
    assert!(!out.exists());
}

/// agent.md, no texts in code: the command line's own messages are in its `--language` too.
#[test]
fn the_command_lines_own_errors_are_in_german() {
    let out = out_file("cli-de.png");

    let scene = Command::new(env!("CARGO_BIN_EXE_klappe-cli"))
        .args([
            "export",
            "--project",
            EXAMPLE,
            "--scene",
            "99",
            "--language",
            "de",
            "--out",
        ])
        .arg(&out)
        .output()
        .unwrap();
    let time = export(&out, &["--at=-1", "--language", "de"]);
    let format = export(&out_file("cli-de.jpg"), &["--language", "de"]);

    assert!(
        stderr(&scene).contains("klappe-cli: Keine Szene 99"),
        "{}",
        stderr(&scene)
    );
    assert!(stderr(&time).contains("Sekunden"), "{}", stderr(&time));
    assert!(
        stderr(&format).contains("--format angeben"),
        "{}",
        stderr(&format)
    );
}

/// Review of PR 297: the other refusals in German too, and `fov` follows `LANG`.
#[test]
fn the_other_refusals_are_in_german() {
    let camera_needs_id = export(
        &out_file("cli-cam.png"),
        &["--format", "camera", "--language", "de"],
    );
    let camera_only = export(
        &out_file("cli-only.png"),
        &["--camera", "k1", "--language", "de"],
    );
    let fov = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_klappe-cli"))
            .arg("fov")
            .args(args)
            .env("LANG", "de_DE.UTF-8")
            .output()
            .unwrap()
    };
    let sensor = fov(&["--lens", "35", "--sensor", "riesig"]);
    let lens = fov(&["--lens", "0"]);

    assert!(
        stderr(&camera_needs_id).contains("braucht --camera"),
        "{}",
        stderr(&camera_needs_id)
    );
    assert!(
        stderr(&camera_only).contains("gehört nur zu"),
        "{}",
        stderr(&camera_only)
    );
    assert!(
        stderr(&sensor).contains("Unbekannter Sensor riesig"),
        "{}",
        stderr(&sensor)
    );
    assert!(stderr(&lens).contains("Brennweite"), "{}", stderr(&lens));
}

/// Every catalogue key the command line names is in both catalogues; a missing one would print
/// the key itself (no scene with a plan missing is in the example to show `cli.noPlan`).
#[test]
fn every_cli_key_is_in_both_catalogues() {
    let source = include_str!("../src/main.rs");
    let keys: Vec<&str> = source
        .split('"')
        .filter(|s| s.starts_with("cli.") && !s.contains(' '))
        .collect();
    let catalogues = [
        include_str!("../../services/i18n/de.json"),
        include_str!("../../services/i18n/en.json"),
    ];

    assert!(keys.len() >= 8, "{keys:?}");
    for key in keys {
        for catalogue in catalogues {
            assert!(catalogue.contains(&format!("\"{key}\":")), "{key} missing");
        }
    }
}
