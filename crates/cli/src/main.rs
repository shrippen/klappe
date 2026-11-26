//! `klappe-cli`: Klappe without the GUI (planung/KONZEPT.md, E-10).
//!
//! ```text
//! klappe-cli fov --lens 35 [--sensor <camera id, e.g. super35|aps-c>|<width mm>]
//! klappe-cli export --project <folder> --scene <nr> --out <file> [--plan <id>] [--at <s>]
//!                   [--paper a4|a3|letter] [--orientation landscape|portrait]
//!                   [--language de|en] [--format pdf|svg|png|csv|ods|xlsx|frames|camera|
//!                                                shotpages|camerapages|shotfiles|camerafiles|mp4]
//!                   [--camera <id>]
//! klappe-cli report --project <folder> --report <szenen|spielorte|figuren|auszug|kontakte> --out <file>
//!                   [--language de|en]      (pdf, csv, ods or xlsx, by the extension of --out)
//! klappe-cli script-pdf --project <folder> --out <file> [--paper a4|letter] [--language de|en]
//!                       [--scene-numbers] [--header] [--revisions] [--changed-pages]
//! klappe-cli script-fdx --project <folder> --out <file> [--language de|en]
//! klappe-cli sides --project <folder> --figure <name> --out <file> [--language de|en]
//! klappe-cli rename-figure --project <folder> --from <name> --to <name> [--language de|en]
//! klappe-cli lock-version --project <folder> --name <name> [--language de|en]
//! klappe-cli remove-version --project <folder> --id <version id> [--language de|en]
//! klappe-cli versions --project <folder> [--language de|en]
//! klappe-cli compare --project <folder> [--from <version id>] [--to <version id>] [--language de|en]
//! ```

mod versions;

use clap::builder::{PossibleValuesParser, TypedValueParser};
use clap::{Args, Parser, Subcommand, ValueEnum};
use klappe_services::{
    builtin_camera_presets, error_text, fdx_loss_text, load_notes_text, read_number, ExportFormat,
    ExportOptions, Field, HeaderLine, Language, OnOutside, Orientation, Paper, PaperSetup, Range,
    Report, ReportFormat, Revisions, Saved, SceneNumbers, ScriptPaper, ScriptPrint, Sensor,
    Session, ShotOrder, Size, Unit,
};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Debug, Parser)]
#[command(name = "klappe-cli", version, about = "Klappe on the command line")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

/// The flags of `export`.
#[derive(Debug, Args)]
struct ExportArgs {
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
    Export(ExportArgs),
    /// A report (DB-61) as PDF, CSV, ODS or XLSX, by the extension of --out: the scenes, the
    /// scripted locations with their shooting locations, the characters by scene, or the
    /// extract of the breakdown.
    Report {
        /// The project folder.
        #[arg(long)]
        project: PathBuf,
        /// `szenen`, `spielorte`, `figuren`, `auszug` or `kontakte`.
        #[arg(long, value_parser = report_arg())]
        report: Report,
        /// Language of the headings; the system's if left out.
        #[arg(long, value_enum)]
        language: Option<LanguageArg>,
        /// The file to write: .pdf, .csv, .ods or .xlsx.
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
        /// Mark what changed since the last locked version: `*` in the margin and the next
        /// revision's colour above the page number.
        #[arg(long)]
        revisions: bool,
        /// Print only the pages that carry a revision mark (DB-43), under their locked
        /// numbers, without a title page; implies --revisions.
        #[arg(long)]
        changed_pages: bool,
    },
    /// The sides of a character as PDF (DB-62): the scenes it speaks in, whole, with their
    /// scene numbers, the character's name above every page.
    Sides {
        /// The project folder.
        #[arg(long)]
        project: PathBuf,
        /// The character's name or an alias, or a cue of the book.
        #[arg(long)]
        figure: String,
        /// The PDF to write.
        #[arg(long)]
        out: PathBuf,
        /// Language of (MORE) and (CONT'D) when the project has none; the system's if left out.
        #[arg(long, value_enum)]
        language: Option<LanguageArg>,
    },
    /// Renames a character in the whole book (DB-04): every cue it answers to, in
    /// `figuren.toml` too; its id, colour and short code stay. Saves the project.
    RenameFigure {
        /// The project folder.
        #[arg(long)]
        project: PathBuf,
        /// The character's name or one of its other names.
        #[arg(long)]
        from: String,
        /// The new name.
        #[arg(long)]
        to: String,
        /// Language of the output; the system's if left out.
        #[arg(long, value_enum)]
        language: Option<LanguageArg>,
    },
    /// The screenplay as a Final Draft file (DB-72); says on stderr what FDX has no place for.
    ScriptFdx {
        /// The project folder.
        #[arg(long)]
        project: PathBuf,
        /// The FDX file to write.
        #[arg(long)]
        out: PathBuf,
        /// Language of the note on what was left out; the system's if left out.
        #[arg(long, value_enum)]
        language: Option<LanguageArg>,
    },
    #[command(flatten)]
    Versions(versions::VersionCommand),
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
        Command::Export(args) => export_command(args),
        Command::Report {
            project,
            report,
            language,
            out,
        } => write_report(&project, report, &out, language_of(language)),
        Command::ScriptPdf {
            project,
            out,
            paper,
            language,
            scene_numbers,
            header,
            revisions,
            changed_pages,
        } => {
            let flags = PrintFlags {
                scene_numbers,
                header,
                revisions,
                changed_pages,
            };
            let print = print_of(language_of(language), flags);

            script_pdf(&project, &out, paper, &print)
        }
        Command::ScriptFdx {
            project,
            out,
            language,
        } => script_fdx(&project, &out, language_of(language)),
        Command::RenameFigure {
            project,
            from,
            to,
            language,
        } => rename_figure(&project, (&from, &to), language_of(language)),
        Command::Sides {
            project,
            figure,
            out,
            language,
        } => sides_pdf(&project, &figure, &out, language_of(language)),
        Command::Versions(command) => command.run(),
    }
}

