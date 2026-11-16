import QtQuick
import QtQuick.Layouts
import QtQuick.Dialogs
import org.kde.kirigami as Kirigami
import Kante

// An image is selected (LH-18): its opacity in percent, and a button that swaps its file. Enter
// in the field or "Apply" sets the opacity; a value out of range marks the field and changes
// nothing. The place, width and turn stay when the file is swapped.
KanteCard {
    id: card

    required property var texts
    required property var controller

    readonly property var info: {
        // Every edit raises the picture revision; the plan may not change (a drag ends as
        // previewed), and the key alone would not tell the new opacity.
        card.controller.pictureRevision
        const key = card.controller.selectedKey
        if (!key || key.indexOf("image:") !== 0) {
            return null
        }
        const d = JSON.parse(card.controller.describe(key))
        return d.kind === "image" ? d : null
    }
    /// The field has the focus: letters and Delete are text, not keys for the selection.
    readonly property bool typing: opacityField.typing

    readonly property real percent: 100

    visible: info !== null
    implicitWidth: row.implicitWidth + Kirigami.Units.largeSpacing * 2
    implicitHeight: row.implicitHeight + Kirigami.Units.largeSpacing * 2

    onInfoChanged: card.fill()
    Component.onCompleted: card.fill()

    function fill() {
        if (!info) {
            return
        }
        opacityField.shown = card.texts.num(info.opacity * card.percent, 0)
        if (!opacityField.typing) {
            opacityField.text = opacityField.shown
        }
    }

    function setOpacity() {
        if (opacityField.text === opacityField.shown) {
            opacityField.invalid = false
            return
        }
        const typed = opacityField.text
        const ok = card.controller.setImageOpacity(typed)
        if (ok) {
            card.fill()
            opacityField.text = opacityField.shown
        }
        opacityField.invalid = !ok
    }

    // The file dialog, made on the first click: one that exists takes the keyboard focus of the
    // plan's fields.
    Component {
        id: replaceDialogComponent
        FileDialog {
            title: card.texts.t("image.replacePick")
            nameFilters: [card.texts.t("background.filter")]
            onAccepted: card.controller.replaceImage(selectedFile.toString())
        }
    }
    property var replaceDialog: null
    function askReplace() {
        if (!replaceDialog) {
            replaceDialog = replaceDialogComponent.createObject(card, { objectName: "replaceDialog" })
        }
        replaceDialog.open()
    }

    RowLayout {
        id: row
        anchors.centerIn: parent
        spacing: Kirigami.Units.smallSpacing

        KanteSectionLabel { text: card.texts.t("image.card") }
        LabeledField {
            id: opacityField
            fieldName: "imageOpacityField"
            label: card.texts.t("field.opacity")
            onAccepted: card.setOpacity()
        }
        KanteButton {
            objectName: "imageOpacityApply"
            Layout.alignment: Qt.AlignBottom
            text: card.texts.t("form.apply")
            onClicked: card.setOpacity()
        }
        KanteButton {
            objectName: "imageReplace"
            Layout.alignment: Qt.AlignBottom
            text: card.texts.t("image.replace")
            onClicked: card.askReplace()
        }
    }
}
