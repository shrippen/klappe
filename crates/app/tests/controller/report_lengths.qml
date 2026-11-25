import QtQuick
import QtCore
import io.github.shrippen.klappe
import "lib"

// A report export lays the screenplay out when the text changed since the last layout, so the
// scene list shows the lengths the report holds, not the older ones, without waiting for the
// editor's timer (which then finds nothing new to lay out).
Item {
    Watchdog {}
    Controller { id: ctl }

    function fail(message) {
        console.error(message)
        Qt.exit(1)
    }

    function lengths() {
        return JSON.stringify(JSON.parse(ctl.projectJson).scenes.map(s => s.length))
    }

    Timer {
        interval: 0
        running: true
        onTriggered: {
            const base = StandardPaths.writableLocation(StandardPaths.TempLocation) + "/klappe-export-test/report-lengths"
            const example = Qt.resolvedUrl("../../../../testdata/beispiel.klappe")
            if (!ctl.openProject(example)) {
                return fail("project did not open")
            }

            // Many lines under the last scene: its length changes, the text is not laid out.
            const before = lengths()
            ctl.editScript(ctl.scriptText + "\n\n" + "Es tropft.\n\n".repeat(200))
            if (lengths() !== before) {
                return fail("typing alone laid the text out: " + lengths())
            }

            if (!ctl.exportReport("szenen", "csv", base + "/Szenen.csv")) {
                return fail("report export failed: " + ctl.status)
            }
            // The editor's timer finds nothing new to lay out; the report's layout must show.
            ctl.paginateScript()
            if (lengths() === before) {
                return fail("the scene list keeps the old lengths after the report: " + lengths())
            }
            Qt.exit(0)
        }
    }
}
