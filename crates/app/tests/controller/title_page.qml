import QtQuick
import io.github.shrippen.klappe
import "lib"
import "../../qml"

// The title page card (DB-06) with the real controller: it reads the fields of the text, Apply
// writes them as the block at the top (an empty field is left out, the body stays) and the
// editor's undo takes it back as one step; Discard changes nothing.
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
            const form = view.titleCard
            const before = area.text

            // Folded at first; the menu's request unfolds it over the text (the focus is in tst_titlecard).
            if (!form.collapsed || form.title !== "Titelseite · Heimat") {
                return root.fail("folded line: " + form.title)
            }
            view.showLayout("print")
            if (form.visible) {
                return root.fail("the card is not shown beside the printed pages")
            }
            view.showTitlePage()
            if (view.layout !== "flow" || form.collapsed) {
                return root.fail("showTitlePage unfolds in the flowing text")
            }
            const fields = form.values()
            if (fields["Title"] !== "Heimat" || fields["Contact"] !== "Eva" || fields["Author"] !== "") {
                return root.fail("the card shows the title page: " + JSON.stringify(fields))
            }

            // Discard: the fields go back, the text stays.
            form.setField("Title", "Zwischen")
            form.discard()
            if (area.text !== before || form.values()["Title"] !== "Heimat") {
                return root.fail("Discard changed the text or kept the field: " + JSON.stringify(area.text))
            }

            // Apply: a new title, a new author, no contact.
            form.setField("Contact", "")
            form.setField("Title", "Neu")
            form.setField("Author", "Anna")
            form.apply()
            if (area.text !== "Title: Neu\nAuthor: Anna\n\n" + body) {
                return root.fail("Apply wrote " + JSON.stringify(area.text))
            }
            if (!form.collapsed || form.title !== "Titelseite · Neu · Anna") {
                return root.fail("folded line after Apply: " + form.title)
            }

            view.undoText()
            if (area.text !== before) {
                return root.fail("one undo step takes it back: " + JSON.stringify(area.text))
            }
            Qt.exit(0)
        }
    }
}
