import QtQuick
import QtCore
import io.github.shrippen.klappe
import "lib"
import "../../qml"

// A project nobody touched stays saved. The screenplay view is part of the window from the start;
// its text area gives back other text than it got (CRLF becomes LF, a no-break space a space),
// which must not count as typing: after the pause the project is still unchanged. The project
// is made by the test that runs this (qml_tests.rs).
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

    Timer {
        id: after
        interval: 400
        onTriggered: {
            if (ctl.dirty) {
                return root.fail("opening made the project unsaved")
            }
            if (ctl.scriptText.indexOf("\r\n") < 0 || ctl.scriptText.indexOf(" ") < 0) {
                return root.fail("the screenplay was changed: " + JSON.stringify(ctl.scriptText))
            }
            Qt.exit(0)
        }
    }

    Timer {
        interval: 0
        running: true
        onTriggered: {
            const base = StandardPaths.writableLocation(StandardPaths.TempLocation) + "/klappe-export-test"
            if (!ctl.openProject(base + "/crlf-projekt")) {
                return root.fail("project did not open: " + ctl.status)
            }
            if (view.area.text === ctl.scriptText) {
                return root.fail("the text area gives the text back as it got it: nothing to test")
            }
            after.start()
        }
    }
}
