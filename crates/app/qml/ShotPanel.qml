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

    // Figures of the plan, for the follow chooser.
    readonly property var figures: controller.tracksJson ? JSON.parse(controller.tracksJson).filter(t => t.kind === "figure") : []

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

    // Couples the shot's camera to the figure `id`, or lets it go with an empty id.
    function follow(id) {
        if (id === shot.follows) {
            return
        }
        if (!controller.setFollow(shot.camera, id)) {
            refused++
        }
    }

    readonly property var sizes: ["extreme-wide", "wide", "medium-wide", "cowboy", "medium", "medium-close", "close-up", "extreme-close-up"]
    readonly property var angles: ["eye-level", "high", "low", "birds-eye", "worms-eye", "dutch"]
    readonly property var movements: ["static", "pan", "tilt", "dolly", "crane", "handheld", "gimbal", "zoom"]

    // New, copy and delete a shot (LH-70). "New" takes the camera selected in the plan, else
    // the selected shot's. Key S is "new"; deleting asks first.
    RowLayout {
        id: bar
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: Kirigami.Units.smallSpacing
        spacing: Kirigami.Units.smallSpacing

        KanteToolButton {
            objectName: "newShotButton"
            text: panel.texts.t("shot.new")
            onClicked: panel.controller.addShot()
        }
        KanteToolButton {
            objectName: "copyShotButton"
            enabled: panel.shot !== null
            text: panel.texts.t("shot.copy")
            onClicked: panel.controller.copyShot(panel.shot.id)
        }
        KanteToolButton {
            objectName: "deleteShotButton"
            enabled: panel.shot !== null
            text: panel.texts.t("shot.delete")
            onClicked: confirmDelete.open()
        }
        Item { Layout.fillWidth: true }
    }

    Shortcut {
        sequence: "S"
        enabled: panel.visible && panel.enabled && !!panel.controller.loaded
        onActivated: panel.controller.addShot()
    }

    KanteDialog {
        id: confirmDelete
        objectName: "confirmDelete"

        parent: QQC2.Overlay.overlay
        anchors.centerIn: parent
        modal: true
        size: KanteDialog.Size.Small
        destructive: true
        standardButtons: QQC2.Dialog.Ok | QQC2.Dialog.Cancel
        acceptText: panel.texts.t("remove.confirm")
        rejectText: panel.texts.t("form.cancel")
        title: panel.texts.t("shot.delete")
        onAccepted: panel.controller.deleteShot(panel.shot.id)

        QQC2.Label {
            width: parent ? parent.width : implicitWidth
            wrapMode: Text.WordWrap
            color: KanteStyle.textColor
            text: panel.shot ? panel.texts.t("shot.deleteAsk", { number: panel.shot.number }) : ""
        }
    }

    KanteEmptyState {
        anchors.top: bar.bottom
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.topMargin: Kirigami.Units.largeSpacing
        width: parent.width - 2 * Kirigami.Units.largeSpacing
        visible: panel.shot === null
        title: panel.texts.t("toggle.shot")
        text: panel.texts.t("shot.none")
    }

    QQC2.ScrollView {
        anchors.top: bar.bottom
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
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

                KanteSectionLabel { text: panel.texts.t("field.perspective") }
                KanteSectionLabel { text: panel.texts.t("field.lens") }
                KanteComboBox {
                    objectName: "angleBox"
                    Layout.fillWidth: true
                    model: panel.angles.map(a => panel.texts.t("angle." + a))
                    currentIndex: panel.shot ? panel.angles.indexOf(panel.shot.angle) : -1
                    onActivated: index => panel.commit("angle", panel.angles[index], panel.shot.angle)
                }
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

            // The shot's camera follows a figure, keeping distance and angle (LH-37).
            ColumnLayout {
                Layout.fillWidth: true
                Layout.leftMargin: Kirigami.Units.largeSpacing
                Layout.rightMargin: Kirigami.Units.largeSpacing
                spacing: Kirigami.Units.smallSpacing
                KanteSectionLabel { text: panel.texts.t("field.follows") }
                KanteComboBox {
                    objectName: "followBox"
                    Layout.fillWidth: true
                    // Index 0 is "none"; the figures come after it.
                    model: [panel.texts.t("follows.none")].concat(panel.figures.map(f => f.name))
                    currentIndex: { panel.refused; return panel.shot ? 1 + panel.figures.findIndex(f => f.id === panel.shot.follows) : -1 }
                    onActivated: index => panel.follow(index === 0 ? "" : panel.figures[index - 1].id)
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
