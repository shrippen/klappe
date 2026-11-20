import QtQuick
import io.github.shrippen.klappe
import "lib"
import "../../qml"

// Review of PR 287: a project opened with the same screenplay as the one before shows its scene
// lengths at once too; the text did not change, so the editor's own call does not come.
Item {
    id: root

    Watchdog {}
    Controller { id: ctl }
    Texts { id: texts; source: ctl.texts }
    ScreenplayView {
        id: view
        width: 800
        height: 600
        texts: texts
        controller: ctl
        debounceMs: 50
    }

    function fail(message) {
        console.error(message)
        Qt.exit(1)
    }

    function length() {
        const scenes = JSON.parse(ctl.projectJson).scenes
        return scenes.length > 0 ? scenes[0].length : ""
    }

    Timer {
        interval: 0
        running: true
        onTriggered: {
            const example = Qt.resolvedUrl("../../../../testdata/beispiel.klappe")
            if (!ctl.openProject(example) || root.length() === "") {
                return root.fail("first opening: no length")
            }
            if (!ctl.openProject(example)) {
                return root.fail("project did not open again")
            }
            if (root.length() === "") {
                return root.fail("the same text again shows no length")
            }
            Qt.exit(0)
        }
    }
}
