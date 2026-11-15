import QtQuick
import io.github.shrippen.klappe
import "lib"

// LH-95: a measure typed on the keyboard. During a drag of the wall tool it is the wall's length
// from the drag's start toward the pointer: one undo step, and the release adds nothing. Else it
// is the height of the selected object or wall. On the example, plan metres (2, 10) to (6, 10) is
// empty ground; the table (key object:tisch) has no height, the north wall (wall:0) is 2.6 m.
Item {
    Watchdog {}
    Controller { id: ctl }

    function fail(message) {
        console.error(message)
        Qt.exit(1)
    }

    function described(key) {
        return JSON.parse(ctl.describe(key))
    }

    // The shapes of the plan that are walls, as { key, points }.
    function walls() {
        return JSON.parse(ctl.planJson).shapes.filter(s => String(s.key).indexOf("wall:") === 0)
    }

    Timer {
        interval: 0
        running: true
        onTriggered: {
            if (!ctl.openProject(Qt.resolvedUrl("../../../../testdata/beispiel.klappe"))) {
                return fail("project did not open")
            }
            ctl.setSnapping(true)

            // Nothing selected, no drag: nothing to apply a measure to.
            if (ctl.typeMeasure("3") || ctl.canUndo) {
                return fail("a measure was taken with nothing to apply it to")
            }

            // A wall drag: 4 m to the right on screen, 3 m typed.
            const before = walls().length
            ctl.setTool("wall", "")
            ctl.drag(2, 10, 6, 10, false)
            if (ctl.typeMeasure("") || ctl.typeMeasure("x") || ctl.canUndo) {
                return fail("an empty or bad length made a wall")
            }
            if (!ctl.typeMeasure("3")) {
                return fail("the typed length was refused")
            }
            const made = walls()
            if (made.length !== before + 1) {
                return fail("no wall was added: " + before + " -> " + made.length)
            }
            const p = made[made.length - 1].points
            if (Math.abs(p[0] - 2) > 1e-9 || Math.abs(p[1] - 10) > 1e-9
                    || Math.abs(p[2] - 5) > 1e-9 || Math.abs(p[3] - 10) > 1e-9) {
                return fail("the wall is not (2,10)-(5,10): " + p)
            }

            // The rest of the drag adds nothing, and the next drag works again.
            ctl.drag(2, 10, 7, 10, false)
            ctl.drag(2, 10, 7, 10, true)
            if (walls().length !== before + 1) {
                return fail("the release added a second wall")
            }
            ctl.undo()
            if (walls().length !== before || ctl.canUndo) {
                return fail("one undo does not take the wall back")
            }
            ctl.drag(2, 10, 4, 10, true)
            if (walls().length !== before + 1) {
                return fail("the drag after the typed wall draws nothing")
            }
            ctl.undo()

            // No drag in progress: a typed length with the wall tool is no wall.
            if (ctl.typeMeasure("3") || ctl.canUndo) {
                return fail("a length without a drag made something")
            }

            // The height of the selection: an object, a wall; text with a comma.
            ctl.setTool("select", "")
            ctl.place(8.5, 6.5)
            if (ctl.selectedKey !== "object:tisch") {
                return fail("the table is not selected: " + ctl.selectedKey)
            }
            if (!ctl.typeMeasure("0,8")) {
                return fail("the height was refused")
            }
            if (Math.abs(described("object:tisch").height - 0.8) > 1e-9) {
                return fail("the table is not 0.8 m: " + ctl.describe("object:tisch"))
            }
            ctl.undo()
            if (described("object:tisch").height !== null) {
                return fail("one undo does not take back the height")
            }

            ctl.place(4, 0)
            if (ctl.selectedKey !== "wall:0" || !ctl.typeMeasure("2.2")) {
                return fail("the wall's height was refused: " + ctl.selectedKey)
            }
            if (Math.abs(described("wall:0").height - 2.2) > 1e-9) {
                return fail("the wall is not 2.2 m: " + ctl.describe("wall:0"))
            }
            Qt.exit(0)
        }
    }
}
