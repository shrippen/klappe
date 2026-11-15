import QtQuick
import io.github.shrippen.klappe
import "lib"

// LH-93: the plan counts the objects without a height and marks them; Shift+H's call selects
// the next one, which takes the typed height; in 3D the view follows. The example's set has
// three such objects: tisch, saeule-1, saeule-2, in this order.
Item {
    Watchdog {}
    Controller { id: ctl }

    function fail(message) {
        console.error(message)
        Qt.exit(1)
    }

    // How many captions of the plan are the warning "height missing".
    function warnings() {
        return JSON.parse(ctl.planJson).shapes.filter(s => s.label === "warning").length
    }

    // The next object takes `height`; its key must be `key`.
    function step(key, height) {
        if (!ctl.nextMissingHeight()) {
            return fail("no next object without height before " + key)
        }
        if (ctl.selectedKey !== key) {
            return fail("selected " + ctl.selectedKey + " instead of " + key)
        }
        if (!ctl.setObjectHeight(height)) {
            return fail("the height " + height + " was refused")
        }
        return true
    }

    Timer {
        interval: 0
        running: true
        onTriggered: {
            if (!ctl.openProject(Qt.resolvedUrl("../../../../testdata/beispiel.klappe"))) {
                return fail("project did not open")
            }
            if (ctl.missingHeights !== 3 || warnings() !== 3) {
                return fail("three objects lack a height: " + ctl.missingHeights + ", " + warnings())
            }

            if (!step("object:tisch", "1")) {
                return
            }
            if (ctl.missingHeights !== 2 || warnings() !== 2) {
                return fail("the table's height was not counted: " + ctl.missingHeights)
            }

            // In 3D the view follows the object: the picture is made again.
            ctl.toggleView()
            ctl.freeSize(800, 600)
            const revision = ctl.freeRevision
            if (!step("object:saeule-1", "2")) {
                return
            }
            if (ctl.freeRevision === revision || ctl.missingHeights !== 1) {
                return fail("the 3D view did not follow the pillar")
            }
            ctl.toggleView()

            if (!step("object:saeule-2", "2")) {
                return
            }
            if (ctl.missingHeights !== 0 || warnings() !== 0) {
                return fail("all heights are set, still " + ctl.missingHeights + " counted")
            }
            if (ctl.nextMissingHeight()) {
                return fail("nothing is left to jump to")
            }
            Qt.exit(0)
        }
    }
}
