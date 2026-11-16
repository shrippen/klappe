import QtQuick
import io.github.shrippen.klappe
import "lib"

// A key arms the next click for the selected figure or camera (LH-104, LH-105, LH-109) without
// switching tools, and Ctrl on the grip of a way moves the whole way (LH-107), on the example:
// Elisabeth walks (3, 6.5), (10, 6.5), (16, 3); camera A is at (8, 11) and (9, 8).
Item {
    id: root

    Watchdog {}
    Controller { id: ctl }

    function fail(message) {
        console.error(message)
        Qt.exit(1)
    }

    function shape(key) {
        return JSON.parse(ctl.planJson).shapes.find(s => s.key === key && s.kind === "circle")
    }

    // The first point of the axis, the dashed line across the plan.
    function axisStart() {
        return JSON.parse(ctl.planJson).shapes.find(s => s.kind === "line" && s.stroke === "dashed").points.slice(0, 2).join(",")
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

            // Nothing selected: nothing to arm.
            if (ctl.canArm("nextMark") || ctl.arm("nextMark")) {
                return root.fail("armed without a selection")
            }

            // Elisabeth (a click on the mark at (3, 6.5) takes her): the next click sets a mark.
            ctl.place(3, 6.5)
            if (!ctl.canArm("nextMark") || !ctl.arm("nextMark") || ctl.armed !== "nextMark") {
                return root.fail("not armed for the next mark: '" + ctl.armed + "' " + ctl.selectedKey)
            }
            ctl.place(14, 9)
            const next = root.shape("mark:elisabeth:3")
            if (!next || Math.abs(next.points[0] - 14) > 1e-6 || ctl.armed !== "") {
                return root.fail("the click did not set the fourth mark: " + JSON.stringify(next) + " '" + ctl.armed + "'")
            }

            // Escape: nothing happens at the next click.
            ctl.arm("nextMark")
            ctl.cancelArm()
            ctl.place(1, 1)
            if (root.shape("mark:elisabeth:4") || ctl.armed !== "") {
                return root.fail("a cancelled arm still set a mark")
            }

            // A mark before the first one: it is the new first mark.
            ctl.place(3, 6.5)
            ctl.arm("markBefore")
            ctl.place(1, 5)
            const first = root.shape("mark:elisabeth:0")
            if (!first || Math.abs(first.points[0] - 1) > 1e-6 || !root.shape("mark:elisabeth:4")) {
                return root.fail("the click did not set the mark before the first: " + JSON.stringify(first))
            }

            // The axis: Burgvogt, then a click on Elisabeth, reverses the pair of the example.
            const before = root.axisStart()
            ctl.setTool("select", "")
            ctl.place(17, 9.5)
            if (ctl.selectedKey !== "figure:burgvogt" && ctl.selectedKey !== "mark:burgvogt:0") {
                return root.fail("Burgvogt was not selected: '" + ctl.selectedKey + "'")
            }
            ctl.arm("axisTo")
            ctl.place(30, 30)
            if (ctl.armed !== "axisTo") {
                return root.fail("a click beside the figures ended the axis question")
            }
            ctl.place(10, 6.5)
            if (ctl.armed !== "" || root.axisStart() === before) {
                return root.fail("the axis did not change: " + before + " " + root.axisStart())
            }

            // A camera has no axis, but marks.
            // Its body is a wedge facing right and up.
            ctl.place(8.26, 10.85)
            if (ctl.selectedKey.indexOf("camera:") !== 0 || ctl.canArm("axisTo") || !ctl.arm("nextMark")) {
                return root.fail("camera: '" + ctl.selectedKey + "'")
            }
            ctl.cancelArm()
            Qt.exit(0)
        }
    }
}
