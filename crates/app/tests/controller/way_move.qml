import QtQuick
import io.github.shrippen.klappe
import "lib"

// A whole way moves with Ctrl held on its grip, and by the arrow keys (LH-107), and becomes
// straight (LH-108), on the example: Elisabeth walks (3, 6.5), (10, 6.5), (16, 3).
Item {
    id: root

    Watchdog {}
    Controller { id: ctl }

    function fail(message) {
        console.error(message)
        Qt.exit(1)
    }

    function mark(index) {
        const m = JSON.parse(ctl.planJson).shapes.find(s => s.key === "mark:elisabeth:" + index && s.kind === "circle")
        return m.points[0].toFixed(2) + "," + m.points[1].toFixed(2)
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

            // The mark takes the figure; its grips show; the one in the middle of the way to
            // mark 1 is picked.
            ctl.place(3, 6.5)
            ctl.place(6.5, 6.5)
            if (ctl.selectedKey !== "figurevia:elisabeth:1:new0") {
                return root.fail("the grip was not selected: '" + ctl.selectedKey + "'")
            }
            if (ctl.wayCurve() !== "smooth") {
                return root.fail("the way is not smooth: '" + ctl.wayCurve() + "'")
            }

            // Without Ctrl the drag bends the way; with it, the whole way moves.
            ctl.lassoSelection = true
            ctl.drag(6.5, 6.5, 6.5, 8.5, false)
            ctl.drag(6.5, 6.5, 6.5, 8.5, true)
            ctl.lassoSelection = false
            if (root.mark(0) !== "3.00,8.50" || root.mark(1) !== "10.00,8.50" || root.mark(2) !== "16.00,3.00") {
                return root.fail("the way did not move: " + root.mark(0) + " " + root.mark(1) + " " + root.mark(2))
            }
            if (ctl.selectedKey !== "figurevia:elisabeth:1:new0") {
                return root.fail("the grip is not selected after the drag: '" + ctl.selectedKey + "'")
            }

            // An arrow key: one grid step (0.25 m) to the right.
            if (!ctl.nudgeWay(1, 0) || root.mark(0) !== "3.25,8.50" || root.mark(1) !== "10.25,8.50") {
                return root.fail("the arrow key did not move the way: " + root.mark(0) + " " + root.mark(1))
            }

            // The shape, for a grip as for a mark.
            if (!ctl.setWayCurve("straight") || ctl.wayCurve() !== "straight") {
                return root.fail("the way did not become straight")
            }
            ctl.undo()
            if (ctl.wayCurve() !== "smooth") {
                return root.fail("undo did not give the curve back")
            }

            // Another selection has no way to move: the arrow keys leave it.
            ctl.clearSelection()
            ctl.place(16, 3)
            if (ctl.nudgeWay(1, 0)) {
                return root.fail("an arrow key moved something that is no way")
            }
            Qt.exit(0)
        }
    }
}
