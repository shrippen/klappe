import QtQuick
import QtCore
import io.github.shrippen.klappe
import "lib"

// DB-40 to DB-42: what the window's version calls do through the controller: a version is
// locked (an empty and a used name are refused with their reason), listed with the colour
// after the last one's, the revision names the lines typed since, the comparison finds them,
// and a deleted version is gone. The folder is a copy of the example (made by qml_tests.rs).
Item {
    id: root

    Watchdog {}
    Controller { id: ctl }

    function fail(message) {
        console.error(message)
        Qt.exit(1)
    }

    function list() { return JSON.parse(ctl.versionsJson()) }

    Timer {
        interval: 0
        running: true
        onTriggered: {
            const base = StandardPaths.writableLocation(StandardPaths.TempLocation) + "/klappe-export-test"
            if (!ctl.openProject(base + "/versionen-projekt")) {
                return root.fail("project did not open: " + ctl.status)
            }
            if (list().versions.length !== 0 || list().next !== null) {
                return root.fail("the example must have no version: " + ctl.versionsJson())
            }
            if (ctl.revisionJson() !== "[]") {
                return root.fail("without a version nothing is marked: " + ctl.revisionJson())
            }

            if (ctl.lockVersion("   ") === "") {
                return root.fail("an empty name must be refused")
            }
            const reason = ctl.lockVersion("Drehfassung")
            if (reason !== "") {
                return root.fail("the version was not locked: " + reason)
            }
            const first = list()
            if (first.versions.length !== 1 || first.versions[0].name !== "Drehfassung"
                    || first.versions[0].color !== "white" || first.next.color !== "blue") {
                return root.fail("the list must show the first version and the next colour: " + ctl.versionsJson())
            }
            if (ctl.lockVersion("DREHFASSUNG") === "") {
                return root.fail("a used name must be refused")
            }

            // Nothing typed since: no line marked, the comparison finds nothing changed.
            if (ctl.revisionJson() !== "[]") {
                return root.fail("right after locking nothing is marked: " + ctl.revisionJson())
            }
            const id = first.versions[0].id
            ctl.editScript(ctl.scriptText + "\nEine neue Zeile.\n")
            const marked = JSON.parse(ctl.revisionJson())
            if (marked.length === 0) {
                return root.fail("the new line must be marked")
            }
            const changed = JSON.parse(ctl.compareJson(id, ""))
            const lines = changed.reduce((all, s) => all.concat(s.lines), [])
            if (!lines.some(l => l.change === "added" && l.text === "Eine neue Zeile.")) {
                return root.fail("the comparison must hold the new line: " + ctl.compareJson(id, ""))
            }
            if (ctl.compareJson("unbekannt", "") !== "[]") {
                return root.fail("an unknown version compares to nothing")
            }

            if (!ctl.removeVersion(id) || list().versions.length !== 0) {
                return root.fail("the version was not deleted: " + ctl.status)
            }
            if (ctl.removeVersion(id)) {
                return root.fail("a version that is gone cannot be deleted again")
            }

            Qt.exit(0)
        }
    }
}
