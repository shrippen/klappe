import QtQuick
import QtTest
import "../../qml"

// ScreenplayView, the pages (DB-30, DN-02): the lines at the page breaks of the last layout, the
// stale state until the next layout, the jump to a scene's page in the print view.
TestCase {
    id: tc
    name: "ScreenplayPages"
    width: 1000
    height: 600
    when: windowShown
    visible: true

    readonly property string book: "# Akt I\n\nINT. SAAL - TAG\n\nSie kommt an.\n\nEXT. HOF - NACHT\n\nEs regnet.\n"
    readonly property var scenes: [
        { id: "s1", number: "1", heading: "INT. SAAL - TAG", inBook: true, plans: 1, shots: 3, line: 2 },
        { id: "s2", number: "2", heading: "EXT. HOF - NACHT", inBook: true, plans: 0, shots: 0, line: 6 }
    ]
    // Page 2 starts at the text's line 6 (the second heading); two pages.
    readonly property var twoPages: ({ pages: 2, breaks: [{ page: 2, line: 6 }],
                                       scenes: [{ line: 2, page: 1 }, { line: 6, page: 2 }] })

    QtObject {
        id: fake
        property string scriptText: tc.book
        property string sceneId: ""
        property string projectJson: JSON.stringify({ scenes: tc.scenes })
        property var selected: []
        property int layouts: 0
        property var pages: tc.twoPages
        function selectScene(id) {
            selected.push(id)
            sceneId = id
            return true
        }
        function editScript(text) { scriptText = text }
        function paginateScript() { layouts += 1 }
        function scriptBreaksJson() { return JSON.stringify(pages) }
        function versionsJson() { return JSON.stringify({ versions: [], next: null }) }
        function revisionJson() { return "[]" }
        function scriptKinds(text) {
            return text.split("\n").map(l => l.startsWith("INT.") || l.startsWith("EXT.") ? "h" : "a").join("")
        }
        function scriptPrintJson() {
            return JSON.stringify({ fontPt: 12, ascentPt: 10, pages: [
                { width: 595, height: 842, texts: [{ x: 108, y: 82, bold: true, text: "INT. SAAL - TAG" }] }
            ] })
        }
        function enterStep(line, col, kind, below) { return JSON.stringify({ step: "break" }) }
        function tabStep(line, kind) { return "" }
        function typedCaps(text, at, pending) { return "" }
        function sameFigureName(a, b) { return a === b }
        function scriptSpeakers(text) { return "[]" }
        function findInScript(text, find, matching, speaker) { return "[]" }
        function scriptLooks() {
            return JSON.stringify({ columns: 60, looks: {
                "h": { kind: "heading", x: 0, width: 60, align: "left" },
                "a": { kind: "action", x: 0, width: 60, align: "left" }
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
        fake.layouts = 0
        fake.pages = twoPages
    }

    function make() {
        const view = createTemporaryObject(viewComponent, tc)
        verify(view)
        return view
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
