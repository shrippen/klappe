import QtQuick
import QtTest
import "../../qml"

// FreeView (LH-94): the controller's 3D picture and names, and what the mouse does in it:
// click and drag edit as on the plan, the right button or Alt turns, the middle button or
// Shift + right shifts, the wheel zooms to the pointer, Escape drops a drag.
TestCase {
    id: tc
    name: "FreeView"
    width: 400
    height: 300
    visible: true
    when: windowShown

    TestTexts { id: catalogue }

    QtObject {
        id: fake
        property int pictureRevision: 0
        property int freeRevision: 0
        property string selectedKey: ""
        property int selectedCount: 0
        property string frameText: ""
        property var requests: []
        property var calls: []

        function freeSize(w, h) { calls.push(["freeSize", w, h]) }
        function freeImage(scale, paletteJson) {
            requests.push({ scale: scale, palette: JSON.parse(paletteJson) })
            return "image " + requests.length
        }
        function freeFrame() { return frameText }
        function turnView(dx, dy) { calls.push(["turnView", dx, dy]) }
        function panView(dx, dy) { calls.push(["panView", dx, dy]) }
        function zoomView(steps, x, y) { calls.push(["zoomView", steps, x, y]) }
        function stepView(step) { calls.push(["stepView", step]) }
        function freePlace(x, y) { calls.push(["freePlace", x, y]); return true }
        function freeDrag(fx, fy, x, y, done) { calls.push(["freeDrag", fx, fy, x, y, done]); return true }
        function cancelDrag() { calls.push(["cancelDrag"]) }
    }

    Component {
        id: viewComponent
        FreeView {
            width: 400
            height: 300
            texts: catalogue
            surfaceSource: Qt.resolvedUrl("StubSurface.qml")
        }
    }

    function make() {
        fake.requests = []
        fake.frameText = JSON.stringify({ width: 400, height: 300, labels: [{ text: "Elisabeth", x: 200, y: 100 }] })
        const view = createTemporaryObject(viewComponent, tc, { controller: fake })
        verify(view)
        tryVerify(() => findChild(view, "freeSurface").item !== null)
        fake.calls = []
        return view
    }

    // The calls without the size the view tells before it asks for a picture.
    function calls() {
        return fake.calls.filter(c => c[0] !== "freeSize")
    }

    function test_makes_the_picture_at_the_device_pixels_of_the_view() {
        const view = make()

        verify(fake.requests.length > 0)
        const last = fake.requests[fake.requests.length - 1]
        compare(last.scale, 1)
        for (const name of ["ground", "text", "muted", "red"]) {
            compare(last.palette[name].length, 3, name)
        }
        compare(findChild(view, "freeSurface").item.shown, "image " + fake.requests.length)
        verify(!findChild(view, "freeHint").visible)
    }

    function test_writes_the_names_over_the_picture() {
        const view = make()
        const label = findChild(view, "freeLabel")

        compare(label.text, "Elisabeth")
        compare(label.x, 200 - label.width / 2)
        compare(label.y, 100 - label.height)
    }

    function test_follows_a_new_revision_of_the_picture_or_of_the_view() {
        make()
        const before = fake.requests.length

        fake.freeRevision += 1
        compare(fake.requests.length, before + 1)

        fake.pictureRevision += 1
        compare(fake.requests.length, before + 2)
    }

    // NF-06: the arrows turn and tilt, Shift + arrows shift, + and - zoom; Ctrl keeps its keys.
    function test_the_keys_step_the_view_while_it_has_the_focus() {
        const view = make()
        view.forceActiveFocus()

        keyClick(Qt.Key_Left)
        keyClick(Qt.Key_Up)
        keyClick(Qt.Key_Right, Qt.ShiftModifier)
        keyClick(Qt.Key_Down, Qt.ShiftModifier)
        keyClick(Qt.Key_Plus)
        keyClick(Qt.Key_Minus)
        keyClick(Qt.Key_Left, Qt.ControlModifier)

        compare(calls(), [["stepView", "turnLeft"], ["stepView", "turnUp"], ["stepView", "panRight"],
                          ["stepView", "panDown"], ["stepView", "zoomIn"], ["stepView", "zoomOut"]])
    }

    function test_a_new_selection_makes_the_picture_again() {
        make()
        const before = fake.requests.length

        fake.selectedKey = "wall:1"
        compare(fake.requests.length, before + 1)
    }

    function test_hint_without_a_picture() {
        const view = make()

        fake.frameText = ""
        fake.freeRevision += 1

        const hint = findChild(view, "freeHint")
        verify(hint.visible)
        compare(hint.text, catalogue.t("view.freeNone"))
    }

    function test_a_click_goes_to_the_controller_in_device_pixels() {
        const view = make()
        const dpr = view.dpr

        mouseClick(view, 100, 80)

        compare(calls(), [["freePlace", 100 * dpr, 80 * dpr]])
        compare(view.clickAt, Qt.point(100, 80))
    }

    function test_a_left_drag_edits_and_a_release_ends_it() {
        const view = make()
        const dpr = view.dpr

        mousePress(view, 100, 80)
        mouseMove(view, 160, 80)
        mouseMove(view, 200, 90)
        mouseRelease(view, 200, 90)

        const drags = calls()
        verify(drags.length >= 2)
        for (const c of drags) {
            compare(c[0], "freeDrag")
            compare(c[1], 100 * dpr)
            compare(c[2], 80 * dpr)
        }
        compare(drags[drags.length - 1].slice(3), [200 * dpr, 90 * dpr, true])
        verify(drags.slice(0, -1).every(c => c[5] === false))
    }

    function test_escape_drops_a_drag() {
        const view = make()

        mousePress(view, 100, 80)
        mouseMove(view, 200, 80)
        verify(view.dragging)
        fake.calls = []

        keyClick(Qt.Key_Escape)
        mouseRelease(view, 200, 80)

        compare(calls(), [["cancelDrag"]])
    }

    // The window takes the press away (Alt+Tab, a popup): the controller drops the drag too.
    function test_a_taken_press_drops_a_drag() {
        const view = make()

        mousePress(view, 100, 80)
        mouseMove(view, 200, 80)
        fake.calls = []

        findChild(view, "freeInput").canceled()

        compare(calls(), [["cancelDrag"]])
        verify(!view.dragging)
        mouseRelease(view, 200, 80)
    }

    function test_the_right_button_turns_and_made_smaller_while_it_runs() {
        const view = make()

        mousePress(view, 100, 100, Qt.RightButton)
        mouseMove(view, 130, 90, 0, Qt.RightButton)
        verify(view.interacting)
        compare(fake.requests[fake.requests.length - 1].scale, view.handScale)
        mouseRelease(view, 130, 90, Qt.RightButton)

        compare(calls(), [["turnView", 30, -10]])
        verify(!view.interacting)
        compare(fake.requests[fake.requests.length - 1].scale, 1)
    }

    function test_alt_and_the_left_button_turn() {
        const view = make()

        mousePress(view, 100, 100, Qt.LeftButton, Qt.AltModifier)
        mouseMove(view, 110, 100, 0, Qt.LeftButton)
        mouseRelease(view, 110, 100, Qt.LeftButton)

        compare(calls(), [["turnView", 10, 0]])
    }

    function test_the_middle_button_and_shift_with_the_right_one_shift() {
        const view = make()
        const dpr = view.dpr

        mousePress(view, 100, 100, Qt.MiddleButton)
        mouseMove(view, 120, 100, 0, Qt.MiddleButton)
        mouseRelease(view, 120, 100, Qt.MiddleButton)
        mousePress(view, 100, 100, Qt.RightButton, Qt.ShiftModifier)
        mouseMove(view, 100, 130, 0, Qt.RightButton)
        mouseRelease(view, 100, 130, Qt.RightButton)

        compare(calls(), [["panView", 20 * dpr, 0], ["panView", 0, 30 * dpr]])
    }

    function test_the_wheel_zooms_to_the_pointer() {
        const view = make()
        const dpr = view.dpr

        mouseWheel(view, 150, 60, 0, 120)

        compare(calls(), [["zoomView", 1, 150 * dpr, 60 * dpr]])
    }

    function test_the_menus_zoom_goes_to_the_middle() {
        const view = make()
        const dpr = view.dpr

        view.zoomBy("out")

        compare(calls(), [["zoomView", -1, 200 * dpr, 150 * dpr]])
    }

    function test_a_controller_without_3d_gets_no_surface_and_no_input() {
        const view = createTemporaryObject(viewComponent, tc, { controller: null })

        compare(findChild(view, "freeSurface").item, null)
        verify(!findChild(view, "freeInput").enabled)
        verify(findChild(view, "freeHint").visible)
    }
}
