import QtQuick
import io.github.shrippen.klappe
import "lib"

// LH-08: Ctrl+D on a figure makes a character of its own with the figure's way, and selects the
// new figure; one undo takes it all back. A mark is copied right behind itself.
// Elisabeth stands at (3, 6.5) at the start.
Item {
    id: root

    Watchdog {}
    Controller { id: ctl }

    function fail(message) {
        console.error(message)
        Qt.exit(1)
    }

    function keys(prefix) {
        return JSON.parse(ctl.planJson).shapes.map(s => s.key).filter(k => k.indexOf(prefix) === 0)
    }

    Timer {
        interval: 0
        running: true
        onTriggered: {
            if (!ctl.openProject(Qt.resolvedUrl("../../../../testdata/beispiel.klappe"))) {
                return root.fail("project did not open")
            }
            ctl.setSnapping(false)
            ctl.setTool("select", "")

            const figures = root.keys("figure:").length
            ctl.place(3.0, 6.5)
            if (ctl.selectedKey !== "figure:elisabeth") {
                return root.fail("the figure was not selected: " + ctl.selectedKey)
            }
            if (!ctl.duplicateSelected()) {
                return root.fail("the figure was not copied")
            }
            if (ctl.selectedCount !== 1 || ctl.selectedKey === "figure:elisabeth" || ctl.selectedKey.indexOf("figure:") !== 0) {
                return root.fail("the copy is not the selection: " + ctl.selectedKey + " " + ctl.selectedCount)
            }
            if (root.keys("figure:").length <= figures) {
                return root.fail("the plan shows no new figure: " + JSON.stringify(root.keys("figure:")) + " was " + figures + " " + ctl.selectedKey)
            }

            ctl.undo()
            if (root.keys("figure:").length !== figures || ctl.selectedCount !== 0) {
                return root.fail("one undo did not take the copy back")
            }
            Qt.exit(0)
        }
    }
}
