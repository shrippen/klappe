import QtQuick
import io.github.shrippen.klappe
import "lib"
import "../../qml"

// Besetzung and Team (D-26) through the real controller: persons, contacts, casting (propose,
// fix, drop), cast numbers (by hand, taken, back to the book, locked), the responsible person of
// an element and the persons in the order of the element's category; deleting a person takes it
// out of the casting and the element. The two views show what the controller holds.
Item {
    id: root

    Watchdog {}
    Controller { id: ctl }
    Texts { id: texts; source: ctl.texts }

    CastingView {
        id: casting
        width: 1000
        height: 600
        texts: texts
        controller: ctl
    }
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

    function castingOf(id) {
        return project().castings.find(k => k.character === id)
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
            const [anna, bert] = project().characters.map(c => c.id)
            if (project().persons.length !== 0 || project().departments.indexOf("cast") < 0) {
                return root.fail("the example has no persons: " + JSON.stringify(project().persons))
            }

            // A refused person says why and makes none.
            if (ctl.addPerson("  ", "cast") !== "" || ctl.addPerson("Jo", "nothing") !== ""
                    || ctl.status === "" || project().persons.length !== 0) {
                return root.fail("a blank name and an unknown department must be refused")
            }

            const lena = ctl.addPerson("Lena Weber", "cast")
            const jo = ctl.addPerson("Jo Berg", "costume")
            if (lena === "" || jo === "" || project().persons.length !== 2) {
                return root.fail("persons were not made: " + ctl.status)
            }
            if (!ctl.editPerson(jo, "Jo Berger", "costume", "Gewandmeister", "Agentur",
                                "0301", "jo@example.org", "Hauptstr. 1")) {
                return root.fail("person was not changed: " + ctl.status)
            }
            const row = project().persons.find(p => p.id === jo)
            if (row.name !== "Jo Berger" || row.function !== "Gewandmeister" || row.phone !== "0301"
                    || row.mail !== "jo@example.org" || row.address !== "Hauptstr. 1") {
                return root.fail("person: " + JSON.stringify(row))
            }

            // The casting: proposals, one fixed.
            if (!ctl.proposeCast(anna, lena) || !ctl.proposeCast(anna, jo) || !ctl.fixCast(anna, lena)) {
                return root.fail("casting was refused: " + ctl.status)
            }
            if (castingOf(anna).fixed !== lena || castingOf(anna).proposed.length !== 2) {
                return root.fail("casting: " + JSON.stringify(castingOf(anna)))
            }
            if (!ctl.dropProposal(anna, jo) || castingOf(anna).proposed.join() !== lena) {
                return root.fail("proposal was not dropped")
            }
            if (!ctl.fixCast(anna, "") || castingOf(anna).fixed !== null) {
                return root.fail("fixing nobody must free the role")
            }
            ctl.fixCast(anna, lena)

            // Cast numbers: by hand, taken, no number, back to the book, locked.
            if (!ctl.setCastNumber(anna, "7") || castingOf(anna).number !== 7 || !castingOf(anna).stored) {
                return root.fail("number by hand: " + JSON.stringify(castingOf(anna)))
            }
            if (ctl.setCastNumber(bert, "7") || ctl.status === "") {
                return root.fail("a taken number must be refused")
            }
            if (ctl.setCastNumber(bert, "x")) {
                return root.fail("no number must be refused")
            }
            if (!ctl.setCastNumber(anna, "") || castingOf(anna).stored) {
                return root.fail("an empty number must go back to the book")
            }
            if (!ctl.lockNumbers() || !project().numbers_locked || !castingOf(bert).stored) {
                return root.fail("numbers were not locked")
            }
            if (!ctl.unlockNumbers() || project().numbers_locked) {
                return root.fail("numbers were not freed")
            }

            // The responsible person; the department of the category first.
            const element = ctl.addElement("costume", "Hut")
            if (element === "" || !ctl.setResponsible(element, jo)) {
                return root.fail("responsible was refused: " + ctl.status)
            }
            if (project().elements.find(e => e.id === element).responsible !== jo) {
                return root.fail("responsible is not listed")
            }
            if (JSON.parse(ctl.personsFor("costume")).map(p => p.id).join() !== jo + "," + lena
                    || JSON.parse(ctl.personsFor("makeup")).map(p => p.id).join() !== lena + "," + jo) {
                return root.fail("order of persons: " + ctl.personsFor("costume"))
            }
            if (JSON.parse(ctl.personsFor("nothing")).length !== 0) {
                return root.fail("an unknown category has no list")
            }

            // The views show what the controller holds.
            if (!child(casting, "role." + anna) || child(casting, "role." + anna).meta !== "Lena Weber") {
                return root.fail("the casting view does not show the fixed person")
            }
            if (!child(team, "person." + jo) || child(team, "person." + jo).text !== "Jo Berger") {
                return root.fail("the team view does not show the person")
            }

            // Deleting takes the person out of the casting and the element.
            if (!ctl.removePerson(jo)) {
                return root.fail("person was not deleted: " + ctl.status)
            }
            if (project().elements.find(e => e.id === element).responsible !== null) {
                return root.fail("the element still has the deleted person")
            }
            if (!ctl.removePerson(lena) || castingOf(anna).fixed !== null || castingOf(anna).proposed.length !== 0) {
                return root.fail("the casting still has the deleted person")
            }
            if (ctl.removePerson(lena) || ctl.status === "") {
                return root.fail("deleting twice must be refused")
            }

            Qt.exit(0)
        }
    }
}
