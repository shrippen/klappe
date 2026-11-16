import QtQuick
import io.github.shrippen.klappe
import "lib"

// The card of the selection goes with the selection (Q1-04): `clearSelection` (Escape, a change
// of mode) clears it. A change of tool keeps it: a typed measure with the wall tool is the
// height of the selected wall (typed_measure.qml).
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
            ctl.setTool("select", "")
            ctl.place(8.5, 6.5)
            if (ctl.selectedKey === "") {
                return root.fail("the click took nothing")
            }

            ctl.setTool("wall", "")
            if (ctl.selectedKey === "") {
                return root.fail("a change of tool cleared the selection")
            }

            ctl.setTool("select", "")
            ctl.place(8.5, 6.5)
            ctl.clearSelection()
            if (ctl.selectedKey !== "" || ctl.selectedCount !== 0) {
                return root.fail("clearSelection kept the selection: '" + ctl.selectedKey + "'")
            }
            Qt.exit(0)
        }
    }
}
