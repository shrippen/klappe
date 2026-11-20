import QtQuick
import QtTest
import "../../qml"

// ScreenplayView (E-09, D-15): the scene list shows the project's scenes (D-13), a click shows
// the scene and moves the cursor to its heading when the text holds it, typing reaches the controller after the pause, and a change from outside (an
// import) shows in the text. Each line gets the look of its kind, in the page's columns scaled
// to the text's width.
TestCase {
    id: tc
    name: "ScreenplayView"
    width: 1000
    height: 600
    when: windowShown
    visible: true

    readonly property string book: "# Akt I\n\nINT. SAAL - TAG\n\nSie kommt an.\n\nEXT. HOF - NACHT\n\nEs regnet.\n"

    // The project's scenes: two are in the text (`line` is their heading's), one is not.
    readonly property var scenes: [
        { id: "s1", number: "1", heading: "INT. SAAL - TAG", inBook: true, plans: 1, shots: 3, line: 2 },
        { id: "s2", number: "2", heading: "EXT. HOF - NACHT", inBook: true, plans: 0, shots: 0, line: 6 },
        { id: "s3", number: "3", heading: "INT. KELLER", inBook: false, plans: 0, shots: 0, line: null }
    ]

    QtObject {
        id: fake
        property string scriptText: tc.book
        property string sceneId: ""
        property string projectJson: JSON.stringify({ scenes: tc.scenes })
        property var selected: []
        property var edits: []
        function selectScene(id) {
            selected.push(id)
            sceneId = id
            return true
        }
        function editScript(text) {
            edits.push(text)
            scriptText = text
        }
        // A heading's line is "h", every other line "a": enough to see the kinds arrive.
        function scriptKinds(text) {
            return text.split("\n").map(l => l.startsWith("INT.") || l.startsWith("EXT.") ? "h" : "a").join("")
        }
        property int prints: 0
        property string printLine: "INT. SAAL - TAG"
        // One page: a heading in bold where the PDF puts it.
        function scriptPrintJson() {
            prints += 1
            return JSON.stringify({ fontPt: 12, ascentPt: 10, pages: [
                { width: 595, height: 842, texts: [{ x: 108, y: 82, bold: true, text: printLine }] }
            ] })
        }
        function scriptLooks() {
            return JSON.stringify({ columns: 60, looks: {
                "h": { kind: "heading", x: 0, width: 60, align: "left" },
                "a": { kind: "action", x: 0, width: 60, align: "left" },
                "c": { kind: "character", x: 22, width: 38, align: "left" },
                "r": { kind: "transition", x: 0, width: 60, align: "right" },
                "n": { kind: "note" }
            } })
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
            toolsSource: Qt.resolvedUrl("StubTools.qml")
        }
    }

    function init() {
        fake.scriptText = book
        fake.projectJson = JSON.stringify({ scenes: scenes })
        fake.sceneId = ""
        fake.selected = []
        fake.edits = []
        fake.prints = 0
        fake.printLine = "INT. SAAL - TAG"
    }

    function make() {
        const view = createTemporaryObject(viewComponent, tc)
        verify(view)
        return view
    }

    function test_the_text_is_in_the_editor() {
        const view = make()

        compare(view.area.text, book)
    }

    function test_the_list_shows_the_projects_scenes() {
        const view = make()
        const list = findChild(view, "sceneRows")

        compare(list.count, 3)
        tryVerify(() => findChild(list, "scene.s1") !== null)
        const first = findChild(list, "scene.s1")
        compare(first.leadingText, "1")
        compare(first.text, "INT. SAAL - TAG")
        // A scene that is not in the text is listed too.
        tryVerify(() => findChild(list, "scene.s3") !== null)
        compare(findChild(list, "scene.s3").subtitle, "nicht im Buch")
    }

    function test_a_click_shows_the_scene_and_moves_the_cursor_to_the_heading() {
        const view = make()
        const list = findChild(view, "sceneRows")
        tryVerify(() => findChild(list, "scene.s2") !== null)

        findChild(list, "scene.s2").clicked()

        compare(fake.selected, ["s2"])
        compare(view.area.cursorPosition, book.indexOf("EXT. HOF"))
        verify(view.area.activeFocus)
    }

    function test_a_scene_not_in_the_text_is_only_shown() {
        const view = make()
        const list = findChild(view, "sceneRows")
        view.area.cursorPosition = 3
        tryVerify(() => findChild(list, "scene.s3") !== null)

        findChild(list, "scene.s3").clicked()

        compare(fake.selected, ["s3"])
        compare(view.area.cursorPosition, 3)
    }

    // An imported book has no numbers: every scene is "", and each still goes to its own heading.
    function test_unnumbered_scenes_go_to_their_own_heading() {
        const rows = [
            { id: "s1", number: "", heading: "INT. SAAL - TAG", inBook: true, plans: 0, shots: 0, line: 2 },
            { id: "s2", number: "", heading: "EXT. HOF - NACHT", inBook: true, plans: 0, shots: 0, line: 6 }
        ]
        fake.projectJson = JSON.stringify({ scenes: rows })
        const view = make()
        const list = findChild(view, "sceneRows")
        tryVerify(() => findChild(list, "scene.s2") !== null)

        findChild(list, "scene.s2").clicked()

        compare(fake.selected, ["s2"])
        compare(view.area.cursorPosition, book.indexOf("EXT. HOF"))
    }

    function test_the_editor_is_monospace_and_wraps() {
        const view = make()

        compare(view.area.wrapMode, TextEdit.Wrap)
        verify(view.area.font.family !== "")
    }

    function test_typing_reaches_the_controller_after_the_pause() {
        const view = make()
        view.area.forceActiveFocus()
        view.area.cursorPosition = view.area.length

        view.area.insert(view.area.length, "Ende.")

        compare(fake.edits.length, 0)
        tryCompare(fake.edits, "length", 1)
        compare(fake.edits[0], book + "Ende.")
    }

    function test_one_pause_sends_once() {
        const view = make()

        view.area.insert(0, "a")
        view.area.insert(1, "b")
        view.area.insert(2, "c")

        tryCompare(fake.edits, "length", 1)
        wait(150)
        compare(fake.edits.length, 1)
    }

    function test_flush_sends_at_once() {
        const view = make()

        view.area.insert(0, "x")
        view.flush()

        compare(fake.edits.length, 1)
        compare(fake.edits[0], "x" + book)
        wait(150)
        compare(fake.edits.length, 1)
    }

    function test_a_loaded_text_is_not_sent_back() {
        const view = make()

        fake.scriptText = "INT. KELLER - NACHT\n"
        compare(view.area.text, "INT. KELLER - NACHT\n")

        wait(150)
        compare(fake.edits.length, 0)
    }

    function test_without_scenes_the_list_says_so() {
        fake.projectJson = JSON.stringify({ scenes: [] })
        fake.scriptText = ""
        const view = make()

        compare(findChild(view, "sceneRows").count, 0)
    }

    // The text area gives back other text than it got (CRLF becomes LF, a no-break space a
    // space): that is no typing, and must not make the project unsaved.
    function test_loading_never_sends() {
        fake.scriptText = "INT. HALLE - TAG\r\n\r\nSie\u00a0kommt an.\r\n"
        const view = make()

        wait(200)
        compare(fake.edits.length, 0)

        fake.scriptText = "EXT. HOF\r\n\u00a0\r\n"
        wait(200)
        compare(fake.edits.length, 0)
    }

    function test_typing_after_a_load_is_sent() {
        fake.scriptText = "INT. HALLE - TAG\r\n"
        const view = make()
        wait(100)

        view.area.insert(0, "x")

        tryCompare(fake.edits, "length", 1)
    }

    // D-17: no text says so and offers the next step.
    function test_an_empty_screenplay_offers_to_load_or_write() {
        fake.scriptText = ""
        const view = make()
        const empty = findChild(view, "emptyBook")
        const imports = createTemporaryObject(spyComponent, tc, { target: view })

        verify(empty.visible)
        compare(empty.title, "Noch kein Drehbuch.")
        findChild(view, "loadScreenplay").clicked()
        compare(imports.count, 1)

        findChild(view, "writeScreenplay").clicked()
        verify(view.area.activeFocus)
        verify(!empty.visible)
    }

    function test_a_screenplay_with_text_shows_no_offer() {
        const view = make()

        verify(!findChild(view, "emptyBook").visible)
    }

    Component {
        id: spyComponent
        SignalSpy { signalName: "importRequested" }
    }

    // The list's fillWidth made its column fill the row and left the text 4 px wide (seen in the
    // app at 1440 px with the example).
    function test_the_text_keeps_its_width_beside_the_list() {
        const view = make()
        const list = findChild(view, "sceneList")
        const area = findChild(view, "scriptArea")

        tryVerify(() => area.width > view.width / 3, 1000, "text is " + area.width + " px wide")
        verify(list.width < view.width / 2, "scene list is " + list.width + " px wide")
    }

    // D-15: the page's columns are scaled from 60 to the text's columns, the same for every line.
    function test_a_cue_is_indented_as_on_the_page() {
        const view = make()
        tryVerify(() => view.tools !== null)
        const unit = view.lineLooks.c.left / 22

        verify(unit > 0)
        compare(view.lineLooks.c.right, 0)
        compare(view.lineLooks.a.left, 0)
        compare(view.lineLooks.r.align, "right")
        verify(view.lineLooks.h.bold)
        verify(view.lineLooks.n.muted)
        // 60 page columns are `columns` characters of the editor's font.
        const metrics = createTemporaryQmlObject("import QtQuick; FontMetrics {}", tc)
        metrics.font = view.area.font
        fuzzyCompare(unit * 60, view.columns * metrics.averageCharacterWidth, 0.01)
    }

    function test_a_loaded_text_gets_its_kinds_anew() {
        const view = make()
        tryVerify(() => view.tools !== null)
        const forgets = view.tools.forgets

        fake.scriptText = "INT. KELLER - NACHT\n\nDunkel.\n"

        compare(view.tools.kinds, "haaa")
        compare(view.tools.forgets, forgets + 1)
    }

    // The kinds follow every keystroke at once, not only the pause before the session.
    function test_typing_sets_the_kinds_at_once() {
        const view = make()
        tryVerify(() => view.tools !== null)
        const forgets = view.tools.forgets

        view.area.insert(view.area.length, "INT. KELLER")

        compare(view.tools.kinds, fake.scriptKinds(view.area.text))
        compare(view.tools.forgets, forgets)
        compare(fake.edits.length, 0)
    }

    // Ctrl+Z in the text is the text's own step back, not the project's.
    function test_ctrl_z_in_the_text_takes_back_typing() {
        const view = make()
        tryVerify(() => view.tools !== null)
        view.area.forceActiveFocus()

        keySequence(StandardKey.Undo)

        compare(view.tools.undos, 1)
    }

    // D-15: the switch shows the pages as they print, laid out from what was typed.
    function test_the_print_view_shows_the_pages() {
        const view = make()
        view.area.insert(0, "x")

        findChild(view, "layoutSwitch").activated(1)

        compare(view.layout, "print")
        compare(fake.edits.length, 1)
        compare(fake.prints, 1)
        verify(view.printView.visible)
        verify(!view.area.visible)
        tryVerify(() => findChild(view.printView, "page.0") !== null)
        const page = findChild(view.printView, "page.0")
        fuzzyCompare(page.height / page.width, 842 / 595, 0.01)
        verify(page.width <= view.printView.width)
    }

    // A scene picked in the list goes back to the text at its heading.
    function test_a_scene_from_the_print_view_goes_to_the_text() {
        const view = make()
        view.showLayout("print")
        const list = findChild(view, "sceneRows")
        tryVerify(() => findChild(list, "scene.s2") !== null)

        findChild(list, "scene.s2").clicked()

        compare(view.layout, "flow")
        verify(!view.printView.visible)
        compare(view.area.cursorPosition, book.indexOf("EXT. HOF"))
    }

    // A change from outside while the pages show lays them out again.
    function test_the_pages_follow_the_text() {
        const view = make()
        view.showLayout("print")

        fake.scriptText = "INT. KELLER - NACHT\n"

        compare(fake.prints, 2)
    }

    // Review of PR 279: the print view is for reading. Picked with the mouse while writing, a
    // key must not change the hidden text.
    function test_keys_in_the_print_view_leave_the_text() {
        const view = make()
        view.area.forceActiveFocus()
        const switcher = findChild(view, "layoutSwitch")

        mouseClick(switcher, switcher.width * 3 / 4, switcher.height / 2)
        keyClick(Qt.Key_Q)

        compare(view.layout, "print")
        verify(!view.area.activeFocus)
        compare(view.area.text, book)
    }

    // "Neu schreiben" in the print view goes to the text it writes in.
    function test_writing_shows_the_text() {
        fake.scriptText = ""
        const view = make()
        view.showLayout("print")

        findChild(view, "writeScreenplay").clicked()

        compare(view.layout, "flow")
        verify(view.area.activeFocus)
    }

    // Review of PR 279: 60 columns are as wide as 60 columns of the page at any size (the size
    // of type is whole pixels; the page is scaled as a whole).
    function test_a_line_is_as_wide_as_on_paper() {
        fake.printLine = "x".repeat(60)
        const view = make()
        view.width = 700
        view.showLayout("print")
        tryVerify(() => findChild(view.printView, "page.0") !== null)
        const page = findChild(view.printView, "page.0")
        const line = findChild(page, "line.0")

        const shown = line.mapToItem(page, line.contentWidth, 0).x - line.mapToItem(page, 0, 0).x
        verify(view.printView.zoom < 1)
        fuzzyCompare(shown, 60 * 7.2 * view.printView.zoom, 60 * 7.2 * view.printView.zoom * 0.02)
    }
}
