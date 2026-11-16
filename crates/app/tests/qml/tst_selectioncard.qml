import QtQuick
import QtTest
import org.kde.kirigami as Kirigami
import "../../qml"

// The card of the selection (D-04) in a PlanArea: where it opens, how it follows the element
// and when it goes.
TestCase {
    id: tc
    name: "SelectionCard"
    width: 900
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
        property bool viewFree: false
        property bool asksNote: false
        property string markJson: "{}"

        function place() {}
        function clearSelection() { selectedKey = ""; selectedCount = 0 }
        function markInfo() { return markJson }
        function describe(key) {
            if (key === "object:o1") {
                return JSON.stringify({ kind: "object", label: "", library: "table", width: 1.6, depth: 0.9, height: 0.8, base: 0, baseDefault: true })
            }
            return JSON.stringify({ kind: "camera", label: "A", lens: 35, fov: 39.6, height: 1.6, shots: "12.1" })
        }
    }

    TestTexts { id: de; language: "de" }

    Component {
        id: areaComponent
        PlanArea {
            width: 600
            height: 400
            controller: fake
        }
    }

    // One box per selectable element, along the bottom of the plan.
    function box(key, x) {
        const y = 11
        return { kind: "polygon", points: [x, y, x + 1, y, x + 1, y + 1, x, y + 1], radius: 0, width: 0,
                 label: "muted", stroke: "solid", fill: "tint", text: "", key: key, selected: false }
    }

    function planWith(x) {
        const keys = ["object:o1", "object:o2", "object:o3", "camera:a", "camera:b", "note:n1"]
        return JSON.stringify({ width: 20, depth: 13, shapes: keys.map((k, i) => box(k, x + i * 2)) })
    }

    function init() {
        fake.planJson = planWith(1)
        fake.selectedKey = ""
        fake.selectedCount = 0
        fake.extendSelection = false
        fake.viewFree = false
        fake.markJson = "{}"
    }

    function drawn(tool, width) {
        const area = createTemporaryObject(areaComponent, tc, { texts: de, tool: tool, width: width || 600 })
        waitForRendering(area)
        return area
    }

    // Room for a card on either side of an element.
    readonly property real wide: 900

    // The card has its size once the layout ran; the place is set again then.
    function settled(card) {
        tryVerify(() => card.visible && card.implicitWidth > Kirigami.Units.gridUnit * 8)
        wait(100)
    }

    // D-04: the card of the selection opens beside the element, with a pointer at it.
    function test_the_card_opens_beside_the_element() {
        const area = drawn("select", wide)
        const card = findChild(area, "selectionCard")
        verify(!card.visible)

        fake.selectedKey = "object:o1"
        settled(card)

        compare(card.pointerSide, "left")
        const element = card.canvas.mapFromModel(2, 11.5)
        verify(card.x > element.x)
        verify(card.y <= element.y && element.y <= card.y + card.height)
    }

    // LH-93: "add heights" opens the rest of the card and puts the keyboard in the height field.
    function test_the_height_field_takes_the_keyboard_when_a_height_is_asked_for() {
        const area = drawn("select", wide)
        const card = findChild(area, "selectionCard")
        fake.selectedKey = "object:o1"
        settled(card)
        const field = findChild(card, "heightField")
        verify(!field.activeFocus)

        area.focusHeight()

        tryVerify(() => field.activeFocus)
        verify(card.expanded)
    }

    // The card goes to the other side of the element when the right has no room.
    function test_the_card_flips_left_at_the_plans_edge() {
        fake.planJson = planWith(17)
        const area = drawn("select", wide)
        const card = findChild(area, "selectionCard")

        fake.selectedKey = "object:o1"
        settled(card)

        compare(card.pointerSide, "right")
        const element = card.canvas.mapFromModel(17, 11.5)

        verify(card.x + card.width < element.x)
    }

    function test_the_card_stays_in_the_picture() {
        fake.planJson = planWith(17)
        const area = drawn("select", wide)
        const card = findChild(area, "selectionCard")

        for (const key of ["object:o1", "camera:a"]) {
            fake.selectedKey = key
            settled(card)
            verify(card.x >= 0, key)
            verify(card.y >= 0, key)
            verify(card.x + card.width <= area.width, key)
            verify(card.y + card.height <= area.height, key)
        }
    }

    // Review of #90: with no room on either side the card left the plan; it covers the
    // element inside the plan then.
    function test_the_card_stays_in_the_plan_without_room_beside() {
        const huge = { kind: "polygon", points: [0, 0, 20, 0, 20, 13, 0, 13], radius: 0, width: 0,
                       label: "muted", stroke: "solid", fill: "tint", text: "", key: "object:o1",
                       selected: false }
        fake.planJson = JSON.stringify({ width: 20, depth: 13, shapes: [huge] })
        const area = drawn("select")
        const card = findChild(area, "selectionCard")

        fake.selectedKey = "object:o1"
        settled(card)

        verify(card.x >= 0, "x " + card.x)
        verify(card.x + card.width <= area.width, "right " + (card.x + card.width))
    }

    function test_the_card_follows_the_element_when_the_plan_is_panned() {
        const area = drawn("select", wide)
        const card = findChild(area, "selectionCard")
        fake.selectedKey = "object:o1"
        settled(card)
        const before = card.x

        card.canvas.panX += 40

        compare(card.x, before + 40)
    }

    // Playing moves the element: the plan hands the canvas new shapes.
    function test_the_card_follows_the_element_when_the_shapes_move() {
        const area = drawn("select", wide)
        const card = findChild(area, "selectionCard")
        fake.selectedKey = "object:o1"
        settled(card)
        const before = card.x

        fake.planJson = planWith(4)

        const element = card.canvas.mapFromModel(4, 11.5)
        tryVerify(() => card.x > before)
        verify(card.x > element.x)
        verify(card.y <= element.y && element.y <= card.y + card.height)
    }

    function test_the_card_stays_inside_when_the_plan_shrinks() {
        fake.planJson = planWith(8)
        const area = drawn("select", wide)
        const card = findChild(area, "selectionCard")
        fake.selectedKey = "object:o1"
        settled(card)

        area.width = 500
        wait(100)

        verify(card.x >= 0)
        verify(card.x + card.width <= area.width)
    }

    // A card that would open under the mouse pointer goes to the other side.
    function test_the_card_keeps_clear_of_the_pointer() {
        fake.planJson = planWith(8)
        const area = drawn("select", wide)
        const card = findChild(area, "selectionCard")
        const element = card.canvas.mapFromModel(8, 11.5)
        card.pointer = Qt.point(element.x + 100, element.y)
        card.pointerOver = true

        fake.selectedKey = "object:o1"
        settled(card)

        compare(card.pointerSide, "right")
        verify(card.x + card.width < element.x)
    }

    function test_a_camera_has_one_card_with_two_sections() {
        const area = drawn("select", wide)
        const card = findChild(area, "selectionCard")
        fake.markJson = JSON.stringify({ kind: "camera", pose: "", arrival: "", departure: "", hold: 0, hasWay: false,
                                         pace: null, height: 1.6, tilt: 0, rig: "tripod", lens: null })

        fake.selectedKey = "camera:a"

        verify(findChild(area, "cameraCard").visible)
        verify(findChild(area, "markCard").visible)
        compare(findChild(area, "cameraCard").parent, findChild(area, "markCard").parent)
        verify(card.visible)
    }

    // Several selected, or none: no card.
    function test_no_card_without_one_selected_element() {
        const area = drawn("select", wide)
        const card = findChild(area, "selectionCard")
        fake.selectedKey = "object:o1"
        verify(card.visible)

        fake.selectedKey = ""
        fake.selectedCount = 2

        verify(!card.visible)
    }

    // The plan is hidden (a tab of the narrow step): the card is a popup in the overlay and
    // must not stay over the other tab.
    function test_the_card_goes_when_the_plan_is_hidden() {
        const area = drawn("select", wide)
        const card = findChild(area, "selectionCard")
        fake.selectedKey = "object:o1"
        settled(card)

        area.visible = false

        verify(!card.visible)

        area.visible = true

        settled(card)
    }

    // LH-94: in 3D the plan's shape is not on view; the card opens beside the click instead.
    function test_in_3d_the_card_opens_beside_the_click() {
        fake.viewFree = true
        const area = drawn("select", wide)
        const card = findChild(area, "selectionCard")
        const view = findChild(area, "freeView")
        view.clickAt = Qt.point(200, 150)

        fake.selectedKey = "object:o1"
        settled(card)

        verify(card.visible)
        verify(card.x > 200)
        verify(card.y <= 150 && 150 <= card.y + card.height)

        view.clickAt = Qt.point(300, 250)
        settled(card)

        verify(card.x > 300)
        verify(card.y <= 250 && 250 <= card.y + card.height)
    }

    // Q1-04: Escape closes the card (it clears the selection, so a click on the element opens
    // it again).
    function test_escape_closes_the_card() {
        const area = drawn("select", wide)
        const card = findChild(area, "selectionCard")
        fake.selectedKey = "object:o1"
        settled(card)

        area.forceActiveFocus()
        keyClick(Qt.Key_Escape)

        tryVerify(() => !card.visible)
        compare(fake.selectedKey, "")
    }

    // In a field of the card Escape first leaves the field; a second one closes the card.
    function test_escape_in_a_field_leaves_it_first() {
        const area = drawn("select", wide)
        const card = findChild(area, "selectionCard")
        fake.selectedKey = "object:o1"
        settled(card)
        card.expanded = true
        const field = findChild(card, "heightField")
        field.forceActiveFocus()
        verify(card.typing)

        keyClick(Qt.Key_Escape)

        verify(!card.typing)
        verify(card.visible)

        keyClick(Qt.Key_Escape)

        tryVerify(() => !card.visible)
    }

    // Q1-04: a change of tool closes the card and keeps the selection; a click on the element
    // or another selection opens it again.
    function test_a_change_of_tool_closes_the_card() {
        const area = drawn("select", wide)
        const card = findChild(area, "selectionCard")
        fake.selectedKey = "object:o1"
        settled(card)

        area.tool = "wall"

        tryVerify(() => !card.visible)
        compare(fake.selectedKey, "object:o1")

        area.tool = "select"
        verify(!card.visible)

        card.canvas.shapeClicked("object:o1", 2, 11.5)
        settled(card)

        area.tool = "wall"
        tryVerify(() => !card.visible)
        fake.selectedKey = ""
        fake.selectedKey = "object:o1"
        settled(card)
    }

    // Review of #173: only a click that selects opens the card again; one of the measure tool on
    // the element's shape (it selects nothing) does not.
    function test_a_click_of_another_tool_does_not_reopen_the_card() {
        const area = drawn("select", wide)
        const card = findChild(area, "selectionCard")
        fake.selectedKey = "object:o1"
        settled(card)
        area.tool = "measure"
        tryVerify(() => !card.visible)

        card.canvas.shapeClicked("object:o1", 2, 11.5)
        wait(100)

        verify(!card.visible)
    }

    // Escape in a text field elsewhere in the window (a cell of the shot table) belongs to
    // that field, not to the plan's selection.
    function test_escape_in_a_field_outside_the_plan_keeps_the_selection() {
        const area = drawn("select", wide)
        const card = findChild(area, "selectionCard")
        const outside = createTemporaryObject(outsideComponent, tc)
        fake.selectedKey = "object:o1"
        settled(card)
        outside.forceActiveFocus()
        verify(outside.activeFocus)

        keyClick(Qt.Key_Escape)

        compare(outside.escapes, 1)
        compare(fake.selectedKey, "object:o1")
        verify(card.visible)
    }

    // "Add heights" after a change of tool opens the card again.
    function test_adding_heights_reopens_a_dismissed_card() {
        const area = drawn("select", wide)
        const card = findChild(area, "selectionCard")
        fake.selectedKey = "object:o1"
        settled(card)
        area.tool = "wall"
        tryVerify(() => !card.visible)

        area.focusHeight()

        settled(card)
    }

    Component {
        id: outsideComponent
        TextInput {
            property int escapes: 0
            width: 100
            height: 20
            Keys.onEscapePressed: escapes++
        }
    }
}
