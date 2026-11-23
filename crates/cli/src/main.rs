//! `klappe-cli`: Klappe without the GUI (planung/KONZEPT.md, E-10).
//!
//! ```text
//! klappe-cli fov --lens 35 [--sensor <camera id, e.g. super35|aps-c>|<width mm>]
//! klappe-cli export --project <folder> --scene <nr> --out <file> [--plan <id>] [--at <s>]
//!                   [--paper a4|a3|letter] [--orientation landscape|portrait]
//!                   [--language de|en] [--format pdf|svg|png|csv|ods|xlsx|frames|camera|
//!                                                shotpages|camerapages|shotfiles|camerafiles|mp4]
//!                   [--camera <id>]
//! klappe-cli script-pdf --project <folder> --out <file> [--paper a4|letter] [--language de|en]
//!                       [--scene-numbers] [--header]
//! ```

use clap::builder::{PossibleValuesParser, TypedValueParser};
use clap::{Parser, Subcommand, ValueEnum};
use klappe_services::{
    builtin_camera_presets, error_text, load_notes_text, read_number, ExportFormat, ExportOptions,
    Field, HeaderLine, Language, Orientation, Paper, PaperSetup, Range, SceneNumbers, ScriptPaper,
    ScriptPrint, Sensor, Session, ShotOrder, Size, Unit,
};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Debug, Parser)]
#[command(name = "klappe-cli", version, about = "Klappe on the command line")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Horizontal angle of view of a lens on a sensor (LH-31).
    Fov {
        /// Focal length in millimetres.
        #[arg(long)]
        lens: f64,
        /// A camera body's id from the presets (super35, full-frame, aps-c, ...) or the sensor
        /// width in millimetres.
        #[arg(long, default_value = "super35")]
        sensor: String,
    },
    /// The plan of a scene at a time as PDF (with shot list), SVG or PNG (LH-80, LH-81); the shot
    /// list as ODS or XLSX (LH-85); the plan at every mark as PNGs in a folder (LH-83); what a
    /// camera sees at a time as PNG (LH-86); the playback as MP4 (LH-84).
    Export {
        /// The project folder.
        #[arg(long)]
        project: PathBuf,
        /// The scene's number as in the script, e.g. 12 or 12A.
        #[arg(long)]
        scene: String,
        /// The plan's id; the scene's first plan if left out.
        #[arg(long)]
        plan: Option<String>,
        /// Seconds into the scene.
        #[arg(long, default_value_t = 0.0)]
        at: f64,
        /// Sheet size; the project's own (A4 if never chosen) if left out.
        #[arg(long, value_parser = paper_arg)]
        paper: Option<Paper>,
        /// Sheet orientation; the project's own (landscape if never chosen) if left out.
        #[arg(long, value_parser = orientation_arg)]
        orientation: Option<Orientation>,
        /// Language of the legend; the system's if left out.
        #[arg(long, value_enum)]
        language: Option<LanguageArg>,
        /// Taken from the extension of --out if left out; `frames` and `camera` are never guessed.
        /// `frames`: one PNG per mark into the folder --out, --at is ignored. `camera`: what
        /// --camera sees at --at, 1920 px wide. `mp4`: the playback as 1920x1080 video, 25 fps,
        /// written by the installed ffmpeg; --at, --paper and --orientation are ignored.
        #[arg(long, value_parser = format_arg())]
        format: Option<ExportFormat>,
        /// The camera `--format camera` shows.
        #[arg(long)]
        camera: Option<String>,
        /// The file to write; the folder for `--format frames`.
        #[arg(long)]
        out: PathBuf,
    },
    /// The screenplay as PDF in Courier on A4 or US Letter, with a title page from the title
    /// page keys (DB-70).
    ScriptPdf {
        /// The project folder.
        #[arg(long)]
        project: PathBuf,
        /// The PDF to write.
        #[arg(long)]
        out: PathBuf,
        /// Paper of the pages; the project's own (A4 if never chosen) if left out.
        #[arg(long, value_parser = script_paper_arg)]
        paper: Option<ScriptPaper>,
        /// Language of (MORE) and (CONT'D) when the project has none; the system's if left out.
        #[arg(long, value_enum)]
        language: Option<LanguageArg>,
        /// Print the scene numbers at both sides of the headings.
        #[arg(long)]
        scene_numbers: bool,
        /// Print title and draft date at the top of every page.
        #[arg(long)]
        header: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum LanguageArg {
    De,
    En,
}

/// The size of the 3D view on screen; the command line has none and offers no export of it.
const NO_VIEW: Size = (0.0, 0.0);

/// What `export` was asked for.
struct ExportJob {
    project: PathBuf,
    scene: String,
    plan: Option<String>,
    at_s: f64,
    paper: Option<Paper>,
    orientation: Option<Orientation>,
    language: Language,
    format: ExportFormat,
    camera: Option<String>,
    out: PathBuf,
}

fn main() -> ExitCode {
    match Cli::parse().command {
        Command::Fov { lens, sensor } => fov(lens, &sensor),
        Command::Export {
            project,
            scene,
            plan,
            at,
            paper,
            orientation,
            language,
            format,
            camera,
            out,
        } => {
            let language = language_of(language);
            let Some(format) = format.or_else(|| format_of(&out)) else {
                let path = format!("{out:?}");
                return refuse(language.text("cli.noFormat").replace("{path}", &path));
            };
            if format == ExportFormat::Camera && camera.is_none() {
                return refuse(language.text("cli.cameraNeedsId"));
            }
            if format != ExportFormat::Camera && camera.is_some() {
                return refuse(language.text("cli.cameraOnly"));
            }
            let job = ExportJob {
                project,
                scene,
                plan,
                at_s: at,
                paper,
                orientation,
                language,
                format,
                camera,
                out,
            };

            export(&job)
        }
        Command::ScriptPdf {
            project,
            out,
            paper,
            language,
            scene_numbers,
            header,
        } => {
            let print = ScriptPrint {
                scene_numbers: if scene_numbers {
                    SceneNumbers::Shown
                } else {
                    SceneNumbers::Hidden
                },
                header: if header {
                    HeaderLine::Title
                } else {
                    HeaderLine::Off
                },
                language: language_of(language),
            };

            script_pdf(&project, &out, paper, &print)
        }
    }
}

/// Writes the screenplay; `paper` replaces the project's for this one PDF.
fn script_pdf(
    project: &Path,
    out: &Path,
    paper: Option<ScriptPaper>,
    print: &ScriptPrint,
) -> ExitCode {
    let written = write_script_pdf(project, out, paper, print);

    match written {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("klappe-cli: {message}");
            ExitCode::FAILURE
        }
    }
}

