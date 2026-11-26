import QtQuick
import QtCore
import io.github.shrippen.klappe
import "lib"

// DB-40: locking saves the project first. A file changed outside and in the window is no save,
// so nothing is locked: the reason is returned for the dialog's field and the question about
// the file is asked (`outsideAsked`). After the answer (overwrite) the lock goes through. The
// second controller on the same folder stands for the terminal beside the window. The folder is
// a copy of the example (made by qml_tests.rs).
Item {
    id: root

    Watchdog {}
    Controller { id: ctl }
    Controller { id: other }

    property int asked: 0

    function fail(message) {
        console.error(message)
        Qt.exit(1)
    }

    Connections {
        target: ctl
        function onOutsideAsked() { root.asked += 1 }
    }

    Timer {
        interval: 0
        running: true
        onTriggered: {
            const base = StandardPaths.writableLocation(StandardPaths.TempLocation) + "/klappe-export-test"
            const folder = base + "/versionen-aussen-projekt"
            if (!ctl.openProject(folder) || !other.openProject(folder)) {
                return root.fail("project did not open: " + ctl.status)
            }

            const text = ctl.scriptText
            other.editScript(text + "\nVon draussen.\n")
            if (!other.save()) {
                return root.fail("the other window did not save: " + other.status)
            }
            ctl.editScript(text + "\nVon hier.\n")

            const reason = ctl.lockVersion("Drehfassung")
            if (reason === "" || root.asked !== 1) {
                return root.fail("a conflict must refuse with a reason and ask: '" + reason + "' " + root.asked)
            }
            if (JSON.parse(ctl.versionsJson()).versions.length !== 0) {
                return root.fail("nothing may be locked on a conflict: " + ctl.versionsJson())
            }
            if (ctl.outsideFiles === "") {
                return root.fail("the conflict must be shown")
            }

            if (!ctl.saveOverwrite()) {
                return root.fail("overwrite did not save: " + ctl.status)
            }
            const again = ctl.lockVersion("Drehfassung")
            if (again !== "" || JSON.parse(ctl.versionsJson()).versions.length !== 1) {
                return root.fail("after the answer the lock must go through: '" + again + "'")
            }

            Qt.exit(0)
        }
    }
}