/// Checks the export flags and runs the export.
fn export_command(args: ExportArgs) -> ExitCode {
    let ExportArgs {
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
    } = args;
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

/// Renames the character `from` to `to` in the book and saves the project.
fn rename_figure(project: &Path, (from, to): (&str, &str), language: Language) -> ExitCode {
    let renamed = Session::open(project)
        .and_then(|mut s| {
            let count = s.rename_figure(from, to)?;
            let saved = s.save(OnOutside::Ask)?;

            Ok((count, saved))
        })
        .map_err(|e| error_text(language, &e));

    match renamed {
        Ok((count, Saved::Done)) => {
            let text = language
                .text("cli.figureRenamed")
                .replace("{from}", from)
                .replace("{to}", to.trim())
                .replace("{count}", &count.to_string());
            println!("{text}");

            ExitCode::SUCCESS
        }
        Ok(_) => refuse(language.text("cli.renameConflict")),
        Err(message) => refuse(message),
    }
}

/// The flags of `script-pdf` that switch something on.
struct PrintFlags {
    scene_numbers: bool,
    header: bool,
    revisions: bool,
    changed_pages: bool,
}

/// What `script-pdf` prints, from its flags.
fn print_of(language: Language, flags: PrintFlags) -> ScriptPrint {
    ScriptPrint {
        scene_numbers: if flags.scene_numbers {
            SceneNumbers::Shown
        } else {
            SceneNumbers::Hidden
        },
        header: if flags.header {
            HeaderLine::Title
        } else {
            HeaderLine::Off
        },
        language,
        revisions: if flags.changed_pages {
            Revisions::ChangedPages
        } else if flags.revisions {
            Revisions::On
        } else {
            Revisions::Off
        },
    }
}

/// Writes `report`; the format is the extension's.
fn write_report(project: &Path, report: Report, out: &Path, language: Language) -> ExitCode {
    let extension = out.extension().and_then(|e| e.to_str()).unwrap_or_default();
    let Some(format) = ReportFormat::from_extension(extension) else {
        let path = format!("{out:?}");
        return refuse(language.text("cli.noReportFormat").replace("{path}", &path));
    };

    let written = Session::open(project)
        .map_err(|e| error_text(language, &e))
        .and_then(|mut session| {
            warn_load_notes(&session, language);

            session
                .export_report(report, format, out, language)
                .map_err(|e| error_text(language, &e))
        });
    match written {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => refuse(message),
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

/// Writes the sides of `figure`; the scene numbers are always printed.
fn sides_pdf(project: &Path, figure: &str, out: &Path, language: Language) -> ExitCode {
    let print = ScriptPrint {
        scene_numbers: SceneNumbers::Shown,
        header: HeaderLine::Off,
        language,
        revisions: Revisions::Off,
    };
    let written = Session::open(project)
        .map_err(|e| error_text(language, &e))
        .and_then(|mut session| {
            warn_load_notes(&session, language);

            session
                .export_sides_pdf(out, figure, &print)
                .map_err(|e| error_text(language, &e))
        });

    match written {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => refuse(message),
    }
}

fn script_fdx(project: &Path, out: &Path, language: Language) -> ExitCode {
    match write_script_fdx(project, out, language) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => refuse(message),
    }
}

/// Writes the screenplay as FDX and notes on stderr what the file lacks.
fn write_script_fdx(project: &Path, out: &Path, language: Language) -> Result<(), String> {
    let session = Session::open(project).map_err(|e| error_text(language, &e))?;
    warn_load_notes(&session, language);

    let loss = session
        .export_script_fdx(out)
        .map_err(|e| error_text(language, &e))?;
    let note = fdx_loss_text(language, &loss);
    if !note.is_empty() {
        eprintln!("{note}");
    }

    Ok(())
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

/// The reports by name, listed in --help.
fn report_arg() -> impl TypedValueParser<Value = Report> {
    PossibleValuesParser::new(Report::ALL.map(Report::name))
        .try_map(|name| Report::from_name(&name).ok_or("unknown report"))
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
            revisions: Revisions::Off,
        };

        let written = write_script_pdf(&project, &out, Some(ScriptPaper::Letter), &print);

        assert_eq!(written, Ok(()));
        let bytes = std::fs::read(&out).unwrap_or_default();
        let _ = std::fs::remove_file(&out);
        assert!(bytes.starts_with(b"%PDF"));
        assert!(String::from_utf8_lossy(&bytes).contains("/MediaBox [0 0 612 792]"));
    }

    #[test]
    fn the_screenplay_goes_to_an_fdx_file() {
        let project = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../testdata/beispiel.klappe");
        let out =
            std::env::temp_dir().join(format!("klappe-cli-script-{}.fdx", std::process::id()));

        let written = write_script_fdx(&project, &out, Language::English);

        assert_eq!(written, Ok(()));
        let text = std::fs::read_to_string(&out).unwrap_or_default();
        let _ = std::fs::remove_file(&out);
        assert!(text.contains("<FinalDraft DocumentType=\"Script\""));
        assert!(text.contains("Type=\"Scene Heading\""));
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
