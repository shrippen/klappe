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

    /// Enter in the field.
    signal accepted()

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
        onAccepted: box.accepted()
    }
}
