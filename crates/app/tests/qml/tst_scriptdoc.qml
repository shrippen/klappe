import QtQuick
import QtQuick.Controls as QQC2
import QtTest
import "../../qml"

// The seam of the screenplay editor (D-36): positions and text of the source, one step per
// replacement, a load that is no typing. Today the source is what the text area holds.
TestCase {
    id: tc
    name: "ScriptDoc"
    width: 400
    height: 300
    when: windowShown
    visible: true

    // A history that records the replacements it is given, as the native one would take them.
    QtObject {
        id: history
        property var calls: []
        function replace(from, to, text) { calls.push([from, to, text]) }
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
        ScriptDoc {}
    }

    function make(area) {
        const doc = createTemporaryObject(docComponent, tc, { area: area })
        verify(doc)
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

    function test_replace_is_one_step_of_the_history() {
        const doc = make(makeArea())
        doc.tools = history

        doc.replace(5, 9, "KELLER")

        compare(history.calls.length, 1)
        compare(history.calls[0], [5, 9, "KELLER"])
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

    Component {
        id: spyComponent
        SignalSpy {}
    }
}
