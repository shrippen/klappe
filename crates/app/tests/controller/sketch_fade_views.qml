import QtQuick
import io.github.shrippen.klappe
import "lib"

// LH-120, LH-15, LH-119: the sketch pen draws a stroke per drag (one undo step) and one command
// clears them all; a layer fades in steps (the plan's shapes carry the strength); a saved view
// is listed by the project view, chosen again and deleted.
Item {
    Watchdog {}
    Controller { id: ctl }

    function fail(message) {
        console.error(message)
        Qt.exit(1)
    }

    function shapes() {
        return JSON.parse(ctl.planJson).shapes
    }

    // The red solid polylines without a key: the strokes of the pen.
    function strokes() {
        return shapes().filter(s => s.kind === "polyline" && s.label === "red" && s.key === "")
    }

    function opacityOf(prefix) {
        const found = shapes().filter(s => String(s.key).indexOf(prefix) === 0)
        return found.length === 0 ? -1 : found[0].opacity
    }

    Timer {
        interval: 0
        running: true
        onTriggered: {
            if (!ctl.openProject(Qt.resolvedUrl("../../../../testdata/beispiel.klappe"))) {
                return fail("project did not open")
            }

            // The pen: the drag so far shows, the release makes one stroke over the plan.
            if (!ctl.setTool("sketch", "")) {
                return fail("the sketch tool is unknown")
            }
            ctl.sketchDrag(2, 3, 2.5, 3, false)
            ctl.sketchDrag(2, 3, 3, 3.2, false)
            if (ctl.canUndo || strokes().length !== 0) {
                return fail("an unfinished stroke is no edit")
            }
            if (!ctl.sketchDrag(2, 3, 3.5, 3.6, true) || !ctl.canUndo) {
                return fail("the stroke was not made: " + ctl.status)
            }
            if (strokes().length !== 1 || strokes()[0].points.length !== 8) {
                return fail("the stroke is not on the plan: " + JSON.stringify(strokes()))
            }
            if (JSON.parse(ctl.fadesJson).sketches !== "100" || JSON.parse(ctl.layersJson).sketches !== true) {
                return fail("the sketch layer is missing")
            }
            ctl.undo()
            if (strokes().length !== 0 || ctl.canUndo) {
                return fail("one undo does not take the stroke back")
            }
            ctl.redo()
            ctl.sketchDrag(5, 5, 6, 6, true)
            if (strokes().length !== 2) {
                return fail("a second stroke is missing")
            }
            if (!ctl.clearSketches() || strokes().length !== 0) {
                return fail("clearing left strokes")
            }
            ctl.undo()
            if (strokes().length !== 2) {
                return fail("undo does not bring the strokes back")
            }
            ctl.setLock("sketches", "locked")
            if (ctl.sketchDrag(7, 7, 8, 8, true) || strokes().length !== 2 || ctl.status === "") {
                return fail("a locked sketch layer took a stroke")
            }
            ctl.setLock("sketches", "unlocked")
            ctl.setTool("select", "")

            // Fading: each step is paler, the others stay; it is no edit.
            const canUndo = ctl.canUndo
            if (opacityOf("figure:") !== 1) {
                return fail("figures start at full strength: " + opacityOf("figure:"))
            }
            ctl.cycleFade("figures")
            if (JSON.parse(ctl.fadesJson).figures !== "60" || opacityOf("figure:") !== 0.6) {
                return fail("the first step is 60 %: " + ctl.fadesJson + " " + opacityOf("figure:"))
            }
            if (opacityOf("wall:") !== 1) {
                return fail("a wall faded with the figures")
            }
            ctl.cycleFade("figures")
            if (JSON.parse(ctl.fadesJson).figures !== "30" || opacityOf("figure:") !== 0.3) {
                return fail("the second step is 30 %")
            }
            ctl.cycleFade("figures")
            if (JSON.parse(ctl.fadesJson).figures !== "100" || opacityOf("figure:") !== 1) {
                return fail("the third click is not back to full")
            }
            if (ctl.canUndo !== canUndo) {
                return fail("fading is a view setting, not an edit")
            }
            if (JSON.stringify(JSON.parse(ctl.choicesJson).fade) !== JSON.stringify(["100", "60", "30"])) {
                return fail("the steps are not listed: " + ctl.choicesJson)
            }

            // Saved views: kept in the project, chosen again, deleted.
            ctl.setLayer("notes", "hidden")
            ctl.setTrack("b", "hidden")
            if (!ctl.saveView("Nur Kamera A")) {
                return fail("the view was not saved: " + ctl.status)
            }
            const views = () => JSON.parse(ctl.projectJson).views
            if (views().length !== 1 || views()[0].name !== "Nur Kamera A") {
                return fail("the project view does not list the view: " + ctl.projectJson)
            }
            ctl.setLayer("notes", "shown")
            ctl.setTrack("b", "shown")
            if (!ctl.applyView(views()[0].id)) {
                return fail("the view was not chosen: " + ctl.status)
            }
            if (JSON.parse(ctl.layersJson).notes !== false) {
                return fail("the view did not hide the notes")
            }
            if (!ctl.removeView(views()[0].id) || views().length !== 0) {
                return fail("the view was not deleted")
            }
            if (ctl.saveView("  ")) {
                return fail("a view without a name was saved")
            }

            Qt.exit(0)
        }
    }
}
