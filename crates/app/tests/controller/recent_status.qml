import QtQuick
import QtCore
import io.github.shrippen.klappe
import "lib"

// A folder action on the start page (rename, archive, trash): a refusal says why in `status`; the
// next try that works takes the message away, or the page keeps showing the old error.
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
            const dir = base + "/recent-status-test"
            // The recent list holds plain paths, not file URLs.
            const path = dir.replace("file://", "")
            if (!ctl.createProject(dir, "Anfang") || !ctl.closeProject()) {
                return root.fail("setup failed: " + ctl.status)
            }

            if (ctl.renameRecent(path, "  ") || ctl.status === "") {
                return root.fail("a blank name must be refused with a message")
            }
            if (!ctl.renameRecent(path, "Neu")) {
                return root.fail("rename failed: " + ctl.status)
            }
            if (ctl.status !== "") {
                return root.fail("a rename that worked left the old message: " + ctl.status)
            }
            Qt.exit(0)
        }
    }
}
