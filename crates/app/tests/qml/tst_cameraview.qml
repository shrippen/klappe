import QtQuick
import QtTest
import "../../qml"

// CameraView: the shapes of the projection, or a hint without a camera (LH-36).
TestCase {
    id: tc
    name: "CameraView"
    width: 480
    height: 300
    visible: true
    when: windowShown

    TestTexts { id: catalogue }

    readonly property string picture: JSON.stringify({
        width: 1, depth: 0.5,
        shapes: [
            { kind: "polygon", points: [0, 0.25, 1, 0.25, 1, 0.5, 0, 0.5], radius: 0, width: 0,
              label: "muted", stroke: "solid", fill: "tint", text: "", key: "", selected: false },
            { kind: "polygon", points: [0.4, 0.1, 0.6, 0.1, 0.6, 0.4, 0.4, 0.4], radius: 0, width: 0,
              label: "red", stroke: "solid", fill: "solid", text: "Elisabeth", key: "", selected: false }
        ]
    })

    Component {
        id: viewComponent
        CameraView {
            width: 400
            texts: catalogue
        }
    }

    function make(json) {
        const view = createTemporaryObject(viewComponent, tc, { json: json })
        verify(view)
        return view
    }

    function test_draws_the_shapes() {
        const view = make(picture)
        const canvas = findChild(view, "viewCanvas")

        verify(canvas.visible)
        compare(canvas.shapes.length, 2)
        verify(!findChild(view, "viewHint").visible)
    }

    function test_frame_has_the_aspect_of_the_picture() {
        const view = make(picture)

        compare(view.implicitHeight, 200)
    }

    function test_follows_a_new_picture() {
        const view = make(picture)

        view.json = JSON.stringify({ width: 1, depth: 0.4, shapes: [] })

        compare(findChild(view, "viewCanvas").shapes.length, 0)
        compare(view.implicitHeight, 160)
    }

    function test_hint_without_a_camera() {
        const view = make("")
        const hint = findChild(view, "viewHint")

        verify(hint.visible)
        compare(hint.text, catalogue.t("view.none"))
        verify(!findChild(view, "viewCanvas").visible)
    }
}
