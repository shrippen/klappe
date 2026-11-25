import QtQuick
import io.github.shrippen.klappe
import "lib"
import "../../qml"

// The element form (D-22, D-26) through the real controller: one Apply that changes the name
// and the responsible person stores both, as one undo step. The edit refreshes the project and
// the form reloads from it, so nothing may be read from the fields after the first edit.
Item {
    id: root

    Watchdog {}
    Controller { id: ctl }
    Texts { id: texts; source: ctl.texts }

    BreakdownView {
        id: breakdown
        width: 1000
        height: 600
        texts: texts
        controller: ctl
        mode: "elements"
    }

    function fail(message) {
        console.error(message)
        Qt.exit(1)
    }

    function project() {
        return JSON.parse(ctl.projectJson)
    }

    // The child of `item` called `name`, at any depth.
    function child(item, name) {
        for (const c of item.children) {
            const found = c.objectName === name ? c : child(c, name)
            if (found) {
                return found
            }
        }
        return null
    }

    Timer {
        interval: 0
        running: true
        onTriggered: {
            const example = Qt.resolvedUrl("../../../../testdata/beispiel.klappe")
            if (!ctl.openProject(example)) {
                return root.fail("project did not open")
            }
            const jo = ctl.addPerson("Jo Berg", "costume")
            const element = ctl.addElement("costume", "Hut")
            if (jo === "" || element === "") {
                return root.fail("setup: " + ctl.status)
            }

            // The element: name and responsible person in one Apply.
            breakdown.pick(element)
            const elementForm = child(breakdown, "elementForm")
            child(elementForm, "nameField").text = "Hut 2"
            child(elementForm, "responsibleBox").currentIndex = 1
            elementForm.apply()
            const hat = project().elements.find(e => e.id === element)
            if (hat.name !== "Hut 2" || hat.responsible !== jo) {
                return root.fail("element form: " + JSON.stringify(hat))
            }

            // One undo takes back the whole form.
            ctl.undo()
            const back = project().elements.find(e => e.id === element)
            if (back.name !== "Hut" || back.responsible !== null) {
                return root.fail("one undo must take back the element form: " + JSON.stringify(back))
            }

            Qt.exit(0)
        }
    }
}
