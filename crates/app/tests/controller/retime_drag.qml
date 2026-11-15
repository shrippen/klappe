import QtQuick
import io.github.shrippen.klappe
import "lib"

// Dragging a mark on the timeline (LH-61): the drag shows the snapped time, the drop moves the
// mark in one undo step, a locked layer refuses it.
Item {
    id: root

    Watchdog {}
    Controller { id: ctl }

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
            const example = Qt.resolvedUrl("../../../../testdata/beispiel.klappe")
            if (!ctl.openProject(example)) {
                return root.fail("project did not open")
            }
            if (arrivals()[1] !== 6) {
                return root.fail("the second mark is not at 6 s: " + arrivals())
            }

            const shown = ctl.dragTime("figure:1:elisabeth", 7.26, "tenths")
            if (Math.abs(shown - 7.3) > 1e-9) {
                return root.fail("the drag shows " + shown + " instead of 7.3")
            }
            if (ctl.dragTime("figure:1:elisabeth", 99, "tenths") !== 14) {
                return root.fail("the drag passes the mark after it")
            }

            if (!ctl.dropTime("figure:1:elisabeth", shown, "free")) {
                return root.fail("the drop was refused")
            }
            if (arrivals()[1] !== 7.3) {
                return root.fail("the mark is at " + arrivals()[1] + " after the drop")
            }

            ctl.undo()
            if (arrivals()[1] !== 6) {
                return root.fail("one undo did not bring the mark back: " + arrivals())
            }

            ctl.setLock("figures", "locked")
            if (ctl.dropTime("figure:1:elisabeth", 8, "free") || arrivals()[1] !== 6) {
                return root.fail("a locked layer let the mark move")
            }
            if (ctl.dropTime("nonsense:1:elisabeth", 8, "free")) {
                return root.fail("an unknown kind was taken")
            }
            Qt.exit(0)
        }
    }
}
