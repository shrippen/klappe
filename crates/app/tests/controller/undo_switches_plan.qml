import QtQuick
import io.github.shrippen.klappe

// `switched` stops play: undo across a plan sends it, a refresh on the same plan does not.
Item {
    id: root
    property int switches: 0

    Controller {
        id: ctl
        onSwitched: root.switches += 1
    }

    function fail(message) {
        console.error(message)
        Qt.exit(1)
    }

    Timer {
        interval: 0
        running: true
        onTriggered: {
            const example = Qt.resolvedUrl("../../../../testdata/beispiel.klappe")
            if (!ctl.openProject(example)) {
                return root.fail("project did not open")
            }
            if (!ctl.copyPlan("Stand 1")) {
                return root.fail("plan was not copied")
            }

            root.switches = 0
            ctl.seek(1.0)
            if (root.switches !== 0) {
                return root.fail("seek on the same plan sent switched")
            }

            ctl.undo()
            if (root.switches !== 1) {
                return root.fail("undo across a plan sent switched " + root.switches + " times")
            }
            Qt.exit(0)
        }
    }
}
