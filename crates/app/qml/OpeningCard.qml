import QtQuick
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import Kante

// A door or window is selected (LH-10), a section of the selection card (D-04): its kind, its
// width, and its sill and head (LH-90) as fields. Enter sets the field, the button the width and
// what was changed of sill and head. The fields show the value in force; an emptied sill or head
// goes back to its kind's. A value that is no length marks the field and changes nothing.
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
    readonly property bool typing: widthField.activeFocus || sillField.typing || headField.typing

    visible: shown
    spacing: Kirigami.Units.smallSpacing

    onInfoChanged: card.fill()
    Component.onCompleted: card.fill()

    function fill() {
        if (!info) {
            return
        }
        if (!widthField.activeFocus) {
            widthField.text = texts.num(info.width, 2)
        }
        if (!sillField.typing) {
            sillField.text = card.metres(info.sill)
        }
        if (!headField.typing) {
            headField.text = card.metres(info.head)
        }
    }

    function metres(value) {
        return texts.num(value, 2)
    }

    function setWidth() {
        widthField.invalid = !controller.setOpeningWidth(widthField.text)
        return !widthField.invalid
    }

    // Sets the `name` ("sill" or "head") typed in `field`; the field then shows the value in
    // force, so an emptied one shows the default again.
    function setHeight(field, name) {
        const ok = controller.setSelectedHeight(name, field.text)
        if (ok && info) {
            field.text = card.metres(info[name])
        }
        field.invalid = !ok
        return ok
    }

    function setSill() {
        return card.setHeight(sillField, "sill")
    }

    function setHead() {
        return card.setHeight(headField, "head")
    }

    // The button sets the width always, sill and head when they differ from what is shown. A
    // refused width stops it. The edits refill the fields, so what was typed is taken first.
    function apply() {
        const sill = sillField.text
        const head = headField.text
        const sillChanged = info !== null && sill.trim() !== card.metres(info.sill)
        const headChanged = info !== null && head.trim() !== card.metres(info.head)
        if (!card.setWidth()) {
            return
        }
        if (sillChanged) {
            sillField.text = sill
            card.setSill()
        }
        if (headChanged) {
            headField.text = head
            card.setHead()
        }
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
            onClicked: card.apply()
        }
    }

    RowLayout {
        spacing: Kirigami.Units.smallSpacing

        LabeledField {
            id: sillField
            fieldName: "openingSillField"
            label: card.texts.t("field.sill")
            onAccepted: card.setSill()
        }
        LabeledField {
            id: headField
            fieldName: "openingHeadField"
            label: card.texts.t("field.head")
            onAccepted: card.setHead()
        }
    }
}
