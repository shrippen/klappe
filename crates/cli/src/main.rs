//! `klappe-cli`: Klappe without the GUI (planung/KONZEPT.md, E-10).
//!
//! ```text
//! klappe-cli fov --lens 35 [--sensor super35|full-frame|mft|<width mm>]
//! klappe-cli export --project <folder> --scene <nr> --out <file> [--plan <id>] [--at <s>]
//!                   [--paper a4|a3|letter] [--language de|en] [--format pdf|svg|png]
//! ```

use clap::{Parser, Subcommand, ValueEnum};
use klappe_services::{
    read_number, Field, Language, Paper, Range, Sensor, Session, ShotOrder, Unit,
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
        /// super35, full-frame, mft or the sensor width in millimetres.
        #[arg(long, default_value = "super35")]
        sensor: String,
    },
    /// The plan of a scene at a time as PDF (with shot list), SVG or PNG (LH-80, LH-81).
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
        #[arg(long, value_enum, default_value = "a4")]
        paper: PaperArg,
        /// Language of the legend; the system's if left out.
        #[arg(long, value_enum)]
        language: Option<LanguageArg>,
        /// Taken from the extension of --out if left out.
        #[arg(long, value_enum)]
        format: Option<FormatArg>,
        /// The file to write.
        #[arg(long)]
        out: PathBuf,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum PaperArg {
    A4,
    A3,
    Letter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum LanguageArg {
    De,
    En,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum FormatArg {
    Pdf,
    Svg,
    Png,
}

/// What `export` was asked for.
struct ExportJob {
    project: PathBuf,
    scene: String,
    plan: Option<String>,
    at_s: f64,
    paper: Paper,
    language: Language,
    format: FormatArg,
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
            language,
            format,
            out,
        } => {
            let Some(format) = format.or_else(|| format_of(&out)) else {
                eprintln!("klappe-cli: cannot tell the format of {out:?}, use --format");
                return ExitCode::FAILURE;
            };
            let job = ExportJob {
                project,
                scene,
                plan,
                at_s: at,
                paper: paper_of(paper),
                language: language_of(language),
                format,
                out,
            };

            export(&job)
        }
    }
}

fn fov(lens_mm: f64, sensor: &str) -> ExitCode {
    let Some(sensor) = parse_sensor(sensor) else {
        eprintln!("klappe-cli: unknown sensor {sensor}");
        return ExitCode::FAILURE;
    };
    if !is_length(lens_mm) {
        eprintln!("klappe-cli: lens must be a positive length in mm, not {lens_mm}");
        return ExitCode::FAILURE;
    }

    let fov = klappe_services::horizontal_fov_deg(sensor, lens_mm);
    println!("{fov:.1}°");
    ExitCode::SUCCESS
}

fn export(job: &ExportJob) -> ExitCode {
    if !job.at_s.is_finite() || job.at_s < 0.0 {
        eprintln!(
            "klappe-cli: time must be a number of seconds from 0, not {}",
            job.at_s
        );
        return ExitCode::FAILURE;
    }

    match write_export(job) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("klappe-cli: {message}");
            ExitCode::FAILURE
        }
    }
}

fn write_export(job: &ExportJob) -> Result<(), String> {
    let mut session = Session::open(&job.project)
        .map_err(|e| format!("cannot open {}: {e}", job.project.display()))?;
    let scene_id = session
        .project()
        .scenes
        .iter()
        .find(|s| s.number == job.scene)
        .map(|s| s.id.clone())
        .ok_or_else(|| format!("no scene {}", job.scene))?;

    session.select_scene(&scene_id).map_err(|e| e.to_string())?;
    if let Some(plan) = &job.plan {
        session.select_plan(plan).map_err(|e| e.to_string())?;
    }
    if session.plan().is_none() {
        return Err(format!("scene {} has no plan", job.scene));
    }
    session.set_time(job.at_s);

    let texts = job.language.print_texts();
    match job.format {
        FormatArg::Pdf => session.export_pdf(&job.out, job.paper, texts, ShotOrder::Cut),
        FormatArg::Svg => session.export_svg(&job.out, job.paper, texts),
        FormatArg::Png => session.export_png(&job.out, job.paper, texts),
    }
    .map_err(|e| e.to_string())
}

fn paper_of(paper: PaperArg) -> Paper {
    match paper {
        PaperArg::A4 => Paper::A4,
        PaperArg::A3 => Paper::A3,
        PaperArg::Letter => Paper::Letter,
    }
}

fn language_of(language: Option<LanguageArg>) -> Language {
    match language {
        Some(LanguageArg::De) => Language::German,
        Some(LanguageArg::En) => Language::English,
        None => Language::pick(None),
    }
}

/// The format a file name asks for by its extension (`plan.PNG`).
fn format_of(out: &Path) -> Option<FormatArg> {
    let extension = out.extension()?.to_str()?.to_lowercase();

    match extension.as_str() {
        "pdf" => Some(FormatArg::Pdf),
        "svg" => Some(FormatArg::Svg),
        "png" => Some(FormatArg::Png),
        _ => None,
    }
}

/// A sensor by name, or by its width in millimetres (e.g. `23.76`).
fn parse_sensor(name: &str) -> Option<Sensor> {
    match name {
        "super35" | "s35" => Some(Sensor::Super35),
        "full-frame" | "ff" => Some(Sensor::FullFrame),
        "mft" => Some(Sensor::MicroFourThirds),
        _ => read_number(Field::Sensor, name, Unit::Millimetre, Range::Positive)
            .ok()
            .map(|width_mm| Sensor::Custom { width_mm }),
    }
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
        assert_eq!(parse_sensor("s35"), Some(Sensor::Super35));
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
        assert_eq!(parse_sensor("imax"), None);
    }

    #[test]
    fn format_follows_the_extension() {
        assert_eq!(format_of(Path::new("a/plan.png")), Some(FormatArg::Png));
        assert_eq!(format_of(Path::new("plan.SVG")), Some(FormatArg::Svg));
        assert_eq!(format_of(Path::new("plan.pdf")), Some(FormatArg::Pdf));
        assert_eq!(format_of(Path::new("plan.jpg")), None);
        assert_eq!(format_of(Path::new("plan")), None);
    }

    /// Review of PR 1: 0, NaN and infinite values printed an angle and exit 0.
    #[test]
    fn nonsense_values_are_refused() {
        assert_eq!(parse_sensor("inf"), None);
        assert_eq!(parse_sensor("NaN"), None);
        assert_eq!(fov(0.0, "s35"), ExitCode::FAILURE);
        assert_eq!(fov(f64::NAN, "s35"), ExitCode::FAILURE);
    }
}
