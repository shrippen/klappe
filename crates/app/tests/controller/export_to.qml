import QtQuick
import QtCore
import io.github.shrippen.klappe
import "lib"

// `exportTo` reports what happened to the file: a path with a space and an umlaut is written,
// a target that cannot be replaced is a failure even though something lies there, and an
// unknown kind is refused. The folder is removed by the test that runs this (qml_tests.rs).
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
            const example = Qt.resolvedUrl("../../../../testdata/beispiel.klappe")
            if (!ctl.openProject(example)) {
                return root.fail("project did not open")
            }

            const project = () => JSON.parse(ctl.projectJson)
            const paperOf = () => project().paper_size + "-" + project().paper_orientation

            const named = base + "/Plan Ärger.pdf"
            if (!ctl.exportTo("pdf", "cut", "a3", named)) {
                return root.fail("export to a path with space and umlaut failed: " + ctl.status)
            }
            if (size(named) <= 0) {
                return root.fail("no file at " + named)
            }

            // A name typed without the format's extension gets it (Q1-06); one that has it, in
            // any case, stays as it is.
            if (!ctl.exportTo("pdf", "cut", "a4", base + "/Ohne")) {
                return root.fail("export to a name without extension failed: " + ctl.status)
            }
            if (size(base + "/Ohne.pdf") <= 0) {
                return root.fail("no extension added: no file at " + base + "/Ohne.pdf")
            }
            if (size(base + "/Ohne") !== -1) {
                return root.fail("a file without extension was written")
            }
            if (!ctl.exportTo("svg", "cut", "a4", base + "/Gross.SVG") || size(base + "/Gross.SVG") <= 0) {
                return root.fail("a name with the extension in capitals was changed: " + ctl.status)
            }

            // The dialog asked about the typed name, not about the file written: where adding
            // the extension would land on a file that exists, nothing is written.
            const before = size(base + "/Ohne.pdf")
            if (ctl.exportTo("pdf", "cut", "a3", base + "/Ohne")) {
                return root.fail("export over Ohne.pdf by typing Ohne reported success")
            }
            if (size(base + "/Ohne.pdf") !== before) {
                return root.fail("Ohne.pdf was overwritten by typing Ohne")
            }
            if (ctl.status.indexOf("Ohne.pdf") < 0) {
                return root.fail("the refusal does not name the file: " + ctl.status)
            }

            // A sheet chosen in the dialog becomes the project's; the Rust side checks the
            // portrait media box in the file.
            const upright = base + "/Hoch.pdf"
            if (!ctl.exportTo("pdf", "cut", "letter-portrait", upright)) {
                return root.fail("portrait export failed: " + ctl.status)
            }
            if (paperOf() !== "letter-portrait") {
                return root.fail("the project did not keep the paper: " + ctl.projectJson)
            }
            if (!ctl.dirty) {
                return root.fail("a new paper left the project saved")
            }

            // A folder lies where the file should go: the old file cannot be replaced.
            const blocked = base + "/Ziel.pdf"
            if (!ctl.exportTo("pdf", "cut", "a4", blocked + "/inner.pdf")) {
                return root.fail("export into a new folder failed: " + ctl.status)
            }
            if (ctl.exportTo("pdf", "cut", "a3-portrait", blocked)) {
                return root.fail("export over something that cannot be replaced reported success")
            }
            // Nothing was printed on A3: the project keeps the sheet it had.
            if (paperOf() === "a3-portrait") {
                return root.fail("a failed export changed the paper")
            }

            // The shot list as ODS and XLSX, the plan per mark as a folder of PNGs; the Rust
            // side (qml_tests.rs) checks what was written. Only what prints a plan on a sheet
            // makes the sheet the project's.
            const kept = paperOf()
            for (const kind of ["ods", "xlsx"]) {
                if (!ctl.exportTo(kind, "shooting", "a3-portrait", base + "/Liste." + kind)) {
                    return root.fail(kind + " export failed: " + ctl.status)
                }
                if (paperOf() !== kept) {
                    return root.fail(kind + " export changed the paper to " + paperOf())
                }
            }
            const printed = [["png", "a4-portrait", "/Blatt.png"], ["svg", "a3-landscape", "/Blatt.svg"],
                             ["frames", "a4-landscape", "/frames"]]
            for (const [kind, paper, name] of printed) {
                if (!ctl.exportTo(kind, "cut", paper, base + name)) {
                    return root.fail(kind + " export failed: " + ctl.status)
                }
                if (paperOf() !== paper) {
                    return root.fail(kind + " export did not keep the paper " + paper + ": " + paperOf())
                }
            }
            if (ctl.exportTo("frames", "cut", "a4", base + "/frames")) {
                return root.fail("frames into a folder that is not empty reported success")
            }

            // The offered file names: the PNGs of the views beside the plan's under their own
            // name, the frames a folder without extension.
            const offered = { pdf: ".pdf", view3d: "-3d.png", camera: "-camera.png" }
            for (const kind in offered) {
                if (!ctl.exportFile(kind).endsWith(offered[kind])) {
                    return root.fail("the file offered for " + kind + " is " + ctl.exportFile(kind))
                }
            }
            if (ctl.exportFile("frames") + ".pdf" !== ctl.exportFile("pdf")) {
                return root.fail("the folder offered for frames is not the plan's name")
            }

            // LH-86: the 3D view (as the view is shown, so after `freeSize`) and the camera view
            // as PNG; the camera view needs a shot or a picked camera. They print nothing on a
            // sheet, so the paper stays.
            const sheet = paperOf()
            if (ctl.exportTo("view3d", "cut", "a3-portrait", base + "/3D.png")) {
                return root.fail("the 3D view of a project never shown in 3D reported success")
            }
            ctl.toggleView()
            ctl.freeSize(800, 450)
            if (!ctl.exportTo("view3d", "cut", "a3-portrait", base + "/3D.png")) {
                return root.fail("3D view export failed: " + ctl.status)
            }
            if (ctl.exportTo("camera", "cut", "a3-portrait", base + "/Kamera.png")) {
                return root.fail("the camera view without a shot or camera reported success")
            }
            ctl.selectShot(JSON.parse(ctl.shotsJson)[0].id)
            if (!ctl.exportTo("camera", "cut", "a3-portrait", base + "/Kamera.png")) {
                return root.fail("camera view export failed: " + ctl.status)
            }
            if (paperOf() !== sheet) {
                return root.fail("a view export changed the paper to " + paperOf())
            }

            // A plan without marks has no frames: nothing is written, and it says so. A new plan of
            // scene 12 holds its speakers (LH-09), so the empty one is in a scene the script lacks.
            if (!ctl.addScene("99", "INT. LEER – TAG", "saengersaal")
                    || !ctl.selectScene(project().scenes.find(s => s.number === "99").id)) {
                return root.fail("no empty plan: " + ctl.status)
            }
            if (ctl.exportTo("frames", "cut", "a4", base + "/leer")) {
                return root.fail("frames of a plan without marks reported success")
            }

            // DB-61: a report is written as the format says, a name without its extension gets
            // it, an unknown report or format is refused, and the file offered is a PDF.
            if (!ctl.exportReport("szenen", "csv", base + "/Szenen.csv") || size(base + "/Szenen.csv") <= 0) {
                return root.fail("report export failed: " + ctl.status)
            }
            if (!ctl.exportReport("figuren", "xlsx", base + "/Figuren") || size(base + "/Figuren.xlsx") <= 0) {
                return root.fail("report without extension failed: " + ctl.status)
            }
            if (ctl.status.indexOf("Figuren.xlsx") < 0) {
                return root.fail("the saved message does not name the file: " + ctl.status)
            }
            if (ctl.exportReport("nope", "csv", base + "/x.csv") || ctl.exportReport("szenen", "png", base + "/x.png")) {
                return root.fail("an unknown report or format reported success")
            }
            // The name follows the language of the surface (CI runs in English).
            if (!/\/(Drehbuchauszug|Breakdown-extract)\.pdf$/.test(ctl.reportFile("auszug"))) {
                return root.fail("the file offered for a report is " + ctl.reportFile("auszug"))
            }

            if (ctl.exportTo("zip", "cut", "a4", base + "/x.zip")) {
                return root.fail("an unknown kind reported success")
            }
            if (ctl.status === "") {
                return root.fail("an unknown kind said nothing")
            }
            Qt.exit(0)
        }
    }
}
