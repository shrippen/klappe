import QtQuick
import QtTest
import "../../qml"

// PlanArea keys of LH-104 ff.: Escape ends an armed click before anything else that answers
// Escape (two enabled shortcuts are ambiguous and Qt does nothing), and the arrow keys move
// the selected way only while a grip of a way is selected.
TestCase {
    id: tc
    name: "PlanAreaArm"
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
        property bool asksNote: false
        property string armed: ""
        property int areaPoints: 0
        property int armCancels: 0
        property int areaCancels: 0
        property int clears: 0
        property var nudges: []

        function place(x, y) { return true }
        function markInfo() { return "{}" }
        function cancelArm() {
            armCancels++
            armed = ""
        }
        function cancelArea() {
            areaCancels++
            areaPoints = 0
        }
        function clearSelection() { clears++ }
        function nudgeWay(right, down) {
            nudges.push([right, down])
            return true
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
        fake.selectedCount = 0
        fake.armed = ""
        fake.areaPoints = 0
        fake.armCancels = 0
        fake.areaCancels = 0
        fake.clears = 0
        fake.nudges = []
    }

    function plan(tool) {
        const area = createTemporaryObject(areaComponent, tc, { tool: tool })
        waitForRendering(area)
        findChild(area, "planCanvas").forceActiveFocus()

        return area
    }

    // With an area open as well, one Escape is enabled, not two; the next one drops the area.
    function test_escape_ends_an_armed_click_before_an_open_area() {
        plan("area")
        fake.selectedKey = "figure:elisabeth"
        fake.areaPoints = 3
        fake.armed = "nextMark"

        keyClick(Qt.Key_Escape)
        compare(fake.armCancels, 1)
        compare(fake.areaCancels, 0)
        compare(fake.clears, 0)

        keyClick(Qt.Key_Escape)
        compare(fake.areaCancels, 1)
    }

    function test_escape_ends_an_armed_click_before_it_clears_the_selection() {
        plan("select")
        fake.selectedKey = "figure:elisabeth"
        fake.armed = "axisTo"

        keyClick(Qt.Key_Escape)
        compare(fake.armCancels, 1)
        compare(fake.clears, 0)

        keyClick(Qt.Key_Escape)
        compare(fake.clears, 1)
    }

    function test_the_arrow_keys_move_the_selected_way() {
        plan("select")
        fake.selectedKey = "figurevia:elisabeth:1:new0"

        keyClick(Qt.Key_Left)
        keyClick(Qt.Key_Right)
        keyClick(Qt.Key_Up)
        keyClick(Qt.Key_Down)

        compare(fake.nudges, [[-1, 0], [1, 0], [0, -1], [0, 1]])
    }

    function test_the_arrow_keys_leave_other_selections_to_the_plan() {
        plan("select")
        fake.selectedKey = "figure:elisabeth"

        keyClick(Qt.Key_Left)
        keyClick(Qt.Key_Down)

        compare(fake.nudges, [])
    }
}
