import QtQuick
import QtCore
import io.github.shrippen.klappe
import "lib"

// DB-51: a scripted location the book names and orte.toml does not is asked about when the project
// opens (the status line counts it), the answer goes to orte.toml at once and is no step of
// undo. The folder is a copy of the example whose scripted location is called FESTSAAL while the
// heading says SÄNGERSAAL; qml_tests.rs makes it and checks the file afterwards.
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
            const base = StandardPaths.writableLocation(StandardPaths.TempLocation) + "/klappe-export-test"
            if (!ctl.openProject(base + "/orte-projekt")) {
                return root.fail("project did not open: " + ctl.status)
            }
            // The count's words follow the system language; "1." is in both.
            const counted = ctl.status
            if (counted.indexOf(": 1.") < 0) {
                return root.fail("the status must count the open place: " + counted)
            }
            const asked = JSON.parse(ctl.projectJson).pendingPlaces
            if (asked.length !== 1 || asked[0].name !== "SÄNGERSAAL" || asked[0].answers[1].kind !== "rename") {
                return root.fail("one place question with a rename expected: " + ctl.projectJson)
            }

            const undoBefore = ctl.canUndo
            if (!ctl.resolvePlace("SÄNGERSAAL", 1)) {
                return root.fail("answer refused: " + ctl.status)
            }
            if (JSON.parse(ctl.projectJson).pendingPlaces.length !== 0) {
                return root.fail("the answered question is still open")
            }
            if (ctl.dirty || ctl.canUndo !== undoBefore) {
                return root.fail("a place answer must be no undo step")
            }
            if (ctl.status.indexOf(counted) >= 0) {
                return root.fail("the status still counts the answered place: " + ctl.status)
            }
            Qt.exit(0)
        }
    }
}
