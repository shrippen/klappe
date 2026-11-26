import QtQuick
import QtCore
import io.github.shrippen.klappe
import "lib"

// D-25: what the window's looks do to the project through the controller: a look is made, worn by
// a figure in the scene on view, filled with a costume (it stands in the sheet's category), taken
// off, renamed and deleted, each an undo step. The folder is a copy of the example (made by
// qml_tests.rs).
Item {
    id: root

    Watchdog {}
    Controller { id: ctl }

    function fail(message) {
        console.error(message)
        Qt.exit(1)
    }

    function project() { return JSON.parse(ctl.projectJson) }

    Timer {
        interval: 0
        running: true
        onTriggered: {
            const base = StandardPaths.writableLocation(StandardPaths.TempLocation) + "/klappe-export-test"
            if (!ctl.openProject(base + "/looks-projekt")) {
                return root.fail("project did not open: " + ctl.status)
            }
            const scene = project().scenes[0].id
            ctl.selectScene(scene)
            const figure = project().sheet.speaking.concat(project().sheet.present)[0]
            if (!figure) {
                return root.fail("the example's first scene has no figure")
            }

            const look = ctl.addLook(figure.id, "Ölzeug, nass")
            if (look === "" || project().looks.length !== 1) {
                return root.fail("the look was not made: " + ctl.status)
            }
            if (ctl.addLook(figure.id, "ÖLZEUG, NASS") !== "") {
                return root.fail("a second look of the same name must be refused")
            }

            if (!ctl.setSceneLook(scene, figure.id, look)) {
                return root.fail("the look was not put on: " + ctl.status)
            }
            const worn = project().sheet.speaking.concat(project().sheet.present).find(f => f.id === figure.id)
            if (worn.look !== look || project().looks[0].scenes.length !== 1) {
                return root.fail("the figure must wear the look in the scene: " + ctl.projectJson)
            }

            const coat = ctl.addElement("costume", "Ölmantel")
            if (!ctl.addLookElement(look, coat)) {
                return root.fail("the costume did not go into the look: " + ctl.status)
            }
            const costume = project().sheet.categories.find(c => c.category === "costume")
            if (!costume || costume.from_looks.length !== 1 || costume.from_looks[0].id !== coat
                    || costume.from_looks[0].figure !== figure.name) {
                return root.fail("the sheet must list the look's costume: " + ctl.projectJson)
            }

            if (!ctl.addElementToLook(look, "makeup", "Schramme")
                    || project().looks[0].elements.length !== 2) {
                return root.fail("the new make-up must go into the look: " + ctl.status)
            }

            // The same name, another case and a new name: the session takes the element of that
            // name or makes one. Same text for the user in each case.
            const before = project().elements.length
            const names = ["Schramme", "SCHRAMME", "Narbe"]
            const steps = []
            for (const name of names) {
                if (!ctl.addElementToLook(look, "makeup", name)) {
                    return root.fail("the make-up '" + name + "' did not go into the look: " + ctl.status)
                }
                steps.push(ctl.undoStep)
            }
            if (project().elements.length !== before + 1 || project().looks[0].elements.length !== 3) {
                return root.fail("only 'Narbe' may be new: " + ctl.projectJson)
            }
            if (steps.join() !== "AddElementToLook,AddElementToLook,AddElementToLook") {
                return root.fail("the undo steps of the three names: " + steps)
            }
            ctl.removeLookElement(look, project().looks[0].elements[2].id)

            const prop = ctl.addElement("prop", "Lampe")
            if (ctl.addLookElement(look, prop)) {
                return root.fail("a prop must not go into a look")
            }

            if (!ctl.editLook(look, "Ölzeug") || project().looks[0].name !== "Ölzeug") {
                return root.fail("the look was not renamed: " + ctl.status)
            }
            if (!ctl.removeLookElement(look, coat) || project().looks[0].elements.length !== 1) {
                return root.fail("the costume did not leave the look")
            }

            // Empty: no look.
            if (!ctl.setSceneLook(scene, figure.id, "")) {
                return root.fail("the look was not taken off: " + ctl.status)
            }
            const bare = project().sheet.speaking.concat(project().sheet.present).find(f => f.id === figure.id)
            if (bare.look !== null || project().looks[0].scenes.length !== 0) {
                return root.fail("the figure must wear no look: " + ctl.projectJson)
            }

            if (!ctl.removeLook(look) || project().looks.length !== 0) {
                return root.fail("the look was not deleted: " + ctl.status)
            }
            if (!ctl.canUndo) {
                return root.fail("a look must be an undo step")
            }
            ctl.undo()
            if (project().looks.length !== 1) {
                return root.fail("undo must bring the look back")
            }

            Qt.exit(0)
        }
    }
}