fn write_script_pdf(
    project: &Path,
    out: &Path,
    paper: Option<ScriptPaper>,
    print: &ScriptPrint,
) -> Result<(), String> {
    let mut session = Session::open(project).map_err(|e| error_text(print.language, &e))?;
    warn_load_notes(&session, print.language);
    if let Some(paper) = paper {
        // Only in memory: the CLI saves nothing.
        let mut setup = session.project().script;
        setup.paper = paper;
        session.set_script_setup(setup);
    }

    session
        .export_script_pdf(out, print)
        .map_err(|e| error_text(print.language, &e))
}

fn fov(lens_mm: f64, sensor: &str) -> ExitCode {
    let language = Language::pick(None);
    let Some(sensor) = parse_sensor(sensor) else {
        let names = sensor_ids().join(", ");

        return refuse(
            language
                .text("cli.unknownSensor")
                .replace("{sensor}", sensor)
                .replace("{names}", &names),
        );
    };
    if !is_length(lens_mm) {
        let lens = lens_mm.to_string();
        return refuse(language.text("cli.badLens").replace("{value}", &lens));
    }

    let fov = klappe_services::horizontal_fov_deg(sensor, lens_mm);
    println!("{fov:.1}°");
    ExitCode::SUCCESS
}

fn export(job: &ExportJob) -> ExitCode {
    if !job.at_s.is_finite() || job.at_s < 0.0 {
        let at = job.at_s.to_string();
        return refuse(job.language.text("cli.badTime").replace("{value}", &at));
    }

    match write_export(job) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("klappe-cli: {message}");
            ExitCode::FAILURE
        }
    }
}

