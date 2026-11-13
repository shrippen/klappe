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
            .filter(|w: &f64| *w > 0.0)
            .map(|width_mm| Sensor::Custom { width_mm }),
    }
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
}
