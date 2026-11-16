import QtQuick
import QtCore
import io.github.shrippen.klappe
import "lib"

// Templates (LH-07) through the real controller: placing is one undo step and adds characters
// when the plan has none free, a locked layer refuses it, filing lists the plan as an own
// template; an own template is renamed and deleted (one undo step each, a built-in one is
// refused), and a template file goes out and comes back in.
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

            const id = own[0].id
            if (!ctl.renameTemplate(id, "Zwiegespräch")) {
                return root.fail("rename was refused: " + ctl.status)
            }
            if (choices().template.filter(t => t.custom)[0].label !== "Zwiegespräch") {
                return root.fail("rename is not listed")
            }
            if (ctl.renameTemplate(id, " ") || ctl.renameTemplate("triangle", "Dreieck 2")) {
                return root.fail("a blank name and a built-in template must be refused")
            }

            const file = StandardPaths.writableLocation(StandardPaths.TempLocation) + "/klappe-templates-test.toml"
            if (!ctl.exportLibrary(file, "templates") || ctl.status.indexOf("1") < 0) {
                return root.fail("export failed: " + ctl.status)
            }
            if (!ctl.removeTemplate(id) || choices().template.length !== 3) {
                return root.fail("delete failed: " + ctl.status)
            }
            if (ctl.removeTemplate("triangle") || choices().template.length !== 3) {
                return root.fail("a built-in template must not be deleted")
            }
            if (ctl.exportLibrary(file, "templates")) {
                return root.fail("nothing to export without own templates")
            }
            if (!ctl.importLibrary(file, "templates")) {
                return root.fail("import failed: " + ctl.status)
            }
            if (choices().template.filter(t => t.custom)[0].label !== "Zwiegespräch") {
                return root.fail("import did not bring the template back")
            }

            for (let step = 0; step < 5; step++) {
                ctl.undo()
            }
            if (choices().figure.length !== 2 || choices().template.length !== 3) {
                return root.fail("undo should take every step back")
            }
            if (ctl.saveTemplate("  ")) {
                return root.fail("a blank name must be refused")
            }
            Qt.exit(0)
        }
    }
}
