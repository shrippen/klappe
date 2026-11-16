import QtQuick
import QtCore
import io.github.shrippen.klappe
import "lib"

// LH-18: the path from the file dialog to the plan. The dialog's accept calls `addImage` with the
// picked file's URL and the click's place (PlanArea.qml); here the same call runs on a copy of
// the example (made by qml_tests.rs, which also checks the copied file): the image is on the plan,
// its file lies in `bilder/`, and one undo takes the image off the plan. Then the card's two
// edits (opacity, other file), each one undo step, and the lock of the notes layer.
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

    // The picture of the image on the plan, or null; the frame, which carries the key, or null.
    function picture() {
        return JSON.parse(ctl.planJson).shapes.find(s => s.kind === "image" && s.key === "") || null
    }
    function frame() {
        return JSON.parse(ctl.planJson).shapes.find(s => s.key === "image:i1") || null
    }

    Timer {
        interval: 0
        running: true
        onTriggered: {
            const base = StandardPaths.writableLocation(StandardPaths.TempLocation) + "/klappe-export-test"
            const dir = base + "/bild-projekt"
            if (!ctl.openProject(dir)) {
                return root.fail("project did not open: " + ctl.status)
            }
            ctl.setSnapping(false)
            if (picture() !== null) {
                return root.fail("the example shows an image already")
            }

            // What the dialog's accept does: a URL, as the dialog hands it over.
            const first = Qt.resolvedUrl("../../../../testdata/skizze.png").toString()
            if (!ctl.addImage(first, 10, 3) || !ctl.canUndo) {
                return root.fail("the picked file was not placed: " + ctl.status)
            }
            const placed = picture()
            if (!placed || !placed.source.endsWith("/bilder/skizze.png") || !frame()) {
                return root.fail("the image is not on the plan: " + ctl.planJson)
            }
            if (placed.width !== 2 || placed.height !== 1 || placed.opacity !== 1) {
                return root.fail("the image has the wrong size or opacity: " + JSON.stringify(placed))
            }
            if (root.size(placed.source) <= 0) {
                return root.fail("no file in bilder/: " + placed.source)
            }

            // One undo takes the image off the plan; the copy stays.
            ctl.undo()
            if (picture() !== null || frame() !== null || ctl.canUndo) {
                return root.fail("undo left the image on the plan")
            }
            if (root.size(placed.source) <= 0) {
                return root.fail("undo removed the user's file")
            }

            // A file that is no image is refused and leaves the plan alone.
            if (ctl.addImage(Qt.resolvedUrl("../../../../testdata/beispiel.klappe/projekt.toml").toString(), 10, 3)
                    || picture() !== null || ctl.canUndo) {
                return root.fail("a file that is no image changed the plan")
            }

            // The card's edits work on the selected image.
            if (!ctl.addImage(first, 10, 3)) {
                return root.fail("placing again failed: " + ctl.status)
            }
            if (ctl.setImageOpacity("40") || ctl.replaceImage(first)) {
                return root.fail("an image that is not selected took a card edit")
            }
            ctl.setTool("select", "")
            ctl.place(10, 3)
            if (ctl.selectedKey !== "image:i1") {
                return root.fail("the image is not selected: " + ctl.selectedKey)
            }

            if (ctl.setImageOpacity("5") || ctl.setImageOpacity("viel") || picture().opacity !== 1) {
                return root.fail("an opacity out of range was taken")
            }
            if (!ctl.setImageOpacity("40") || picture().opacity !== 0.4) {
                return root.fail("opacity not set: " + ctl.status)
            }
            const second = Qt.resolvedUrl("../../../../testdata/skizze2.png").toString()
            if (!ctl.replaceImage(second)) {
                return root.fail("file not replaced: " + ctl.status)
            }
            const replaced = picture()
            if (!replaced.source.endsWith("/bilder/skizze2.png") || replaced.opacity !== 0.4
                    || replaced.width !== 2 || replaced.height !== 4 || root.size(replaced.source) <= 0) {
                return root.fail("replacement lost place, width or opacity: " + JSON.stringify(replaced))
            }

            // One undo step each.
            ctl.undo()
            if (!picture().source.endsWith("/bilder/skizze.png") || picture().opacity !== 0.4) {
                return root.fail("undo of the replacement did more or less")
            }
            ctl.undo()
            if (picture().opacity !== 1 || !ctl.canUndo) {
                return root.fail("undo of the opacity did more or less")
            }

            // A locked notes layer refuses both.
            ctl.setLock("notes", "locked")
            if (ctl.setImageOpacity("40") || ctl.replaceImage(second) || picture().opacity !== 1) {
                return root.fail("the lock did not hold")
            }
            // A file dropped on the plan goes through addImage as well.
            if (ctl.addImage(first, 4, 4) || JSON.parse(ctl.planJson).shapes.filter(s => s.key === "image:i2").length) {
                return root.fail("an image was placed on a locked layer")
            }
            Qt.exit(0)
        }
    }
}
