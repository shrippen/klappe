import QtQuick
import io.github.shrippen.klappe
import "lib"

// LH-10: a selected door's side and hinge are set through the controller, each in one undo
// step, and the plan shows the leaf on the other side or at the other end. On the example a
// door set at (8, 0) in the north wall (from (0, 0) to (20, 0), y runs down) has its hinge at
// x = 7.55 and its leaf tip below the wall; the card reads both from `describe`.
Item {
    Watchdog {}
    Controller { id: ctl }

    function fail(message) {
        console.error(message)
        Qt.exit(1)
    }

    // The door's points, flat: hinge, then the leaf tip.
    function doorPoints() {
        const door = JSON.parse(ctl.planJson).shapes.find(s => s.key === "opening:0:0" && s.kind === "polygon")
        return door ? door.points : null
    }

    function swing() {
        const d = JSON.parse(ctl.describe("opening:0:0"))
        return d.side + "/" + d.hinge
    }

    Timer {
        interval: 0
        running: true
        onTriggered: {
            const example = Qt.resolvedUrl("../../../../testdata/beispiel.klappe")
            if (!ctl.openProject(example)) {
                return fail("project did not open")
            }
            ctl.setSnapping(false)
            ctl.setTool("door", "")
            ctl.place(8.0, 0.0)
            ctl.setTool("select", "")
            ctl.place(8.0, 0.4)
            if (ctl.selectedKey !== "opening:0:0") {
                return fail("the door was not selected: '" + ctl.selectedKey + "'")
            }

            const start = doorPoints()
            if (swing() !== "right/start" || Math.abs(start[0] - 7.55) > 1e-6 || start[3] < 0.8) {
                return fail("not a right/start door to begin with: " + swing() + " " + JSON.stringify(start))
            }

            if (!ctl.setSelectedField("side", "left")) {
                return fail("the side was refused")
            }
            const left = doorPoints()
            if (swing() !== "left/start" || Math.abs(left[0] - 7.55) > 1e-6 || left[3] > -0.8) {
                return fail("the leaf did not swing to the left: " + swing() + " " + JSON.stringify(left))
            }

            if (!ctl.setSelectedField("hinge", "end")) {
                return fail("the hinge was refused")
            }
            const ended = doorPoints()
            if (swing() !== "left/end" || Math.abs(ended[0] - 8.45) > 1e-6) {
                return fail("the hinge did not move: " + swing() + " " + JSON.stringify(ended))
            }

            if (ctl.setSelectedField("side", "up")) {
                return fail("a side that is none was taken")
            }

            ctl.undo()
            if (swing() !== "left/start") {
                return fail("undo did not take back the hinge alone: " + swing())
            }
            ctl.undo()
            if (swing() !== "right/start") {
                return fail("undo did not take back the side: " + swing())
            }
            Qt.exit(0)
        }
    }
}
