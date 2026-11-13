//! `klappe-cli`: Klappe without the GUI (planung/KONZEPT.md, E-10).
//!
//! ```text
//! klappe-cli fov --lens 35 [--sensor super35|full-frame|mft|<width mm>]
//! ```

use clap::{Parser, Subcommand};
use klappe_model::Sensor;
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
}

fn main() -> ExitCode {
    match Cli::parse().command {
        Command::Fov { lens, sensor } => fov(lens, &sensor),
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

    let fov = klappe_stellprobe::horizontal_fov_deg(sensor, lens_mm);
    println!("{fov:.1}°");
    ExitCode::SUCCESS
}

/// A sensor by name, or by its width in millimetres (e.g. `23.76`).
fn parse_sensor(name: &str) -> Option<Sensor> {
    match name {
        "super35" | "s35" => Some(Sensor::Super35),
        "full-frame" | "ff" => Some(Sensor::FullFrame),
        "mft" => Some(Sensor::MicroFourThirds),
        _ => name
            .parse()
            .ok()
            .filter(|w: &f64| is_length(*w))
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
        assert_eq!(parse_sensor("-1"), None);
        assert_eq!(parse_sensor("imax"), None);
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
