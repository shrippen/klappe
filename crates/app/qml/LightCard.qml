import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// A lamp is selected (LH-50, LH-51): its kind, a lamp of the library, direction, beam angle,
// power and colour temperature. Enter in a field sets that field; "Apply" sets the fields that
// were typed in. A value that is not a number marks its field and changes nothing. Only an LED
// takes another colour temperature. No optics here (agent.md): the controller sets, the
// session checks, Stellprobe draws the cone.
KanteCard {
    id: card

    required property var texts
    required property var controller

    // The picture revision rises with every edit; the plan does not draw colour and power,
    // and the key alone would not tell the new values.
    readonly property var info: {
        card.controller.pictureRevision
        const key = card.controller.selectedKey
        if (!key || key.indexOf("light:") !== 0) {
            return null
        }
        const d = JSON.parse(card.controller.describe(key))
        return d.kind === "light" ? d : null
    }
    readonly property var kinds: {
        const choices = card.controller.choicesJson ? JSON.parse(card.controller.choicesJson) : {}
        return choices.lightKind || []
    }
    readonly property var presets: {
        const choices = card.controller.choicesJson ? JSON.parse(card.controller.choicesJson) : {}
        return choices.light || []
    }

    visible: card.info !== null
    implicitWidth: cardRows.implicitWidth + Kirigami.Units.largeSpacing * 2
    implicitHeight: cardRows.implicitHeight + Kirigami.Units.largeSpacing * 2

    onInfoChanged: card.fill()
    Component.onCompleted: card.fill()

    // Whether an apply is running: its edits refresh the plan, and `fill()` waits for the end.
    property bool applying: false

    readonly property var numberFields: [
        ["facing", facingField, "facing", 1],
        ["beam", beamField, "beam", 1],
        ["power", powerField, "power", 0],
        ["temp", tempField, "temp", 0],
    ]

    // A combo box drops its binding once the user picks; the box follows the lamp again.
    function fill() {
        if (!card.info || card.applying) {
            return
        }
        for (const [, field, key, digits] of card.numberFields) {
            // An old file may keep a power on a bounce or flag: leave the field empty.
            const hidden = key === "power" && !card.info.gives_light
            field.shown = hidden ? "" : card.texts.num(card.info[key], digits)
            if (!field.activeFocus) {
                field.text = field.shown
            }
        }
        kindBox.currentIndex = Math.max(0, kindBox.indexOfValue(card.info.type))
        presetBox.currentIndex = 0
    }

    // Sets each typed field of `fields` ([name, field] pairs). The texts are read first and
    // the card filled once at the end: each edit refreshes the plan, and a fill in between
    // would write the lamp's values over fields not yet applied. A sent field then shows the
    // lamp's value, so a second apply sends nothing; a refused one keeps its text and is
    // marked.
    function apply(fields) {
        const typed = fields.map(([name, field]) => [name, field, field.text])
        const sent = []
        const refused = []

        card.applying = true
        for (const [name, field, text] of typed) {
            if (text === field.shown) {
                continue
            }
            if (card.controller.setLightField(name, text)) {
                sent.push(field)
            } else {
                refused.push([field, text])
            }
        }
        card.applying = false

        // Only an edit refreshes the plan; without one the fields stay as typed.
        if (sent.length > 0) {
            card.fill()
        }
        for (const field of sent) {
            field.text = field.shown
        }
        for (const [field, text] of refused) {
            field.text = text
            field.invalid = true
        }
    }

    function applyAll() {
        card.apply(card.numberFields.map(([name, field]) => [name, field]))
    }

    ColumnLayout {
        id: cardRows
        anchors.centerIn: parent
        spacing: Kirigami.Units.smallSpacing

        RowLayout {
            spacing: Kirigami.Units.smallSpacing

            KanteSectionLabel {
                text: card.info ? card.texts.t("light.card", { label: card.info.label }) : ""
            }
            KanteComboBox {
                id: kindBox
                objectName: "lightKindBox"
                Layout.preferredWidth: Kirigami.Units.gridUnit * 8
                Accessible.name: card.texts.t("light.kind")
                textRole: "text"
                valueRole: "value"
                model: card.kinds.map(k => ({ value: k, text: card.texts.t("lightKind." + k) }))
                onActivated: {
                    card.controller.setLightField("kind", String(currentValue))
                    card.fill()
                }
            }
            KanteComboBox {
                id: presetBox
                objectName: "lightPresetBox"
                Layout.preferredWidth: Kirigami.Units.gridUnit * 10
                Accessible.name: card.texts.t("light.preset")
                textRole: "text"
                valueRole: "value"
                model: [{ value: "", text: card.texts.t("light.presetPick") }].concat(
                    card.presets.map(p => ({ value: p, text: card.texts.t("lightPreset." + p) })))
                onActivated: {
                    if (currentValue !== "") {
                        card.controller.setLightField("preset", String(currentValue))
                    }
                    card.fill()
                }
            }
        }

        RowLayout {
            spacing: Kirigami.Units.smallSpacing

            KanteTextField {
                id: facingField
                objectName: "lightFacingField"
                property string shown: ""
                Layout.preferredWidth: Kirigami.Units.gridUnit * 6
                Accessible.name: card.texts.t("light.facing")
                placeholderText: card.texts.t("light.facing")
                inputMethodHints: Qt.ImhFormattedNumbersOnly
                onTextChanged: invalid = false
                onAccepted: card.apply([["facing", facingField]])
            }
            KanteTextField {
                id: beamField
                objectName: "lightBeamField"
                property string shown: ""
                Layout.preferredWidth: Kirigami.Units.gridUnit * 8
                Accessible.name: card.texts.t("light.beam")
                placeholderText: card.texts.t("light.beam")
                inputMethodHints: Qt.ImhFormattedNumbersOnly
                onTextChanged: invalid = false
                onAccepted: card.apply([["beam", beamField]])
            }
            KanteTextField {
                id: powerField
                objectName: "lightPowerField"
                property string shown: ""
                Layout.preferredWidth: Kirigami.Units.gridUnit * 6
                // Bounce and flag make no light: no power.
                enabled: card.info !== null && card.info.gives_light
                Accessible.name: card.texts.t("light.power")
                placeholderText: card.texts.t("light.power")
                inputMethodHints: Qt.ImhFormattedNumbersOnly
                onTextChanged: invalid = false
                onAccepted: card.apply([["power", powerField]])
            }
            KanteTextField {
                id: tempField
                objectName: "lightTempField"
                property string shown: ""
                Layout.preferredWidth: Kirigami.Units.gridUnit * 8
                // Tungsten keeps its colour; only an LED is set to another.
                enabled: card.info !== null && card.info.tunable
                Accessible.name: card.texts.t("light.temp")
                placeholderText: card.texts.t("light.temp")
                QQC2.ToolTip.text: card.texts.t("light.tempTip")
                QQC2.ToolTip.visible: hovered && !enabled
                QQC2.ToolTip.delay: Kirigami.Units.toolTipDelay
                inputMethodHints: Qt.ImhFormattedNumbersOnly
                onTextChanged: invalid = false
                onAccepted: card.apply([["temp", tempField]])
            }
            KanteButton {
                objectName: "lightApply"
                text: card.texts.t("form.apply")
                onClicked: card.applyAll()
            }
        }
    }
}
