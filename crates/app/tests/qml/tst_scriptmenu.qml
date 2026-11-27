import QtQuick
import QtTest
import "../../qml"
import "ScriptStub.js" as Stub

// ScreenplayView, the keys and the menu of the text (D-36, D-37): Ctrl+B, Ctrl+I and Ctrl+U
// toggle the emphasis of the selection or of what is typed next; the menu is the editor's own,
// with the entries and states of the selection; the source switch shows the text as it is.
// The rules of the emphasis are the real controller's (tests/controller/script_menu.qml); the
// stand-in here only records what the view asks for.
TestCase {
    id: tc
    name: "ScriptMenu"
    width: 1000
    height: 600
    when: windowShown
    visible: true

    readonly property string book: "# Akt I\n\nINT. SAAL - TAG\n\nSie kommt an.\n\nEXT. HOF - NACHT\n\nEs regnet.\n"

    QtObject {
        id: fake
        property string scriptText: tc.book
        property string sceneId: ""
        property string projectJson: JSON.stringify({ scenes: [] })
        property var changes: []
        property var toggles: []
        property var emphasis: ({ bold: false, italic: false, underline: false, heading: false })
        function editScript(text) { scriptText = text }
        function paginateScript() {}
        function scriptBreaksJson() { return JSON.stringify({ pages: 0, breaks: [], scenes: [] }) }
        function scriptOutlineJson() { return "[]" }
        function scriptShownJson(source, view) { return Stub.shown(source) }
        function scriptEditJson(source, change, view) {
            changes.push(JSON.parse(change))
            return Stub.edit(source, change)
        }
        function scriptReplaceJson(source, line, from, endLine, to, text, view) {
            return Stub.replace(source, line, from, endLine, to, text)
        }
        // A toggle changes nothing here; the call is what counts.
        function scriptToggleJson(source, line, from, endLine, to, mark, view) {
            toggles.push({ line: line, from: from, endLine: endLine, to: to, mark: mark, view: view })
            return Stub.replace(source, line, from, line, from, "")
        }
        function scriptEmphasisJson(source, line, from, endLine, to) {
            return JSON.stringify(emphasis)
        }
        function versionsJson() { return JSON.stringify({ versions: [], next: null }) }
        function revisionJson() { return "[]" }
        function scriptKinds(text) { return text.split("\n").map(l => "a").join("") }
        function enterStep(line, col, kind, below) { return JSON.stringify({ step: "break" }) }
        function tabStep(line, kind) { return "" }
        function typedCaps(text, at, pending) { return "" }
        function titleFields(text) { return JSON.stringify({ "Title": "", "Credit": "", "Author": "", "Source": "", "Draft date": "", "Contact": "" }) }
        function scriptSuggest(text, line, kind, pending) { return "" }
        function sameFigureName(a, b) { return a === b }
        function scriptSpeakers(text) { return "[]" }
        function findInScript(text, find, matching, speaker) { return "[]" }
        function scriptLooks() {
            return JSON.stringify({ columns: 60, looks: {
                "a": { kind: "action", x: 0, width: 60, align: "left" }
            }, letters: { blank: " ", heading: "a", action: "a" } })
        }
    }

    TestTexts { id: de; language: "de" }

    Component {
        id: viewComponent
        ScreenplayView {
            texts: de
            controller: fake
            width: tc.width
            height: tc.height
            debounceMs: 50
            layoutRestMs: 200
            toolsSource: Qt.resolvedUrl("StubTools.qml")
        }
    }

    function init() {
        fake.scriptText = book
        fake.changes = []
        fake.toggles = []
        fake.emphasis = { bold: false, italic: false, underline: false, heading: false }
    }

    function make() {
        const view = createTemporaryObject(viewComponent, tc)
        verify(view)
        return view
    }

    // D-36, D-37: Ctrl+B, Ctrl+I and Ctrl+U in the text toggle the emphasis of the selection, as
    // the source positions of its ends say.
    function test_the_emphasis_keys_toggle_the_selection() {
        const view = make()
        view.area.forceActiveFocus()
        const from = book.indexOf("Sie kommt")
        view.area.select(from, from + 3)

        keyClick(Qt.Key_B, Qt.ControlModifier)
        keyClick(Qt.Key_I, Qt.ControlModifier)
        keyClick(Qt.Key_U, Qt.ControlModifier)

        compare(fake.toggles.map(t => t.mark), ["bold", "italic", "underline"])
        compare(fake.toggles[0].line, 4)
        compare(fake.toggles[0].from, 0)
        compare(fake.toggles[0].endLine, 4)
        compare(fake.toggles[0].to, 3)
        compare(fake.toggles[0].view, "shown")
        compare(view.area.text, book)
    }

    // Without a selection a key sets the emphasis of what is typed next, at that place only.
    function test_an_emphasis_key_without_a_selection_is_for_the_next_typing() {
        const view = make()
        view.area.forceActiveFocus()
        view.area.cursorPosition = book.indexOf("Sie kommt") + 3

        keyClick(Qt.Key_I, Qt.ControlModifier)
        compare(fake.toggles.length, 0)
        view.area.insert(view.area.cursorPosition, "x")

        compare(fake.changes.length, 1)
        compare(fake.changes[0].typed, "x")
        compare(fake.changes[0].runs.length, 1)
        verify(fake.changes[0].runs[0].italic)
        verify(!fake.changes[0].runs[0].bold)

        // The choice was for one typing: the next character takes after the one before.
        view.area.insert(view.area.cursorPosition + 1, "y")
        compare(fake.changes[1].runs.length, 0)
    }

    function test_the_choice_of_a_key_goes_when_the_cursor_leaves() {
        const view = make()
        view.area.forceActiveFocus()
        view.area.cursorPosition = 5

        keyClick(Qt.Key_B, Qt.ControlModifier)
        view.area.cursorPosition = 20
        wait(50)
        view.area.cursorPosition = 5
        view.area.insert(5, "x")

        compare(fake.changes.length, 1)
        compare(fake.changes[0].runs.length, 0)
    }

    // The menu of the text is the editor's own, in German, with the keys beside the entries.
    function test_the_menu_of_the_text_has_its_entries() {
        const view = make()
        const menu = view.scriptMenu
        const wanted = { menuCut: "Ausschneiden", menuCopy: "Kopieren", menuPaste: "Einfügen",
                         menuUndo: "Rückgängig", menuRedo: "Wiederholen", menuBold: "Fett",
                         menuItalic: "Kursiv", menuUnderline: "Unterstrichen",
                         menuNote: "Notiz an dieser Stelle" }
        const found = {}
        for (let i = 0; i < menu.count; i++) {
            found[menu.itemAt(i).objectName] = menu.itemAt(i)
        }

        for (const name in wanted) {
            verify(found[name], name)
            compare(found[name].text, wanted[name])
        }
        compare(found.menuBold.shownShortcut, "Strg+B")
        compare(found.menuRedo.shownShortcut, "Strg+Umschalt+Z")
        verify(found.menuBold.checkable)
    }

    function test_the_menu_knows_what_it_can_do() {
        const view = make()
        const menu = view.scriptMenu
        const item = name => {
            for (let i = 0; i < menu.count; i++) {
                if (menu.itemAt(i).objectName === name) {
                    return menu.itemAt(i)
                }
            }
            return null
        }
        view.area.cursorPosition = 3
        menu.popup(view.area, 10, 10)
        verify(!item("menuCut").enabled)
        verify(!item("menuCopy").enabled)
        verify(!item("menuUndo").enabled)
        verify(!item("menuBold").checked)
        menu.close()

        // A selection takes cut and copy; the emphasis the controller names is ticked, a heading's
        // bold is grey.
        view.area.select(3, 8)
        fake.emphasis = { bold: true, italic: false, underline: true, heading: true }
        menu.popup(view.area, 10, 10)
        verify(item("menuCut").enabled)
        verify(item("menuCopy").enabled)
        verify(item("menuBold").checked)
        verify(!item("menuBold").enabled)
        verify(!item("menuItalic").checked)
        verify(item("menuUnderline").checked)
        menu.close()
    }

    function test_the_menu_key_opens_the_menu_at_the_cursor() {
        const view = make()
        view.area.forceActiveFocus()
        view.area.cursorPosition = 3

        let opened = 0
        view.scriptMenu.aboutToShow.connect(() => opened++)

        verify(view.scriptMenu.key({ key: Qt.Key_Menu, modifiers: Qt.NoModifier }))
        compare(opened, 1)
        view.scriptMenu.close()
        verify(view.scriptMenu.key({ key: Qt.Key_F10, modifiers: Qt.ShiftModifier }))
        compare(opened, 2)
        view.scriptMenu.close()
        verify(!view.scriptMenu.key({ key: Qt.Key_F10, modifiers: Qt.NoModifier }))
    }

    function test_the_source_switch_shows_the_text_as_it_is() {
        const view = make()
        verify(!view.sourceShown)

        view.showSource(true)
        verify(view.sourceShown)
        compare(view.area.text, book)

        view.showSource(false)
        verify(!view.sourceShown)
        compare(view.area.text, book)
    }
}
