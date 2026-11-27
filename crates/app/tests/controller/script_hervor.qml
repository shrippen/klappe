import QtQuick
import QtCore
import io.github.shrippen.klappe
import "lib"
import "../../qml"

// D-36: the editor shows emphasis, not its marks. A text with `**fett**` shows `fett` in bold;
// typing in the bold word stays bold and writes `**fexxtt**`; a typed `*x*` becomes italic; a
// step back and forth restores source and display; a copy of a bold word is `**fett**`; the
// lines nobody touched stay byte for byte, saved too (the folder is checked by qml_tests.rs).
Item {
    id: root

    Watchdog {}
    Controller { id: ctl }
    Texts { id: texts; source: ctl.texts }
    ScreenplayView {
        id: view
        width: 1200
        height: 800
        texts: texts
        controller: ctl
        debounceMs: 50
    }

    function fail(message) {
        console.error(message)
        Qt.exit(1)
    }

    // Types `text` at the cursor, a character at a time, as the keyboard does.
    function type(text) {
        for (const c of text) {
            const at = view.area.cursorPosition
            view.area.insert(at, c)
            view.area.cursorPosition = at + 1
        }
    }

    function shown(line) {
        return view.area.text.split("\n")[line]
    }

    function sourceLine(line) {
        return view.doc.read().split("\n")[line]
    }

    Timer {
        interval: 0
        running: true
        onTriggered: {
            const base = StandardPaths.writableLocation(StandardPaths.TempLocation) + "/klappe-export-test"
            if (!ctl.openProject(base + "/hervor-projekt")) {
                return root.fail("project did not open: " + ctl.status)
            }
            const area = view.area
            const doc = view.doc
            const original = ctl.scriptText

            // Loaded: no marks, the words drawn bold and italic, the source as it was.
            if (shown(2) !== "Das ist fett und kursiv." || doc.read() !== original) {
                return root.fail("the loaded line: " + JSON.stringify(shown(2)))
            }
            if (view.tools.emphasisOf(2) !== "8-12b 17-23i") {
                return root.fail("the emphasis drawn: " + view.tools.emphasisOf(2))
            }
            if (view.tools.canUndo) {
                return root.fail("loading is no step to take back")
            }

            // A copy over a bold word is Fountain; a plain one has no marks.
            const second = area.text.indexOf("Das ist") + 8
            area.select(second, second + 4)
            if (doc.selectedSource() !== "**fett**") {
                return root.fail("the copy of a bold word: " + doc.selectedSource())
            }
            area.select(second - 8, second - 1)
            if (doc.selectedSource() !== "Das ist") {
                return root.fail("the copy of plain words: " + doc.selectedSource())
            }

            // Typing inside a bold word stays bold.
            const start = area.text.indexOf("Das ist")
            area.cursorPosition = start + 10
            type("xx")
            if (sourceLine(2) !== "Das ist **fexxtt** und *kursiv*." || shown(2) !== "Das ist fexxtt und kursiv.") {
                return root.fail("typed in a bold word: " + JSON.stringify(sourceLine(2)) + " / " + JSON.stringify(shown(2)))
            }
            if (view.tools.emphasisOf(2) !== "8-14b 19-25i") {
                return root.fail("the emphasis after typing: " + view.tools.emphasisOf(2))
            }

            // The other lines are untouched: not canonical, with a backslash and a note.
            const typedXx = doc.read()
            const before = original.split("\n")
            const now = typedXx.split("\n")
            for (let i = 0; i < before.length; i++) {
                if (i !== 2 && before[i] !== now[i]) {
                    return root.fail("line " + i + " changed: " + JSON.stringify(now[i]))
                }
            }

            // `*x*` typed by hand is italic `x`.
            area.cursorPosition = area.length
            type("\nNeu ")
            type("*x*")
            const last = doc.read().split("\n").length - 1
            if (sourceLine(last) !== "Neu *x*" || shown(last) !== "Neu x") {
                return root.fail("typed *x*: " + JSON.stringify(sourceLine(last)) + " / " + JSON.stringify(shown(last)))
            }
            if (view.tools.emphasisOf(last) !== "4-5i" || area.cursorPosition !== area.length) {
                return root.fail("italic x: " + view.tools.emphasisOf(last) + ", cursor " + area.cursorPosition)
            }
            const typedAll = doc.read()

            // A step back takes back what was typed behind the line break (one step, typed
            // without a pause), source and display together; a step forward gives it back.
            view.undoText()
            if (doc.read() !== typedXx + "\n" || shown(last) !== "Neu " && shown(last) !== "") {
                return root.fail("undo: " + JSON.stringify(doc.read().slice(-12)) + " / " + JSON.stringify(shown(last)))
            }
            if (view.tools.emphasisOf(last) !== "") {
                return root.fail("undo leaves emphasis: " + view.tools.emphasisOf(last))
            }
            view.redoText()
            if (doc.read() !== typedAll || shown(last) !== "Neu x" || view.tools.emphasisOf(last) !== "4-5i") {
                return root.fail("redo: " + JSON.stringify(sourceLine(last)) + " / " + JSON.stringify(shown(last)))
            }

            // Back to the bold word: the file keeps what was typed there.
            view.undoText()
            view.undoText()
            if (doc.read() !== typedXx || !view.tools.canUndo) {
                return root.fail("undo goes wrong: " + JSON.stringify(doc.read().slice(-20)))
            }
            const lines = JSON.parse(ctl.scriptShownJson(doc.read())).map(l => l.text).join("\n")
            if (area.text !== lines) {
                return root.fail("the display is not the projection of the source: " + JSON.stringify(area.text))
            }
            view.flush()
            if (ctl.scriptText !== doc.read() || !ctl.save()) {
                return root.fail("save: " + ctl.status)
            }
            Qt.exit(0)
        }
    }
}
