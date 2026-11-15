import QtQuick
import QtTest
import "../../qml"

// PlanArea lasso (LH-08): Ctrl at the press makes the drag on empty ground a lasso, Shift
// still adds to the selection; the canvas hands its drag on either way.
TestCase {
    id: tc
    name: "PlanAreaLasso"
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
        property var drags: []

        function drag(fromX, fromY, x, y, done) {
            drags.push({ from: [fromX, fromY], to: [x, y], done: done })
            return true
        }
        function place(x, y) { return true }
    }

    TestTexts { id: de; language: "de" }

    Component {
        id: areaComponent
        PlanArea {
            width: 600
            height: 400
            controller: fake
            texts: de
            tool: "select"
        }
    }

    function init() {
        fake.planJson = JSON.stringify({ width: 20, depth: 13, shapes: [] })
        fake.extendSelection = false
        fake.lassoSelection = false
        fake.drags = []
    }

    function lasso(area, modifiers) {
        mousePress(area, 100, 100, Qt.LeftButton, modifiers)
        mouseMove(area, 160, 100, 0, Qt.LeftButton)
        mouseMove(area, 160, 160, 0, Qt.LeftButton)
        mouseMove(area, 100, 160, 0, Qt.LeftButton)
        mouseRelease(area, 100, 160, Qt.LeftButton, modifiers)
    }

    function test_a_plain_drag_is_a_box() {
        const area = createTemporaryObject(areaComponent, tc)
        waitForRendering(area)

        lasso(area, Qt.NoModifier)

        verify(!fake.lassoSelection)
        verify(fake.drags.length >= 1)
    }

    function test_ctrl_at_the_press_makes_the_drag_a_lasso() {
        const area = createTemporaryObject(areaComponent, tc)
        waitForRendering(area)

        lasso(area, Qt.ControlModifier)

        verify(fake.lassoSelection)
        verify(!fake.extendSelection)
        verify(fake.drags.length >= 3)
        verify(fake.drags[fake.drags.length - 1].done)
    }

    function test_ctrl_and_shift_make_a_lasso_that_adds() {
        const area = createTemporaryObject(areaComponent, tc)
        waitForRendering(area)

        lasso(area, Qt.ControlModifier | Qt.ShiftModifier)

        verify(fake.lassoSelection)
        verify(fake.extendSelection)
    }

    function test_the_next_press_without_ctrl_is_a_box_again() {
        const area = createTemporaryObject(areaComponent, tc)
        waitForRendering(area)
        lasso(area, Qt.ControlModifier)
        verify(fake.lassoSelection)

        lasso(area, Qt.NoModifier)

        verify(!fake.lassoSelection)
    }
}
