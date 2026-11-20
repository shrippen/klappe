import QtQuick
import io.github.shrippen.klappe

// The native helpers of the screenplay editor (D-15): ScriptLook sets each line as its kind,
// ScriptHistory keeps the undo history of the text. Their own file, so the QML tests, which do
// not know native types, load `ScreenplayView` with a stand-in (`toolsSource`).
Item {
    property alias document: look.document
    property alias mutedColor: look.mutedColor
    readonly property alias canUndo: history.canUndo
    readonly property alias canRedo: history.canRedo

    /// `kinds`: one letter per line; `looks`: per letter its margins in pixels and its face.
    function setLooks(kinds, looks) {
        look.apply(kinds, looks)
    }

    /// How line `line` is drawn: "bold", "muted" or ""; only the test of the look asks.
    function faceOf(line) {
        return look.drawnFace(line)
    }

    /// The text came from outside: nothing to take back.
    function forget() {
        history.reset()
    }

    /// Puts `text` in place of `from` to `to` as one change of the history.
    function replace(from, to, text) {
        history.replace(from, to, text)
    }

    /// Where the cursor goes after the step, -1 for none.
    function undo() {
        return history.undo()
    }

    function redo() {
        return history.redo()
    }

    ScriptLook { id: look }
    ScriptHistory {
        id: history
        document: look.document
    }
}
