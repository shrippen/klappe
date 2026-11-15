import QtQuick
import io.github.shrippen.klappe
import "lib"

// The door and window tools (LH-10) on the example: a click on the north wall (the line from
// (0, 0) to (20, 0)) sets a door in one undo step, the plan shows the gap and the door, a
// click with the select tool takes it, a drag slides it along the wall, the card's width
// edit and Delete change it, each in one undo step.
Item {
    id: root

    Watchdog {}
    Controller { id: ctl }

    function fail(message) {
        console.error(message)
        Qt.exit(1)
    }

    function shapes() {
        return JSON.parse(ctl.planJson).shapes
    }

    // The pieces of the north wall (wall:0), and the door's polygon.
    function wallPieces() {
        return root.shapes().filter(s => s.key === "wall:0").length
    }

    function doorPoints() {
        const door = root.shapes().find(s => s.key === "opening:0:0" && s.kind === "polygon")
        return door ? door.points : null
    }

    Timer {
        interval: 0
        running: true
        onTriggered: {
            const example = Qt.resolvedUrl("../../../../testdata/beispiel.klappe")
            if (!ctl.openProject(example)) {
                return root.fail("project did not open")
            }
            ctl.setSnapping(false)

            if (root.wallPieces() !== 1) {
                return root.fail("the north wall was not whole to begin with")
            }

            // A click away from every wall sets nothing.
            ctl.setTool("door", "")
            ctl.place(10.0, 6.0)
            if (ctl.canUndo || root.doorPoints() !== null) {
                return root.fail("a click in the room set a door")
            }

            ctl.place(8.0, 0.0)
            if (root.wallPieces() !== 2) {
                return root.fail("the wall has no gap: " + root.wallPieces() + " pieces")
            }
            const first = root.doorPoints()
            if (!first) {
                return root.fail("no door in the plan")
            }
            // The hinge is at 8 - 0.45.
            if (Math.abs(first[0] - 7.55) > 1e-6) {
                return root.fail("the hinge is not at 7.55: " + first[0])
            }

            ctl.setTool("window", "")
            ctl.place(14.0, 0.0)
            if (!root.shapes().some(s => s.key === "opening:0:1")) {
                return root.fail("no window in the plan")
            }

            ctl.setTool("select", "")
            ctl.place(8.0, 0.4)
            if (ctl.selectedKey !== "opening:0:0") {
                return root.fail("the click did not take the door: '" + ctl.selectedKey + "'")
            }

            // Along the wall by 2 m.
            ctl.drag(8.0, 0.4, 10.0, 0.4, true)
            const moved = root.doorPoints()
            if (!moved || Math.abs(moved[0] - 9.55) > 1e-6) {
                return root.fail("the drag did not slide the door: " + JSON.stringify(moved))
            }

            if (!ctl.setOpeningWidth("1,2")) {
                return root.fail("the width was refused")
            }
            const widened = root.doorPoints()
            if (!widened || Math.abs(widened[0] - 9.4) > 1e-6) {
                return root.fail("the width did not change: " + JSON.stringify(widened))
            }
            if (ctl.setOpeningWidth("breit")) {
                return root.fail("a width that is no number was taken")
            }

            ctl.removeSelected()
            // The window is the only opening left, now with index 0.
            if (root.doorPoints() !== null || !root.shapes().some(s => s.key === "opening:0:0")) {
                return root.fail("Delete did not remove the door alone")
            }

            ctl.undo()
            if (!root.doorPoints()) {
                return root.fail("undo did not bring the door back")
            }
            Qt.exit(0)
        }
    }
}
