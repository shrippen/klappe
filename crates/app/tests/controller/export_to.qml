import QtQuick
import QtCore
import io.github.shrippen.klappe
import "lib"

// `exportTo` reports what happened to the file: a path with a space and an umlaut is written,
// a target that cannot be replaced is a failure even though something lies there, and an
// unknown kind is refused. The folder is removed by the test that runs this (qml_tests.rs).
Item {
    id: root

    Watchdog {}
    Controller { id: ctl }

    function fail(message) {
        console.error(message)
        Qt.exit(1)
    }

    function size(url) {
        const request = new XMLHttpRequest()
        request.open("GET", url, false)
        request.send()
        return request.status === 200 ? request.responseText.length : -1
    }

    Timer {
        interval: 0
        running: true
        onTriggered: {
            const base = StandardPaths.writableLocation(StandardPaths.TempLocation) + "/klappe-export-test"
            const example = Qt.resolvedUrl("../../../../testdata/beispiel.klappe")
            if (!ctl.openProject(example)) {
                return root.fail("project did not open")
            }

            const named = base + "/Plan Ärger.pdf"
            if (!ctl.exportTo("pdf", "cut", "a3", named)) {
                return root.fail("export to a path with space and umlaut failed: " + ctl.status)
            }
            if (size(named) <= 0) {
                return root.fail("no file at " + named)
            }

            // A sheet chosen in the dialog becomes the project's; the Rust side checks the
            // portrait media box in the file.
            const upright = base + "/Hoch.pdf"
            if (!ctl.exportTo("pdf", "cut", "letter-portrait", upright)) {
                return root.fail("portrait export failed: " + ctl.status)
            }
            if (JSON.parse(ctl.projectJson).paper !== "letter-portrait") {
                return root.fail("the project did not keep the paper: " + ctl.projectJson)
            }
            if (!ctl.dirty) {
                return root.fail("a new paper left the project saved")
            }

            // A folder lies where the file should go: the old file cannot be replaced.
            const blocked = base + "/Ziel.pdf"
            if (!ctl.exportTo("pdf", "cut", "a4", blocked + "/inner.pdf")) {
                return root.fail("export into a new folder failed: " + ctl.status)
            }
            if (ctl.exportTo("pdf", "cut", "a3-portrait", blocked)) {
                return root.fail("export over something that cannot be replaced reported success")
            }
            // Nothing was printed on A3: the project keeps the sheet it had.
            if (JSON.parse(ctl.projectJson).paper === "a3-portrait") {
                return root.fail("a failed export changed the paper")
            }

            // The shot list as ODS and XLSX, the plan per mark as a folder of PNGs; the Rust
            // side (qml_tests.rs) checks what was written.
            for (const kind of ["ods", "xlsx"]) {
                if (!ctl.exportTo(kind, "shooting", "a4", base + "/Liste." + kind)) {
                    return root.fail(kind + " export failed: " + ctl.status)
                }
            }
            if (!ctl.exportTo("frames", "cut", "a4", base + "/frames")) {
                return root.fail("frames export failed: " + ctl.status)
            }
            if (ctl.exportTo("frames", "cut", "a4", base + "/frames")) {
                return root.fail("frames into a folder that is not empty reported success")
            }

            // LH-86: the 3D view (as the view is shown, so after `freeSize`) and the camera view
            // as PNG; the camera view needs a shot or a picked camera.
            if (ctl.exportTo("view3d", "cut", "a4", base + "/3D.png")) {
                return root.fail("the 3D view of a project never shown in 3D reported success")
            }
            ctl.toggleView()
            ctl.freeSize(800, 450)
            if (!ctl.exportTo("view3d", "cut", "a4", base + "/3D.png")) {
                return root.fail("3D view export failed: " + ctl.status)
            }
            if (ctl.exportTo("camera", "cut", "a4", base + "/Kamera.png")) {
                return root.fail("the camera view without a shot or camera reported success")
            }
            ctl.selectShot(JSON.parse(ctl.shotsJson)[0].id)
            if (!ctl.exportTo("camera", "cut", "a4", base + "/Kamera.png")) {
                return root.fail("camera view export failed: " + ctl.status)
            }

            // A plan without marks has no frames: nothing is written, and it says so.
            if (!ctl.addPlan("Leer")) {
                return root.fail("no empty plan: " + ctl.status)
            }
            if (ctl.exportTo("frames", "cut", "a4", base + "/leer")) {
                return root.fail("frames of a plan without marks reported success")
            }

            if (ctl.exportTo("zip", "cut", "a4", base + "/x.zip")) {
                return root.fail("an unknown kind reported success")
            }
            if (ctl.status === "") {
                return root.fail("an unknown kind said nothing")
            }
            Qt.exit(0)
        }
    }
}
