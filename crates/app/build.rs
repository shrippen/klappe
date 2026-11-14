// Builds the QML module `io.github.shrippen.klappe` (pages compiled into the binary) and the
// Rust bridge. Kante is loaded from its folder at run time (see main.rs), copied unchanged.
use cxx_qt_build::{CxxQtBuilder, QmlModule};

const PAGES: &[&str] = &[
    "qml/Main.qml",
    "qml/BackupNotice.qml",
    "qml/Texts.qml",
    "qml/ScriptPanel.qml",
    "qml/ShotPanel.qml",
    "qml/PlanArea.qml",
    "qml/PlanTools.qml",
    "qml/Timeline.qml",
];

fn main() {
    CxxQtBuilder::new_qml_module(QmlModule::new("io.github.shrippen.klappe").qml_files(PAGES))
        .files(["src/bridge.rs"])
        .build();
}
