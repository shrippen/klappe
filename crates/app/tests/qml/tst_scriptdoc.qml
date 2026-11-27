import QtQuick
import QtQuick.Controls as QQC2
import QtTest
import "../../qml"
import "ScriptStub.js" as Stub

// The seam of the screenplay editor (D-36): positions and text of the source, one step per
// replacement, a load that is no typing, and the way between the source and the shown text
// when emphasis is hidden (the projection is the controller's; here a stand-in).
TestCase {
    id: tc
    name: "ScriptDoc"
    width: 400
    height: 300
    when: windowShown
    visible: true

    // A history that records the changes it is given, as the native one would take them.
    QtObject {
        id: history
        property var calls: []
        function record(at, removed, put) { calls.push([at, removed, put]) }
    }

    // The source with emphasis: its shown text is `Das ist fett und kursiv`.
    readonly property string marked: "Das ist **fett** und *kursiv*"
    readonly property var markedLine: ({
        text: "Das ist fett und kursiv",
        runs: [{ from: 8, to: 12, bold: true, italic: false, underline: false },
               { from: 17, to: 23, bold: false, italic: true, underline: false }],
        cols: [0, 1, 2, 3, 4, 5, 6, 7, 10, 11, 12, 13, 16, 17, 18, 19, 20, 22, 23, 24, 25, 26, 27, 29]
    })

    // Projects `marked` as above and everything else without emphasis.
    QtObject {
        id: fake
        function scriptShownJson(source) {
            return source === tc.marked ? JSON.stringify([tc.markedLine]) : Stub.shown(source)
        }
        function scriptEditJson(source, change) { return Stub.edit(source, change) }
        function scriptReplaceJson(source, line, from, endLine, to, text) {
            return Stub.replace(source, line, from, endLine, to, text)
        }
    }

    Component {
        id: areaComponent
        QQC2.TextArea {
            width: tc.width
            height: tc.height
            text: "INT. SAAL - TAG\nSie kommt an.\n"
        }
    }

    Component {
        id: docComponent
        ScriptDoc { controller: fake }
    }

    function make(area) {
        const doc = createTemporaryObject(docComponent, tc, { area: area })
        verify(doc)
        doc.load(area.text)
        return doc
    }

    function makeArea() {
        const area = createTemporaryObject(areaComponent, tc)
        verify(area)
        return area
    }

    function init() {
        history.calls = []
    }

    function test_it_reads_the_source() {
        const doc = make(makeArea())

        compare(doc.source, "INT. SAAL - TAG\nSie kommt an.\n")
        compare(doc.read(), doc.source)
    }

    function test_the_cursor_and_the_selection_are_source_positions() {
        const area = makeArea()
        const doc = make(area)

        doc.setCursor(4)
        compare(doc.cursor, 4)
        compare(area.cursorPosition, 4)

        doc.select(5, 9)
        compare(doc.selectionStart, 5)
        compare(doc.selectionEnd, 9)
        compare(doc.selectedText, "SAAL")
    }

    function test_rectOf_follows_the_lines() {
        const doc = make(makeArea())

        verify(doc.rectOf(16).y > doc.rectOf(0).y)
        compare(doc.rectOf(3).y, doc.rectOf(0).y)
    }

    function test_lineStart_is_where_a_line_begins() {
        const doc = make(makeArea())

        // "INT. SAAL - TAG\nSie kommt an.\n": three lines, the last one empty.
        compare([0, 1, 2].map(n => doc.lineStart(n)).join(), "0,16,30")
        compare(doc.lineStart(3), -1)
        compare(doc.lineStart(99), -1)
    }

    function test_lineStarts_gives_minus_one_for_lines_the_text_lacks() {
        const area = makeArea()
        const doc = make(area)

        compare(doc.lineStarts([0, 2, 2, 3, 9]).join(), "0,30,30,-1,-1")
        compare(doc.lineStarts([]).length, 0)

        area.text = ""
        compare(doc.lineStarts([0, 1]).join(), "0,-1")
    }

    function test_replace_is_one_step_of_the_history() {
        const doc = make(makeArea())
        doc.tools = history

        doc.replace(5, 9, "KELLER")

        compare(history.calls.length, 1)
        compare(history.calls[0], [5, "SAAL", "KELLER"])
    }

    function test_replace_without_a_history_edits_the_text() {
        const doc = make(makeArea())

        doc.replace(5, 9, "KELLER")

        compare(doc.read(), "INT. KELLER - TAG\nSie kommt an.\n")
    }

    function test_load_is_no_typing() {
        const area = makeArea()
        const doc = make(area)
        const seen = []
        area.onTextChanged.connect(() => seen.push(doc.loading))

        doc.load("Neu\n")

        compare(doc.read(), "Neu\n")
        compare(seen, [true])
        compare(doc.loading, false)
    }

    function test_the_source_signals_a_change() {
        const doc = make(makeArea())
        const changed = createTemporaryObject(spyComponent, tc, { target: doc, signalName: "sourceChanged" })

        doc.insert(0, "x")

        compare(changed.count, 1)
    }

    function test_typing_in_the_area_changes_the_source() {
        const area = makeArea()
        const doc = make(area)
        doc.tools = history

        area.insert(5, "ABC")

        compare(doc.read(), "INT. ABCSAAL - TAG\nSie kommt an.\n")
        compare(history.calls, [[5, "", "ABC"]])
    }

    function test_a_replacement_is_shown_in_the_area() {
        const area = makeArea()
        const doc = make(area)

        doc.replace(5, 9, "KELLER")

        compare(area.text, "INT. KELLER - TAG\nSie kommt an.\n")
    }

    function test_positions_run_between_the_source_and_the_shown_text() {
        const area = createTemporaryObject(areaComponent, tc, { text: marked })
        const doc = make(area)

        compare(area.text, "Das ist fett und kursiv")
        compare(doc.read(), marked)
        // A position in a hidden mark is the next one that shows.
        compare([7, 8, 9, 10, 14, 15, 16, 28].map(p => doc.toDisplay(p)).join(), "7,8,8,8,12,12,12,23")
        compare([8, 12, 17, 23].map(p => doc.toSource(p)).join(), "10,16,22,29")
        // The end of a selection stands behind its last character, not behind the marks.
        compare([12, 23].map(p => doc.toSourceEnd(p)).join(), "14,28")

        doc.setCursor(22)
        compare(area.cursorPosition, 17)
        compare(doc.cursor, 22)

        doc.select(10, 14)
        compare([area.selectionStart, area.selectionEnd].join(), "8,12")
        compare([doc.selectionStart, doc.selectionEnd].join(), "10,14")
    }

    function test_a_copy_has_the_marks_of_its_emphasis() {
        const area = createTemporaryObject(areaComponent, tc, { text: marked })
        const doc = make(area)

        area.select(8, 12)
        compare(doc.selectedSource(), "**fett**")
        area.select(17, 23)
        compare(doc.selectedSource(), "*kursiv*")
        area.select(0, 7)
        compare(doc.selectedSource(), "Das ist")
        area.select(10, 20)
        // A copy is closed Fountain: the marks of the emphasis at its ends close, so pasting it
        // shows no stray mark.
        compare(doc.selectedSource(), "**tt** und *kur*")
        area.select(5, 5)
        compare(doc.selectedSource(), "")
    }

    function test_a_change_behind_the_cursor_leaves_it() {
        const area = makeArea()
        const doc = make(area)
        doc.load("Hallo Welt\nZweite")

        doc.setCursor(0)
        doc.replace(6, 10, "Erde und Mond")
        compare(doc.cursor, 0)

        doc.setCursor(2)
        doc.replace(6, 19, "Welt")
        compare(doc.cursor, 2)
    }

    function test_a_change_before_the_cursor_moves_it_by_the_difference() {
        const area = makeArea()
        const doc = make(area)
        doc.load("Hallo Welt\nZweite")

        doc.setCursor(13)
        doc.replace(6, 10, "Erde und Mond")

        compare(doc.cursor, 22)
    }

    function test_a_selection_outside_the_change_stays() {
        const area = makeArea()
        const doc = make(area)
        doc.load("Hallo Welt\nZweite")

        doc.select(11, 17)
        doc.replace(6, 10, "Erde und Mond")

        compare(area.selectedText, "Zweite")
        compare([doc.selectionStart, doc.selectionEnd].join(), "20,26")
    }

    function test_the_end_of_a_crlf_line_is_behind_its_return() {
        const area = createTemporaryObject(areaComponent, tc)
        const doc = make(area)
        doc.load("Sie kommt an.\r\nX")

        doc.setCursor(14)
        compare(area.cursorPosition, 13)
        compare(doc.cursor, 14)
        compare(doc.cursor, doc.lineStart(0) + "Sie kommt an.\r".length)
    }

    Component {
        id: spyComponent
        SignalSpy {}
    }
}
