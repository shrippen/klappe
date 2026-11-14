import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// A camera is selected (LH-32, LH-38): its body, sensor width, lens and frame. A body sets
// sensor and frame together; a lens comes from the chosen set or is typed. Enter in a field
// sets that field; "Apply" sets the fields that were typed in. A value that is not a number
// marks its field and changes nothing. No optics here (agent.md): the controller sets, the
// session checks, Stellprobe works out the angle of view.
KanteCard {
    id: card

    required property var texts
    required property var controller

    // The plan changes with every edit; the key alone would not tell the new values.
    readonly property var info: {
        card.controller.planJson
        const key = card.controller.selectedKey
        if (!key || key.indexOf("camera:") !== 0) {
            return null
        }
        const d = JSON.parse(card.controller.describe(key))
        return d.kind === "camera" ? d : null
    }
    readonly property var presets: {
        const choices = card.controller.choicesJson ? JSON.parse(card.controller.choicesJson) : {}
        return choices.camera || { bodies: [], lens_sets: [] }
    }
    // The set the user picked; until then the first one that holds the camera's lens.
    property string pickedSet: ""
    readonly property var lensSet: {
        const sets = card.presets.lens_sets
        let found = sets.find(s => s.id === card.pickedSet)
        if (!found && card.info) {
            found = sets.find(s => s.stops_mm.indexOf(card.info.ownLens) >= 0)
        }
        return found || sets[0] || null
    }

    visible: card.info !== null
    implicitWidth: cardRows.implicitWidth + Kirigami.Units.largeSpacing * 2
    implicitHeight: cardRows.implicitHeight + Kirigami.Units.largeSpacing * 2

    onInfoChanged: card.fill()
    onLensSetChanged: card.syncBoxes()
    Component.onCompleted: card.fill()

    // A combo box drops its binding once the user picks; the boxes follow the camera again.
    function syncBoxes() {
        if (!card.info) {
            return
        }
        bodyBox.currentIndex = Math.max(0, bodyBox.indexOfValue(card.info.body))
        setBox.currentIndex = Math.max(0, setBox.indexOfValue(card.lensSet ? card.lensSet.id : ""))
        lensBox.currentIndex = lensBox.indexOfValue(card.info.ownLens)
    }

    function fill() {
        if (!card.info) {
            return
        }
        const values = [
            [sensorField, card.info.sensor],
            [lensField, card.info.ownLens],
            [aspectField, card.info.aspect],
        ]
        for (const [field, value] of values) {
            field.shown = card.texts.num(value, 2)
            if (!field.activeFocus) {
                field.text = field.shown
            }
        }
        card.syncBoxes()
    }

    // Sets one field when it was typed in; marks it when the value is refused.
    function applyField(name, field) {
        if (field.text === field.shown) {
            return
        }
        const ok = card.controller.setCameraField(name, field.text)
        field.invalid = !ok
        if (ok) {
            field.shown = field.text
        }
    }

    function applyAll() {
        card.applyField("sensor", sensorField)
        card.applyField("lens", lensField)
        card.applyField("aspect", aspectField)
    }

    function choose(name, value) {
        card.controller.setCameraField(name, String(value))
        card.syncBoxes()
    }

    ColumnLayout {
        id: cardRows
        anchors.centerIn: parent
        spacing: Kirigami.Units.smallSpacing

        RowLayout {
            spacing: Kirigami.Units.smallSpacing

            KanteSectionLabel {
                text: card.info ? card.texts.t("tip.camera", { label: card.info.label }) : ""
            }
            KanteComboBox {
                id: bodyBox
                objectName: "cameraBodyBox"
                Layout.preferredWidth: Kirigami.Units.gridUnit * 10
                Accessible.name: card.texts.t("camera.body")
                textRole: "text"
                valueRole: "value"
                model: [{ value: "", text: card.texts.t("camera.bodyCustom") }].concat(
                    card.presets.bodies.map(b => ({ value: b.id, text: b.name })))
                onActivated: {
                    if (currentValue !== "") {
                        card.choose("body", currentValue)
                    }
                    card.syncBoxes()
                }
            }
            KanteTextField {
                id: sensorField
                objectName: "cameraSensorField"
                property string shown: ""
                Layout.preferredWidth: Kirigami.Units.gridUnit * 6
                Accessible.name: card.texts.t("camera.sensor")
                placeholderText: card.texts.t("camera.sensor")
                inputMethodHints: Qt.ImhFormattedNumbersOnly
                onTextChanged: invalid = false
                onAccepted: card.applyField("sensor", sensorField)
            }
            KanteTextField {
                id: aspectField
                objectName: "cameraAspectField"
                property string shown: ""
                Layout.preferredWidth: Kirigami.Units.gridUnit * 8
                Accessible.name: card.texts.t("camera.aspect")
                placeholderText: card.texts.t("camera.aspect")
                QQC2.ToolTip.text: card.texts.t("camera.aspectTip")
                QQC2.ToolTip.visible: hovered
                QQC2.ToolTip.delay: Kirigami.Units.toolTipDelay
                onTextChanged: invalid = false
                onAccepted: card.applyField("aspect", aspectField)
            }
        }

        RowLayout {
            spacing: Kirigami.Units.smallSpacing

            KanteComboBox {
                id: setBox
                objectName: "cameraLensSetBox"
                Layout.preferredWidth: Kirigami.Units.gridUnit * 10
                Accessible.name: card.texts.t("camera.lensSet")
                textRole: "text"
                valueRole: "value"
                model: card.presets.lens_sets.map(s => ({ value: s.id, text: card.texts.t("lensSet." + s.id) }))
                onActivated: card.pickedSet = String(currentValue)
            }
            KanteComboBox {
                id: lensBox
                objectName: "cameraLensBox"
                Layout.preferredWidth: Kirigami.Units.gridUnit * 7
                Accessible.name: card.texts.t("camera.lensPick")
                textRole: "text"
                valueRole: "value"
                displayText: currentIndex >= 0 ? currentText : card.texts.t("camera.lensPick")
                model: card.lensSet ? card.lensSet.stops_mm.map(v => ({ value: v, text: card.texts.num(v, 1) + " mm" })) : []
                onActivated: card.choose("lens", currentValue)
            }
            KanteTextField {
                id: lensField
                objectName: "cameraLensField"
                property string shown: ""
                Layout.preferredWidth: Kirigami.Units.gridUnit * 6
                Accessible.name: card.texts.t("camera.lens")
                placeholderText: card.texts.t("camera.lens")
                inputMethodHints: Qt.ImhFormattedNumbersOnly
                onTextChanged: invalid = false
                onAccepted: card.applyField("lens", lensField)
            }
            KanteButton {
                objectName: "cameraApply"
                text: card.texts.t("form.apply")
                onClicked: card.applyAll()
            }
        }
    }
}
