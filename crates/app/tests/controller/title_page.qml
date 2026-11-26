import QtQuick
import io.github.shrippen.klappe
import "lib"
import "../../qml"

// The title page form (DB-06) with the real controller: it reads the fields of the text, Apply
// writes them as the block at the top (an empty field is left out, the body stays) and the
// editor's undo takes it back as one step; Cancel changes nothing.
Item {
    id: root

    Watchdog {}
    Controller { id: ctl }
    Texts { id: texts; source: ctl.texts }
    ScreenplayView {
        id: view
        width: 1200
        height: 800
        texts: texts
        controller: ctl
        debounceMs: 50
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
            const body = "INT. SAAL - TAG\n\nSie kommt an.\n"
            ctl.editScript("Title: Heimat\nContact: Eva\n\n" + body)
            const area = view.area
            const form = view.titlePage
            const before = area.text

            form.show()
            const fields = form.values()
            if (fields["Title"] !== "Heimat" || fields["Contact"] !== "Eva" || fields["Author"] !== "") {
                return root.fail("the form shows the title page: " + JSON.stringify(fields))
            }

            // Cancel: nothing changes.
            form.reject()
            if (area.text !== before) {
                return root.fail("Cancel changed the text: " + JSON.stringify(area.text))
            }

            // Apply: a new title, a new author, no contact.
            form.show()
            form.setField("Contact", "")
            form.setField("Title", "Neu")
            form.setField("Author", "Anna")
            form.accept()
            if (area.text !== "Title: Neu\nAuthor: Anna\n\n" + body) {
                return root.fail("Apply wrote " + JSON.stringify(area.text))
            }

            view.undoText()
            if (area.text !== before) {
                return root.fail("one undo step takes it back: " + JSON.stringify(area.text))
            }
            Qt.exit(0)
        }
    }
}
