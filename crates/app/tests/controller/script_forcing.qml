import QtQuick
import io.github.shrippen.klappe
import "lib"
import "../../qml"

// D-36: the characters that force an element (`.`, `!`, `@`, `>…<`, `~`, ` ^`) are not in the
// text the editor shows. The element still comes from the whole text; typing keeps the kind and
// the character in the source; Enter, Tab, search, copy, undo and the source view work on the
// source. A capital is typed for a forced cue as for any (DB-01: `@mcClane` becomes `@MCCLANE`).
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

    readonly property string book: ".flashback\n\n!INT. Text\n\n@mcClane\n(leise)\nHallo.\n\n"
        + "BOB ^\nHi.\n\n>ENDE<\n\n~la la\n\nAnna geht."

    function fail(message) {
        console.error(message)
        Qt.exit(1)
    }

    function row(line) {
        return view.area.text.split("\n")[line]
    }

    function sourceRow(line) {
        return view.doc.read().split("\n")[line]
    }

    /// The text position of the shown line `line`, and its end.
    function lineAt(line) {
        const lines = view.area.text.split("\n")
        const start = lines.slice(0, line).reduce((sum, l) => sum + l.length + 1, 0)
        return [start, start + lines[line].length]
    }

    function type(text) {
        for (const c of text) {
            const at = view.area.cursorPosition
            view.area.insert(at, c)
            view.area.cursorPosition = at + 1
        }
    }

    function copyOf(line, from, to) {
        const [start] = lineAt(line)
        view.area.select(start + from, start + to)
        return view.doc.selectedSource()
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
            const flow = view.flow
            ctl.editScript(book)

            // Loading: the lines show without their characters, the source stays.
            const want = ["flashback", "", "INT. Text", "", "mcClane", "(leise)", "Hallo.", "",
                          "BOB", "Hi.", "", "ENDE", "", "la la", "", "Anna geht."]
            if (JSON.stringify(area.text.split("\n")) !== JSON.stringify(want)) {
                return root.fail("shown: " + JSON.stringify(area.text.split("\n")))
            }
            if (doc.read() !== book) {
                return root.fail("the source changed by loading")
            }
            const kinds = view.flow.shown
            if (kinds[0] !== "h" || kinds[2] !== "a" || kinds[4] !== "c" || kinds[8] !== "c") {
                return root.fail("kinds " + kinds)
            }

            // A copy takes the hidden character before it from the start of a line and the one behind it
            // from the end, one in the middle of the line none.
            const wholes = [[0, ".flashback"], [2, "!INT. Text"], [4, "@mcClane"], [8, "BOB ^"],
                            [11, ">ENDE<"], [13, "~la la"]]
            for (const [line, source] of wholes) {
                const got = copyOf(line, 0, row(line).length)
                if (got !== source) {
                    return root.fail("copy of line " + line + ": " + JSON.stringify(got))
                }
            }
            if (copyOf(4, 2, 5) !== "Cla" || copyOf(4, 0, 3) !== "@mcC" || copyOf(4, 3, 7) !== "lane") {
                return root.fail("copy inside the cue: " + copyOf(4, 2, 5) + ", " + copyOf(4, 0, 3) + ", "
                                 + copyOf(4, 3, 7))
            }
            if (copyOf(8, 1, 3) !== "OB ^" || copyOf(8, 0, 3) !== "BOB ^") {
                return root.fail("copy of the dual cue: " + copyOf(8, 0, 3))
            }

            // Typing in the cue keeps its kind and its character.
            area.cursorPosition = lineAt(4)[1]
            type("X")
            if (sourceRow(4) !== "@MCCLANEX" || row(4) !== "MCCLANEX" || view.flow.shown[4] !== "c") {
                return root.fail("typed in the cue: " + sourceRow(4) + " / " + row(4) + " / " + view.flow.shown)
            }
            // Lines nobody touched are as they were.
            const lines = doc.read().split("\n")
            const before = book.split("\n")
            for (let n = 0; n < lines.length; n++) {
                if (n !== 4 && lines[n] !== before[n]) {
                    return root.fail("line " + n + " changed: " + lines[n])
                }
            }

            // Undo gives the source and the display back: the capitals are a step of their own, as
            // for any line that is not typed from its start.
            view.undoText()
            view.undoText()
            if (doc.read() !== book || area.text.split("\n")[4] !== "mcClane") {
                return root.fail("undo: " + JSON.stringify(sourceRow(4)) + " / " + row(4))
            }

            // Enter after a forced cue opens the dialogue below it; after ` ^` too, behind it.
            area.cursorPosition = lineAt(4)[1]
            ctl.editScript("ANNA\nHi.\n\n@mcClane")
            area.cursorPosition = lineAt(3)[1]
            if (!flow.enter() || flow.shown[4] !== "d" || doc.read() !== "ANNA\nHi.\n\n@MCCLANE\n") {
                return root.fail("Enter after the cue: " + JSON.stringify(doc.read()) + " " + flow.shown)
            }
            ctl.editScript("ANNA\nHi.\n\n@BOB ^")
            area.cursorPosition = lineAt(3)[1]
            if (!flow.enter() || flow.shown[4] !== "d" || doc.read() !== "ANNA\nHi.\n\n@BOB ^\n") {
                return root.fail("Enter after the dual cue: " + JSON.stringify(doc.read()) + " " + flow.shown)
            }

            // Tab on an empty line is as before: action to cue.
            ctl.editScript("Er geht.\n\n")
            area.cursorPosition = area.text.length
            flow.tab()
            if (flow.shown[2] !== "c") {
                return root.fail("Tab on an empty line: " + flow.shown)
            }

            // Search finds the word behind the hidden `@`.
            ctl.editScript(book)
            const search = view.search
            area.cursorPosition = 0
            search.open()
            search.findField.text = "mcClane"
            search.from(0)
            if (search.hits.length !== 1 || area.selectedText !== "mcClane") {
                return root.fail("search: " + JSON.stringify(search.hits) + " " + area.selectedText)
            }
            search.close()

            // Ctrl+B over a forced line keeps the character.
            area.select(lineAt(13)[0], lineAt(13)[1])
            doc.toggleMark("bold")
            if (sourceRow(13) !== "~**la la**" || row(13) !== "la la") {
                return root.fail("bold: " + sourceRow(13) + " / " + row(13))
            }

            // The source view shows every character. (`editScript` of the text the session has
            // already is no change, so the area is loaded directly.)
            doc.load(book)
            view.showSource(true)
            if (row(4) !== "@mcClane" || row(8) !== "BOB ^" || row(0) !== ".flashback") {
                return root.fail("source view: " + row(0) + " / " + row(4) + " / " + row(8))
            }
            view.showSource(false)
            if (row(4) !== "mcClane" || doc.read() !== book) {
                return root.fail("back to the shown text: " + row(4) + " / " + JSON.stringify(doc.read()))
            }
            Qt.exit(0)
        }
    }
}
