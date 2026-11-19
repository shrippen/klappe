import QtQuick
import QtQuick.Window
import io.github.shrippen.klappe
import "lib"
import "lib/count.js" as Count

// The shot tiles with the real controller: a change to the plan, its undo and redo paint the
// tiles again; a selection and the playhead do not (NF-05). The stand-in surface counts the
// pictures. `projectJson` (the overview) does not hold the plan, so it cannot be the trigger.
Item {
    id: root

    Watchdog {}
    Controller { id: ctl }

    readonly property url tilesUrl: "qrc:/qt/qml/io/github/shrippen/klappe/qml/ShotTiles.qml"
    property int seen: -1
    property int stillTicks: 0
    property var steps: []
    property string stepName: ""

    function fail(message) {
        console.error(message)
        Qt.exit(1)
    }

    Window {
        id: window
        width: 600
        height: 500
        visible: true
    }

    // Runs `change`, waits until the count of pictures rests, and then `expect` tells if the
    // tiles were painted again (`true`) or not.
    function after(name, change, repainted) {
        steps.push({ name: name, change: change, repainted: repainted })
    }

    Timer {
        id: tick
        interval: 100
        repeat: true
        running: false
        onTriggered: {
            if (Count.shows !== root.seen) {
                root.seen = Count.shows
                root.stillTicks = 0
                return
            }
            // 8 ticks without a picture: the tiles are at rest (a tick of theirs is 120 ms).
            if (++root.stillTicks < 8) {
                return
            }
            root.stillTicks = 0
            if (root.stepName !== "") {
                const step = root.steps.shift()
                const painted = Count.shows > root.before
                if (painted !== step.repainted) {
                    return root.fail(step.name + ": the tiles were " + (painted ? "" : "not ")
                        + "painted again, expected " + (step.repainted ? "" : "not ") + "(pictures: "
                        + root.before + " -> " + Count.shows + ")")
                }
            }
            if (root.steps.length === 0) {
                return Qt.exit(0)
            }
            root.stepName = root.steps[0].name
            root.before = Count.shows
            root.steps[0].change()
        }
    }
    property int before: 0

    Timer {
        interval: 0
        running: true
        onTriggered: {
            if (!ctl.openProject(Qt.resolvedUrl("../../../../testdata/beispiel.klappe"))) {
                return root.fail("project did not open")
            }
            // The rows as the panel hands them over: the labels come from the camera list.
            const shots = JSON.parse(ctl.shotsJson).map(row => Object.assign(
                { camera_label: "A", lens_label: "35 mm", short_name: "", axis_jump: false }, row))
            const component = Qt.createComponent(root.tilesUrl)
            if (component.status !== Component.Ready) {
                return root.fail("ShotTiles does not load: " + component.errorString())
            }
            component.createObject(window.contentItem, {
                texts: { t: key => key }, controller: ctl, shots: shots,
                width: 600, height: 500, surfaceSource: Qt.resolvedUrl("lib/CountSurface.qml")
            })

            root.after("select", () => ctl.selectShot(shots[shots.length - 1].id), false)
            root.after("playhead", () => ctl.seek(4), false)
            root.after("move", () => {
                ctl.setTool("select", "")
                ctl.drag(6.5, 6.5, 6.5, 8.5, true)
            }, true)
            root.after("undo", () => ctl.undo(), true)
            root.after("redo", () => ctl.redo(), true)
            tick.start()
        }
    }
}
