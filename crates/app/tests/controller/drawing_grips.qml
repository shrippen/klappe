import QtQuick
import io.github.shrippen.klappe
import "lib"

// A selected drawing shows grips (LH-19): a drag from one reshapes the drawing in one undo
// step, the drawing stays selected, and R turns it. The example's arrow d1 runs from (6, 10)
// to (6, 7.5).
Item {
    id: root

    Watchdog {}
    Controller { id: ctl }

    function fail(message) {
        console.error(message)
        Qt.exit(1)
    }

    // The points of the shaft of drawing d1 as the plan hands them to the canvas.
    function shaft() {
        const shapes = JSON.parse(ctl.planJson).shapes
        const found = shapes.find(s => s.key === "drawing:d1" && s.kind === "line")
        return found ? found.points : []
    }

    function grips() {
        return JSON.parse(ctl.planJson).shapes.filter(s => s.key.indexOf("drawingpoint:d1:") === 0).length
    }

    Timer {
        interval: 0
        running: true
        onTriggered: {
            const example = Qt.resolvedUrl("../../../../testdata/beispiel.klappe")
            if (!ctl.openProject(example)) {
                return root.fail("project did not open")
            }

            ctl.setTool("select", "")
            if (root.grips() !== 0) {
                return root.fail("grips shown without a selection")
            }
            ctl.place(6.0, 9.0)
            if (ctl.selectedKey !== "drawing:d1" || root.grips() !== 2) {
                return root.fail("the arrow was not selected with two grips: " + ctl.selectedKey + " " + root.grips())
            }

            // The end of the arrow, one metre to the right.
            ctl.drag(6.0, 7.5, 6.5, 7.5, false)
            ctl.drag(6.0, 7.5, 7.0, 7.5, true)
            if (JSON.stringify(root.shaft()) !== JSON.stringify([6, 10, 7, 7.5])) {
                return root.fail("the end did not move alone: " + JSON.stringify(root.shaft()))
            }
            if (ctl.selectedKey !== "drawing:d1" || root.grips() !== 2) {
                return root.fail("the drawing lost its selection: " + ctl.selectedKey)
            }

            ctl.undo()
            if (JSON.stringify(root.shaft()) !== JSON.stringify([6, 10, 6, 7.5])) {
                return root.fail("one undo did not take the drag back: " + JSON.stringify(root.shaft()))
            }

            ctl.turnSelected(90)
            if (JSON.stringify(root.shaft()) === JSON.stringify([6, 10, 6, 7.5])) {
                return root.fail("the turn changed nothing")
            }
            Qt.exit(0)
        }
    }
}
