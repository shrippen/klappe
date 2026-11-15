import QtQuick
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import Kante

// A door or window is selected (LH-10), a section of the selection card (D-04): its kind, its
// width, and its sill and head (LH-90) as fields. Enter sets the field, the button the width and
// what was changed of sill and head. Sill and head show the opening's own value; without one the
// field is empty and shows its kind's greyed; an emptied one goes back to its kind's. A value that is no length marks the field and changes nothing.
// A door also shows two choices: the side it opens to and the end its hinge is at.
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

    readonly property var sides: ["left", "right"]
    readonly property var hinges: ["start", "end"]
    readonly property bool isDoor: info !== null && info.type === "door"

    visible: shown
    spacing: Kirigami.Units.smallSpacing

    onInfoChanged: card.fill()
    Component.onCompleted: card.fill()

    function fill() {
        if (!info) {
            return
        }
        sideChoice.currentIndex = card.sides.indexOf(info.side)
        hingeChoice.currentIndex = card.hinges.indexOf(info.hinge)
        if (!widthField.activeFocus) {
            widthField.text = texts.num(info.width, 2)
        }
        sillField.placeholderText = card.metres(info.sill)
        headField.placeholderText = card.metres(info.head)
        if (!sillField.typing) {
            sillField.text = card.own("sill")
        }
        if (!headField.typing) {
            headField.text = card.own("head")
        }
    }

    function metres(value) {
        return texts.num(value, 2)
    }

    // The opening's own `name` ("sill" or "head") as its field shows it: empty for the kind's,
    // which the placeholder shows.
    function own(name) {
        return info[name + "Default"] ? "" : card.metres(info[name])
    }

    // Sets a door's `side` or `hinge` to `value`; the plan change refills the choice.
    function setSwing(name, value) {
        if (!controller.setSelectedField(name, value)) {
            card.fill()
        }
    }

    function setWidth() {
        widthField.invalid = !controller.setOpeningWidth(widthField.text)
        return !widthField.invalid
    }

    // Sets the `name` ("sill" or "head") typed in `field`; an untouched field sends nothing, so
    // a default is never pinned as the opening's own. An emptied one goes back to the kind's.
    function setHeight(field, name) {
        if (info && field.text.trim() === card.own(name)) {
            field.invalid = false
            return true
        }
        const ok = controller.setSelectedField(name, field.text)
        if (ok && info) {
            field.text = card.own(name)
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
        const sillChanged = info !== null && sill.trim() !== card.own("sill")
        const headChanged = info !== null && head.trim() !== card.own("head")
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

    // A door's swing: which side of the wall it opens to and which end its hinge is at.
    ColumnLayout {
        objectName: "swingRow"
        visible: card.isDoor
        spacing: Kirigami.Units.smallSpacing

        KanteSectionLabel { text: card.texts.t("opening.swing") }
        KanteSegmented {
            id: sideChoice
            objectName: "sideChoice"
            model: card.sides.map(s => card.texts.t("swing." + s))
            onActivated: index => card.setSwing("side", card.sides[index])
        }

        KanteSectionLabel { text: card.texts.t("opening.hinge") }
        KanteSegmented {
            id: hingeChoice
            objectName: "hingeChoice"
            model: card.hinges.map(h => card.texts.t("hinge." + h))
            onActivated: index => card.setSwing("hinge", card.hinges[index])
        }
    }
}
