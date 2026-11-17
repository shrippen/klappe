import QtQuick
import io.github.shrippen.klappe
import "lib"

// `speechJson` (LH-64): the dialogue of the scene on view with the time each line is read at;
// `null` without a project.
Item {
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
            if (ctl.speechJson() !== "null") {
                return fail("no project, no speech: " + ctl.speechJson())
            }
            if (!ctl.openProject(Qt.resolvedUrl("../../../../testdata/beispiel.klappe"))) {
                return fail("project did not open")
            }

            const speech = JSON.parse(ctl.speechJson())
            if (speech.language !== "de" && speech.language !== "en") {
                return fail("language: " + speech.language)
            }
            const lines = speech.lines.map(l => l.at_s + " " + l.text)
            if (lines.join("|") !== "6 Ihr kommt spät.|9 Der Weg war lang.") {
                return fail("lines: " + lines.join("|"))
            }
            Qt.exit(0)
        }
    }
}
