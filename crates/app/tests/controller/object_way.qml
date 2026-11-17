import QtQuick
import io.github.shrippen.klappe
import "lib"

// A set object gets a way of marks like a figure (LH-24), on the example: the pillar stands at
// (6, 4). Selected, the next-mark key arms the click; the body then follows the playhead, a
// figure can carry it, and undo takes the marks away.
Item {
    id: root

    Watchdog {}
    Controller { id: ctl }

    function fail(message) {
        console.error(message)
        Qt.exit(1)
    }

    function shapes(key) {
        return JSON.parse(ctl.planJson).shapes.filter(s => s.key === key)
    }

    // The middle of the pillar's body, the mean of its corners.
    function bodyY() {
        const body = root.shapes("object:saeule-1").find(s => s.kind === "polygon")
        let sum = 0
        for (let i = 1; i < body.points.length; i += 2) {
            sum += body.points[i]
        }
        return sum / (body.points.length / 2)
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

            ctl.place(6, 4)
            if (ctl.selectedKey !== "object:saeule-1") {
                return root.fail("the pillar was not selected: '" + ctl.selectedKey + "'")
            }
            if (!ctl.canArm("nextMark") || !ctl.arm("nextMark")) {
                return root.fail("an object cannot be armed for a mark")
            }
            ctl.place(6, 10)
            if (root.shapes("objectmark:saeule-1:0").length === 0
                    || root.shapes("objectmark:saeule-1:1").length === 0) {
                return root.fail("the click did not set the marks")
            }

            // The playhead moves the body along its way.
            ctl.seek(30)
            if (Math.abs(root.bodyY() - 10) > 0.01) {
                return root.fail("the body is not at the last mark: " + root.bodyY())
            }

            // The card's fields: the pause, and the figure that carries it.
            ctl.seek(0)
            const info = JSON.parse(ctl.markInfo())
            if (info.kind !== "object" || info.carriedBy !== null) {
                return root.fail("the mark fields: " + ctl.markInfo())
            }
            if (!ctl.setMarkCarrier("elisabeth")
                    || JSON.parse(ctl.markInfo()).carriedBy !== "elisabeth") {
                return root.fail("the carrier was not set: " + ctl.markInfo())
            }
            if (ctl.setMarkCarrier("nobody")) {
                return root.fail("a figure the plan lacks was accepted")
            }

            ctl.undo()
            ctl.undo()
            if (root.shapes("objectmark:saeule-1:0").length !== 0) {
                return root.fail("undo left the marks")
            }
            Qt.exit(0)
        }
    }
}
