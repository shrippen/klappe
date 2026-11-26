import QtQuick
import io.github.shrippen.klappe
import "lib"
import "../../qml"

// The outline in the window (DB-21) with the real controller: the list's outline follows the text,
// a scene moves down with its text and the outline follows at once, a section moves with its
// scenes, each move is one step of the text's undo history, and a move that is none changes nothing.
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

    function names() {
        return view.outline.map(e => e.text).join("|")
    }

    Timer {
        interval: 0
        running: true
        onTriggered: {
            const example = Qt.resolvedUrl("../../../../testdata/beispiel.klappe")
            if (!ctl.openProject(example)) {
                return root.fail("project did not open")
            }
            const area = view.area
            const book = "# Akt I\n\n= Es beginnt.\n\nINT. A - TAG\n\nEins.\n\nINT. B - TAG\n\nZwei.\n\n# Akt II\n\nINT. C - TAG\n\nDrei.\n"
            ctl.editScript(book)
            if (root.names() !== "Akt I|Es beginnt.|INT. A - TAG|INT. B - TAG|Akt II|INT. C - TAG") {
                return root.fail("the outline of the text: " + root.names())
            }

            // Scene A (line 4) down: it passes B.
            view.moveBlock(4, "down")
            const swapped = "# Akt I\n\n= Es beginnt.\n\nINT. B - TAG\n\nZwei.\n\nINT. A - TAG\n\nEins.\n\n# Akt II\n\nINT. C - TAG\n\nDrei.\n"
            if (area.text !== swapped || ctl.scriptText !== swapped) {
                return root.fail("scene down: " + JSON.stringify(area.text))
            }
            if (root.names() !== "Akt I|Es beginnt.|INT. B - TAG|INT. A - TAG|Akt II|INT. C - TAG") {
                return root.fail("the outline follows at once: " + root.names())
            }

            // One undo step takes the move back.
            view.undoText()
            if (area.text !== book) {
                return root.fail("undo of the move: " + JSON.stringify(area.text))
            }
            view.flush()

            // The first scene cannot go up past its section's heading and the first section not up.
            view.moveBlock(0, "up")
            if (area.text !== book) {
                return root.fail("the first section does not move up")
            }

            // Akt I with its scenes goes below Akt II.
            view.moveBlock(0, "down")
            const sections = "# Akt II\n\nINT. C - TAG\n\nDrei.\n\n# Akt I\n\n= Es beginnt.\n\nINT. A - TAG\n\nEins.\n\nINT. B - TAG\n\nZwei.\n"
            if (area.text !== sections || ctl.scriptText !== sections) {
                return root.fail("section down: " + JSON.stringify(area.text))
            }
            Qt.exit(0)
        }
    }
}
