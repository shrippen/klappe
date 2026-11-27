import QtQuick
import QtQuick.Window
import io.github.shrippen.klappe
import "lib"
import "../../qml"

// D-36, D-37: the context menu of the text, Ctrl+B, Ctrl+I, Ctrl+U and the Fountain source.
// The menu has its entries and their states, copy and cut give Fountain and a cut is one step
// back; Ctrl+B toggles a selection (over lines too) in one step; Ctrl+I without a selection
// makes what is typed next italic, only there; a note comes in as `[[]]`; the source view shows
// `**fett**`, typing in it writes what is typed, and the way back shows the emphasis again.
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

    readonly property string book: "Das ist **fett** und kursiv.\n\nEin Wort\n\nINT. SAAL - TAG\n\nHallo Welt\n"

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

    // Types `text` at the cursor, a character at a time, as the keyboard does.
    function type(text) {
        for (const c of text) {
            const at = view.area.cursorPosition
            view.area.insert(at, c)
            view.area.cursorPosition = at + 1
        }
    }

    // Selects the shown text `word` of line `line`.
    function selectWord(line, word) {
        const text = shown(line)
        const from = view.area.text.indexOf(text) + text.indexOf(word)
        view.area.select(from, from + word.length)
    }

    // A key press as the text area gets it.
    function press(key, mod) {
        const event = { key: key, modifiers: mod, accepted: false,
                        matches: seq => (key === Qt.Key_B && seq === StandardKey.Bold)
                            || (key === Qt.Key_I && seq === StandardKey.Italic)
                            || (key === Qt.Key_U && seq === StandardKey.Underline) }
        view.keyPressed(event)
        return event.accepted
    }

    function entry(name) {
        const menu = view.scriptMenu
        for (let i = 0; i < menu.count; i++) {
            if (menu.itemAt(i).objectName === name) {
                return menu.itemAt(i)
            }
        }
        return null
    }

    // What the clipboard holds, read the way a paste reads it.
    function clipboard() {
        const probe = Qt.createQmlObject("import QtQuick; TextEdit {}", view)
        probe.paste()
        const text = probe.text
        probe.destroy()
        return text
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
            const doc = view.doc

            ctl.editScript(root.book)

            // The style's own menu does not open: no handler of the area but ours takes the
            // right button.
            for (const child of area.data) {
                if (child.acceptedButtons !== undefined && (child.acceptedButtons & Qt.RightButton)
                        && child.objectName !== "menuTap" && child.enabled) {
                    return root.fail("a handler of the style takes the right button: " + child)
                }
            }

            // The entries are there. Cut and copy want a selection, Undo a step, Bold is off
            // in plain text.
            const names = ["menuCut", "menuCopy", "menuPaste", "menuUndo", "menuRedo", "menuBold",
                           "menuItalic", "menuUnderline", "menuNote"]
            for (const name of names) {
                if (!entry(name)) {
                    return root.fail("the menu has no " + name)
                }
            }
            area.cursorPosition = 2
            view.scriptMenu.popup(area, 10, 10)
            if (entry("menuCut").enabled || entry("menuCopy").enabled || entry("menuUndo").enabled
                    || entry("menuRedo").enabled) {
                return root.fail("cut, copy, undo or redo is on without anything to act on")
            }
            if (entry("menuBold").checked || !entry("menuBold").enabled || !entry("menuNote").enabled) {
                return root.fail("bold in plain text, or no note")
            }
            view.scriptMenu.close()

            // The state at a bold word: bold ticked; on a heading bold is ticked and grey.
            area.cursorPosition = 12
            view.scriptMenu.popup(area, 10, 10)
            if (!entry("menuBold").checked || entry("menuItalic").checked) {
                return root.fail("the menu at a bold word: " + JSON.stringify(view.scriptMenu.marks))
            }
            view.scriptMenu.close()
            selectWord(4, "SAAL")
            view.scriptMenu.popup(area, 10, 10)
            if (!entry("menuBold").checked || entry("menuBold").enabled) {
                return root.fail("the menu at a heading: " + JSON.stringify(view.scriptMenu.marks))
            }
            view.scriptMenu.close()

            // Copy and cut from the menu give Fountain, as the keys do.
            selectWord(0, "fett")
            view.scriptMenu.popup(area, 10, 10)
            if (!entry("menuCopy").enabled || !entry("menuCut").enabled) {
                return root.fail("cut and copy are off with a selection")
            }
            entry("menuCopy").triggered()
            if (root.clipboard() !== "**fett**") {
                return root.fail("copy from the menu: " + JSON.stringify(root.clipboard()))
            }
            entry("menuCut").triggered()
            view.scriptMenu.close()
            if (sourceLine(0) !== "Das ist  und kursiv." || root.clipboard() !== "**fett**") {
                return root.fail("cut from the menu: " + JSON.stringify(sourceLine(0)))
            }
            // The cut is a step back; the menu says so.
            view.scriptMenu.popup(area, 10, 10)
            if (!entry("menuUndo").enabled) {
                return root.fail("undo is off after a cut")
            }
            entry("menuUndo").triggered()
            view.scriptMenu.close()
            if (doc.read() !== root.book || view.tools.emphasisOf(0) !== "8-12b") {
                return root.fail("undo of the cut: " + JSON.stringify(sourceLine(0)))
            }
            view.scriptMenu.popup(area, 10, 10)
            if (!entry("menuRedo").enabled) {
                return root.fail("redo is off after an undo")
            }
            view.scriptMenu.close()
            view.redoText()
            view.undoText()

            // Ctrl+B on a selection, then Ctrl+Z.
            selectWord(2, "Wort")
            if (!press(Qt.Key_B, Qt.ControlModifier)) {
                return root.fail("Ctrl+B is not taken")
            }
            if (sourceLine(2) !== "Ein **Wort**" || shown(2) !== "Ein Wort"
                    || view.tools.emphasisOf(2) !== "4-8b") {
                return root.fail("Ctrl+B on a word: " + JSON.stringify(sourceLine(2)) + " " + view.tools.emphasisOf(2))
            }
            if (area.selectedText !== "Wort") {
                return root.fail("the selection stays: " + JSON.stringify(area.selectedText))
            }
            // Again: all of it is bold, so it goes.
            press(Qt.Key_B, Qt.ControlModifier)
            if (sourceLine(2) !== "Ein Wort") {
                return root.fail("Ctrl+B twice: " + JSON.stringify(sourceLine(2)))
            }
            press(Qt.Key_B, Qt.ControlModifier)
            view.undoText()
            if (sourceLine(2) !== "Ein Wort") {
                return root.fail("one step back of Ctrl+B: " + JSON.stringify(sourceLine(2)))
            }
            view.redoText()
            if (sourceLine(2) !== "Ein **Wort**") {
                return root.fail("a step forward of Ctrl+B: " + JSON.stringify(sourceLine(2)))
            }
            // Three presses were three steps.
            view.undoText()
            view.undoText()
            view.undoText()
            if (sourceLine(2) !== "Ein Wort" || view.tools.canUndo) {
                return root.fail("back to the start: " + JSON.stringify(sourceLine(2)))
            }

            // Over lines: all of it italic; the bold word keeps its bold.
            area.select(0, area.text.indexOf("Ein Wort") + 8)
            press(Qt.Key_I, Qt.ControlModifier)
            if (sourceLine(0) !== "*Das ist **fett** und kursiv.*" || sourceLine(2) !== "*Ein Wort*"
                    || sourceLine(1) !== "") {
                return root.fail("Ctrl+I over lines: " + JSON.stringify(doc.read()))
            }
            view.undoText()
            if (doc.read() !== root.book) {
                return root.fail("one step back over lines: " + JSON.stringify(doc.read()))
            }

            // A bold shortcut on the heading changes nothing; italic does.
            selectWord(4, "INT. SAAL")
            press(Qt.Key_B, Qt.ControlModifier)
            if (doc.read() !== root.book) {
                return root.fail("Ctrl+B on a heading: " + JSON.stringify(doc.read()))
            }

            // Without a selection: what is typed next is italic, there only.
            area.cursorPosition = area.text.indexOf("Hallo Welt") + 10
            press(Qt.Key_I, Qt.ControlModifier)
            if (doc.read() !== root.book) {
                return root.fail("Ctrl+I without a selection writes something")
            }
            view.scriptMenu.popup(area, 10, 10)
            if (!entry("menuItalic").checked) {
                return root.fail("the menu does not show the italic to come")
            }
            view.scriptMenu.close()
            type("xy")
            if (sourceLine(6) !== "Hallo Welt*xy*" || view.tools.emphasisOf(6) !== "10-12i") {
                return root.fail("typed after Ctrl+I: " + JSON.stringify(sourceLine(6)) + " " + view.tools.emphasisOf(6))
            }
            type("z")
            if (sourceLine(6) !== "Hallo Welt*xyz*") {
                return root.fail("the next key goes on: " + JSON.stringify(sourceLine(6)))
            }
            // Typing before the closing mark is a step of its own: a few steps take it all back.
            for (let i = 0; i < 5 && sourceLine(6) !== "Hallo Welt"; i++) {
                view.undoText()
            }
            if (sourceLine(6) !== "Hallo Welt") {
                return root.fail("a step back of the typing: " + JSON.stringify(sourceLine(6)))
            }

            // A note comes in as `[[]]`, the cursor between the brackets.
            area.cursorPosition = area.text.indexOf("Hallo Welt") + 5
            view.scriptMenu.popup(area, 10, 10)
            entry("menuNote").triggered()
            view.scriptMenu.close()
            if (sourceLine(6) !== "Hallo[[]] Welt" || doc.cursor !== doc.lineStart(6) + 7) {
                return root.fail("the note: " + JSON.stringify(sourceLine(6)) + " cursor " + doc.cursor)
            }
            view.undoText()
            if (doc.read() !== root.book) {
                return root.fail("a step back of the note: " + JSON.stringify(doc.read()))
            }

            // An emphasis chosen for the next typing is gone once the cursor left its place.
            area.cursorPosition = area.text.indexOf("Hallo Welt") + 10
            press(Qt.Key_I, Qt.ControlModifier)
            area.cursorPosition = area.text.indexOf("Hallo Welt") + 3
            pendingGone.start()
        }
    }

    // The cursor moved: the choice goes after the cursor's change is handled.
    Timer {
        id: pendingGone
        interval: 50
        onTriggered: {
            const area = view.area
            const doc = view.doc

            area.cursorPosition = area.text.indexOf("Hallo Welt") + 10
            type("q")
            if (sourceLine(6) !== "Hallo Weltq") {
                return root.fail("the emphasis stayed after the cursor left: " + JSON.stringify(sourceLine(6)))
            }
            view.undoText()

            // The source view shows every mark; the way back shows the emphasis again.
            area.select(0, 0)
            area.cursorPosition = area.text.indexOf("kursiv") + 2
            const at = doc.cursor
            view.showSource(true)
            if (shown(0) !== "Das ist **fett** und kursiv." || area.text.split("\n").length !== 8) {
                return root.fail("the source view: " + JSON.stringify(shown(0)))
            }
            if (doc.read() !== root.book) {
                return root.fail("showing the source writes something")
            }
            if (doc.cursor !== at) {
                return root.fail("the cursor moved in the source view: " + doc.cursor + " of " + at)
            }
            if (view.tools.emphasisOf(0) !== "") {
                return root.fail("the source view draws emphasis: " + view.tools.emphasisOf(0))
            }

            // Typing writes what is typed: only the line typed in changes.
            area.cursorPosition = area.text.indexOf("fett") + 2
            type("*")
            const lines = root.book.split("\n")
            lines[0] = "Das ist **fe*tt** und kursiv."
            if (doc.read() !== lines.join("\n") || shown(0) !== lines[0]) {
                return root.fail("typing in the source view: " + JSON.stringify(sourceLine(0)))
            }
            view.undoText()
            if (doc.read() !== root.book) {
                return root.fail("undo in the source view: " + JSON.stringify(doc.read()))
            }

            // Ctrl+B writes its marks in sight; without a selection, a pair.
            selectWord(2, "Wort")
            press(Qt.Key_B, Qt.ControlModifier)
            if (sourceLine(2) !== "Ein **Wort**" || shown(2) !== "Ein **Wort**") {
                return root.fail("Ctrl+B in the source view: " + JSON.stringify(sourceLine(2)) + " / " + JSON.stringify(shown(2)))
            }
            view.undoText()
            area.cursorPosition = area.text.indexOf("Hallo Welt") + 5
            press(Qt.Key_U, Qt.ControlModifier)
            if (sourceLine(6) !== "Hallo__ Welt" || doc.cursor !== doc.lineStart(6) + 6) {
                return root.fail("Ctrl+U in the source view: " + JSON.stringify(sourceLine(6)))
            }
            view.undoText()

            // The way back: the emphasis is drawn again, the text is as it was.
            view.showSource(false)
            if (shown(0) !== "Das ist fett und kursiv." || view.tools.emphasisOf(0) !== "8-12b"
                    || doc.read() !== root.book) {
                return root.fail("the way back: " + JSON.stringify(shown(0)) + " " + view.tools.emphasisOf(0))
            }
            Qt.exit(0)
        }
    }
}
