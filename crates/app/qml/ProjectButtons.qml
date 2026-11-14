import QtQuick
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import Kante

// New, Open and Project of the tool bar: what changes or replaces the project. `locked` while
// the backup question is open: edits then are not backed up, and the answer (restore) would
// overwrite them.
RowLayout {
    id: buttons

    required property var texts
    property bool locked: false

    signal newRequested()
    signal openRequested()
    signal projectRequested()

    enabled: !locked
    spacing: Kirigami.Units.smallSpacing

    KanteButton {
        objectName: "newProjectButton"
        text: buttons.texts.t("menu.new")
        onClicked: buttons.newRequested()
    }
    KanteButton {
        objectName: "openProjectButton"
        text: buttons.texts.t("menu.open")
        onClicked: buttons.openRequested()
    }
    KanteButton {
        objectName: "projectButton"
        text: buttons.texts.t("menu.project")
        onClicked: buttons.projectRequested()
    }
}
