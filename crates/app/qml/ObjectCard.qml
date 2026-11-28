import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante
import "ProjectData.js" as ProjectData

// An object is selected (LH-25, LH-12, LH-17, LH-90), a section of the selection card (D-04):
// its size as two fields (its name or kind is the card's head); under "More" its height (empty
// is none), its underside (the value in force; emptied it stands on what is below again) and
// filing it in the library under a name. Enter in a field, or the keyboard leaving it with a
// changed text, sets that field's value (D-46); a value that is no length marks the field and changes nothing. Under "More"
// too: its kind (a list of the library, LH-114), mirroring (LH-115), the size of its entry again
// (LH-116) and a label bound to it (LH-118).
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
    /// The library as the kind list shows it: built in first, then the project's own.
    readonly property var kinds: {
        const choices = card.controller.choicesJson ? JSON.parse(card.controller.choicesJson) : {}
        return (choices.object || []).map(o => ({
            value: o.kind,
            text: o.custom ? (o.label || o.kind) : card.texts.t("objectKind." + o.kind),
            group: !o.custom ? card.texts.t("objectGroup." + o.group) : (o.group || card.texts.t("library.custom"))
        }))
    }
    /// The breakdown's elements the object can stand for (D-22), each as the box shows it.
    readonly property var elements: ProjectData.of(card.controller.projectJson).elements || []
    readonly property var elementNames: elements.map(e => card.texts.t("object.elementEntry",
        { category: card.texts.t("elementCategory." + e.category), name: e.name }))
    /// Entry 0 of the box is "none".
    readonly property int elementIndex: info && info.element ? 1 + elements.findIndex(e => e.id === info.element) : 0
    readonly property int kindIndex: info ? kinds.findIndex(k => k.value === info.library) : -1
    /// A field has the focus: letters and Delete are text, not keys for the selection.
    readonly property bool typing: widthField.typing || depthField.typing
        || heightField.typing || baseField.typing || libraryName.activeFocus || kindBox.popupOpen
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
    }

    KanteButton {
        objectName: "objectMore"
        visible: card.showMore
        emphasis: KanteButton.Emphasis.Quiet
        size: KanteButton.Size.Small
        text: card.texts.t(card.expanded ? "card.less" : "card.more")
        onClicked: card.expanded = !card.expanded
    }

    // Kind, mirror, size of the entry, label (LH-114 to LH-116, LH-118).
    KanteSearchCombo {
        id: kindBox
        objectName: "kindBox"
        visible: card.expanded
        Layout.fillWidth: true
        Accessible.name: card.texts.t("object.kind")
        placeholderText: card.texts.t("object.kind")
        textRole: "text"
        sectionRole: "group"
        model: card.kinds
        // A pick sets the kind itself, which ends the binding; Binding keeps it.
        onActivated: index => card.controller.changeSelected("kind", model[index].value)
    }
    Binding {
        target: kindBox
        property: "currentIndex"
        value: card.kindIndex
    }

    // The breakdown element the object stands for: scenes shot on this set get it as a suggestion.
    KanteSectionLabel {
        visible: card.expanded
        text: card.texts.t("object.element")
    }
    KanteComboBox {
        id: elementBox
        objectName: "elementBox"
        visible: card.expanded
        Layout.fillWidth: true
        Accessible.name: card.texts.t("object.element")
        model: [card.texts.t("object.noElement")].concat(card.elementNames)
        // A pick sets the index itself, which ends the binding; Binding keeps it.
        onActivated: index => card.controller.setObjectElement(card.info.location, card.info.id,
            index > 0 ? card.elements[index - 1].id : "")
    }
    Binding {
        target: elementBox
        property: "currentIndex"
        value: card.elementIndex
    }

    RowLayout {
        visible: card.expanded
        spacing: Kirigami.Units.smallSpacing

        KanteButton {
            objectName: "objectMirror"
            // The state is the button's checked mark, which a screen reader announces; the click
            // asks the controller, the card shows the answer.
            checked: card.info !== null && card.info.mirrored === true
            Accessible.checkable: true
            text: card.texts.t("object.mirror")
            onClicked: card.controller.changeSelected("mirror", "")
            QQC2.ToolTip.text: card.texts.t("object.mirrorTip", { key: card.texts.t("key.shift") + "+M" })
            QQC2.ToolTip.visible: hovered
            QQC2.ToolTip.delay: Kirigami.Units.toolTipDelay
        }
        KanteButton {
            objectName: "objectResetSize"
            text: card.texts.t("object.resetSize")
            onClicked: card.controller.changeSelected("resetSize", "")
            QQC2.ToolTip.text: card.texts.t("object.resetSizeTip", { key: card.texts.t("key.shift") + "+G" })
            QQC2.ToolTip.visible: hovered
            QQC2.ToolTip.delay: Kirigami.Units.toolTipDelay
        }
        KanteButton {
            objectName: "objectLabel"
            text: card.texts.t("object.label")
            onClicked: card.controller.labelSelected()
            QQC2.ToolTip.text: card.texts.t("object.labelTip", { key: card.texts.t("key.shift") + "+L" })
            QQC2.ToolTip.visible: hovered
            QQC2.ToolTip.delay: Kirigami.Units.toolTipDelay
        }
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
