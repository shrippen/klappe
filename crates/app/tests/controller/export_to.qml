import QtQuick
import QtCore
import io.github.shrippen.klappe
import "lib"

// `exportTo` reports what happened to the file: a path with a space and an umlaut is written,
// a target that cannot be replaced is a failure even though something lies there, and an
// unknown kind is refused. The folder is removed by the test that runs this (qml_tests.rs).
Item {
    id: root

    Watchdog {}
    Controller { id: ctl }

    function fail(message) {
        console.error(message)
        Qt.exit(1)
    }

    function size(url) {
        const request = new XMLHttpRequest()
        request.open("GET", url, false)
        request.send()
        return request.status === 200 ? request.responseText.length : -1
    }

    Timer {
        interval: 0
        running: true
        onTriggered: {
            const base = StandardPaths.writableLocation(StandardPaths.TempLocation) + "/klappe-export-test"
            const example = Qt.resolvedUrl("../../../../testdata/beispiel.klappe")
            if (!ctl.openProject(example)) {
                return root.fail("project did not open")
            }

            const named = base + "/Plan Ärger.pdf"
            if (!ctl.exportTo("pdf", "cut", "a3", named)) {
                return root.fail("export to a path with space and umlaut failed: " + ctl.status)
            }
            if (size(named) <= 0) {
                return root.fail("no file at " + named)
            }

            // A folder lies where the file should go: the old file cannot be replaced.
            const blocked = base + "/Ziel.pdf"
            if (!ctl.exportTo("pdf", "cut", "a4", blocked + "/inner.pdf")) {
                return root.fail("export into a new folder failed: " + ctl.status)
            }
            if (ctl.exportTo("pdf", "cut", "a4", blocked)) {
                return root.fail("export over something that cannot be replaced reported success")
            }

            if (ctl.exportTo("zip", "cut", "a4", base + "/x.zip")) {
                return root.fail("an unknown kind reported success")
            }
            if (ctl.status === "") {
                return root.fail("an unknown kind said nothing")
            }
            Qt.exit(0)
        }
    }
}
