import QtQuick
import QtTest
import "../../qml"

// PlanArea tooltip: numbers use the catalogue's decimal separator (review gui-r1).
TestCase {
    id: tc
    name: "PlanArea"
    width: 600
    height: 400
    when: windowShown
    visible: true

    QtObject {
        id: fake
        property bool loaded: true
        property string planJson: ""
        property string selectedKey: ""
        property bool grab: false
        property var drags: []
        property var places: []
        property var turns: []
        property int removes: 0
        property int cancels: 0

        function describe(key) {
            return JSON.stringify({ kind: "camera", label: "A", lens: 35, fov: 39.6, height: 1.6, shots: "12.1" })
        }
        function drag(fromX, fromY, x, y, done) {
            drags.push({ from: [fromX, fromY], to: [x, y], done: done })
            return true
        }
        function place(x, y) {
            places.push([x, y])
            return true
        }
        function grabbable(x, y) { return grab }
        function removeSelected() { removes++ }
        function turnSelected(step) { turns.push(step) }
        function cancelDrag() { cancels++ }
    }

    TestTexts { id: de; language: "de" }
    TestTexts { id: en; language: "en" }

    Component {
        id: areaComponent
        PlanArea {
            width: 600
            height: 400
            controller: fake
        }
    }

    function test_german_tooltip_has_commas() {
        const area = createTemporaryObject(areaComponent, tc, { texts: de })

        compare(area.tipLines("camera:a"), ["Kamera A", "35 mm · 39,6°", "Höhe 1,6 m", "Shots 12.1"])
    }

    function test_english_tooltip_has_points() {
        const area = createTemporaryObject(areaComponent, tc, { texts: en })
        const lines = area.tipLines("camera:a")

        verify(lines[1].indexOf("39.6") >= 0, lines[1])
        verify(lines[2].indexOf("1.6") >= 0, lines[2])
    }

    function init() {
        fake.planJson = JSON.stringify({ width: 20, depth: 13, shapes: [] })
        fake.selectedKey = ""
        fake.drags = []
        fake.places = []
        fake.turns = []
        fake.removes = 0
        fake.cancels = 0
    }

    function drawn(tool) {
        const area = createTemporaryObject(areaComponent, tc, { texts: de, tool: tool })
        waitForRendering(area)
        return area
    }

    function drag(area, steps) {
        mousePress(area, 100, 100, Qt.LeftButton)
        for (let i = 1; i <= steps; i++) {
            mouseMove(area, 100 + i * 40, 100 + i * 20, 0, Qt.LeftButton)
        }
        mouseRelease(area, 100 + steps * 40, 100 + steps * 20, Qt.LeftButton)
    }

    function test_a_wall_drag_goes_to_the_controller() {
        const area = drawn("wall")

        drag(area, 3)

        verify(fake.drags.length >= 2, fake.drags.length)
        verify(!fake.drags[0].done)
        verify(fake.drags[fake.drags.length - 1].done)
        verify(fake.drags[0].to[0] > fake.drags[0].from[0])
    }

    function test_select_pans_on_empty_ground() {
        const area = drawn("select")

        drag(area, 3)

        compare(fake.drags.length, 0)
    }

    function test_select_moves_what_the_pointer_is_over() {
        const area = drawn("select")
        fake.planJson = JSON.stringify({ width: 20, depth: 13, shapes: [
            { kind: "polygon", points: [0, 0, 20, 0, 20, 13, 0, 13], radius: 0, width: 0, label: "muted",
              stroke: "solid", fill: "solid", text: "", key: "object:t", selected: false }] })
        fake.grab = true
        mouseMove(area, 90, 90)
        mouseMove(area, 100, 100)

        verify(area.overGrab)
        drag(area, 2)

        verify(fake.drags.length >= 1)
    }

    function test_a_click_is_passed_on() {
        const area = drawn("camera")

        mouseClick(area, 120, 150)

        compare(fake.places.length, 1)
        compare(fake.drags.length, 0)
    }

    function test_escape_aborts_the_drag_until_release() {
        const area = drawn("wall")

        mousePress(area, 100, 100, Qt.LeftButton)
        mouseMove(area, 160, 120, 0, Qt.LeftButton)
        mouseMove(area, 200, 140, 0, Qt.LeftButton)
        const before = fake.drags.length
        verify(before >= 1)

        keyClick(Qt.Key_Escape)
        compare(fake.cancels, 1)

        mouseMove(area, 260, 160, 0, Qt.LeftButton)
        mouseRelease(area, 260, 160, Qt.LeftButton)
        compare(fake.drags.length, before)

        // The next drag is a new one.
        drag(area, 2)
        verify(fake.drags.length > before)
    }

    function test_delete_and_turn_keys_act_on_the_selection() {
        const area = drawn("select")

        keyClick(Qt.Key_R)
        compare(fake.turns.length, 0)

        fake.selectedKey = "camera:a"
        keyClick(Qt.Key_R)
        keyClick(Qt.Key_R, Qt.ShiftModifier)
        keyClick(Qt.Key_Delete)

        compare(fake.turns, [15, -15])
        compare(fake.removes, 1)
    }
}
