import QtQuick
import io.github.shrippen.klappe
import "../controller/lib"
import "../../qml"

// D-36: the editor loads a script of 120 pages and takes a key. Run by `make perf` (a release
// build, quiet machine): `cargo test --release -p klappe-app the_editor_of_120_pages -- --ignored
// --nocapture`. The numbers are in planung/STAND.md. Loading goes up to the text being laid
// out; a key goes up to the lines being set and the text sent after the pause being scheduled.
Item {
    id: root

    /// Loading 120 pages, and a key: one frame (D-36).
    readonly property int loadMs: 300
    readonly property int keyMs: 16
    readonly property int keys: 30
    readonly property int pages: 120
    readonly property int linesPerPage: 50

    Watchdog {}
    Controller { id: ctl }
    Texts { id: texts; source: ctl.texts }
    ScreenplayView {
        id: view
        width: 1200
        height: 800
        texts: texts
        controller: ctl
    }

    function fail(message) {
        console.error(message)
        Qt.exit(1)
    }

    // Scenes of action with emphasis and dialogue, up to the lines of 120 pages.
    function script() {
        const rows = []
        for (let scene = 1; rows.length < pages * linesPerPage; scene++) {
            rows.push("INT. ROOM " + scene + " - DAY", "")
            rows.push("Rain taps on the window. A **clock** ticks, somebody *coughs* in the next room, and the light flickers twice before it settles. Nobody moves.", "")
            for (let line = 0; line < 6; line++) {
                rows.push("ANNA", "(quietly)", "This is line " + line + ". It is _long_ enough to wrap over two or three rows. Then it ends.", "")
            }
            rows.push("CUT TO:", "")
        }
        return rows.join("\n")
    }

    function median(times) {
        times.sort((a, b) => a - b)
        return times[Math.floor(times.length / 2)]
    }

    Timer {
        interval: 0
        running: true
        onTriggered: {
            const example = Qt.resolvedUrl("../../../../testdata/beispiel.klappe")
            if (!ctl.openProject(example)) {
                return root.fail("project did not open")
            }
            const text = script()
            const area = view.area

            // Loading: the projection, the text area and its layout (asking for the end of the
            // text lays out all of it).
            let start = Date.now()
            view.doc.load(text)
            area.positionToRectangle(area.length)
            const loaded = Date.now() - start
            console.log("perf script editor: load of", text.split("\n").length, "lines:", loaded, "ms")

            // A key in the middle of a line with emphasis, in the middle of the book.
            const times = []
            const at = area.text.indexOf("A clock ticks", area.length / 2) + 4
            area.cursorPosition = at
            for (let i = 0; i < keys; i++) {
                start = Date.now()
                area.insert(area.cursorPosition, "x")
                area.cursorPosition += 1
                times.push(Date.now() - start)
            }
            const key = median(times)
            console.log("perf script editor: a key:", key, "ms (median of", keys + ")")

            if (loaded > loadMs || key > keyMs) {
                return root.fail("over the budgets: load " + loaded + " ms (" + loadMs + "), key " + key + " ms (" + keyMs + ")")
            }
            Qt.exit(0)
        }
    }
}
