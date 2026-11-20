import QtQuick
import io.github.shrippen.klappe
import "lib"
import "../../qml"

// Find and replace in the window (DB-60) with the real controller and text tools: the hits are
// positions of the text area, also after a character outside the BMP; "Alle ersetzen" changes
// every hit and is one step of the text's undo history.
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
            const book = "INT. SAAL - TAG\n\n🎬 Anna sieht anna.\n\nANNA\nJa."
            ctl.editScript(book)
            const area = view.area
            const search = view.search
            area.cursorPosition = 0

            search.open()
            search.findField.text = "anna"
            search.from(0)
            if (search.hits.length !== 3 || area.selectedText !== "Anna") {
                return root.fail("hits " + JSON.stringify(search.hits) + ", selected " + area.selectedText)
            }

            search.replaceField.text = "Eva"
            search.replaceAll()
            const all = "INT. SAAL - TAG\n\n🎬 Eva sieht Eva.\n\nEva\nJa."
            if (area.text !== all) {
                return root.fail("replace all: " + JSON.stringify(area.text))
            }

            view.undoText()
            if (area.text !== book) {
                return root.fail("one step back is the text before: " + JSON.stringify(area.text))
            }
            Qt.exit(0)
        }
    }
}
