import QtQuick
import io.github.shrippen.klappe
import "lib"

// After a grip drag the element itself is selected again, so its card shows the new values:
// the width grip of a door on the example's north wall (the line from (0, 0) to (20, 0)), the
// point of a free form and of an area (a click on those selects the point itself).
Item {
    id: root

    Watchdog {}
    Controller { id: ctl }

    function fail(message) {
        console.error(message)
        Qt.exit(1)
    }

    function doorPoints() {
        const door = JSON.parse(ctl.planJson).shapes.find(s => s.key === "opening:0:0" && s.kind === "polygon")
        return door ? door.points : null
    }

    Timer {
        interval: 0
        running: true
        onTriggered: {
            if (!ctl.openProject(Qt.resolvedUrl("../../../../testdata/beispiel.klappe"))) {
                return root.fail("project did not open")
            }
            ctl.setSnapping(false)
            ctl.setTool("door", "")
            ctl.place(8.0, 0.0)
            ctl.setTool("select", "")
            ctl.place(8.0, 0.4)
            if (ctl.selectedKey !== "opening:0:0") {
                return root.fail("the door was not selected: '" + ctl.selectedKey + "'")
            }

            // The grip at the door's far end (8.45), pulled to 9: the door is 1.4 m wide.
            ctl.drag(8.45, 0.0, 9.0, 0.0, false)
            ctl.drag(8.45, 0.0, 9.0, 0.0, true)
            if (Math.abs(root.doorPoints()[0] - 7.55) > 1e-6 || ctl.canUndo === false) {
                return root.fail("the grip did not widen the door: " + JSON.stringify(root.doorPoints()))
            }
            if (ctl.selectedKey !== "opening:0:0") {
                return root.fail("the door is not selected after the grip drag: '" + ctl.selectedKey + "'")
            }

            // A free form's point: a click on it selects the point (Delete takes it out), a
            // drag of it ends with the form selected.
            ctl.setTool("freeform", "")
            for (const [x, y] of [[2, 10], [4, 10], [4, 12], [3, 12], [3, 14], [2, 14]]) {
                ctl.place(x, y)
            }
            ctl.place(2, 10)
            ctl.setTool("select", "")
            ctl.place(2.5, 10.5)
            const form = ctl.selectedKey
            ctl.place(4, 10)
            if (ctl.selectedKey.indexOf("objectgrip:") !== 0) {
                return root.fail("the click did not take the point: '" + ctl.selectedKey + "'")
            }
            ctl.drag(4, 10, 5, 9, false)
            ctl.drag(4, 10, 5, 9, true)
            if (ctl.selectedKey !== form) {
                return root.fail("the form is not selected after its point was dragged: '" + ctl.selectedKey + "'")
            }

            // An area's point likewise: the drawing.
            ctl.setTool("area", "")
            for (const [x, y] of [[12, 2], [16, 2], [14, 5]]) {
                ctl.place(x, y)
            }
            ctl.finishArea()
            ctl.setTool("select", "")
            ctl.place(13, 2)
            const area = ctl.selectedKey
            ctl.place(14, 5)
            if (ctl.selectedKey.indexOf("drawingpoint:") !== 0) {
                return root.fail("the click did not take the corner: '" + ctl.selectedKey + "'")
            }
            ctl.drag(14, 5, 14, 6, false)
            ctl.drag(14, 5, 14, 6, true)
            if (ctl.selectedKey !== area) {
                return root.fail("the area is not selected after its point was dragged: '" + ctl.selectedKey + "'")
            }
            Qt.exit(0)
        }
    }
}
