import QtQuick
import io.github.shrippen.klappe
import "lib"

// A change of tool ends what a key armed for the next click: after Shift+F and the wall
// tool, a click sets no mark (LH-104). A new choice in the same tool keeps it armed.
Item {
    id: root

    Watchdog {}
    Controller { id: ctl }

    function fail(message) {
        console.error(message)
        Qt.exit(1)
    }

    function marks() {
        return JSON.parse(ctl.planJson).shapes.filter(s => s.key.indexOf("mark:elisabeth:") === 0).length
    }

    Timer {
        interval: 0
        running: true
        onTriggered: {
            if (!ctl.openProject(Qt.resolvedUrl("../../../../testdata/beispiel.klappe"))) {
                return root.fail("project did not open")
            }
            ctl.setSnapping(false)
            ctl.setTool("select", "")
            ctl.place(3, 6.5)
            const before = root.marks()

            ctl.arm("nextMark")
            ctl.setTool("select", "")
            if (ctl.armed !== "nextMark") {
                return root.fail("the same tool ended the arm: '" + ctl.armed + "'")
            }

            ctl.setTool("wall", "")
            if (ctl.armed !== "") {
                return root.fail("a tool change left the arm: '" + ctl.armed + "'")
            }
            ctl.place(14, 9)
            if (root.marks() !== before) {
                return root.fail("a click after the tool change set a mark: " + root.marks() + " of " + before)
            }
            Qt.exit(0)
        }
    }
}
