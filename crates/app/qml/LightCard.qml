import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// A lamp is selected (LH-50, LH-51), a section of the selection card (D-46): its kind, a lamp of
// the library, direction, beam angle, power and colour temperature. Enter in a field, or the
// keyboard leaving it with a changed text, sets that field. A value that is not a number marks its field and changes nothing. Only an LED
// takes another colour temperature. No optics here (agent.md): the controller sets, the
// session checks, Stellprobe draws the cone.
ColumnLayout {
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

    readonly property bool shown: info !== null
    /// The head of the card: the lamp's label, else its kind.
    readonly property string title: !info ? "" : (info.label !== "" ? info.label : card.texts.t("lightKind." + info.type))
    /// A field has the focus: letters and Delete are text, not keys for the selection.
    readonly property bool typing: facingField.typing || beamField.typing || powerField.typing
        || tempField.typing

    visible: card.shown
    spacing: Kirigami.Units.smallSpacing

    onInfoChanged: card.fill()
    Component.onCompleted: card.fill()

    // Whether an apply is running: each of its edits raises the picture revision, and `fill()`
    // waits for the end.
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
            if (!field.typing) {
                field.text = field.shown
            }
        }
        kindBox.currentIndex = Math.max(0, kindBox.indexOfValue(card.info.type))
        presetBox.currentIndex = 0
    }

    // Sets each typed field of `fields` ([name, field] pairs). The texts are read first and
    // the card filled once at the end: each edit rereads the lamp, and a fill in between
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

        // Only an edit rereads the lamp; without one the fields stay as typed.
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

    RowLayout {
        spacing: Kirigami.Units.smallSpacing

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

    // Two by two: a card beside the element stays narrow.
    GridLayout {
        columns: 2
        columnSpacing: Kirigami.Units.smallSpacing
        rowSpacing: Kirigami.Units.smallSpacing

        LabeledField {
            id: facingField
            fieldName: "lightFacingField"
            label: card.texts.t("light.facing")
            fieldWidth: Kirigami.Units.gridUnit * 6
            onAccepted: card.apply([["facing", facingField]])
        }
        LabeledField {
            id: beamField
            fieldName: "lightBeamField"
            label: card.texts.t("light.beam")
            fieldWidth: Kirigami.Units.gridUnit * 8
            onAccepted: card.apply([["beam", beamField]])
        }
        LabeledField {
            id: powerField
            fieldName: "lightPowerField"
            label: card.texts.t("light.power")
            fieldWidth: Kirigami.Units.gridUnit * 6
            // Bounce and flag make no light: no power.
            enabled: card.info !== null && card.info.gives_light
            onAccepted: card.apply([["power", powerField]])
        }
        LabeledField {
            id: tempField
            fieldName: "lightTempField"
            label: card.texts.t("light.temp")
            fieldWidth: Kirigami.Units.gridUnit * 8
            // Tungsten keeps its colour; only an LED is set to another.
            enabled: card.info !== null && card.info.tunable
            QQC2.ToolTip.text: card.texts.t("light.tempTip")
            QQC2.ToolTip.visible: hovered && !enabled
            QQC2.ToolTip.delay: Kirigami.Units.toolTipDelay
            onAccepted: card.apply([["temp", tempField]])
        }
    }
}
