import QtQuick
import io.github.shrippen.klappe
import "lib"
import "../../qml"

// The person form of Team (D-26) through the real controller: one Apply that changes the name
// and the phone stores both, as one undo step. The edit refreshes the project and the form
// reloads from it, so nothing may be read from the fields after the first edit.
Item {
    id: root

    Watchdog {}
    Controller { id: ctl }
    Texts { id: texts; source: ctl.texts }

    TeamView {
        id: team
        width: 1000
        height: 600
        texts: texts
        controller: ctl
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
            if (jo === "") {
                return root.fail("setup: " + ctl.status)
            }

            // The person: name and phone in one Apply.
            team.pick(jo)
            const personForm = child(team, "personForm")
            child(personForm, "nameField").text = "Jo Berger"
            child(personForm, "phoneField").text = "0301"
            personForm.apply()
            const row = project().persons.find(p => p.id === jo)
            if (row.name !== "Jo Berger" || row.phone !== "0301") {
                return root.fail("person form: " + JSON.stringify(row))
            }

            // One undo takes back the whole form.
            ctl.undo()
            const before = project().persons.find(p => p.id === jo)
            if (before.name !== "Jo Berg" || before.phone !== "") {
                return root.fail("one undo must take back the person form: " + JSON.stringify(before))
            }

            Qt.exit(0)
        }
    }
}
