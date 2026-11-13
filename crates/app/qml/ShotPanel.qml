import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// The selected shot: preview (camera view comes with LH-36) and its fields, edited in place
// (LH-71). Every change is one undo step.
Item {
    id: panel

    required property var texts
    property var shot: null
    signal edit(string field, string value)

    readonly property var sizes: ["extreme-wide", "wide", "medium-wide", "cowboy", "medium", "medium-close", "close-up", "extreme-close-up"]
    readonly property var movements: ["static", "pan", "tilt", "dolly", "crane", "handheld", "gimbal", "zoom"]

    KanteEmptyState {
        anchors.centerIn: parent
        width: parent.width - 2 * Kirigami.Units.largeSpacing
        visible: panel.shot === null
        title: panel.texts.t("toggle.shot")
        text: panel.texts.t("shot.none")
    }

    QQC2.ScrollView {
        anchors.fill: parent
        visible: panel.shot !== null
        contentWidth: availableWidth

        ColumnLayout {
            width: parent.width
            spacing: Kirigami.Units.largeSpacing

            RowLayout {
                Layout.fillWidth: true
                Layout.margins: Kirigami.Units.largeSpacing
                KanteHeading {
                    Layout.fillWidth: true
                    level: 4
                    text: panel.shot ? panel.texts.t("shot.title", { number: panel.shot.number }) : ""
                }
                KanteChip {
                    visible: panel.shot !== null && panel.shot.axis_jump
                    interactive: false
                    chipColor: KanteStyle.warningColor
                    iconName: "dialog-warning"
                    text: panel.texts.t("shot.axisJump")
                }
            }

            KanteCallout {
                Layout.fillWidth: true
                Layout.leftMargin: Kirigami.Units.largeSpacing
                Layout.rightMargin: Kirigami.Units.largeSpacing
                text: panel.texts.t("shot.preview")
            }

            GridLayout {
                Layout.fillWidth: true
                Layout.leftMargin: Kirigami.Units.largeSpacing
                Layout.rightMargin: Kirigami.Units.largeSpacing
                columns: 2
                columnSpacing: Kirigami.Units.largeSpacing
                rowSpacing: Kirigami.Units.smallSpacing

                KanteSectionLabel { text: panel.texts.t("field.number") }
                KanteSectionLabel { text: panel.texts.t("field.lens") }
                KanteTextField {
                    Layout.fillWidth: true
                    text: panel.shot ? panel.shot.number : ""
                    onEditingFinished: panel.edit("number", text)
                }
                KanteTextField {
                    Layout.fillWidth: true
                    text: panel.shot ? String(panel.shot.lens_mm) : ""
                    inputMethodHints: Qt.ImhFormattedNumbersOnly
                    onEditingFinished: panel.edit("lens", text)
                }

                KanteSectionLabel { text: panel.texts.t("field.size") }
                KanteSectionLabel { text: panel.texts.t("field.movement") }
                KanteComboBox {
                    Layout.fillWidth: true
                    model: panel.sizes.map(s => panel.texts.t("size." + s))
                    currentIndex: panel.shot ? panel.sizes.indexOf(panel.shot.size) : -1
                    onActivated: index => panel.edit("size", panel.sizes[index])
                }
                KanteComboBox {
                    Layout.fillWidth: true
                    model: panel.movements.map(m => panel.texts.t("movement." + m))
                    currentIndex: panel.shot ? panel.movements.indexOf(panel.shot.movement) : -1
                    onActivated: index => panel.edit("movement", panel.movements[index])
                }
            }

            Repeater {
                model: ["description", "props", "gear", "crew", "note"]
                delegate: ColumnLayout {
                    required property string modelData
                    Layout.fillWidth: true
                    Layout.leftMargin: Kirigami.Units.largeSpacing
                    Layout.rightMargin: Kirigami.Units.largeSpacing
                    spacing: Kirigami.Units.smallSpacing
                    KanteSectionLabel { text: panel.texts.t("field." + parent.modelData) }
                    KanteTextField {
                        Layout.fillWidth: true
                        text: panel.shot ? panel.shot[parent.modelData] : ""
                        onEditingFinished: panel.edit(parent.modelData, text)
                    }
                }
            }
        }
    }
}
