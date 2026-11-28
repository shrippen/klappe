import QtQuick
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import Kante

// A drawing is selected (LH-19), a section of the selection card (D-46): its colour, line and, for a shape with an inside, its fill.
// Each pick sets the drawing at once. The style comes from the session; no geometry here.
ColumnLayout {
    id: card

    required property var texts
    required property var controller

    readonly property var choices: controller.choicesJson ? JSON.parse(controller.choicesJson) : ({})
    readonly property var colors: choices.color || []
    readonly property var dashes: choices.dash || []
    readonly property var fills: choices.fill || []
    readonly property var info: {
        // Every edit raises the picture revision; the plan may not change (a drag ends as
        // previewed), and the key alone would not tell the new style.
        card.controller.pictureRevision
        const key = card.controller.selectedKey
        if (!key || key.indexOf("drawing:") !== 0) {
            return null
        }
        const d = JSON.parse(card.controller.describe(key))
        return d.kind === "drawing" ? d : null
    }

    readonly property bool shown: info !== null
    readonly property string title: card.texts.t("legend.drawing")
    /// No field takes the keyboard: the style is picked.
    readonly property bool typing: false

    visible: card.shown
    spacing: Kirigami.Units.smallSpacing

    // Sets the style with one part changed: `part` is "color", "dash" or "fill".
    function pick(part, value) {
        const next = { color: info.color, dash: info.dash, fill: info.fill }
        next[part] = value
        card.controller.setDrawingStyle(next.color, next.dash, next.fill)
    }

    RowLayout {
        spacing: Kirigami.Units.smallSpacing

        KanteSwatchPicker {
            objectName: "drawingColor"
            Accessible.name: card.texts.t("draw.color")
            colors: card.colors.map(n => KanteStyle.labelColor(n))
            names: card.colors.map(n => card.texts.t("color." + n))
            currentIndex: card.info ? card.colors.indexOf(card.info.color) : -1
            onPicked: index => card.pick("color", card.colors[index])
        }
        KanteComboBox {
            objectName: "drawingDash"
            Accessible.name: card.texts.t("draw.dash")
            textRole: "text"
            valueRole: "value"
            model: card.dashes.map(k => ({ value: k, text: card.texts.t("dash." + k) }))
            currentIndex: card.info ? Math.max(0, card.dashes.indexOf(card.info.dash)) : 0
            onActivated: card.pick("dash", String(currentValue))
        }
        KanteComboBox {
            objectName: "drawingFill"
            visible: card.info !== null && card.info.closed
            Accessible.name: card.texts.t("draw.fill")
            textRole: "text"
            valueRole: "value"
            model: card.fills.map(k => ({ value: k, text: card.texts.t("fill." + k) }))
            currentIndex: card.info ? Math.max(0, card.fills.indexOf(card.info.fill)) : 0
            onActivated: card.pick("fill", String(currentValue))
        }
    }
}
