import QtQuick
import io.github.shrippen.klappe
import "lib"

// LH-96: the room tool. A drag pulls up a rectangle of four walls, one undo step; a click does
// nothing. A typed `5;4` (LH-95) finishes the room from the drag's start, width along x and depth
// along y toward the pointer; the release adds nothing.
Item {
    Watchdog {}
    Controller { id: ctl }

    function fail(message) {
        console.error(message)
        Qt.exit(1)
    }

    // The shapes of the plan that are walls, as { key, points }.
    function walls() {
        return JSON.parse(ctl.planJson).shapes.filter(s => String(s.key).indexOf("wall:") === 0)
    }

    // The last four walls as "x1,y1,x2,y2" strings, rounded.
    function lastFour() {
        return walls().slice(-4).map(w => w.points.map(v => Math.round(v * 100) / 100).join(","))
    }

    Timer {
        interval: 0
        running: true
        onTriggered: {
            if (!ctl.openProject(Qt.resolvedUrl("../../../../testdata/beispiel.klappe"))) {
                return fail("project did not open")
            }
            ctl.setSnapping(true)
            const before = walls().length
            ctl.setTool("room", "")

            // A click does nothing.
            ctl.place(2, 10)
            if (walls().length !== before || ctl.canUndo) {
                return fail("a click made something")
            }

            // A drag: four walls round the rectangle, one undo.
            ctl.drag(2, 10, 6, 13, true)
            if (walls().length !== before + 4) {
                return fail("no four walls: " + before + " -> " + walls().length)
            }
            const want = ["2,10,6,10", "6,10,6,13", "6,13,2,13", "2,13,2,10"]
            if (JSON.stringify(lastFour()) !== JSON.stringify(want)) {
                return fail("the walls are not the rectangle: " + JSON.stringify(lastFour()))
            }
            ctl.undo()
            if (walls().length !== before || ctl.canUndo) {
                return fail("one undo does not take back all four walls")
            }

            // A drag along a line makes no room.
            ctl.drag(2, 10, 6, 10, true)
            if (walls().length !== before || ctl.canUndo) {
                return fail("a flat drag made walls")
            }

            // A typed size: toward the pointer (up-left of the start).
            ctl.drag(10, 14, 8, 12, false)
            if (ctl.typeMeasure("5") || ctl.typeMeasure("0;4") || ctl.canUndo) {
                return fail("a bad size made a room")
            }
            if (!ctl.typeMeasure("5;4")) {
                return fail("the typed size was refused")
            }
            const typed = ["10,14,5,14", "5,14,5,10", "5,10,10,10", "10,10,10,14"]
            if (walls().length !== before + 4 || JSON.stringify(lastFour()) !== JSON.stringify(typed)) {
                return fail("the typed room is wrong: " + JSON.stringify(lastFour()))
            }

            // The rest of the drag adds nothing; one undo takes the room back.
            ctl.drag(10, 14, 12, 16, false)
            ctl.drag(10, 14, 12, 16, true)
            if (walls().length !== before + 4) {
                return fail("the release added walls")
            }
            ctl.undo()
            if (walls().length !== before || ctl.canUndo) {
                return fail("one undo does not take back the typed room")
            }
            Qt.exit(0)
        }
    }
}
