import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// Shown while no project is open (LH-01): create one, open one, or take one of the last.
Item {
    id: start

    required property var texts
    required property var controller

    signal newRequested()
    signal openRequested()
    signal recentRequested(string path)

    readonly property var recent: controller.recentJson ? JSON.parse(controller.recentJson) : []

    ColumnLayout {
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.verticalCenter: parent.verticalCenter
        width: Math.min(parent.width - 2 * Kirigami.Units.largeSpacing, Kirigami.Units.gridUnit * 36)
        spacing: Kirigami.Units.largeSpacing

        KanteHeading {
            text: start.texts.t("start.title")
            level: 2
        }
        QQC2.Label {
            Layout.fillWidth: true
            text: start.texts.t("start.hint")
        }

        RowLayout {
            spacing: Kirigami.Units.smallSpacing

            KanteButton {
                objectName: "newButton"
                emphasis: KanteButton.Emphasis.Primary
                text: start.texts.t("start.new")
                onClicked: start.newRequested()
            }
            KanteButton {
                objectName: "openButton"
                text: start.texts.t("start.open")
                onClicked: start.openRequested()
            }
        }

        KanteSectionLabel { text: start.texts.t("start.recent") }
        QQC2.Label {
            objectName: "noRecent"
            Layout.fillWidth: true
            visible: start.recent.length === 0
            color: KanteStyle.mutedTextColor
            text: start.texts.t("start.noRecent")
        }
        Repeater {
            model: start.recent
            delegate: KanteListRow {
                required property var modelData
                objectName: "recent"
                Layout.fillWidth: true
                text: modelData.name
                subtitle: modelData.path
                onClicked: start.recentRequested(modelData.path)
            }
        }
    }
}
