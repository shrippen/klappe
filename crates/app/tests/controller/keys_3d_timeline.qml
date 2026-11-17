import QtQuick
import io.github.shrippen.klappe
import "lib"

// NF-06: the keyboard's ways. stepView turns, shifts and zooms the 3D view without an edit and
// not in the plan; nudgeTime moves the selected mark or the edges of the selected shot by a
// tenth of a second, one undo step each, and refuses without a selection.
Item {
    Watchdog {}
    Controller { id: ctl }

    readonly property string colors: JSON.stringify({ ground: [255, 255, 255], text: [0, 0, 0] })

    function fail(message) {
        console.error(message)
        Qt.exit(1)
    }

    function arrivals() {
        const tracks = JSON.parse(ctl.tracksJson)
        return tracks.find(t => t.id === "elisabeth").markers.map(m => m.at_s)
    }

    Timer {
        interval: 0
        running: true
        onTriggered: {
            if (!ctl.openProject(Qt.resolvedUrl("../../../../testdata/beispiel.klappe"))) {
                return fail("project did not open")
            }

            // The plan has no 3D view to step.
            ctl.stepView("turnLeft")
            ctl.toggleView()
            ctl.freeSize(800, 600)
            ctl.freeImage(1, colors)
            for (const step of ["turnLeft", "turnUp", "panRight", "panDown", "zoomIn", "zoomOut"]) {
                const was = ctl.freeRevision
                ctl.stepView(step)
                if (ctl.freeRevision !== was + 1) {
                    return fail(step + " did not make another picture")
                }
            }
            ctl.stepView("spin")
            if (ctl.canUndo) {
                return fail("stepping the view is an edit")
            }

            // A mark: nothing selected, nothing moves; selected, a tenth later, one undo step.
            if (ctl.nudgeTime("mark", 0.1)) {
                return fail("nudged a mark without a selection")
            }
            ctl.toggleView()
            ctl.setTool("select", "")
            ctl.place(10, 6.5)
            if (ctl.selectedKey !== "mark:elisabeth:1") {
                return fail("the mark was not selected: '" + ctl.selectedKey + "'")
            }
            if (!ctl.nudgeTime("mark", 0.1) || arrivals()[1] !== 6.1) {
                return fail("the mark did not arrive a tenth later: " + arrivals())
            }
            if (!ctl.nudgeTime("mark", -0.1) || arrivals()[1] !== 6) {
                return fail("the mark did not come back: " + arrivals())
            }
            ctl.undo()
            if (arrivals()[1] !== 6.1) {
                return fail("one undo did not take back one nudge: " + arrivals())
            }

            // The edges of the selected shot.
            ctl.selectShot("s12-2")
            if (!ctl.nudgeTime("shot-to", 0.1) || !ctl.nudgeTime("shot-from", -0.1)) {
                return fail("the edges of the selected shot did not move")
            }
            if (ctl.nudgeTime("nonsense", 0.1)) {
                return fail("an unknown edge was taken")
            }
            Qt.exit(0)
        }
    }
}
