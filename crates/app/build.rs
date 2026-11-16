// Builds the QML module `io.github.shrippen.klappe` (pages compiled into the binary) and the
// Rust bridge. Kante is loaded from its folder at run time (see main.rs), copied unchanged.
use cxx_qt_build::{CxxQtBuilder, QmlModule};

const PAGES: &[&str] = &[
    "qml/Main.qml",
    "qml/AsideNotice.qml",
    "qml/BackupNotice.qml",
    "qml/ProjectGuard.qml",
    "qml/Texts.qml",
    "qml/ScriptPanel.qml",
    "qml/ShotPanel.qml",
    "qml/CameraView.qml",
    "qml/CameraSurface.qml",
    "qml/ViewPalette.qml",
    "qml/FreeView.qml",
    "qml/ShotTiles.qml",
    "qml/ShotTable.qml",
    "qml/PlanArea.qml",
    "qml/CameraCard.qml",
    "qml/DrawingCard.qml",
    "qml/ImageCard.qml",
    "qml/LightCard.qml",
    "qml/MarkCard.qml",
    "qml/ObjectCard.qml",
    "qml/OpeningCard.qml",
    "qml/WallCard.qml",
    "qml/LabeledField.qml",
    "qml/SelectionCard.qml",
    "qml/PlanTools.qml",
    "qml/PlanMenu.qml",
    "qml/HeaderMenu.qml",
    "qml/CommandAction.qml",
    "qml/AppHeader.qml",
    "qml/ExportDialog.qml",
    "qml/LayerPopover.qml",
    "qml/PlanForm.qml",
    "qml/TemplateForm.qml",
    "qml/ViewForm.qml",
    "qml/ViewCommands.qml",
    "qml/Timeline.qml",
    "qml/Workspace.qml",
    "qml/TileDialog.qml",
    "qml/FormDialog.qml",
    "qml/NewProjectDialog.qml",
    "qml/RefCard.qml",
    "qml/EditorPane.qml",
    "qml/SceneEditor.qml",
    "qml/LocationEditor.qml",
    "qml/CharacterEditor.qml",
    "qml/ManageList.qml",
    "qml/ProjectDialog.qml",
    "qml/StartPage.qml",
];

fn main() {
    CxxQtBuilder::new_qml_module(QmlModule::new("io.github.shrippen.klappe").qml_files(PAGES))
        .qt_module("Quick")
        .qt_module("Widgets")
        .files([
            "src/application.rs",
            "src/bridge.rs",
            "src/camera_picture.rs",
        ])
        .cpp_file("src/application.cpp")
        .build();
}
