import QtQuick
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import Kante

// A wall is selected (LH-90), a section of the selection card (D-04): its height as a field. The
// field shows the height in force, the room's when the wall has none of its own; emptying it
// goes back to the room's. Enter or the button sets it; a value that is no length marks the
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
        if (info && !heightField.typing) {
            heightField.text = texts.num(info.height, 2)
        }
    }

    // The field then shows the height in force, so an emptied one shows the room's again.
    function setHeight() {
        const ok = controller.setSelectedHeight("wall", heightField.text)
        if (ok && info) {
            heightField.text = texts.num(info.height, 2)
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
