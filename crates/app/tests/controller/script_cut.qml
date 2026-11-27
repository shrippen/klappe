import QtQuick
import io.github.shrippen.klappe
import "lib"
import "../../qml"

// D-36: a cut or a copy of the shown text is closed Fountain. A selection over the edge of an
// emphasis takes the marks of its own emphasis along and leaves the rest standing, so what is
// pasted shows no stray `*` or `_`; a cut is the deletion typed in the text area would be. The
// text area's own undo list stays off: its context menu would undo half a step of the history.
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

    function shown(line) {
        return view.area.text.split("\n")[line]
    }

    function sourceLine(line) {
        return view.doc.read().split("\n")[line]
    }

    Timer {
        interval: 0
        running: true
        onTriggered: {
            const example = Qt.resolvedUrl("../../../../testdata/beispiel.klappe")
            if (!ctl.openProject(example)) {
                return root.fail("project did not open")
            }
            const area = view.area
            const doc = view.doc

            ctl.editScript("Das ist **fett** und *kursiv*.\n\nEin **a**_b_ Wort\n\nHallo Welt\n")

            // A copy over the edge of the bold word: the marks of the bold word close it.
            area.select(10, 16)
            if (doc.selectedSource() !== "**tt** und") {
                return root.fail("copy of `tt und`: " + doc.selectedSource())
            }
            // A copy inside the bold word is bold too.
            area.select(9, 11)
            if (doc.selectedSource() !== "**et**") {
                return root.fail("copy of `et`: " + doc.selectedSource())
            }

            // The cut is the deletion: the bold word keeps its first half.
            area.select(10, 16)
            doc.cutSelection()
            if (sourceLine(0) !== "Das ist **fe** *kursiv*." || shown(0) !== "Das ist fe kursiv.") {
                return root.fail("cut of `tt und`: " + JSON.stringify(sourceLine(0)) + " / " + JSON.stringify(shown(0)))
            }
            if (view.tools.emphasisOf(0) !== "8-10b 11-17i") {
                return root.fail("emphasis after the cut: " + view.tools.emphasisOf(0))
            }

            // Two emphasis side by side: the cut of `a` leaves the underlined `b` alone.
            const line = area.text.indexOf("Ein ab Wort")
            area.select(line + 4, line + 5)
            if (doc.selectedSource() !== "**a**") {
                return root.fail("copy of `a`: " + doc.selectedSource())
            }
            doc.cutSelection()
            if (sourceLine(2) !== "Ein _b_ Wort") {
                return root.fail("cut of `a`: " + JSON.stringify(sourceLine(2)))
            }

            // The text area's own undo list is off: only the history of the source undoes.
            doc.replace(doc.read().indexOf("Welt"), doc.read().indexOf("Welt") + 4, "Erde")
            if (area.canUndo || area.canRedo) {
                return root.fail("the text area keeps an undo list of its own")
            }
            Qt.exit(0)
        }
    }
}
