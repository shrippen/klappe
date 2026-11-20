import QtQuick
import io.github.shrippen.klappe
import "lib"
import "../../qml"

// The element flow in the window (Lastenheft, Anhang A; D-15): Enter at the end of a heading
// opens an action after a blank line, Tab turns it into a cue, the cue takes capitals as it is
// typed, Enter after it opens the dialogue right below. Each new line shows as its element at
// once. Undo takes back the speech, the break and the cue, each as one step.
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

    function type(text) {
        for (const c of text) {
            const at = view.area.cursorPosition
            view.area.insert(at, c)
            view.area.cursorPosition = at + 1
        }
    }

    function row(line) {
        return view.area.text.split("\n")[line]
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
            const flow = view.flow
            ctl.editScript("INT. SAAL - TAG\n\nSie kommt an.")
            area.cursorPosition = "INT. SAAL - TAG".length

            if (!flow.enter() || flow.pendingLine !== 2 || flow.shown[2] !== "a") {
                return root.fail("Enter after a heading: pending " + flow.pendingLine + ", shown " + flow.shown)
            }
            const opened = area.text

            flow.tab()
            if (flow.shown[2] !== "c") {
                return root.fail("Tab makes the empty line a cue: " + flow.shown)
            }

            root.type("anna")
            if (row(2) !== "ANNA" || area.cursorPosition !== opened.indexOf("\n\n\n") + 2 + 4) {
                return root.fail("the cue takes capitals as typed: " + JSON.stringify(row(2)))
            }

            if (!flow.enter() || flow.shown[3] !== "d") {
                return root.fail("Enter after a cue opens the dialogue: " + flow.shown)
            }
            root.type("Hallo.")
            const kinds = ctl.scriptKinds(area.text)
            if (kinds[2] !== "c" || kinds[3] !== "d") {
                return root.fail("cue and speech are read as such: " + kinds)
            }

            for (let i = 0; i < 3; i++) {
                view.undoText()
            }
            if (area.text !== opened) {
                return root.fail("three steps back are the opened line: " + JSON.stringify(area.text))
            }
            Qt.exit(0)
        }
    }
}
