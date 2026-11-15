import QtQuick
import io.github.shrippen.klappe
import "lib"

// LH-95: the height tool (H) drags an object's or a wall's height, in the plan and in 3D: up is
// higher, the new height shows at the pointer, a release is one undo step, and a drag from
// anything else does nothing. On the example the table (6 m wide around (10, 6.5), a mark
// stands in its middle) and the pillars have no height (shown as 0.9 m), the north wall
// (0, 0)-(20, 0) is as high as the room, 2.6 m.
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

    function described(key) {
        return JSON.parse(ctl.describe(key))
    }

    // The captions of the plan that are a height in metres, "1,4 m".
    function captions() {
        return JSON.parse(ctl.planJson).shapes.map(s => s.text).filter(t => /^[0-9]+[,.][0-9]+ m$/.test(t))
    }

    // The first pixel of a grid over the 3D picture where a click selects an element whose key
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
            ctl.setSnapping(true)
            if (!ctl.setTool("height", "")) {
                return fail("the height tool is unknown")
            }

            // A drag from empty ground does nothing, not even a preview.
            ctl.drag(2, 10, 2, 8, false)
            ctl.drag(2, 10, 2, 8, true)
            if (ctl.canUndo) {
                return fail("a drag from empty ground edited the plan")
            }

            // Up on the table: 0.5 m higher than the stand-in 0.9 m; the caption shows while
            // the button is down, the project stays as it was.
            ctl.drag(8.5, 6.5, 8.5, 6, false)
            if (captions().indexOf("1,4 m") < 0 && captions().indexOf("1.4 m") < 0) {
                return fail("no caption 1,4 m at the pointer: " + captions())
            }
            if (ctl.canUndo || described("object:tisch").height !== null) {
                return fail("the preview changed the project")
            }
            ctl.drag(8.5, 6.5, 8.5, 6, true)
            if (Math.abs(described("object:tisch").height - 1.4) > 1e-9) {
                return fail("the table is not 1.4 m: " + ctl.describe("object:tisch"))
            }
            if (captions().indexOf("1,4 m") < 0 && captions().indexOf("1.4 m") < 0) {
                return fail("the table's height is not on the plan: " + captions())
            }
            ctl.undo()
            if (described("object:tisch").height !== null || ctl.canUndo) {
                return fail("one undo does not take back the height")
            }

            // Down on the north wall (from the click on its line): 2.6 - 0.6 = 2 m.
            ctl.drag(4, 0, 4, 0.6, true)
            if (Math.abs(described("wall:0").height - 2.0) > 1e-9) {
                return fail("the wall is not 2 m: " + ctl.describe("wall:0"))
            }
            ctl.undo()
            if (Math.abs(described("wall:0").height - 2.6) > 1e-9 || ctl.canUndo) {
                return fail("one undo does not take back the wall's height")
            }

            // Far down stops at the least height, not at a refusal.
            ctl.drag(8.5, 6.5, 8.5, 12, true)
            if (!(described("object:tisch").height > 0)) {
                return fail("a drag far down left no height: " + ctl.describe("object:tisch"))
            }
            ctl.undo()

            // In 3D a drag up on a pillar raises it above the stand-in.
            ctl.setTool("select", "")
            ctl.toggleView()
            ctl.freeSize(width3d, height3d)
            ctl.freeImage(1, colors)
            const at = find("saeule")
            if (at === null) {
                return fail("no click selected a pillar")
            }
            const key = ctl.selectedKey
            ctl.setTool("height", "")
            ctl.freeDrag(at.x, at.y, at.x, at.y - 3 * step, false)
            if (JSON.parse(ctl.freeFrame()).labels.every(l => !/ m$/.test(l.text))) {
                return fail("the height shows nowhere near the pointer: " + ctl.freeFrame())
            }
            ctl.freeDrag(at.x, at.y, at.x, at.y - 3 * step, true)
            if (!(described(key).height > 0.9)) {
                return fail("the pillar did not grow: " + ctl.describe(key))
            }
            if (JSON.parse(ctl.freeFrame()).labels.some(l => / m$/.test(l.text))) {
                return fail("the caption stays after the release")
            }
            ctl.undo()
            if (described(key).height !== null || ctl.canUndo) {
                return fail("one undo does not take back the lift in 3D")
            }

            // A drag in 3D from the sky takes nothing.
            ctl.freeDrag(width3d / 2, 1, width3d / 2, 1 + 3 * step, true)
            if (ctl.canUndo) {
                return fail("a drag from the sky edited the plan")
            }
            Qt.exit(0)
        }
    }
}
