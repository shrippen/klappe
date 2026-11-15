import QtQuick
import io.github.shrippen.klappe
import "lib"

// LH-94: J switches to the free 3D view and back; turning and zooming make another picture;
// a click selects what is seen there; a drag moves it in one undo step; above the horizon
// nothing happens.
Item {
    Watchdog {}
    Controller { id: ctl }

    readonly property string colors: JSON.stringify({ ground: [255, 255, 255], text: [0, 0, 0] })
    readonly property real width3d: 800
    readonly property real height3d: 600
    readonly property real step: 20

    function fail(message) {
        console.error(message)
        Qt.exit(1)
    }

    // The first pixel of a grid over the picture where a click selects an element whose key
    // holds `part`; null if none does.
    function find(part) {
        for (let y = height3d - step; y > 0; y -= step) {
            for (let x = step; x < width3d; x += step) {
                ctl.freePlace(x, y)
                if (ctl.selectedKey.indexOf(part) >= 0) {
                    return Qt.point(x, y)
                }
            }
        }
        return null
    }

    Timer {
        interval: 0
        running: true
        onTriggered: {
            if (!ctl.openProject(Qt.resolvedUrl("../../../../testdata/beispiel.klappe"))) {
                return fail("project did not open")
            }
            if (ctl.viewFree) {
                return fail("the plan is the first view")
            }

            // J: the 3D view, and back.
            ctl.toggleView()
            if (!ctl.viewFree) {
                return fail("toggleView did not switch to 3D")
            }
            ctl.freeSize(width3d, height3d)
            ctl.freeImage(1, colors)
            const frame = JSON.parse(ctl.freeFrame())
            if (frame.width !== width3d || frame.height <= 0) {
                return fail("the 3D picture is not as large as the view: " + ctl.freeFrame())
            }
            ctl.freeImage(0.5, colors)
            if (JSON.parse(ctl.freeFrame()).width !== width3d / 2) {
                return fail("the picture under a drag is not half as wide")
            }

            // Turning and zooming change the picture; shifting too.
            let revision = ctl.freeRevision
            ctl.turnView(40, 10)
            ctl.zoomView(2, width3d / 2, height3d / 2)
            ctl.panView(30, 0)
            if (ctl.freeRevision !== revision + 3) {
                return fail("navigating does not change the picture")
            }
            if (ctl.canUndo) {
                return fail("navigating is an edit")
            }

            // A click on a wall selects it, as on the plan.
            ctl.turnView(-40, -10)
            ctl.zoomView(-2, width3d / 2, height3d / 2)
            ctl.setTool("select", "")
            if (find("wall") === null) {
                return fail("no click selected a wall")
            }

            // A click above the horizon does nothing: the selection stays.
            ctl.turnView(0, -1000)
            const kept = ctl.selectedKey
            if (ctl.freePlace(width3d / 2, 1) || ctl.selectedKey !== kept) {
                return fail("a click above the horizon changed something")
            }

            // A drag of a selected element moves it, in one undo step.
            ctl.turnView(0, 1000)
            const at = find("saeule")
            if (at === null) {
                return fail("no click selected a pillar")
            }
            const plan = ctl.planJson
            ctl.freeDrag(at.x, at.y, at.x + 3 * step, at.y, false)
            ctl.freeDrag(at.x, at.y, at.x + 3 * step, at.y, true)
            if (ctl.planJson === plan) {
                return fail("the drag did not move the pillar")
            }
            ctl.undo()
            if (ctl.planJson !== plan) {
                return fail("one undo does not take back the drag")
            }

            ctl.toggleView()
            if (ctl.viewFree) {
                return fail("toggleView did not come back to the plan")
            }
            Qt.exit(0)
        }
    }
}
