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
        property int layouts: 0
        function paginateScript() { layouts += 1 }
        // The last layout: page 2 starts at the text's line 6 (the second heading); two pages.
        property var pages: ({ pages: 2, breaks: [{ page: 2, line: 6 }],
                               scenes: [{ line: 2, page: 1 }, { line: 6, page: 2 }] })
        function scriptBreaksJson() { return JSON.stringify(pages) }
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
        // The element flow is the real controller's (tests/controller/script_flow.qml).
        property int enters: 0
        function enterStep(line, col, kind, below) {
            enters += 1
            return JSON.stringify({ step: "break" })
        }
        function tabStep(line, kind) { return "" }
        function typedCaps(text, at, pending) { return "" }
        // Search (DB-07) by a plain `indexOf` without case; the rules are `script/`'s, tested there.
        // A speaker keeps the hits below their cue: the last line in capitals above the hit.
        function cueAt(text, at) {
            const lines = text.slice(0, at).split("\n")
            return lines.reverse().find(l => l !== "" && l === l.toUpperCase()) || ""
        }
        // The session's rules, as far as the tests need them.
        function sameFigureName(a, b) { return a.trim().toUpperCase() === b.trim().toUpperCase() }

        function scriptSpeakers(text) {
            return JSON.stringify(speakers)
        }
        property var speakers: []
        function findInScript(text, find, matching, speaker) {
            const hits = []
            const low = text.toLowerCase(), key = find.toLowerCase()
            for (let at = low.indexOf(key); at >= 0; at = low.indexOf(key, at + key.length)) {
                if (speaker === "" || cueAt(text, at) === speaker) {
                    hits.push([at, at + key.length])
                }
            }
            return JSON.stringify(hits)
        }
        function replaceInScript(text, find, matching, speaker, replacement, index) {
            const hits = JSON.parse(findInScript(text, find, matching, speaker))
            const chosen = index < 0 ? hits : hits.slice(index, index + 1)
            if (chosen.length === 0) {
                return ""
            }
            const from = chosen[0][0], to = chosen[chosen.length - 1][1]
            let out = "", at = from
            chosen.forEach(h => { out += text.slice(at, h[0]) + replacement; at = h[1] })
            return JSON.stringify({ from: from, to: to, text: out })
        }
        function scriptLooks() {
            return JSON.stringify({ columns: 60, looks: {
                "h": { kind: "heading", x: 0, width: 60, align: "left" },
                "a": { kind: "action", x: 0, width: 60, align: "left" },
                "c": { kind: "character", x: 22, width: 38, align: "left" },
                "r": { kind: "transition", x: 0, width: 60, align: "right" },
                "n": { kind: "note" }
            }, letters: { blank: " ", heading: "h", action: "a" } })
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
        fake.projectJson = JSON.stringify({ scenes: scenes })
        fake.sceneId = ""
        fake.selected = []
        fake.edits = []
        fake.prints = 0
        fake.enters = 0
        fake.layouts = 0
        fake.pages = { pages: 2, breaks: [{ page: 2, line: 6 }],
                       scenes: [{ line: 2, page: 1 }, { line: 6, page: 2 }] }
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

    // DB-30: the print view stays on a click in the scene list and scrolls to the page of the
    // scene's heading; the title page, if there is one, comes before the counted pages.
    function test_a_scene_in_the_print_view_goes_to_its_page() {
        const view = make()
        fake.pages = { pages: 3, breaks: [{ page: 2, line: 4 }, { page: 3, line: 6 }],
                       scenes: [{ line: 2, page: 1 }, { line: 6, page: 3 }] }
        view.showLayout("print")
        const list = findChild(view, "sceneRows")
        tryVerify(() => findChild(list, "scene.s2") !== null)
        // Title page and three pages of text.
        view.printView.sheet = { fontPt: 12, ascentPt: 10, pages: [0, 1, 2, 3].map(i => ({ width: 595, height: 842, texts: [] })) }
        view.printView.positionViewAtBeginning()

        findChild(list, "scene.s2").clicked()

        compare(view.layout, "print")
        verify(view.printView.visible)
        compare(view.printView.indexAt(10, view.printView.contentY + 10), 3)
        compare(fake.selected[fake.selected.length - 1], "s2")
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

    // Review of PR 280: the Enter of the number pad follows the element flow too.
    function test_the_pad_enter_follows_the_flow() {
        const view = make()
        view.area.forceActiveFocus()
        view.area.cursorPosition = book.indexOf("\n")

        keyClick(Qt.Key_Enter, Qt.KeypadModifier)

        compare(fake.enters, 1)
    }

    // Review of PR 280: Enter over a selection replaces it, as without the flow.
    function test_enter_replaces_a_selection() {
        const view = make()
        view.area.forceActiveFocus()
        view.area.select(2, 7)

        keyClick(Qt.Key_Return)

        compare(fake.enters, 0)
        compare(view.area.text, book.slice(0, 2) + "\n" + book.slice(7))
    }

    // DN-02: the pages and scene lengths are laid out on opening and after the rest without a
    // key, never at each key.
    function test_the_layout_waits_for_a_rest() {
        const view = make()
        compare(fake.layouts, 1, "opening sets the text from outside")
        view.area.forceActiveFocus()
        view.area.cursorPosition = 0

        keyClick("x")
        wait(100)
        keyClick("y")
        compare(fake.layouts, 1)

        tryCompare(fake, "layouts", 2)
        verify(fake.edits.length > 0)
    }

    // Review of PR 287: a text from outside (import, undo in the project) is laid out at once.
    function test_a_text_from_outside_is_laid_out_at_once() {
        const view = make()
        const before = fake.layouts

        fake.scriptText = "INT. KELLER\n\nDunkel.\n"

        compare(fake.layouts, before + 1)
    }

    // DB-07: Ctrl+F opens the search; Enter walks the hits, the shown one selected.
    function test_ctrl_f_finds_in_the_text() {
        const view = make()
        view.area.forceActiveFocus()
        view.area.cursorPosition = 0

        keySequence(StandardKey.Find)
        verify(view.search.visible)
        verify(view.search.findField.activeFocus)
        keyClick("t")
        keyClick("a")
        keyClick("g")

        compare(view.search.hits.length, 1)
        compare(view.area.selectedText, "TAG")
        compare(findChild(view, "hitCount").text, "1 von 1")
    }

    function test_enter_goes_to_the_next_hit_and_wraps() {
        const view = make()
        view.area.cursorPosition = 0
        view.search.open()
        keyClick("e")
        const first = view.area.selectionStart

        keyClick(Qt.Key_Return)
        verify(view.area.selectionStart > first)
        keyClick(Qt.Key_Return, Qt.ShiftModifier)
        compare(view.area.selectionStart, first)
        keyClick(Qt.Key_Return, Qt.ShiftModifier)
        compare(view.search.current, view.search.hits.length - 1)
    }

    function test_replace_one_then_all_and_close() {
        const view = make()
        view.area.cursorPosition = 0
        view.search.open()
        view.search.findField.text = "e"
        view.search.from(0)
        view.search.replaceField.text = "E"

        view.search.replaceOne()
        compare(view.area.text, book.replace("Sie", "SiE"))
        compare(view.area.selectedText.toLowerCase(), "e")
        verify(view.area.selectionStart > book.indexOf("Sie"))

        view.search.replaceAll()
        compare(view.area.text, book.replace(/e/g, "E"))
        // Without case the replacement still matches: the count, no "0 of".
        compare(findChild(view, "hitCount").text, view.search.hits.length + " Treffer")

        view.search.findField.forceActiveFocus()
        keyClick(Qt.Key_Escape)
        verify(!view.search.visible)
        verify(view.area.activeFocus)
    }

    // Review of PR 288: typing between two steps moves the hit; Ersetzen takes the marked one.
    function test_replace_takes_the_marked_hit_after_typing() {
        const view = make()
        fake.scriptText = "Anna sieht anna."
        view.area.cursorPosition = 0
        view.search.open()
        view.search.findField.text = "anna"
        view.search.from(0)
        view.search.next()
        compare(view.area.selectionStart, 11)

        view.area.insert(0, "anna ")
        compare(view.area.selectionStart, 16)
        view.search.replaceField.text = "Eva"
        view.search.replaceOne()

        compare(view.area.text, "anna Anna sieht Eva.")
    }

    // Review of PR 288: Ctrl+F works anywhere in the view, not only in the text.
    function test_ctrl_f_works_outside_the_text() {
        const view = make()
        const list = findChild(view, "sceneList")
        list.forceActiveFocus()

        keySequence(StandardKey.Find)

        verify(view.search.visible)
    }

    // Placement C: the search floats over the text; opening it moves nothing.
    function test_the_search_floats_over_the_text() {
        const view = make()
        const switchBar = findChild(view, "layoutSwitch")
        const before = switchBar.mapToItem(view, 0, 0)

        view.search.open()

        const after = switchBar.mapToItem(view, 0, 0)
        compare(after.x, before.x)
        compare(after.y, before.y)
        verify(view.search.height > 0)
        verify(view.search.z > 0)
        verify(view.search.width >= view.area.width * 0.5)
        // At the top right of the text, clear of the layout switch above.
        const box = view.search.mapToItem(view, 0, 0)
        verify(box.y >= before.y + switchBar.height)
        verify(box.x + view.search.width <= view.width)
        verify(box.x > view.width / 3)
    }

    /// The hit shown, as a rectangle in the view's coordinates.
    function hitBox(view) {
        const a = view.area.positionToRectangle(view.area.selectionStart)
        const b = view.area.positionToRectangle(view.area.selectionEnd)
        const p = view.area.mapToItem(view, a.x, a.y)
        return Qt.rect(p.x, p.y, Math.max(b.x - a.x, 1), a.height)
    }

    /// The first line of the selection; a selection that wraps fills it to the text's right edge.
    function firstLineBox(view) {
        const a = view.area.positionToRectangle(view.area.selectionStart)
        const b = view.area.positionToRectangle(view.area.selectionEnd)
        const right = b.y > a.y ? view.area.width : b.x
        const p = view.area.mapToItem(view, a.x, a.y)
        return Qt.rect(p.x, p.y, Math.max(right - a.x, 1), a.height)
    }

    function overlaps(a, b) {
        return a.x < b.x + b.width && b.x < a.x + a.width && a.y < b.y + b.height && b.y < a.y + a.height
    }

    function cardBox(view) {
        const p = view.search.mapToItem(view, 0, 0)
        return Qt.rect(p.x, p.y, view.search.width, view.search.height)
    }

    // A hit in the top lines of the view, right of the card's left edge, must not sit under it.
    function test_the_card_moves_off_a_hit_under_it() {
        fake.scriptText = "INT. SAAL - TAG\n\nSie kommt an und sieht sich lange im Saal um, dann ZIEL.\n\nEs regnet.\n"
        const view = make()
        view.search.open()
        view.search.findField.text = "ziel"
        view.search.from(0)

        compare(view.area.selectedText, "ZIEL")
        verify(!overlaps(hitBox(view), cardBox(view)), "hit " + JSON.stringify(hitBox(view)) + " under card " + JSON.stringify(cardBox(view)))
        // At the bottom right, same right margin.
        const box = cardBox(view)
        verify(box.y > view.height / 2, "card at y " + box.y)
    }

    // A hit that starts left of the card and wraps lies under it with the rest of its first line.
    function test_the_card_moves_off_a_wrapped_hit() {
        let words = []
        for (let i = 0; i < 30; i++) {
            words.push("wort" + i)
        }
        fake.scriptText = "INT. SAAL - TAG\n\n" + words.join(" ") + " \n\nEs regnet.\n"
        const view = make()
        view.search.open()
        view.showText(30, 115)

        verify(view.area.positionToRectangle(115).y > view.area.positionToRectangle(30).y, "hit wraps")
        verify(!overlaps(firstLineBox(view), cardBox(view)), "hit " + JSON.stringify(firstLineBox(view)) + " under card " + JSON.stringify(cardBox(view)))
    }

    // A later hit under the card at the bottom sends it back to the top.
    function test_the_card_goes_back_up_for_a_hit_at_the_bottom() {
        let text = "INT. SAAL - TAG\n\nSie kommt an und sieht sich lange im Saal um, dann ZIEL.\n\n"
        for (let i = 0; i < 40; i++) {
            text += "Es regnet.\n\n"
        }
        text += "Am Ende " + "x".repeat(60) + " ZIEL.\n"
        fake.scriptText = text
        const view = make()
        view.search.open()
        view.search.findField.text = "ziel"
        view.search.from(0)
        verify(cardBox(view).y > view.height / 2)

        view.search.next()
        verify(!overlaps(hitBox(view), cardBox(view)), "hit " + JSON.stringify(hitBox(view)) + " under card " + JSON.stringify(cardBox(view)))
        verify(cardBox(view).y < view.height / 2, "card at y " + cardBox(view).y)
    }

    // A narrow text area: the card stays inside it, not over the scene list.
    function test_the_card_stays_in_a_narrow_text_area() {
        const view = make()
        view.width = 500
        view.search.open()

        const box = cardBox(view)
        const area = view.area.mapToItem(view, 0, 0)
        verify(box.x >= area.x, "card at " + box.x + ", text from " + area.x)
        verify(box.x + box.width <= area.x + view.area.width, "card to " + (box.x + box.width) + ", text to " + (area.x + view.area.width))
    }

    // "Nur Dialog von": the cues of the text, most frequent first, after "Alle Figuren"; the
    // choice limits the hits, and a choice that is no longer a cue falls back to everyone.
    function test_the_speaker_limits_the_hits() {
        const view = make()
        fake.speakers = ["ANNA", "EVA"]
        fake.scriptText = "ANNA\nHallo Anna.\n\nEVA\nHallo Eva.\n"
        view.search.open()
        const box = findChild(view, "speakerBox")
        compare(box.count, 3)
        compare(box.textAt(0), "Alle Figuren")
        compare(box.textAt(1), "ANNA")

        view.search.findField.text = "hallo"
        view.search.from(0)
        compare(view.search.hits.length, 2)

        box.currentIndex = 2
        box.activated(2)
        compare(view.search.speaker, "EVA")
        compare(view.search.hits.length, 1)
        compare(view.area.text.slice(view.area.selectionStart, view.area.selectionEnd), "Hallo")
        verify(view.area.selectionStart > view.area.text.indexOf("EVA"))

        fake.speakers = ["ANNA"]
        view.search.open()
        compare(box.count, 2)
        compare(view.search.speaker, "")
        fake.speakers = []
    }

    // DB-30: a thin line with the page number above the first line of each page after the first,
    // placed from the layout's lines; it moves no text.
    function test_a_page_break_stands_above_its_line() {
        const view = make()
        const mark = findChild(view.area, "pageBreak.2")
        verify(mark !== null)

        const at = book.indexOf("EXT. HOF")
        compare(view.marks[0].at, at)
        fuzzyCompare(mark.y + mark.height, view.area.positionToRectangle(at).y, 1)
        compare(mark.width, view.area.width - view.area.leftPadding - view.area.rightPadding)
    }

    // A line the text does not have (it got shorter since) gives no position.
    function test_a_break_beyond_the_text_has_no_place() {
        const view = make()

        compare(view.breakMarks([{ page: 2, line: 99 }], "a\nb")[0].at, -1)
        compare(view.breakMarks([{ page: 2, line: 1 }, { page: 3, line: 2 }], "a\nb\nc").map(m => m.at).join(), "2,4")
        compare(view.lineStart("a\nb", 0), 0)
    }

    // DN-02: typing makes the lines and lengths stale until the layout is made anew; the note
    // "Seiten veraltet" stands beside the switch for that time.
    function test_typing_makes_the_pages_stale_until_laid_out() {
        const view = make()
        const note = findChild(view, "staleNote")
        const mark = findChild(view.area, "pageBreak.2")
        const before = fake.layouts
        verify(!view.stale)
        verify(!note.visible)
        compare(mark.opacity, 1)
        compare(findChild(view, "sceneRows").parent.lengthsStale, false)

        view.area.forceActiveFocus()
        view.area.insert(view.area.length, "x")

        verify(view.stale)
        verify(note.visible)
        compare(note.text, "Seiten veraltet")
        verify(mark.opacity < 1)
        compare(findChild(view, "sceneRows").parent.lengthsStale, true)

        tryVerify(() => !view.stale)
        verify(fake.layouts > before)
        verify(!note.visible)
        compare(findChild(view.area, "pageBreak.2").opacity, 1)
    }

    // Another project with the same text, or another paper: the breaks are read anew.
    function test_a_project_change_reads_the_breaks_again() {
        const view = make()
        compare(view.pageMap.pages, 2)

        fake.pages = { pages: 3, breaks: [{ page: 2, line: 4 }, { page: 3, line: 6 }], scenes: [] }
        fake.projectJson = JSON.stringify({ scenes: tc.scenes, paper: "letter" })

        compare(view.pageMap.pages, 3)
        verify(!view.stale)
    }
}
