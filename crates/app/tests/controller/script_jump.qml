import QtQuick
import io.github.shrippen.klappe
import "lib"

// DB-63: from a script line to its mark or shot on the plan and back. showLink selects the
// mark (and goes to its time) or the shot; scriptLine says which line the selected mark or
// shot is linked to, -1 for one that no line names (the scene's head is shown then).
Item {
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
            if (!ctl.openProject(Qt.resolvedUrl("../../../../testdata/beispiel.klappe"))) {
                return fail("project did not open")
            }

            if (ctl.scriptLine() !== -1) {
                return fail("a line without a selection: " + ctl.scriptLine())
            }

            // Line 3 names Burgvogt's mark 2: the mark is selected, the time is its arrival.
            if (!ctl.showLink("mark", "burgvogt", 1)) {
                return fail("the mark was not shown")
            }
            if (ctl.selectedKey !== "mark:burgvogt:1" || ctl.time !== 9) {
                return fail("mark: '" + ctl.selectedKey + "' at " + ctl.time)
            }
            if (ctl.scriptLine() !== 3) {
                return fail("the mark's line is " + ctl.scriptLine())
            }

            // A shot replaces the mark in the selection; line 1 names it.
            if (!ctl.showLink("shot", "s12-2", 0)) {
                return fail("the shot was not shown")
            }
            if (ctl.selectedShot !== "s12-2" || ctl.selectedKey !== "") {
                return fail("shot: '" + ctl.selectedShot + "', key '" + ctl.selectedKey + "'")
            }
            if (ctl.scriptLine() !== 1) {
                return fail("the shot's line is " + ctl.scriptLine())
            }

            // A mark and a shot selected: the mark's line wins.
            ctl.showLink("mark", "elisabeth", 0)
            if (ctl.scriptLine() !== 0) {
                return fail("the mark did not win: " + ctl.scriptLine())
            }

            // A mark that no line names; and what is not there.
            ctl.showLink("mark", "elisabeth", 1)
            if (ctl.scriptLine() !== -1) {
                return fail("an unlinked mark has line " + ctl.scriptLine())
            }
            if (ctl.showLink("mark", "elisabeth", 99) || ctl.showLink("shot", "nope", 0) || ctl.showLink("nonsense", "x", 0)) {
                return fail("something that is not there was shown")
            }
            if (ctl.canUndo) {
                return fail("showing a link is an edit")
            }
            Qt.exit(0)
        }
    }
}
