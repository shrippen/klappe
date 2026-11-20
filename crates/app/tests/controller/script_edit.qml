import QtQuick
import io.github.shrippen.klappe
import "lib"

// E-09: the screenplay as text in the window. `editScript` replaces the text, which stays
// unsaved. `importScript` takes a Fountain file; a file that is not text or not there changes
// nothing and says so in the status line.
Item {
    Watchdog {}
    Controller { id: ctl }

    function fail(message) {
        console.error(message)
        Qt.exit(1)
    }

    // The text line of each scene whose heading the text holds, in order.
    function lines() {
        return JSON.parse(ctl.projectJson).scenes.map(e => e.line).filter(l => l !== null)
    }

    Timer {
        interval: 0
        running: true
        onTriggered: {
            const example = Qt.resolvedUrl("../../../../testdata/beispiel.klappe")
            if (!ctl.openProject(example)) {
                return fail("project did not open")
            }

            const original = ctl.scriptText
            const originalLines = lines()
            if (original === "" || originalLines.length === 0 || ctl.dirty) {
                return fail("the example has a screenplay, saved: " + original.length + " " + ctl.dirty)
            }

            const typed = "# Akt\n\nINT. KELLER - NACHT\n\nEs tropft.\n\nEXT. HOF - TAG\n\nSonne.\n"
            ctl.editScript(typed)
            if (ctl.scriptText !== typed || !ctl.dirty) {
                return fail("the typed text must be the screenplay and unsaved")
            }
            // The same text again changes nothing.
            ctl.editScript(typed)
            if (ctl.scriptText !== typed) {
                return fail("the same text again")
            }

            // A file that is not text, and one that is not there.
            if (ctl.importScript(Qt.resolvedUrl("../../../../testdata/skizze.png")) || ctl.status === "") {
                return fail("a PNG is no screenplay")
            }
            if (ctl.importScript(Qt.resolvedUrl("../../../../testdata/gibt-es-nicht.fountain")) || ctl.scriptText !== typed) {
                return fail("a missing file must change nothing")
            }

            // The project's own screenplay file, loaded again over the typed text.
            if (!ctl.importScript(example + "/drehbuch.fountain")) {
                return fail("the file was refused: " + ctl.status)
            }
            if (ctl.scriptText !== original || JSON.stringify(lines()) !== JSON.stringify(originalLines)) {
                return fail("the import did not replace the text")
            }
            if (ctl.status === "") {
                return fail("an import says so in the status line")
            }
            const report = JSON.parse(ctl.importReport)
            if (report.scenes !== originalLines.length || report.created !== 0) {
                return fail("the import report: " + ctl.importReport)
            }
            Qt.exit(0)
        }
    }
}