/// What loaded but is odd in the project goes to stderr; the export goes on.
fn warn_load_notes(session: &Session, language: Language) {
    let notes = load_notes_text(language, session.load_notes());
    if !notes.is_empty() {
        eprintln!("{notes}");
    }
}

fn write_export(job: &ExportJob) -> Result<(), String> {
    let mut session = Session::open(&job.project).map_err(|e| error_text(job.language, &e))?;
    warn_load_notes(&session, job.language);
    let scene_id = session
        .project()
        .scenes
        .iter()
        .find(|s| s.number == job.scene)
        .map(|s| s.id.clone())
        .ok_or_else(|| {
            job.language
                .text("cli.noScene")
                .replace("{number}", &job.scene)
        })?;

    session
        .select_scene(&scene_id)
        .map_err(|e| error_text(job.language, &e))?;
    if let Some(plan) = &job.plan {
        session
            .select_plan(plan)
            .map_err(|e| error_text(job.language, &e))?;
    }
    if session.plan().is_none() {
        return Err(job
            .language
            .text("cli.noPlan")
            .replace("{number}", &job.scene));
    }
    session.set_time(job.at_s);

    // What the options leave out comes from the project.
    let kept = session.project().paper;
    let paper = PaperSetup {
        size: job.paper.unwrap_or(kept.size),
        orientation: job.orientation.unwrap_or(kept.orientation),
    };
    let options = ExportOptions {
        paper,
        order: ShotOrder::Cut,
        texts: job.language.print_texts(),
        shown: NO_VIEW,
        camera: job.camera.clone(),
    };
    session
        .export(job.format, &job.out, &options)
        .map_err(|e| error_text(job.language, &e))
}

fn paper_arg(name: &str) -> Result<Paper, String> {
    let names: Vec<&str> = Paper::ALL.iter().map(|p| p.name()).collect();

    Paper::from_name(name).ok_or_else(|| format!("use one of {}", names.join(", ")))
}

fn script_paper_arg(name: &str) -> Result<ScriptPaper, String> {
    let names: Vec<&str> = ScriptPaper::ALL.iter().map(|p| p.name()).collect();

    ScriptPaper::from_name(name).ok_or_else(|| format!("use one of {}", names.join(", ")))
}

fn orientation_arg(name: &str) -> Result<Orientation, String> {
    let names: Vec<&str> = Orientation::ALL.iter().map(|o| o.name()).collect();

    Orientation::from_name(name).ok_or_else(|| format!("use one of {}", names.join(", ")))
}

/// Says why the command line refuses, in the user's language, and fails.
fn refuse(message: String) -> ExitCode {
    eprintln!("klappe-cli: {message}");
    ExitCode::FAILURE
}

fn language_of(language: Option<LanguageArg>) -> Language {
    match language {
        Some(LanguageArg::De) => Language::German,
        Some(LanguageArg::En) => Language::English,
        None => Language::pick(None),
    }
}

/// The formats by name, listed in --help; the 3D view is left out, the command line has no
/// state for it.
fn format_arg() -> impl TypedValueParser<Value = ExportFormat> {
    let names = ExportFormat::ALL
        .into_iter()
        .filter(|f| *f != ExportFormat::View3d)
        .map(ExportFormat::name);

    PossibleValuesParser::new(names)
        .try_map(|name| ExportFormat::from_name(&name).ok_or("unknown format"))
}

/// The format a file name asks for by its extension (`plan.PNG`). `png` is the plan's, the
/// views and the folder of frames are never guessed.
fn format_of(out: &Path) -> Option<ExportFormat> {
    let extension = out.extension()?.to_str()?.to_lowercase();

    ExportFormat::ALL
        .into_iter()
        .find(|f| f.extension() == Some(extension.as_str()))
}

/// The ids of the camera bodies the window offers.
fn sensor_ids() -> Vec<&'static str> {
    let bodies = &builtin_camera_presets().bodies;

    bodies.iter().map(|body| body.id.as_str()).collect()
}

