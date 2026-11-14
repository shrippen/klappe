import QtQuick
import io.github.shrippen.klappe
import "lib"

// A figure dragged while play goes on: the preview must show what the release stores. The
// drag picks the mark by the time, so a frozen preview moves one mark, the release another.
Item {
    Controller { id: ctl }
    Watchdog {}

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

            ctl.seek(4)
            ctl.setTool("select", "")
            ctl.drag(6.5, 6.5, 6.5, 8.5, false)
            ctl.advance(3.0, 1.0, "off")
            const preview = ctl.planJson
            ctl.drag(6.5, 6.5, 6.5, 8.5, true)

            if (ctl.planJson !== preview) {
                console.error("the preview was not what the release stored")
                Qt.exit(1)
                return
            }
            Qt.exit(0)
        }
    }
}
