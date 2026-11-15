import QtQuick
import io.github.shrippen.klappe
import "lib"

// LH-92: the free form tool. Clicks set the points of an outline; a click on the first point or
// Enter (finishArea) closes it into one new object without a height, one undo step. Escape
// (cancelArea) and another tool drop the points. In 3D the clicks set points on the floor.
Item {
    Watchdog {}
    Controller { id: ctl }

    function fail(message) {
        console.error(message)
        Qt.exit(1)
    }

    // The keys of the plan's objects.
    function objectKeys() {
        const keys = JSON.parse(ctl.planJson).shapes.map(s => String(s.key)).filter(k => k.indexOf("object:") === 0)

        return Array.from(new Set(keys))
    }

    // The outline of the object `key` as "x,y" strings, rounded.
    function outline(key) {
        const shape = JSON.parse(ctl.planJson).shapes.find(s => s.key === key && s.points.length > 2)
        const points = []
        for (let i = 0; i < shape.points.length; i += 2) {
            points.push(Math.round(shape.points[i] * 100) / 100 + "," + Math.round(shape.points[i + 1] * 100) / 100)
        }

        return points
    }

    // The key that is in `after` and not in `before`.
    function added(before, after) {
        const fresh = after.filter(k => before.indexOf(k) < 0)

        return fresh.length === 1 ? fresh[0] : ""
    }

    // Clicks the points of an L, 2 m wide and 4 m deep, from (2, 10).
    function clickL() {
        for (const [x, y] of [[2, 10], [4, 10], [4, 12], [3, 12], [3, 14], [2, 14]]) {
            ctl.place(x, y)
        }
    }

    Timer {
        interval: 0
        running: true
        onTriggered: {
            if (!ctl.openProject(Qt.resolvedUrl("../../../../testdata/beispiel.klappe"))) {
                return fail("project did not open")
            }
            ctl.setSnapping(true)
            const keys = objectKeys()
            const missing = ctl.missingHeights
            if (!ctl.setTool("freeform", "")) {
                return fail("no tool freeform")
            }

            // The points are a draft: nothing is made before the outline closes.
            clickL()
            if (ctl.areaPoints !== 6 || objectKeys().length !== keys.length || ctl.canUndo) {
                return fail("the draft is not six points: " + ctl.areaPoints)
            }

            // A click on the first point closes it: one object with that outline, no height.
            if (!ctl.place(2, 10)) {
                return fail("the closing click was refused")
            }
            const key = added(keys, objectKeys())
            if (key === "" || ctl.areaPoints !== 0) {
                return fail("no new object: " + JSON.stringify(objectKeys()) + ", " + ctl.areaPoints)
            }
            const want = ["2,10", "4,10", "4,12", "3,12", "3,14", "2,14"]
            if (JSON.stringify(outline(key)) !== JSON.stringify(want)) {
                return fail("the outline is wrong: " + JSON.stringify(outline(key)))
            }
            if (ctl.missingHeights !== missing + 1) {
                return fail("the form has a height: " + ctl.missingHeights)
            }

            // One undo takes it back.
            ctl.undo()
            if (objectKeys().length !== keys.length || ctl.missingHeights !== missing || ctl.canUndo) {
                return fail("one undo does not take back the form")
            }

            // Enter (finishArea) closes it too; below three points it does nothing.
            ctl.place(2, 10)
            ctl.place(6, 10)
            if (ctl.finishArea()) {
                return fail("two points closed")
            }
            ctl.place(2, 13)
            if (!ctl.finishArea() || objectKeys().length !== keys.length + 1) {
                return fail("Enter did not close the triangle")
            }
            ctl.undo()

            // Escape drops the draft; so does another tool.
            ctl.place(2, 10)
            ctl.place(6, 10)
            ctl.cancelArea()
            if (ctl.areaPoints !== 0 || ctl.canUndo) {
                return fail("Escape kept the points")
            }
            ctl.place(2, 10)
            ctl.setTool("area", "")
            if (ctl.areaPoints !== 0) {
                return fail("another tool kept the points")
            }
            ctl.setTool("freeform", "")

            // In 3D the clicks land on the floor.
            ctl.setSnapping(false)
            ctl.toggleView()
            ctl.freeSize(800, 600)
            for (const [x, y] of [[300, 400], [500, 400], [400, 520]]) {
                if (!ctl.freePlace(x, y)) {
                    return fail("no floor at " + x + "," + y)
                }
            }
            if (!ctl.freePlace(300, 400) || objectKeys().length !== keys.length + 1) {
                return fail("the closing click in 3D made no form")
            }
            ctl.undo()
            Qt.exit(0)
        }
    }
}
