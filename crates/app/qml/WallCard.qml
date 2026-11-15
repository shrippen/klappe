import QtQuick
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import Kante

// A wall is selected (LH-90), a section of the selection card (D-04): its height as a field. The
// field shows the wall's own height; without one it is empty and shows the room's greyed.
// Emptying it goes back to the room's. Enter or the button sets it; a value that is no length marks the
// field and changes nothing.
ColumnLayout {
    id: card

    required property var texts
    required property var controller

    readonly property var info: {
        // The plan changes with every edit; the key alone would not tell the new height.
        card.controller.planJson
        const key = card.controller.selectedKey
        if (!key || key.indexOf("wall:") !== 0) {
            return null
        }
        const d = JSON.parse(card.controller.describe(key))
        return d.kind === "wall" ? d : null
    }
    readonly property bool shown: info !== null
    /// A field has the focus: letters and Delete are text, not keys for the selection.
    readonly property bool typing: heightField.typing

    visible: shown
    spacing: Kirigami.Units.smallSpacing

    onInfoChanged: card.fill()
    Component.onCompleted: card.fill()

    function fill() {
        if (!info) {
            return
        }
        heightField.placeholderText = texts.num(info.height, 2)
        if (!heightField.typing) {
            heightField.text = card.own()
        }
    }

    // The wall's own height as the field shows it: empty for the room's, which the placeholder
    // shows.
    function own() {
        return info.heightDefault ? "" : texts.num(info.height, 2)
    }

    // An untouched field sends nothing, so the room's height is never pinned as the wall's own
    // and follows the room. An emptied one goes back to the room's.
    function setHeight() {
        if (info && heightField.text.trim() === card.own()) {
            heightField.invalid = false
            return
        }
        const ok = controller.setSelectedHeight("wall", heightField.text)
        if (ok && info) {
            heightField.text = card.own()
        }
        heightField.invalid = !ok
    }

    KanteSectionLabel {
        objectName: "wallTitle"
        text: card.texts.t("tool.wall")
    }

    RowLayout {
        spacing: Kirigami.Units.smallSpacing

        LabeledField {
            id: heightField
            fieldName: "wallHeightField"
            label: card.texts.t("field.height")
            Layout.alignment: Qt.AlignBottom
            onAccepted: card.setHeight()
        }
        KanteButton {
            objectName: "wallApply"
            Layout.alignment: Qt.AlignBottom
            text: card.texts.t("form.apply")
            onClicked: card.setHeight()
        }
    }
}
