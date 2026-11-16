import QtQuick
import io.github.shrippen.klappe
import "lib"

// LH-19: the points of an area. The middle grip of an edge of the selected area, dragged, makes a
// point where it lands, one undo step; a click on a point selects it, and Delete (removeSelected)
// takes it out, one undo step. An area keeps three points: Delete is refused, the reason on the
// status line.
Item {
    id: root

    Watchdog {}
    Controller { id: ctl }

    function fail(message) {
        console.error(message)
        Qt.exit(1)
    }

    // The points of the outline of the area `d2` as "x,y" strings, rounded.
    function outline() {
        const shapes = JSON.parse(ctl.planJson).shapes.filter(s => s.key === "drawing:d2" && s.points.length > 4)
        if (shapes.length === 0) {
            return []
        }
        const points = []
        for (let i = 0; i < shapes[0].points.length; i += 2) {
            points.push(Math.round(shapes[0].points[i] * 100) / 100 + "," + Math.round(shapes[0].points[i + 1] * 100) / 100)
        }

        // The outline is closed: it ends where it began.
        return points.slice(0, -1)
    }

    function grips(kind) {
        return JSON.parse(ctl.planJson).shapes.filter(s => s.key.indexOf(kind + ":d2:") === 0).length
    }

    Timer {
        interval: 0
        running: true
        onTriggered: {
            if (!ctl.openProject(Qt.resolvedUrl("../../../../testdata/beispiel.klappe"))) {
                return root.fail("project did not open")
            }
            ctl.setSnapping(true)

            // A triangle (2, 2), (6, 2), (4, 5), selected by a click on its top edge.
            ctl.setTool("area", "")
            ctl.place(2, 2)
            ctl.place(6, 2)
            ctl.place(4, 5)
            if (!ctl.finishArea()) {
                return root.fail("the area was not made")
            }
            ctl.setTool("select", "")
            ctl.place(3, 2)
            if (ctl.selectedKey !== "drawing:d2" || root.grips("drawingedge") !== 3 || root.grips("drawingpoint") !== 3) {
                return root.fail("the area was not selected with its grips: " + ctl.selectedKey)
            }
            const triangle = ["2,2", "6,2", "4,5"]
            if (JSON.stringify(root.outline()) !== JSON.stringify(triangle)) {
                return root.fail("not the triangle: " + JSON.stringify(root.outline()))
            }

            // The middle of the top edge, two metres up: a new point, the drawing stays selected.
            ctl.drag(4, 2, 4, 1, false)
            ctl.drag(4, 2, 4, 0, true)
            const four = ["2,2", "4,0", "6,2", "4,5"]
            if (JSON.stringify(root.outline()) !== JSON.stringify(four)) {
                return root.fail("no point on the edge: " + JSON.stringify(root.outline()))
            }
            if (ctl.selectedKey !== "drawing:d2" || root.grips("drawingedge") !== 4) {
                return root.fail("the drawing lost its selection: " + ctl.selectedKey)
            }
            ctl.undo()
            if (JSON.stringify(root.outline()) !== JSON.stringify(triangle)) {
                return root.fail("one undo did not take the point back: " + JSON.stringify(root.outline()))
            }
            ctl.redo()

            // A click on the new point selects it; Delete takes it out again.
            ctl.place(4, 0)
            if (ctl.selectedKey !== "drawingpoint:d2:1") {
                return root.fail("the point was not selected: " + ctl.selectedKey)
            }
            if (!ctl.removeSelected() || JSON.stringify(root.outline()) !== JSON.stringify(triangle)) {
                return root.fail("Delete did not take the point out: " + JSON.stringify(root.outline()))
            }
            ctl.undo()
            if (root.outline().length !== 4) {
                return root.fail("one undo did not bring the point back")
            }
            ctl.undo()
            ctl.place(3, 2)

            // Three points stay: the refusal says why.
            ctl.place(4, 5)
            if (ctl.selectedKey !== "drawingpoint:d2:2") {
                return root.fail("the corner was not selected: " + ctl.selectedKey)
            }
            if (ctl.removeSelected() || root.outline().length !== 3 || ctl.status === "") {
                return root.fail("the third point went or no reason: " + ctl.status)
            }
            Qt.exit(0)
        }
    }
}
