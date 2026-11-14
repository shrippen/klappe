import QtQuick
import io.github.shrippen.klappe

// Play ticks (advance) must not take a drag preview off the plan.
Item {
    Controller { id: ctl }

    Timer {
        interval: 0
        running: true
        onTriggered: {
            const example = Qt.resolvedUrl("../../../../testdata/beispiel.klappe")
            if (!ctl.openProject(example)) {
                console.error("project did not open")
                Qt.exit(1)
                return
            }
            const project = ctl.planJson

            ctl.setTool("rect", "")
            ctl.drag(0.5, 0.5, 3.5, 2.5, false)
            const preview = ctl.planJson
            if (preview === project) {
                console.error("the drag showed no preview")
                Qt.exit(1)
                return
            }

            ctl.advance(0.04, 1.0, "off")
            if (ctl.planJson !== preview) {
                console.error("advance dropped the preview")
                Qt.exit(1)
                return
            }
            Qt.exit(0)
        }
    }
}
