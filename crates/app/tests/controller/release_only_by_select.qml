import QtQuick
import io.github.shrippen.klappe
import "lib"

// Review of #173: a drag ends with the element selected again only when the select tool
// grabbed the point. With the point selected and another tool (a wall drag), the selection
// stays as it was.
Item {
    id: root

    Watchdog {}
    Controller { id: ctl }

    function fail(message) {
        console.error(message)
        Qt.exit(1)
    }

    Timer {
        interval: 0
        running: true
        onTriggered: {
            if (!ctl.openProject(Qt.resolvedUrl("../../../../testdata/beispiel.klappe"))) {
                return root.fail("project did not open")
            }
            ctl.setSnapping(false)
            ctl.setTool("freeform", "")
            for (const [x, y] of [[2, 10], [4, 10], [4, 12], [3, 12], [3, 14], [2, 14]]) {
                ctl.place(x, y)
            }
            ctl.place(2, 10)
            ctl.setTool("select", "")
            ctl.place(2.5, 10.5)
            ctl.place(4, 10)
            const point = ctl.selectedKey
            if (point.indexOf("objectgrip:") !== 0) {
                return root.fail("the click did not take the point: '" + point + "'")
            }

            ctl.setTool("wall", "")
            ctl.drag(10, 11, 14, 11, false)
            ctl.drag(10, 11, 14, 11, true)
            if (ctl.selectedKey !== point) {
                return root.fail("a wall drag changed the selection: '" + ctl.selectedKey + "'")
            }
            Qt.exit(0)
        }
    }
}
