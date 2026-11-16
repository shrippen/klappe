import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// An object is selected (LH-25, LH-12, LH-17, LH-90), a section of the selection card (D-04):
// its name or kind, its size as two fields; under "More" its height (empty is none), its
// underside (the value in force; emptied it stands on what is below again) and filing it in the
// library under a name. Enter in a size field sets the size, Enter in the height or underside
// field that one; a value that is no length marks the field and changes nothing.
ColumnLayout {
    id: card

    required property var texts
    required property var controller

    /// The rest of the fields: the height and the library.
    property bool expanded: false
    /// The button for "More"; the selection card has one for all its sections.
    property bool showMore: true

    readonly property var info: {
        // Every edit raises the picture revision; the plan may not change (a drag ends as
        // previewed), and the key alone would not tell the new size.
        card.controller.pictureRevision
        const key = card.controller.selectedKey
        if (!key || key.indexOf("object:") !== 0) {
            return null
        }
        const d = JSON.parse(card.controller.describe(key))
        return d.kind === "object" ? d : null
    }
    readonly property bool shown: info !== null
    /// A field has the focus: letters and Delete are text, not keys for the selection.
    readonly property bool typing: widthField.typing || depthField.typing
        || heightField.typing || baseField.typing || libraryName.activeFocus
    readonly property string title: info ? (info.label !== "" ? info.label : card.kindOf(info)) : ""

    visible: shown
    spacing: Kirigami.Units.smallSpacing

    onInfoChanged: card.fill()
    Component.onCompleted: card.fill()

    // The kind's name; empty for a kind the catalogue does not know (an old project).
    function kindOf(info) {
        const name = texts.t("objectKind." + info.library)
        return name !== "objectKind." + info.library ? name : ""
    }

    function fill() {
        if (!info) {
            return
        }
        if (!widthField.typing) {
            widthField.text = texts.num(info.width, 2)
        }
        if (!depthField.typing) {
            depthField.text = texts.num(info.depth, 2)
        }
        if (!heightField.typing) {
            heightField.text = card.shownHeight(info)
        }
        baseField.placeholderText = texts.num(info.base, 2)
        if (!baseField.typing) {
            baseField.text = card.ownBase()
        }
        // The name a new library entry starts with.
        if (!libraryName.activeFocus) {
            libraryName.text = info.label !== "" ? info.label : card.kindOf(info)
        }
    }

    // The height as the field shows it: empty when the object has none.
    function shownHeight(info) {
        return info.height === null ? "" : texts.num(info.height, 2)
    }

    // The height field takes the keyboard, its text selected (LH-93).
    function focusHeight() {
        heightField.focusAll()
    }

    function setHeight() {
        heightField.invalid = !controller.setObjectHeight(heightField.text)
    }

    // The object's own underside as the field shows it: empty when it stands on what is below,
    // which the placeholder shows.
    function ownBase() {
        return info.baseDefault ? "" : texts.num(info.base, 2)
    }

    // Sets the underside; an untouched field sends nothing, so stacking is never pinned. An
    // emptied one stands it on what is below again.
    function setBase() {
        if (info && baseField.text.trim() === card.ownBase()) {
            baseField.invalid = false
            return
        }
        const ok = controller.setSelectedField("base", baseField.text)
        if (ok && info) {
            baseField.text = card.ownBase()
        }
        baseField.invalid = !ok
    }

    // The button sets what was changed: the size always, height and underside when they
    // differ. The size edit refills the fields, so what was typed is taken before and put back.
    function apply() {
        const typed = heightField.text
        const typedBase = baseField.text
        const changed = info !== null && typed.trim() !== card.shownHeight(info)
        const baseChanged = info !== null && typedBase.trim() !== card.ownBase()
        // A refused size stops the apply: the height alone would be half of what was asked.
        if (!card.setSize()) {
            return
        }
        if (changed) {
            heightField.text = typed
            card.setHeight()
        }
        if (baseChanged) {
            baseField.text = typedBase
            card.setBase()
        }
    }

    function setSize() {
        const ok = controller.resizeSelected(widthField.text, depthField.text)
        widthField.invalid = !ok
        depthField.invalid = !ok
        return ok
    }

    function file() {
        if (!controller.saveToLibrary(libraryName.text)) {
            libraryName.invalid = true
        }
    }

    KanteSectionLabel {
        objectName: "objectTitle"
        text: card.title
    }

    RowLayout {
        spacing: Kirigami.Units.smallSpacing

        LabeledField {
            id: widthField
            fieldName: "widthField"
            label: card.texts.t("field.width")
            onAccepted: card.setSize()
        }
        LabeledField {
            id: depthField
            fieldName: "depthField"
            label: card.texts.t("field.depth")
            onAccepted: card.setSize()
        }
        KanteButton {
            objectName: "objectApply"
            Layout.alignment: Qt.AlignBottom
            text: card.texts.t("form.apply")
            onClicked: card.apply()
        }
    }

    KanteButton {
        objectName: "objectMore"
        visible: card.showMore
        emphasis: KanteButton.Emphasis.Quiet
        size: KanteButton.Size.Small
        text: card.texts.t(card.expanded ? "card.less" : "card.more")
        onClicked: card.expanded = !card.expanded
    }

    RowLayout {
        visible: card.expanded
        spacing: Kirigami.Units.smallSpacing

        LabeledField {
            id: heightField
            fieldName: "heightField"
            label: card.texts.t("field.height")
            Layout.alignment: Qt.AlignBottom
            onAccepted: card.setHeight()
        }
        LabeledField {
            id: baseField
            fieldName: "baseField"
            label: card.texts.t("field.base")
            Layout.alignment: Qt.AlignBottom
            onAccepted: card.setBase()
        }
    }

    RowLayout {
        visible: card.expanded
        spacing: Kirigami.Units.smallSpacing

        KanteTextField {
            id: libraryName
            objectName: "libraryName"
            Layout.preferredWidth: Kirigami.Units.gridUnit * 8
            Accessible.name: card.texts.t("library.name")
            placeholderText: card.texts.t("library.name")
            onTextChanged: invalid = false
            onAccepted: card.file()
        }
        KanteButton {
            objectName: "libraryFile"
            text: card.texts.t("library.save")
            onClicked: card.file()
            QQC2.ToolTip.text: card.texts.t("library.saveTip")
            QQC2.ToolTip.visible: hovered
            QQC2.ToolTip.delay: Kirigami.Units.toolTipDelay
        }
    }
}
