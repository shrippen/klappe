import QtQuick
import QtTest
import "../../qml"

// CameraView: the controller's picture and names, or a hint without a camera (LH-36, LH-99).
TestCase {
    id: tc
    name: "CameraView"
    width: 480
    height: 300
    visible: true
    when: windowShown

    TestTexts { id: catalogue }

    // What the controller offers: a picture 4:2 and one name; `frameText` empty is no camera.
    QtObject {
        id: fake
        property int pictureRevision: 0
        property string frameText: JSON.stringify({
            width: 400, height: 200,
            labels: [{ text: "Elisabeth", x: 200, y: 100 }]
        })
        property var requests: []

        function cameraImage(widthPx, paletteJson) {
            requests.push({ widthPx: widthPx, palette: JSON.parse(paletteJson) })
            return "image " + requests.length
        }
        function cameraFrame() {
            return frameText
        }
    }

    Component {
        id: viewComponent
        CameraView {
            width: 400
            texts: catalogue
            surfaceSource: Qt.resolvedUrl("StubSurface.qml")
        }
    }

    function make() {
        fake.requests = []
        fake.frameText = JSON.stringify({
            width: 400, height: 200,
            labels: [{ text: "Elisabeth", x: 200, y: 100 }]
        })
        const view = createTemporaryObject(viewComponent, tc, { controller: fake })
        verify(view)
        tryVerify(() => findChild(view, "viewSurface").item !== null)
        return view
    }

    function test_asks_for_the_picture_at_the_device_pixels_of_its_width() {
        const view = make()

        verify(fake.requests.length > 0)
        compare(fake.requests[fake.requests.length - 1].widthPx, Math.round(400 * Screen.devicePixelRatio))
        compare(findChild(view, "viewSurface").item.shown, "image " + fake.requests.length)
        verify(!findChild(view, "viewHint").visible)
    }

    function test_hands_over_kantes_colours() {
        make()
        const palette = fake.requests[fake.requests.length - 1].palette

        for (const name of ["ground", "text", "muted", "red", "orange", "purple"]) {
            compare(palette[name].length, 3, name)
        }
    }

    function test_frame_has_the_aspect_of_the_picture() {
        const view = make()

        compare(view.implicitHeight, 200)
    }

    function test_frame_never_takes_more_than_its_share_of_the_width() {
        fake.frameText = JSON.stringify({ width: 100, height: 100, labels: [] })
        const view = createTemporaryObject(viewComponent, tc, { controller: fake })
        tryVerify(() => view.frame !== null)

        compare(view.implicitHeight, 400 * view.maxHeightShare)
    }

    function test_writes_the_names_scaled_with_the_picture() {
        const view = make()
        const label = findChild(view, "viewLabel")

        compare(label.text, "Elisabeth")
        // The picture is shown at the view's width: the frame's 400 pixels are 400 units.
        compare(label.x, 200 - label.width / 2)
        compare(label.y, 100 - label.height)
    }

    function test_follows_a_new_revision() {
        const view = make()
        const before = fake.requests.length
        fake.frameText = JSON.stringify({ width: 400, height: 160, labels: [] })

        fake.pictureRevision += 1

        compare(fake.requests.length, before + 1)
        compare(view.implicitHeight, 160)
        compare(findChild(view, "viewLabel"), null)
    }

    function test_hint_without_a_camera() {
        const view = make()

        fake.frameText = ""
        fake.pictureRevision += 1

        const hint = findChild(view, "viewHint")
        verify(hint.visible)
        compare(hint.text, catalogue.t("view.none"))
        verify(!findChild(view, "viewBox").visible)
    }
}
