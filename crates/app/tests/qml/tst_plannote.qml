import QtQuick
import QtTest
import "../../qml"

// PlanArea: a double click on a note card edits its text (LH-16).
TestCase {
    id: tc
    name: "PlanNote"
    width: 600
    height: 400
    when: windowShown
    visible: true

    QtObject {
        id: fake
        property bool loaded: true
        property string planJson: ""
        property string selectedKey: ""
        property int selectedCount: 0
        property bool extendSelection: false
        property bool asksNote: false
        property string noteText: ""
        property bool measuring: false
        property int areaPoints: 0
        property int scalePoints: 0
        property int noteEdits: 0
        property int keyed: 0

        function markInfo() { return "{}" }
        function describe(key) { return "{}" }
        function place(x, y) {}
        function hover(x, y) { return true }
        function drag(a, b, c, d, done) {}
        function editSelectedNote() { noteEdits++ }
        function removeSelected() { keyed++ }
        function duplicateSelected() { keyed++ }
        function straightenSelected() { keyed++ }
        function turnSelected(deg) { keyed++ }
    }

    TestTexts { id: de; language: "de" }

    Component {
        id: areaComponent
        PlanArea {
            width: 600
            height: 400
            controller: fake
            texts: de
        }
    }

    function card(key) {
        return JSON.stringify({ width: 20, depth: 13, shapes: [
            { kind: "polygon", points: [0, 0, 20, 0, 20, 13, 0, 13], radius: 0, width: 0, label: "text",
              stroke: "solid", fill: "tint", text: "Vorhang", textAnchor: "topLeft", key: key, selected: false }] })
    }

    function drawn(tool, key) {
        fake.noteEdits = 0
        fake.planJson = card(key)
        const area = createTemporaryObject(areaComponent, tc, { tool: tool })
        waitForRendering(area)
        return area
    }

    function test_a_double_click_on_a_note_edits_it() {
        const area = drawn("select", "note:n1")

        mouseDoubleClickSequence(area, 120, 150)

        compare(fake.noteEdits, 1)
    }

    function test_a_double_click_elsewhere_edits_no_note() {
        const area = drawn("select", "object:o1")

        mouseDoubleClickSequence(area, 120, 150)

        compare(fake.noteEdits, 0)
    }

    function test_the_note_tool_does_not_edit_twice() {
        const area = drawn("note", "note:n1")

        mouseDoubleClickSequence(area, 120, 150)

        compare(fake.noteEdits, 0)
    }

    Component {
        id: tableComponent
        ShotTable {
            width: 600
            height: 200
            texts: de
            selectedId: "s1"
            shots: [{ id: "s1", number: "1.1", camera: "a", camera_label: "A", lens_label: "35", own_lens_mm: null,
                      lens_from_marks: false, size: "wide", angle: "eye", movement: "static", from_s: 0, to_s: 2,
                      setup: 1, description: "", props: "", gear: "", crew: "", note: "" }]
        }
    }

    // In the shot table the keys are the table's: F2 edits the cell, the plan's selection is left alone.
    function test_the_keys_in_the_shot_table_leave_the_plan_selection() {
        const area = drawn("select", "note:n1")
        fake.selectedKey = "note:n1"
        fake.keyed = 0
        const table = createTemporaryObject(tableComponent, tc)
        table.forceActiveFocus()
        verify(table.activeFocus)

        keyClick(Qt.Key_F2)
        compare(fake.noteEdits, 0)
        compare(table.editId, "s1")

        keyClick(Qt.Key_Delete)
        keyClick(Qt.Key_R)
        keyClick(Qt.Key_G)
        keyClick(Qt.Key_D, Qt.ControlModifier)
        compare(fake.keyed, 0)
        fake.selectedKey = ""
    }
}
