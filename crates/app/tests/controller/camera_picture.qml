import QtQuick
import QtQuick.Window
import io.github.shrippen.klappe
import "lib"

// The shot card's camera view with the real controller and the real `CameraPicture`, in a
// window (offscreen): the picture is made at the item's width, the frame follows the camera's
// aspect and the element paints it without a crash.
Item {
    id: root

    Watchdog {}
    Controller { id: ctl }

    readonly property url viewUrl: "qrc:/qt/qml/io/github/shrippen/klappe/qml/CameraView.qml"
    readonly property var texts: ({ t: key => key })
    property var view: null

    function fail(message) {
        console.error(message)
        Qt.exit(1)
    }

    Window {
        id: window
        width: 400
        height: 300
        visible: true
    }

    Timer {
        interval: 0
        running: true
        onTriggered: {
            if (!ctl.openProject(Qt.resolvedUrl("../../../../testdata/beispiel.klappe"))) {
                return root.fail("project did not open")
            }
            ctl.selectShot(JSON.parse(ctl.shotsJson)[0].id)

            const component = Qt.createComponent(root.viewUrl)
            if (component.status !== Component.Ready) {
                return root.fail("CameraView does not load: " + component.errorString())
            }
            root.view = component.createObject(window.contentItem,
                { texts: root.texts, controller: ctl, width: 400 })
            check.start()
        }
    }

    // Gives the window a few frames to paint.
    Timer {
        id: check
        interval: 300
        onTriggered: {
            const view = root.view
            if (view.frame === null) {
                return root.fail("the view has no picture")
            }
            if (view.frame.width !== Math.round(400 * window.devicePixelRatio)) {
                return root.fail("the picture is not as wide as the view in device pixels")
            }
            if (Math.abs(view.implicitHeight - Math.min(400 * view.aspect, 400 * view.maxHeightShare)) > 1) {
                return root.fail("the frame does not have the camera's aspect")
            }

            // Another playhead is another picture.
            const revision = ctl.pictureRevision
            ctl.seek(10)
            if (ctl.pictureRevision === revision) {
                return root.fail("the picture does not follow the playhead")
            }
            Qt.exit(0)
        }
    }
}
