//! Klappe desktop app (E-10): Stellprobe in a Kirigami window styled by Kante.
//!
//! Window size for checks: Qt's own `-qwindowgeometry 1440x900`; offscreen with
//! `QT_QPA_PLATFORM=offscreen` (agent.md: never a visible window in tests).

mod args;
mod bridge;
mod catalog;
mod notice;
mod plan;
mod tools;

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QString, QUrl};
use std::path::PathBuf;

const MAIN_QML: &str = "qrc:/qt/qml/io/github/shrippen/klappe/qml/Main.qml";

/// Exit status for a wrong command line (sysexits EX_USAGE).
const EXIT_USAGE: i32 = 64;

/// Folder with Kante's QML module, for installs that keep it elsewhere.
const QML_DIR_VAR: &str = "KLAPPE_QML_DIR";

fn main() {
    let options = args::AppArgs::from_env();
    if options.help {
        print!("{}", args::USAGE);
        return;
    }
    if let Some(option) = options.unknown {
        eprint!("klappe: unknown option {option}\n\n{}", args::USAGE);
        std::process::exit(EXIT_USAGE);
    }

    let mut app = QGuiApplication::new();
    if let Some(mut app) = app.as_mut() {
        app.as_mut().set_application_name(&QString::from("Klappe"));
        app.as_mut()
            .set_organization_domain(&QString::from("shrippen.github.io"));
    }

    let mut engine = QQmlApplicationEngine::new();
    if let Some(mut engine) = engine.as_mut() {
        for dir in qml_dirs() {
            engine
                .as_mut()
                .add_import_path(&QString::from(dir.to_string_lossy().as_ref()));
        }
        // Without this a broken UI leaves a process without a window that never exits.
        engine
            .as_mut()
            .on_object_creation_failed(|_, url| {
                eprintln!("klappe: could not load {}", url.to_qstring());
                std::process::exit(1);
            })
            .release();
        engine.as_mut().load(&QUrl::from(MAIN_QML));
    }

    if let Some(app) = app.as_mut() {
        app.exec();
    }
}

/// Where Kante's QML folder lives: `KLAPPE_QML_DIR`, next to the binary
/// (`../share/klappe/qml`) or, in development, in the crate.
fn qml_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(dir) = std::env::var_os(QML_DIR_VAR) {
        dirs.push(PathBuf::from(dir));
    }
    if let Some(installed) = std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(|p| p.join("../share/klappe/qml")))
    {
        dirs.push(installed);
    }
    dirs.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("qml"));

    dirs.into_iter()
        .filter(|d| d.join("Kante/qmldir").exists())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The QML module's name and the main page's address must agree, or no window comes up.
    #[test]
    fn main_qml_lies_in_the_module() {
        let build = include_str!("../build.rs");
        let module = build
            .split("QmlModule::new(\"")
            .nth(1)
            .and_then(|rest| rest.split('"').next())
            .unwrap_or_default();
        let wanted = format!("qrc:/qt/qml/{}/qml/Main.qml", module.replace('.', "/"));

        assert_eq!(MAIN_QML, wanted);
    }
}
