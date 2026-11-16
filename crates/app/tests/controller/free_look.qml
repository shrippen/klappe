import QtQuick
import io.github.shrippen.klappe
import "lib"

// LH-98, LH-99: the 3D view's wall cut (Y) and the look through the selected shot's camera:
// both make another picture, a turn leaves the camera, the playhead moves the view through it.
Item {
    Watchdog {}
    Controller { id: ctl }

    readonly property string colors: JSON.stringify({ ground: [255, 255, 255], text: [0, 0, 0] })
    readonly property real width3d: 800
    readonly property real height3d: 600

    function fail(message) {
        console.error(message)
        Qt.exit(1)
    }

    Timer {
        interval: 0
        running: true
        onTriggered: {
            if (!ctl.openProject(Qt.resolvedUrl("../../../../testdata/beispiel.klappe"))) {
                return fail("project did not open")
            }
            ctl.toggleView()
            ctl.freeSize(width3d, height3d)
            ctl.freeImage(1, colors)
            if (ctl.freeLook("") !== "") {
                return fail("a new 3D view has nothing on: " + ctl.freeLook(""))
            }

            // The cut: another picture, on and off; asking changes nothing.
            let revision = ctl.freeRevision
            if (ctl.freeLook("cut") !== "cut" || ctl.freeRevision === revision) {
                return fail("the cut did not switch on")
            }
            revision = ctl.freeRevision
            ctl.freeImage(1, colors)
            if (ctl.freeLook("cut") !== "" || ctl.freeRevision === revision) {
                return fail("the cut did not switch off")
            }
            if (ctl.canUndo) {
                return fail("the cut is an edit")
            }

            // Nothing selected: no camera to look through; it says so.
            if (ctl.freeLook("camera").split(" ").includes("camera")) {
                return fail("a look through no camera: " + ctl.freeLook(""))
            }
            if (ctl.status === "") {
                return fail("the refused look said nothing")
            }

            // Through the camera of the selected shot.
            ctl.selectShot(JSON.parse(ctl.shotsJson)[0].id)
            revision = ctl.freeRevision
            if (ctl.freeLook("camera") !== "camera" || ctl.freeRevision === revision) {
                return fail("the view did not go through the camera")
            }
            ctl.freeImage(1, colors)
            if (JSON.parse(ctl.freeFrame()).width !== width3d) {
                return fail("the picture through the camera is not as wide as the view")
            }

            // The playhead moves it.
            revision = ctl.pictureRevision
            ctl.seek(10)
            if (ctl.pictureRevision === revision) {
                return fail("the playhead does not make the view through a camera again")
            }
            if (ctl.freeLook("") !== "camera") {
                return fail("the look through the camera ended by itself")
            }

            // Cut and camera together; a turn leaves the camera only.
            ctl.freeLook("cut")
            if (ctl.freeLook("") !== "cut camera") {
                return fail("cut and camera: " + ctl.freeLook(""))
            }
            ctl.turnView(10, 0)
            if (ctl.freeLook("") !== "cut") {
                return fail("a turn did not leave the camera: " + ctl.freeLook(""))
            }
            Qt.exit(0)
        }
    }
}
