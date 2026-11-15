import QtQuick
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import Kante

// A door or window is selected (LH-10), a section of the selection card (D-04): its kind and
// its width as a field. Enter or the button sets the width; a value that is no length marks
// the field and changes nothing.
ColumnLayout {
    id: card

    required property var texts
    required property var controller

    readonly property var info: {
        // The plan changes with every edit; the key alone would not tell the new width.
        card.controller.planJson
        const key = card.controller.selectedKey
        if (!key || key.indexOf("opening:") !== 0) {
            return null
        }
        const d = JSON.parse(card.controller.describe(key))
        return d.kind === "opening" ? d : null
    }
    readonly property bool shown: info !== null
    /// A field has the focus: letters and Delete are text, not keys for the selection.
    readonly property bool typing: widthField.activeFocus

    visible: shown
    spacing: Kirigami.Units.smallSpacing

    onInfoChanged: card.fill()
    Component.onCompleted: card.fill()

    function fill() {
        if (info && !widthField.activeFocus) {
            widthField.text = texts.num(info.width, 2)
        }
    }

    function setWidth() {
        widthField.invalid = !controller.setOpeningWidth(widthField.text)
    }

    KanteSectionLabel {
        objectName: "openingTitle"
        text: card.info ? card.texts.t("openingKind." + card.info.type) : ""
    }

    RowLayout {
        spacing: Kirigami.Units.smallSpacing

        KanteTextField {
            id: widthField
            objectName: "openingWidthField"
            Layout.preferredWidth: Kirigami.Units.gridUnit * 5
            Accessible.name: card.texts.t("field.width")
            placeholderText: card.texts.t("field.width")
            inputMethodHints: Qt.ImhFormattedNumbersOnly
            onTextChanged: invalid = false
            onAccepted: card.setWidth()
        }
        KanteButton {
            objectName: "openingApply"
            text: card.texts.t("form.apply")
            onClicked: card.setWidth()
        }
    }
}
