import QtQuick
import io.github.shrippen.klappe
import "lib"

// Templates (LH-07) through the real controller: placing is one undo step and adds characters
// when the plan has none free, a locked layer refuses it, filing lists the plan as an own
// template.
Item {
    id: root

    Watchdog {}
    Controller { id: ctl }

    function fail(message) {
        console.error(message)
        Qt.exit(1)
    }

    function choices() {
        return JSON.parse(ctl.choicesJson)
    }

    Timer {
        interval: 0
        running: true
        onTriggered: {
            const example = Qt.resolvedUrl("../../../../testdata/beispiel.klappe")
            if (!ctl.openProject(example)) {
                return root.fail("project did not open")
            }
            const ids = choices().template.map(t => t.id)
            if (ids.join() !== "shot-reverse,triangle,long-take") {
                return root.fail("built-in templates: " + ids)
            }
            if (choices().figure.length !== 2) {
                return root.fail("the example has two characters")
            }

            ctl.setLock("cameras", "locked")
            if (ctl.applyTemplate("shot-reverse") || ctl.status === "" || ctl.canUndo) {
                return root.fail("a locked camera layer must refuse the template")
            }
            ctl.setLock("cameras", "unlocked")

            if (!ctl.applyTemplate("shot-reverse")) {
                return root.fail("template was refused: " + ctl.status)
            }
            if (choices().figure.length !== 4) {
                return root.fail("two characters should have been made")
            }
            if (!ctl.canUndo || !ctl.dirty) {
                return root.fail("placing is a change")
            }

            if (!ctl.saveTemplate("Mein Dialog")) {
                return root.fail("plan was not filed: " + ctl.status)
            }
            if (ctl.status.indexOf("Mein Dialog") < 0) {
                return root.fail("status does not name the template: " + ctl.status)
            }
            const own = choices().template.filter(t => t.custom)
            if (own.length !== 1 || own[0].label !== "Mein Dialog" || own[0].figures !== 4 || own[0].cameras !== 5) {
                return root.fail("own template: " + JSON.stringify(own))
            }

            ctl.undo()
            ctl.undo()
            if (choices().figure.length !== 2 || choices().template.length !== 3) {
                return root.fail("undo should take both steps back")
            }
            if (ctl.saveTemplate("  ")) {
                return root.fail("a blank name must be refused")
            }
            Qt.exit(0)
        }
    }
}
