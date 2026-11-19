import QtQuick
import io.github.shrippen.klappe
import "lib"
import "../../qml"

// D-15: the screenplay view sets each line in the page's columns (ScriptLook). A cue stands
// indented, a heading at the edge. Undo and redo are the text's own (ScriptHistory): every step
// leaves each line where its kind belongs, and no redo is lost to a new look.
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

    // Where line `line` starts, from the edge of the text.
    function lineX(line) {
        const rows = view.area.text.split("\n")
        let at = 0
        for (let i = 0; i < line; i++) {
            at += rows[i].length + 1
        }
        return view.area.positionToRectangle(at).x - view.area.leftPadding
    }

    // The first line that stands elsewhere than its kind says, or -1.
    function misplaced() {
        const kinds = ctl.scriptKinds(view.area.text)
        for (let i = 0; i < kinds.length; i++) {
            const look = view.lineLooks[kinds[i]]
            if (look.align !== "left") {
                continue
            }
            if (Math.abs(lineX(i) - look.left) > 1) {
                return i
            }
        }
        return -1
    }

    Timer {
        interval: 0
        running: true
        onTriggered: {
            const example = Qt.resolvedUrl("../../../../testdata/beispiel.klappe")
            if (!ctl.openProject(example)) {
                return root.fail("project did not open")
            }

            const kinds = ctl.scriptKinds(view.area.text)
            const cue = kinds.indexOf("c")
            if (kinds[0] !== "h" || cue < 0) {
                return root.fail("the example starts with a heading and has a cue: " + kinds)
            }
            if (lineX(0) > 1 || lineX(cue) < view.lineLooks.c.left - 1 || view.lineLooks.c.left < 50) {
                return root.fail("the cue is not indented: heading " + lineX(0) + ", cue " + lineX(cue))
            }
            if (misplaced() >= 0) {
                return root.fail("line " + misplaced() + " of the example is misplaced")
            }
            if (view.tools.canUndo) {
                return root.fail("the look of a loaded text is no undo step")
            }

            // Typed in steps: the action line becomes a cue once its speech follows.
            const area = view.area
            area.insert(area.length, "\n\nANNA")
            area.insert(area.length, "\n")
            area.insert(area.length, "Hallo.")
            const typed = area.text
            if (ctl.scriptKinds(typed).slice(-2) !== "cd" || misplaced() >= 0) {
                return root.fail("the typed cue is not set as one: line " + misplaced())
            }

            let undone = 0
            while (view.tools.canUndo) {
                view.undoText()
                undone += 1
                if (misplaced() >= 0) {
                    const m = misplaced()
                    return root.fail("after undo " + undone + " line " + m + " is misplaced: kind '"
                        + ctl.scriptKinds(area.text)[m] + "' at " + lineX(m) + " text "
                        + JSON.stringify(area.text.split("\n")[m]) + " kinds " + ctl.scriptKinds(area.text))
                }
            }
            // Each line break starts a step: the cue, the break, the speech.
            if (undone !== 3 || area.text.indexOf("ANNA") >= 0) {
                return root.fail("undo did not take the typing back in 3 steps: " + undone)
            }

            let redone = 0
            while (view.tools.canRedo) {
                view.redoText()
                redone += 1
                if (misplaced() >= 0) {
                    return root.fail("after redo " + redone + " line " + misplaced() + " is misplaced")
                }
            }
            if (redone !== undone || area.text !== typed) {
                return root.fail("redo lost steps: " + undone + " undone, " + redone + " redone")
            }
            // Review of PR 278: lines put in above a heading move it to another number; it
            // turns into action and must lose the bold of its old kind.
            area.text = "Text\n\nINT. A\n\nMehr"
            if (view.tools.faceOf(2) !== "bold") {
                return root.fail("a heading is bold: " + view.tools.faceOf(2))
            }
            area.insert(5, "y\nz\nw")
            if (ctl.scriptKinds(area.text)[4] !== "a" || view.tools.faceOf(4) !== "") {
                return root.fail("the old heading in line 4 is " + ctl.scriptKinds(area.text)[4]
                    + ", drawn '" + view.tools.faceOf(4) + "'")
            }
            Qt.exit(0)
        }
    }
}
