import QtQuick
import QtCore
import io.github.shrippen.klappe
import "lib"

// The screenplay as PDF (DB-70) and its setup (DN-05, DB-31, DB-33): a project without
// screenplay writes nothing; the paper chosen for the output stays in the project; a name typed
// without `.pdf` gets it and never replaces a file; the project form's setup is kept whole, the
// words are German ones only. The project is the test's own.
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
            const dir = base + "/script-pdf-test"
            const script = () => JSON.parse(ctl.projectJson).script
            if (!ctl.createProject(dir, "Nachtwache")) {
                return root.fail("project did not open: " + ctl.status)
            }

            // The file offered is named by the title, next to the project folder.
            if (!ctl.scriptFile().endsWith("/Nachtwache.pdf")) {
                return root.fail("the file offered is " + ctl.scriptFile())
            }

            if (ctl.exportScript("a4", true, false, base + "/Leer.pdf") || size(base + "/Leer.pdf") !== -1) {
                return root.fail("a project without screenplay wrote a PDF")
            }

            ctl.editScript("INT. SAAL - TAG\n\nAnna tritt ein.\n")
            if (!ctl.exportScript("letter", true, true, base + "/Buch Ärger")) {
                return root.fail("export failed: " + ctl.status)
            }
            if (size(base + "/Buch Ärger.pdf") <= 0) {
                return root.fail("no file at " + base + "/Buch Ärger.pdf")
            }
            if (script().paper !== "letter") {
                return root.fail("the project did not keep the paper: " + ctl.projectJson)
            }
            if (ctl.status.indexOf("Buch Ärger.pdf") < 0) {
                return root.fail("the status does not name the file: " + ctl.status)
            }

            // Typing the name without the extension does not replace the file under it.
            if (ctl.exportScript("a4", true, true, base + "/Buch Ärger")) {
                return root.fail("export over an existing file by typing its name reported success")
            }
            if (script().paper !== "letter") {
                return root.fail("a refused export changed the paper")
            }

            // The form: the setup is kept whole, English words are never international.
            if (!ctl.setScriptSetup("de", "international", "a4", true, "automatic", 5)) {
                return root.fail("the setup was refused: " + ctl.status)
            }
            let kept = script()
            if (kept.words !== "international" || kept.paper !== "a4" || !kept.contd_after_action
                    || kept.scene_numbering !== "automatic" || kept.first_scene !== 5) {
                return root.fail("the setup is not kept: " + JSON.stringify(kept))
            }
            if (!ctl.setScriptSetup("en", "international", "a4", true, "automatic", 5)) {
                return root.fail("the english setup was refused: " + ctl.status)
            }
            if (script().words !== "local" || script().language !== "en") {
                return root.fail("English words stayed international: " + JSON.stringify(script()))
            }

            Qt.exit(0)
        }
    }
}
