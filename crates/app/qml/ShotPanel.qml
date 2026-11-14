import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// The selected shot: preview (camera view comes with LH-36) and its fields, edited in place
// (LH-71). Every change is one undo step; a field left unchanged edits nothing.
Item {
    id: panel

    required property var texts
    required property var controller
    property var shot: null

    // Cameras of the plan (the timeline's camera tracks), for the camera chooser.
    readonly property var cameras: controller.tracksJson ? JSON.parse(controller.tracksJson).filter(t => t.kind === "camera") : []

    // Counts refused values; the fields' text bindings read it, so they snap back to the model.
    property int refused: 0

    // The shot's own lens as text; empty if it follows its camera.
    readonly property string ownLens: shot && shot.own_lens_mm !== null ? texts.num(shot.own_lens_mm, 1) : ""

    // Sends `value` unless it is what the field showed; a refusal resets the fields.
    function commit(field, value, shown) {
        if (value === shown) {
            return
        }
        if (!controller.editShot(shot.id, field, value)) {
            refused++
        }
    }

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
                KanteSectionLabel { text: panel.texts.t("field.camera") }
                KanteTextField {
                    objectName: "numberField"
                    Layout.fillWidth: true
                    text: { panel.refused; return panel.shot ? panel.shot.number : "" }
                    onEditingFinished: panel.commit("number", text, panel.shot.number)
                }
                KanteComboBox {
                    objectName: "cameraBox"
                    Layout.fillWidth: true
                    model: panel.cameras.map(c => c.name)
                    currentIndex: panel.shot ? panel.cameras.findIndex(c => c.id === panel.shot.camera) : -1
                    onActivated: index => panel.commit("camera", panel.cameras[index].id, panel.shot.camera)
                }

                KanteSectionLabel { text: panel.texts.t("field.size") }
                KanteSectionLabel { text: panel.texts.t("field.movement") }
                KanteComboBox {
                    Layout.fillWidth: true
                    model: panel.sizes.map(s => panel.texts.t("size." + s))
                    currentIndex: panel.shot ? panel.sizes.indexOf(panel.shot.size) : -1
                    onActivated: index => panel.commit("size", panel.sizes[index], panel.shot.size)
                }
                KanteComboBox {
                    Layout.fillWidth: true
                    model: panel.movements.map(m => panel.texts.t("movement." + m))
                    currentIndex: panel.shot ? panel.movements.indexOf(panel.shot.movement) : -1
                    onActivated: index => panel.commit("movement", panel.movements[index], panel.shot.movement)
                }

                KanteSectionLabel { text: panel.texts.t("field.lens") }
                Item { Layout.fillWidth: true }
                // Empty: the shot follows its camera, whose lens is the hint.
                KanteTextField {
                    objectName: "lensField"
                    Layout.fillWidth: true
                    text: { panel.refused; return panel.ownLens }
                    placeholderText: panel.shot ? panel.texts.num(panel.shot.lens_mm, 1) : ""
                    inputMethodHints: Qt.ImhFormattedNumbersOnly
                    onEditingFinished: panel.commit("lens", text, panel.ownLens)
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
                        text: { panel.refused; return panel.shot ? panel.shot[parent.modelData] : "" }
                        onEditingFinished: panel.commit(parent.modelData, text, panel.shot[parent.modelData])
                    }
                }
            }
        }
    }
}
