import QtQuick
import io.github.shrippen.klappe
import "lib"

// What the window shows after each kind of change: the plain properties, and which properties
// signal a change. A property must not signal more or less often than here. The JSON texts
// are tested by their signals (they change when they signal) and for being filled.
Item {
    id: root

    Watchdog {}
    Controller { id: ctl }

    readonly property var plain: ["loaded", "title", "planTitle", "time", "duration",
        "selectedShot", "canUndo", "canRedo", "dirty", "selectedKey", "selectedCount",
        "snappingOn", "gridName", "hasBackground", "scalePoints", "measuring", "areaPoints", "asksNote",
        "backupState", "backupMessage", "sceneId", "status"]
    readonly property var texts: ["planJson", "tracksJson", "shotsJson", "shotsShootingJson", "scriptJson",
        "layersJson", "locksJson", "choicesJson", "projectJson"]
    property var counts: ({})

    // What the start shows without a project: the first scenario step is relative to it.
    readonly property var started: ({
        loaded: false, title: "", planTitle: "", time: 0, duration: 0, selectedShot: "",
        canUndo: false, canRedo: false, dirty: false, selectedKey: "", selectedCount: 0,
        snappingOn: true, gridName: "", hasBackground: false, scalePoints: 0, measuring: false,
        areaPoints: 0, asksNote: false, backupState: "", backupMessage: "", sceneId: "",
        status: ""
    })
    readonly property string planA: "12 · INT. SÄNGERSAAL – TAG · Plan A"
    readonly property string stand1: "12 · INT. SÄNGERSAAL – TAG · Stand 1"
    readonly property var opened: Object.assign({}, started, {
        loaded: true, title: "Die Wartburg-Szene", planTitle: planA, duration: 20,
        backupState: "none", sceneId: "m4djbtj6", gridName: "cm25"
    })
    readonly property var moved: Object.assign({}, opened, {
        time: 4, canUndo: true, dirty: true, selectedKey: "figure:elisabeth", selectedCount: 1
    })
    readonly property var undone: Object.assign({}, moved, { canUndo: false, canRedo: true })
    readonly property var redone: Object.assign({}, moved, { canUndo: true, canRedo: false })
    readonly property var copied: Object.assign({}, redone, {
        planTitle: stand1, time: 0, selectedKey: "", selectedCount: 0
    })
    readonly property var back: Object.assign({}, copied, {
        planTitle: planA, canUndo: true, canRedo: true
    })
    readonly property var shot: Object.assign({}, back, { selectedShot: "s12-1" })
    readonly property var unsnapped: Object.assign({}, shot, { snappingOn: false })

    // Kante's colours as the view hands them over; only the names matter here.
    readonly property string colors: JSON.stringify({
        ground: [250, 250, 250], text: [10, 10, 10], muted: [100, 100, 100], red: [200, 0, 0]
    })

    function fail(message) {
        console.error(message)
        Qt.exit(1)
    }

    // Checks the plain values, and the signals since the last step: `expected` lists them.
    function check(step, values, signals) {
        for (const name of plain) {
            if (ctl[name] !== values[name]) {
                return fail(step + ": " + name + " is " + ctl[name] + ", expected " + values[name])
            }
        }
        const got = JSON.stringify(Object.keys(counts).sort().map(n => n + "=" + counts[n]))
        const want = JSON.stringify(Object.keys(signals).sort().map(n => n + "=" + signals[n]))
        counts = ({})
        if (got !== want) {
            return fail(step + ": signals " + got + ", expected " + want)
        }
        return true
    }

    Component.onCompleted: {
        for (const name of plain.concat(texts)) {
            ctl[name + "Changed"].connect(() => { counts[name] = (counts[name] || 0) + 1 })
        }
    }

    Timer {
        interval: 0
        running: true
        onTriggered: {
            if (!root.check("start", root.started, {})) {
                return
            }

            if (!ctl.openProject(Qt.resolvedUrl("../../../../testdata/beispiel.klappe"))) {
                return root.fail("project did not open")
            }
            if (!root.check("open", root.opened, {
                loaded: 1, projectJson: 1, sceneId: 1, title: 1, planTitle: 1, duration: 1,
                choicesJson: 1, gridName: 1, planJson: 1, tracksJson: 1, shotsJson: 1, shotsShootingJson: 1, scriptJson: 1,
                layersJson: 1, locksJson: 1, backupState: 1
            })) {
                return
            }
            for (const name of root.texts) {
                if (ctl[name].length === 0) {
                    return root.fail("open: " + name + " is empty")
                }
            }
            if (JSON.parse(ctl.layersJson).notes !== true
                    || JSON.parse(ctl.locksJson).notes !== false) {
                return root.fail("open: layers or locks are not the project's")
            }

            // A counter, not a value to check in each step: the camera's picture follows it.
            const revision = ctl.pictureRevision
            ctl.seek(4)
            if (ctl.pictureRevision === revision) {
                return root.fail("seek: the picture revision did not change")
            }
            if (!root.check("seek", Object.assign({}, root.opened, { time: 4 }),
                    { planJson: 1, time: 1 })) {
                return
            }

            ctl.setTool("select", "")
            ctl.drag(6.5, 6.5, 6.5, 8.5, true)
            if (!root.check("move", root.moved, {
                planJson: 1, selectedKey: 1, selectedCount: 1, canUndo: 1, dirty: 1
            })) {
                return
            }

            ctl.undo()
            if (!root.check("undo", root.undone, { planJson: 1, canUndo: 1, canRedo: 1 })) {
                return
            }
            ctl.redo()
            if (!root.check("redo", root.redone, { planJson: 1, canUndo: 1, canRedo: 1 })) {
                return
            }

            ctl.copyPlan("Stand 1")
            if (!root.check("copy plan", root.copied, {
                projectJson: 1, planTitle: 1, planJson: 1, time: 1, selectedKey: 1,
                selectedCount: 1
            })) {
                return
            }
            ctl.undo()
            if (!root.check("undo plan", root.back, {
                projectJson: 1, planTitle: 1, canRedo: 1
            })) {
                return
            }

            ctl.selectShot(JSON.parse(ctl.shotsJson)[0].id)
            if (!root.check("shot", root.shot, { planJson: 1, tracksJson: 1, selectedShot: 1 })) {
                return
            }
            ctl.cameraImage(160, root.colors)
            const frame = JSON.parse(ctl.cameraFrame() || "{}")
            if (frame.width !== 160 || frame.height <= 0) {
                return root.fail("shot: the camera picture is empty: " + JSON.stringify(frame))
            }

            // The shot's tile: its picture at the shot's start, as wide as asked for. QML has no
            // accessor for a QImage, only its text: `QImage(QSize(160, 67), ...)`, or `QImage(null)`.
            const shotPicture = String(ctl.shotImage(JSON.parse(ctl.shotsJson)[0].id, 160, root.colors))
            if (!shotPicture.includes("QSize(160, ")) {
                return root.fail("shot: the tile picture is not 160 px wide: " + shotPicture)
            }
            const noPicture = String(ctl.shotImage("none", 160, root.colors))
            if (!noPicture.includes("QImage(null)")) {
                return root.fail("shot: an unknown shot has a picture: " + noPicture)
            }

            ctl.setLayer("notes", "hidden")
            if (!root.check("layer", root.shot, { planJson: 1, layersJson: 1 })
                    || JSON.parse(ctl.layersJson).notes !== false) {
                return
            }

            ctl.setSnapping(false)
            if (!root.check("snapping", root.unsnapped, { snappingOn: 1 })) {
                return
            }

            ctl.setGrid("cm50")
            if (!root.check("grid", Object.assign({}, root.unsnapped, { gridName: "cm50" }),
                    { gridName: 1 })) {
                return
            }
            Qt.exit(0)
        }
    }
}
