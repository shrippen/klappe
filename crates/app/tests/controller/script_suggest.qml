import QtQuick
import QtQuick.Window
import io.github.shrippen.klappe
import "lib"
import "../../qml"

// The choices under the line being typed (DB-03) with the real controller: a heading takes its
// location and then its time of day from the list and goes on to the action, an empty cue in a
// conversation is the other speaker, Esc closes the list.
Window {
    id: root
    width: 1200
    height: 800
    visible: true

    Watchdog {}
    Controller { id: ctl }
    Texts { id: texts; source: ctl.texts }
    ScreenplayView {
        id: view
        anchors.fill: parent
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
        view.flow.updateSuggest()
    }

    function options() {
        return view.flow.suggestions ? view.flow.suggestions.options : []
    }

    Timer {
        interval: 300
        running: true
        onTriggered: {
            const example = Qt.resolvedUrl("../../../../testdata/beispiel.klappe")
            if (!ctl.openProject(example)) {
                return root.fail("project did not open")
            }
            const area = view.area
            const flow = view.flow
            ctl.editScript("INT. KÜCHE - TAG\n\nANNA\nHi.\n\nBEN\nHo.\n\nANNA\nHm.\n\n")
            area.forceActiveFocus()
            area.cursorPosition = area.text.length
            if (!area.activeFocus) {
                return root.fail("the text has no focus")
            }

            root.type("int. k")
            if (JSON.stringify(root.options()) !== JSON.stringify(["KÜCHE - "])) {
                return root.fail("the location is offered: " + JSON.stringify(root.options()))
            }
            if (!flow.enter() || !area.text.endsWith("INT. KÜCHE - ")) {
                return root.fail("Enter takes the location: " + JSON.stringify(area.text))
            }

            root.type("n")
            if (root.options()[0] !== "NACHT") {
                return root.fail("the times are offered: " + JSON.stringify(root.options()))
            }
            flow.updateSuggest()
            flow.dismissSuggest()
            if (flow.suggestions !== null) {
                return root.fail("Esc closes the list")
            }
            root.type("a")
            flow.moveSuggest(1)
            flow.moveSuggest(-1)
            if (!flow.enter()) {
                return root.fail("Enter takes the time")
            }
            if (!area.text.endsWith("INT. KÜCHE - NACHT\n\n") || flow.pendingLine < 0
                    || flow.shown[flow.pendingLine] !== "a") {
                return root.fail("the time goes on to the action: " + JSON.stringify(area.text))
            }
            if (flow.suggestions !== null) {
                return root.fail("no list on the action line")
            }

            flow.tab()
            flow.updateSuggest()
            if (JSON.stringify(root.options()) !== JSON.stringify(["BEN"]) && root.options()[0] !== "BEN") {
                return root.fail("the other speaker is offered: " + JSON.stringify(root.options()))
            }
            Qt.exit(0)
        }
    }
}
