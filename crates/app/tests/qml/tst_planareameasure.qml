import QtQuick
import QtTest
import "../../qml"

// PlanArea measure field (LH-95): a digit typed on the plan opens a field with it, for the length
// of the wall being drawn or the height of the selected object or wall; Enter hands the text to
// the controller, Escape closes the field.
TestCase {
    id: tc
    name: "PlanAreaMeasure"
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
        property bool lassoSelection: false
        property var typed: []
        property bool typeOk: true
        property int removes: 0
        property var turns: []

        function drag(fromX, fromY, x, y, done) { return true }
        function place(x, y) { return true }
        function cancelDrag() {}
        function removeSelected() { removes++ }
        function turnSelected(step) { turns.push(step) }
        function typeMeasure(text) {
            typed.push(text)
            return typeOk
        }
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

    function init() {
        fake.planJson = JSON.stringify({ width: 20, depth: 13, shapes: [] })
        fake.selectedKey = ""
        fake.typed = []
        fake.typeOk = true
        fake.removes = 0
        fake.turns = []
    }

    function drawn(tool) {
        const area = createTemporaryObject(areaComponent, tc, { tool: tool })
        waitForRendering(area)
        findChild(area, "planCanvas").forceActiveFocus()
        return area
    }

    function test_enter_opens_the_height_field() {
        const area = drawn("select")
        const card = findChild(area, "measureCard")
        const field = findChild(area, "measureField")
        fake.selectedKey = "object:o1"

        verify(!card.visible)
        keyClick(Qt.Key_Return)
        verify(card.visible)
        compare(field.text, "")
        verify(field.activeFocus)

        keyClick(Qt.Key_0)
        keyClick(Qt.Key_Comma)
        keyClick(Qt.Key_8)
        compare(field.text, "0,8")
        keyClick(Qt.Key_Return)

        compare(fake.typed, ["0,8"])
        verify(!card.visible)
        verify(findChild(area, "planCanvas").activeFocus)
    }

    function test_the_field_names_what_enter_does() {
        const area = drawn("select")
        fake.selectedKey = "wall:0"

        keyClick(Qt.Key_Return)

        compare(area.asksMeasure, "height")
        compare(findChild(area, "measureField").Accessible.name, de.t("measure.height"))
    }

    function test_a_refused_measure_stays_open_and_escape_closes_it() {
        const area = drawn("select")
        const card = findChild(area, "measureCard")
        fake.selectedKey = "object:o1"
        fake.typeOk = false

        keyClick(Qt.Key_Return)
        keyClick(Qt.Key_9)
        keyClick(Qt.Key_Return)
        verify(card.visible)
        verify(findChild(area, "measureField").invalid)

        keyClick(Qt.Key_Escape)
        verify(!card.visible)
    }

    // Nothing to apply it to: Enter opens nothing.
    function test_enter_without_a_target_opens_nothing() {
        const area = drawn("select")
        const card = findChild(area, "measureCard")

        keyClick(Qt.Key_Return)
        verify(!card.visible)

        fake.selectedKey = "camera:a"
        keyClick(Qt.Key_Return)
        verify(!card.visible)
        compare(fake.typed.length, 0)
    }

    function test_a_wall_drag_asks_for_the_length() {
        const area = drawn("wall")
        const card = findChild(area, "measureCard")

        mousePress(area, 100, 100, Qt.LeftButton)
        mouseMove(area, 160, 120, 0, Qt.LeftButton)
        mouseMove(area, 200, 140, 0, Qt.LeftButton)
        keyClick(Qt.Key_Return)
        keyClick(Qt.Key_3)

        verify(card.visible)
        compare(area.asksMeasure, "length")
        keyClick(Qt.Key_Return)
        compare(fake.typed, ["3"])
        verify(!card.visible)
        mouseRelease(area, 200, 140, Qt.LeftButton)
    }

    // The field is text: Delete and letters do not reach the selection.
    function test_typing_a_measure_leaves_the_selection_alone() {
        const area = drawn("select")
        fake.selectedKey = "object:o1"

        keyClick(Qt.Key_Return)
        keyClick(Qt.Key_1)
        keyClick(Qt.Key_Delete)
        keyClick(Qt.Key_R)

        compare(fake.removes, 0)
        compare(fake.turns.length, 0)
    }

    // Digits stay the mode keys and the fit (0) with a selection.
    function test_a_digit_opens_nothing() {
        const area = drawn("select")
        fake.selectedKey = "object:o1"

        keyClick(Qt.Key_2)
        keyClick(Qt.Key_0)

        verify(!findChild(area, "measureCard").visible)
    }
}