/// A sensor by a camera body's id (e.g. `aps-c`), or by its width in millimetres (e.g. `23.76`).
fn parse_sensor(name: &str) -> Option<Sensor> {
    let body = builtin_camera_presets()
        .bodies
        .iter()
        .find(|b| b.id == name);
    if let Some(body) = body {
        return Some(Sensor::of_width_mm(body.width_mm));
    }

    read_number(Field::Sensor, name, Unit::Millimetre, Range::Positive)
        .ok()
        .map(|width_mm| Sensor::Custom { width_mm })
}

/// A finite, positive number of millimetres.
fn is_length(mm: f64) -> bool {
    mm.is_finite() && mm > 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sensors_by_name_and_width() {
        assert_eq!(parse_sensor("super35"), Some(Sensor::Super35));
        assert_eq!(
            parse_sensor("23.5"),
            Some(Sensor::Custom { width_mm: 23.5 })
        );
        assert_eq!(
            parse_sensor("23,76"),
            Some(Sensor::Custom { width_mm: 23.76 })
        );
        assert_eq!(
            parse_sensor("36 mm"),
            Some(Sensor::Custom { width_mm: 36.0 })
        );
        assert_eq!(parse_sensor("-1"), None);
        assert_eq!(parse_sensor("s35"), None);
        assert_eq!(parse_sensor("imax"), None);
    }

    /// Audit 19, finding 9: the window offers these, the command line refused them.
    #[test]
    fn every_preset_body_is_a_sensor() {
        assert_eq!(
            parse_sensor("aps-c"),
            Some(Sensor::Custom { width_mm: 23.5 })
        );
        assert_eq!(
            parse_sensor("micro-four-thirds"),
            Some(Sensor::MicroFourThirds)
        );
        assert_eq!(parse_sensor("full-frame"), Some(Sensor::FullFrame));
        for id in sensor_ids() {
            assert!(parse_sensor(id).is_some(), "{id}");
        }
    }

    #[test]
    fn the_screenplay_goes_to_a_pdf() {
        let project = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe");
        let out =
            std::env::temp_dir().join(format!("klappe-cli-script-{}.pdf", std::process::id()));
        let print = ScriptPrint {
            scene_numbers: SceneNumbers::Shown,
            header: HeaderLine::Off,
            language: Language::English,
        };

        let written = write_script_pdf(&project, &out, Some(ScriptPaper::Letter), &print);

        assert_eq!(written, Ok(()));
        let bytes = std::fs::read(&out).unwrap_or_default();
        assert!(bytes.starts_with(b"%PDF"));
        assert!(String::from_utf8_lossy(&bytes).contains("/MediaBox [0 0 612 792]"));
    }

    #[test]
    fn format_follows_the_extension() {
        assert_eq!(format_of(Path::new("a/plan.png")), Some(ExportFormat::Png));
        assert_eq!(format_of(Path::new("plan.SVG")), Some(ExportFormat::Svg));
        assert_eq!(format_of(Path::new("plan.pdf")), Some(ExportFormat::Pdf));
        assert_eq!(format_of(Path::new("list.ods")), Some(ExportFormat::Ods));
        assert_eq!(format_of(Path::new("list.XLSX")), Some(ExportFormat::Xlsx));
        assert_eq!(format_of(Path::new("list.csv")), Some(ExportFormat::Csv));
        assert_eq!(format_of(Path::new("plan.MP4")), Some(ExportFormat::Video));
        assert_eq!(format_of(Path::new("frames")), None);
        assert_eq!(format_of(Path::new("plan.jpg")), None);
        assert_eq!(format_of(Path::new("plan")), None);
    }

    /// Review of PR 1: 0, NaN and infinite values printed an angle and exit 0.
    #[test]
    fn nonsense_values_are_refused() {
        assert_eq!(parse_sensor("inf"), None);
        assert_eq!(parse_sensor("NaN"), None);
        assert_eq!(fov(0.0, "super35"), ExitCode::FAILURE);
        assert_eq!(fov(f64::NAN, "super35"), ExitCode::FAILURE);
    }
}
