import QtQuick
import io.github.shrippen.klappe
import "lib"

// LH-08: with a drawing selected, a box over its grips takes the grips (several at once) and
// a drag of one moves them all, in one undo step. A box over the grips of an element that is
// not selected takes elements only. The example's arrow d1 runs from (6, 10) to (6, 7.5).
Item {
    id: root

    Watchdog {}
    Controller { id: ctl }

    function fail(message) {
        console.error(message)
        Qt.exit(1)
    }

    function shaft() {
        const found = JSON.parse(ctl.planJson).shapes.find(s => s.key === "drawing:d1" && s.kind === "line")
        return found ? JSON.stringify(found.points) : ""
    }

    function grips() {
        return JSON.parse(ctl.planJson).shapes.filter(s => s.key.indexOf("drawingpoint:d1:") === 0).length
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

            // Another element selected: the box takes elements, never the arrow's grips.
            ctl.place(3.0, 11.5)
            if (ctl.selectedKey !== "note:n1") {
                return root.fail("the note was not selected: " + ctl.selectedKey)
            }
            ctl.drag(5.5, 7, 6.5, 10.5, false)
            ctl.drag(5.5, 7, 6.5, 10.5, true)
            if (ctl.selectedKey !== "drawing:d1") {
                return root.fail("the box did not take the arrow: " + ctl.selectedKey)
            }

            ctl.drag(5, 6, 7, 11, false)
            ctl.drag(5, 6, 7, 11, true)
            if (ctl.selectedCount !== 2 || root.grips() !== 2) {
                return root.fail("the box did not take both grips: " + ctl.selectedCount + " " + root.grips())
            }

            ctl.drag(6.0, 7.5, 6.5, 7.5, false)
            ctl.drag(6.0, 7.5, 7.0, 7.5, true)
            if (root.shaft() !== "[7,10,7,7.5]") {
                return root.fail("the grips did not move together: " + root.shaft())
            }
            if (ctl.selectedCount !== 2) {
                return root.fail("the grips are not selected after the drag: " + ctl.selectedCount)
            }

            ctl.undo()
            if (root.shaft() !== "[6,10,6,7.5]") {
                return root.fail("one undo did not take the drag back: " + root.shaft())
            }
            Qt.exit(0)
        }
    }
}
