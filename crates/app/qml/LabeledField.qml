import QtQuick
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import Kante

// A number field with its label above it (LH-90), so a field is never named by its
// placeholder alone. `invalid` marks a refused text until it is edited.
ColumnLayout {
    id: box

    required property string label
    /// The object name of the text field inside, for tests and for the card that reads it.
    property alias fieldName: input.objectName
    property alias text: input.text
    /// Shown greyed while the field is empty, e.g. the default in force.
    property alias placeholderText: input.placeholderText
    property alias invalid: input.invalid
    /// The text the card put in last; a card compares what is typed against it.
    property string shown: ""
    /// The keyboard hints of the field; a field that takes more than a number (16:9) changes them.
    property alias inputHints: input.inputMethodHints
    /// The width of the field; the card sets it where the value is longer than a few digits.
    property real fieldWidth: Kirigami.Units.gridUnit * 5
    /// The pointer is over the field, for a tooltip on the box.
    readonly property bool hovered: input.hovered
    /// The field has the focus: letters and Delete are text, not keys for the selection.
    readonly property bool typing: input.activeFocus

    /// What the field held when it got the keyboard or last applied: a value applies on Enter
    /// and when the field loses the keyboard, only if it differs from this (D-46).
    property string since: ""

    /// The selection when the field got the keyboard.
    property string focusKey: ""

    /// Enter in the field, or the keyboard left it with a changed text.
    signal accepted()

    // The selection now, read from the controller of the card the field is in (empty without
    // one): not from a binding, which may not have followed a change yet.
    function selection() {
        let item = box.parent
        while (item && item.controller === undefined) {
            item = item.parent
        }
        return item ? item.controller.selectedKey : ""
    }

    spacing: 0

    /// Puts the keyboard in the field with its text selected.
    function focusAll() {
        input.forceActiveFocus()
        input.selectAll()
    }

    KanteSectionLabel { text: box.label }
    KanteTextField {
        id: input
        Layout.preferredWidth: box.fieldWidth
        Accessible.name: box.label
        inputMethodHints: Qt.ImhFormattedNumbersOnly
        onTextChanged: invalid = false
        onAccepted: {
            box.accepted()
            box.since = input.text
        }
        onActiveFocusChanged: {
            if (activeFocus) {
                box.since = text
                box.focusKey = box.selection()
                return
            }
            // Another element is selected: what was typed was for the one before.
            if (text === box.since || box.selection() !== box.focusKey) {
                return
            }
            // Set first: the card's apply may put the plan's value in the field.
            box.since = text
            box.accepted()
            box.since = input.text
        }
    }
}
