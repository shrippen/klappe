import QtQuick
import QtCore
import io.github.shrippen.klappe
import "lib"

// `closeProject` (Q1-02, D-07): the project is released, the window falls back to what it shows
// without one (nothing left of the old project), the project stays in the recent list, and unsaved
// changes are backed up as when another project opens. The folder is removed by the test that
// runs this (qml_tests.rs).
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
            const dir = base + "/close-test"
            if (!ctl.closeProject() || ctl.loaded) {
                return root.fail("closing without a project must be harmless and leave nothing loaded")
            }
            if (!ctl.createProject(dir, "Schlusstest") || !ctl.loaded || ctl.title !== "Schlusstest") {
                return root.fail("project did not open: " + ctl.status)
            }
            if (!ctl.addLocation("Halle", "8", "6", "3") || !ctl.dirty) {
                return root.fail("an edit must leave the project unsaved: " + ctl.status)
            }

            if (ctl.gridName === "") {
                return root.fail("an open project must show its grid")
            }

            if (!ctl.closeProject()) {
                return root.fail("close failed: " + ctl.status)
            }
            if (ctl.loaded || ctl.title !== "" || ctl.dirty || ctl.planJson !== "" || ctl.projectJson !== ""
                    || ctl.canUndo || ctl.shotsJson !== "" || ctl.sceneId !== "" || ctl.selectedShot !== ""
                    || ctl.gridName !== "") {
                return root.fail("the closed project left state behind: " + ctl.title + " " + ctl.projectJson + " grid " + ctl.gridName)
            }
            if (ctl.save() || ctl.addLocation("Hof", "5", "5", "3")) {
                return root.fail("a closed project must refuse edits and saving")
            }

            const recent = JSON.parse(ctl.recentJson)
            if (recent.length !== 1 || !recent[0].path.endsWith("/close-test")) {
                return root.fail("the closed project is not first in the recent list: " + ctl.recentJson)
            }

            // The unsaved edit went to the backup, as when another project opens.
            if (!ctl.openProject(dir) || ctl.backupState !== "found") {
                return root.fail("reopening must offer the backup, got: " + ctl.backupState)
            }
            Qt.exit(0)
        }
    }
}
