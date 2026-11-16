import QtQuick
import io.github.shrippen.klappe
import "lib"

// LH-08: a drag on empty ground with Ctrl held draws a lasso; on release it takes the
// elements whose middle lies in it, and Shift adds to the selection.
Item {
    Watchdog {}
    Controller { id: ctl }

    function fail(message) {
        console.error(message)
        Qt.exit(1)
    }

    // A square lasso, `side` m wide, with its corner at (x, y), drawn in four steps.
    function lasso(x, y, side) {
        ctl.drag(x, y, x + side, y, false)
        ctl.drag(x, y, x + side, y + side, false)
        ctl.drag(x, y, x, y + side, false)
        return ctl.drag(x, y, x, y + side, true)
    }

    Timer {
        interval: 0
        running: true
        onTriggered: {
            const example = Qt.resolvedUrl("../../../../testdata/beispiel.klappe")
            if (!ctl.openProject(example)) {
                return fail("project did not open")
            }

            ctl.setTool("select", "")
            ctl.lassoSelection = true

            // Counted by kind: a selected pillar shows its size grips (LH-25), which are shapes too.
            const lines = () => JSON.parse(ctl.planJson).shapes.filter(s => s.kind === "polyline").length
            const before = lines()

            // Pillar 1 stands at (6, 4), pillar 2 at (14, 4).
            ctl.drag(5, 3, 7, 3, false)
            ctl.drag(5, 3, 7, 5, false)
            const shown = JSON.parse(ctl.planJson).shapes
            if (lines() !== before + 1 || shown[shown.length - 1].kind !== "polyline") {
                return fail("the lasso line was not shown")
            }
            ctl.drag(5, 3, 5, 5, true)
            if (ctl.selectedCount !== 1 || ctl.selectedKey.indexOf("saeule-1") < 0) {
                return fail("the lasso did not take pillar 1: " + ctl.selectedKey)
            }
            if (lines() !== before) {
                return fail("the lasso line stayed after the release")
            }

            // Without Shift a new lasso replaces the selection ...
            if (!lasso(13, 3, 2) || ctl.selectedCount !== 1) {
                return fail("a second lasso did not replace the selection")
            }

            // ... with Shift it adds.
            ctl.extendSelection = true
            if (!lasso(5, 3, 2) || ctl.selectedCount !== 2) {
                return fail("Shift did not add: " + ctl.selectedCount)
            }
            Qt.exit(0)
        }
    }
}
