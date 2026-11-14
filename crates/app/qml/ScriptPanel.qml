import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// The scene's script with chips to the marks and shots its lines are linked to (E-14,
// design B). The line the playback is at is marked with a bar, not by colour alone.
Item {
    id: panel

    required property var texts
    property var lines: []
    signal chipClicked(string kind, string target, real at)

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: Kirigami.Units.largeSpacing
        spacing: Kirigami.Units.smallSpacing

        RowLayout {
            KanteHeading {
                Layout.fillWidth: true
                level: 4
                text: panel.texts.t("script.title")
            }
            KanteSectionLabel { text: panel.texts.t("script.page") }
        }

        QQC2.Label {
            Layout.fillWidth: true
            visible: panel.lines.length === 0
            wrapMode: Text.Wrap
            color: KanteStyle.mutedTextColor
            text: panel.texts.t("script.empty")
        }

        ListView {
            id: list
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            spacing: Kirigami.Units.smallSpacing
            model: panel.lines

            delegate: Rectangle {
                id: row
                required property var modelData
                width: ListView.view.width
                height: content.implicitHeight + 2 * Kirigami.Units.smallSpacing
                color: modelData.current ? KanteStyle.selectionColor : "transparent"

                // The current line's bar (shape, not only colour).
                Rectangle {
                    width: Kirigami.Units.smallSpacing
                    height: parent.height
                    color: KanteStyle.focusColor
                    visible: row.modelData.current
                }

                RowLayout {
                    id: content
                    anchors.fill: parent
                    anchors.margins: Kirigami.Units.smallSpacing
                    anchors.leftMargin: Kirigami.Units.largeSpacing
                    spacing: Kirigami.Units.smallSpacing

                    QQC2.Label {
                        Layout.fillWidth: true
                        Layout.alignment: Qt.AlignTop
                        wrapMode: Text.Wrap
                        font.family: KanteStyle.monoFamily
                        // Screenplay layout: cues and dialogue indented, as on paper.
                        leftPadding: row.modelData.kind === "character" ? Kirigami.Units.gridUnit * 4
                            : row.modelData.kind === "parenthetical" ? Kirigami.Units.gridUnit * 3
                            : row.modelData.kind === "dialogue" ? Kirigami.Units.gridUnit * 2 : 0
                        color: row.modelData.kind === "character" ? KanteStyle.strongTextColor
                            : row.modelData.kind === "parenthetical" ? KanteStyle.mutedTextColor : KanteStyle.textColor
                        text: row.modelData.kind === "parenthetical" ? "(" + row.modelData.text + ")" : row.modelData.text
                    }

                    ColumnLayout {
                        Layout.alignment: Qt.AlignTop
                        spacing: Kirigami.Units.smallSpacing
                        Repeater {
                            model: row.modelData.chips
                            delegate: KanteChip {
                                required property var modelData
                                text: modelData.text
                                chipColor: modelData.label !== "" ? KanteStyle.labelColor(modelData.label) : KanteStyle.textColor
                                onClicked: panel.chipClicked(modelData.kind, modelData.target, modelData.at_s)
                            }
                        }
                    }
                }
            }
        }
    }
}
