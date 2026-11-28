import QtQuick
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import Kante

// A door or window is selected (LH-10), a section of the selection card (D-04): its kind, its
// width, and its sill and head (LH-90) as fields. Enter, or the keyboard leaving a field, sets that field. Sill and head show the opening's own value; without one the
// field is empty and shows its kind's greyed; an emptied one goes back to its kind's. A value that is no length marks the field and changes nothing.
// A door also shows two choices: the side it opens to and the end its hinge is at.
ColumnLayout {
    id: card

    required property var texts
    required property var controller

    readonly property var info: {
        // Every edit raises the picture revision; the plan may not change (a drag ends as
        // previewed), and the key alone would not tell the new width.
        card.controller.pictureRevision
        const key = card.controller.selectedKey
        if (!key || key.indexOf("opening:") !== 0) {
            return null
        }
        const d = JSON.parse(card.controller.describe(key))
        return d.kind === "opening" ? d : null
    }
    readonly property bool shown: info !== null
    readonly property string title: info ? card.texts.t("openingKind." + info.type) : ""
    /// A field has the focus: letters and Delete are text, not keys for the selection.
    readonly property bool typing: widthField.typing || sillField.typing || headField.typing

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
        if (!widthField.typing) {
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

    RowLayout {
        spacing: Kirigami.Units.smallSpacing

        LabeledField {
            id: widthField
            fieldName: "openingWidthField"
            label: card.texts.t("field.width")
            onAccepted: card.setWidth()
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
