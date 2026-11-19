import QtQuick
import io.github.shrippen.klappe
import "lib"

// `openBlank` (D-07): a new project opens with one scene and its empty plan on view, saved, so
// closing asks nothing; a second one gets the next free folder. The data folder is the test's
// own (qml_tests.rs sets XDG_DATA_HOME).
Item {
    id: root

    Watchdog {}
    Controller { id: ctl }

    function fail(message) {
        console.error(message)
        Qt.exit(1)
    }

    Timer {
        interval: 0
        running: true
        onTriggered: {
            if (!ctl.openBlank() || !ctl.loaded) {
                return root.fail("the blank plan did not open: " + ctl.status)
            }

            const project = JSON.parse(ctl.projectJson)
            if (project.scenes.length !== 1 || project.plans.length !== 1 || project.plan !== "plan-a") {
                return root.fail("one scene with its plan expected: " + ctl.projectJson)
            }
            if (ctl.sceneId === "" || ctl.planJson === "" || ctl.dirty || ctl.title !== "Unbenannt") {
                return root.fail("the blank plan must show its plan, saved, titled: " + ctl.title)
            }

            const first = JSON.parse(ctl.recentJson)[0].path
            if (!first.endsWith("/entwuerfe/unbenannt-1.klappe")) {
                return root.fail("not in the data folder: " + first)
            }

            if (!ctl.openBlank() || JSON.parse(ctl.recentJson)[0].path === first || ctl.title !== "Unbenannt 2") {
                return root.fail("the second blank plan needs its own folder: " + ctl.title)
            }
            Qt.exit(0)
        }
    }
}
