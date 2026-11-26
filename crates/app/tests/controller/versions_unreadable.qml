import QtQuick
import QtCore
import io.github.shrippen.klappe
import "lib"

// DB-41, DB-42: a version whose text cannot be read (the list names it, the file is gone, as
// after a Git switch) is no book without changes: the comparison and the revision marks say
// why in the status line. Only "no version yet" stays silent. The folder is a copy of the
// example with such a list (made by qml_tests.rs).
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
            if (!ctl.openProject(base + "/versionen-fehlt-projekt")) {
                return root.fail("project did not open: " + ctl.status)
            }

            if (ctl.status !== "") {
                return root.fail("nothing was said yet: " + ctl.status)
            }
            if (ctl.compareJson("fehlt", "") !== "[]") {
                return root.fail("an unreadable version compares to nothing")
            }
            if (ctl.status === "") {
                return root.fail("the comparison must say why it found nothing")
            }

            // A change of the book clears the message.
            ctl.editScript(ctl.scriptText + "\nEine neue Zeile.\n")
            if (ctl.status !== "") {
                return root.fail("the message must be gone: " + ctl.status)
            }
            if (ctl.revisionJson() !== "[]") {
                return root.fail("an unreadable version marks nothing")
            }
            if (ctl.status === "") {
                return root.fail("the revision must say why it marked nothing")
            }

            Qt.exit(0)
        }
    }
}
