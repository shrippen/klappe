import QtQuick
import QtCore
import io.github.shrippen.klappe
import "lib"

// A file changed outside the window (here: a second controller on the same folder, as the
// terminal beside it): the window takes over what it did not change itself and tells which
// (`outsideTaken`); a file both changed is asked about on save (`outsideAsked`, nothing
// written) and settled by `saveOverwrite` or `reloadProject`.
Item {
    id: root

    Watchdog {}
    Controller { id: ctl }
    Controller { id: other }

    property var told: []
    property int asked: 0

    function fail(message) {
        console.error(message)
        Qt.exit(1)
    }

    Connections {
        target: ctl
        function onOutsideTaken(text) { root.told.push(text) }
        function onOutsideAsked() { root.asked += 1 }
    }

    Timer {
        interval: 0
        running: true
        onTriggered: {
            const base = StandardPaths.writableLocation(StandardPaths.TempLocation) + "/klappe-export-test"
            if (!ctl.openProject(base + "/aussen-projekt") || !other.openProject(base + "/aussen-projekt")) {
                return root.fail("project did not open: " + ctl.status)
            }

            // Changed outside only: taken over, told, no question.
            const outside = ctl.scriptText + "\nVon draussen.\n"
            other.editScript(outside)
            if (!other.save()) {
                return root.fail("the other window did not save: " + other.status)
            }
            ctl.checkOutside()
            if (ctl.scriptText !== outside || root.told.length !== 1 || ctl.outsideFiles !== "") {
                return root.fail("the change must be taken over and told: " + root.told.length)
            }
            if (ctl.canUndo || ctl.dirty) {
                return root.fail("a take-over without own work leaves nothing to undo or save")
            }

            // Changed in both: the save asks and writes nothing. The other window's save also
            // moved the register, which the save takes over and tells (the second text).
            const own = outside + "Von hier.\n"
            ctl.editScript(own)
            const second = outside + "Noch einmal von draussen.\n"
            other.editScript(second)
            if (!other.save()) {
                return root.fail("the other window did not save again")
            }
            if (ctl.save() || root.asked !== 1) {
                return root.fail("the save must ask, not write: " + root.asked)
            }
            if (ctl.outsideFiles === "" || ctl.scriptText !== own) {
                return root.fail("the conflict is shown and the text is the window's")
            }

            // Overwrite: the window's version goes to the disk.
            if (!ctl.saveOverwrite() || ctl.outsideFiles !== "" || ctl.dirty) {
                return root.fail("overwrite must save and end the conflict: " + ctl.status)
            }

            // Reload: the window's changes go, the folder's text comes.
            // The other window sees the overwrite first (its focus returned), then types.
            other.checkOutside()
            const third = second + "Und ein drittes Mal.\n"
            other.editScript(third)
            if (!other.save()) {
                return root.fail("the other window did not save a third time")
            }
            ctl.editScript(own + "Noch mehr.\n")
            ctl.reloadProject()
            if (ctl.scriptText !== third || ctl.dirty || ctl.canUndo || root.told.length !== 3) {
                return root.fail("reload must take the folder's text: " + ctl.scriptText.length)
            }

            // A save finds a change outside only: it takes it over, tells it and still saves.
            const fourth = third + "Ein viertes Mal.\n"
            other.checkOutside()
            other.editScript(fourth)
            if (!other.save()) {
                return root.fail("the other window did not save a fourth time")
            }
            if (!ctl.save() || ctl.scriptText !== fourth || root.told.length !== 4) {
                return root.fail("a save must tell what it took over: " + root.told.join("|"))
            }

            // The dialog stays open while the window comes back to the front and the other
            // side writes the window's own text: nothing is left to settle, "Neu laden" then
            // has no file to tell about, so it says nothing.
            const fifth = fourth + "Von beiden.\n"
            ctl.editScript(fifth)
            other.checkOutside()
            other.editScript(fourth + "Nur von draussen.\n")
            if (!other.save() || ctl.save() || root.asked !== 2) {
                return root.fail("the save must ask again: " + root.asked)
            }
            other.editScript(fifth)
            if (!other.save()) {
                return root.fail("the other window did not save the shared text")
            }
            const before = root.told.length
            ctl.checkOutside()
            ctl.reloadProject()
            if (ctl.outsideFiles !== "" || root.told.length !== before) {
                return root.fail("a reload with no file outside must tell nothing: "
                    + root.told.slice(before).join("|"))
            }
            Qt.exit(0)
        }
    }
}
