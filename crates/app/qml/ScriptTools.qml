import QtQuick
import io.github.shrippen.klappe

// The native helpers of the screenplay editor (D-15): ScriptLook sets each line as its kind and
// draws its emphasis (D-36), ScriptHistory keeps the undo history of the source, ScriptClipboard
// copies it. Their own file, so the QML tests, which do
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

    /// The source had `removed` at `at` replaced by `put`: one change of the history.
    function record(at, removed, put) {
        history.record(at, removed, put)
    }

    /// The next change is a step of its own, however soon it comes.
    function seal() {
        history.seal()
    }

    /// The change that takes back the last one, `{ at, removed, put }`; empty for none.
    function undo() {
        return history.undo()
    }

    function redo() {
        return history.redo()
    }

    /// The emphasis of the lines `from` to `from + removed` is that of `lines`: per line three
    /// numbers for each stretch, first character, end and 1 bold + 2 italic + 4 underline;
    /// -1 for `removed` is all lines, which are drawn only with the text that is set next.
    function setRuns(from, removed, lines) {
        look.setRuns(from, removed, lines)
    }

    /// Draws every line again, after the emphasis of all lines was set for a text that stands.
    function redraw() {
        look.redraw()
    }

    /// The emphasis drawn in line `line` as "from-to" and b, i, u per stretch; only tests ask.
    function emphasisOf(line) {
        return look.drawnRuns(line)
    }

    /// Puts `text` on the clipboard.
    function copy(text) {
        clipboard.setText(text)
    }

    ScriptLook { id: look }
    ScriptHistory { id: history }
    ScriptClipboard { id: clipboard }
}
