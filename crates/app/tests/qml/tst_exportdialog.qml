import QtQuick
import QtTest
import "../../qml"

// ExportDialog (LH-80, LH-81): what it sends the controller, that it starts from the project's
// paper and that the folder stays for the session. The file dialog is the platform's; the tests hand it the chosen file with `save`.
TestCase {
    id: tc
    name: "ExportDialog"
    width: 800
    height: 600
    when: windowShown
    visible: true

    QtObject {
        id: fake
        property string status: ""
        property var calls: []

        function call(name, args) {
            calls.push([name].concat(args))
            return true
        }
        function clearStatus() {}
        function exportFile(extension) { return "file:///work/Wartburg-Plan-A." + extension }
        function exportTo(kind, order, paper, path) { return call("exportTo", [kind, order, paper, path]) }
    }

    TestTexts { id: de; language: "de" }

    Component {
        id: dialogComponent
        ExportDialog {
            texts: de
            controller: fake
            papers: [{ id: "a4", label: "A4" }, { id: "a3", label: "A3" }, { id: "letter", label: "Letter" }]
            orientations: ["portrait", "landscape"]
            formats: [
                { id: "pdf", paper: true, extension: "pdf" },
                { id: "png", paper: true, extension: "png" },
                { id: "svg", paper: true, extension: "svg" },
                { id: "csv", paper: false, extension: "csv" },
                { id: "ods", paper: false, extension: "ods" },
                { id: "xlsx", paper: false, extension: "xlsx" },
                { id: "frames", paper: true, extension: null },
                { id: "view3d", paper: false, extension: "png" },
                { id: "camera", paper: false, extension: "png" }
            ]
        }
    }

    function init() {
        fake.calls = []
    }

    function make() {
        const dialog = createTemporaryObject(dialogComponent, tc)
        verify(dialog)
        return dialog
    }

    function test_a_plan_export_asks_for_the_paper_first() {
        const dialog = make()

        dialog.request("pdf")
        tryCompare(dialog, "opened", true)
        compare(findChild(dialog.contentItem, "paper").currentIndex, 0)
        compare(findChild(dialog.contentItem, "orientation").currentIndex, 1)
        compare(fake.calls, [])
    }

    // What prints on a sheet asks for it; the lists and the views of the plan do not.
    function test_only_the_formats_that_print_on_paper_ask_for_it() {
        const asks = { pdf: true, png: true, svg: true, frames: true,
                       csv: false, ods: false, xlsx: false, view3d: false, camera: false }

        for (const kind in asks) {
            const dialog = make()
            const files = findChild(dialog, "files")
            const folders = findChild(dialog, "folders")

            dialog.request(kind)

            compare(dialog.opened || dialog.visible, asks[kind], kind)
            compare(files.visible || folders.visible, !asks[kind], kind)
            files.close()
            folders.close()
            dialog.close()
        }
    }

    // A format without extension is a folder.
    function test_the_frames_ask_for_a_folder() {
        const dialog = make()
        const folders = findChild(dialog, "folders")

        dialog.request("frames")
        tryCompare(dialog, "opened", true)
        dialog.pick()

        tryCompare(folders, "visible", true)
        folders.close()
        dialog.close()
    }

    function test_it_starts_from_the_projects_paper() {
        const dialog = make()
        dialog.projectSize = "letter"
        dialog.projectOrientation = "portrait"

        dialog.request("pdf")
        tryCompare(dialog, "opened", true)

        compare(findChild(dialog.contentItem, "paper").currentIndex, 2)
        compare(findChild(dialog.contentItem, "orientation").currentIndex, 0)
        compare(dialog.paper, "letter-portrait")
    }

    function test_the_choices_are_named_in_the_language() {
        const dialog = make()
        dialog.request("pdf")
        tryCompare(dialog, "opened", true)

        compare(findChild(dialog.contentItem, "paper").model, ["A4", "A3", "Letter"])
        compare(findChild(dialog.contentItem, "orientation").model, ["Hoch", "Quer"])
    }

    function test_the_shot_list_asks_for_the_file_only() {
        const dialog = make()

        dialog.request("csv")

        verify(!dialog.opened)
        compare(dialog.kind, "csv")
    }

    // The shot list opens the file dialog straight from the menu, with the paper form closed:
    // the dialog still needs the window to show in (QA Q1-01: nothing came).
    function test_the_file_dialog_of_the_shot_list_has_the_window() {
        const dialog = make()
        const files = findChild(dialog, "files")

        dialog.request("csv")

        verify(files.parentWindow !== null)
        compare(files.parentWindow, tc.Window.window)
        files.close()
    }

    // The file dialog adds the suffix itself, before it asks about replacing a file.
    function test_the_file_dialog_carries_the_suffix_of_the_format() {
        const dialog = make()
        const files = findChild(dialog, "files")
        const suffixes = { pdf: "pdf", png: "png", svg: "svg", csv: "csv", ods: "ods", xlsx: "xlsx",
                           view3d: "png", camera: "png" }

        for (const kind in suffixes) {
            dialog.request(kind)
            dialog.pick()
            compare(files.defaultSuffix, suffixes[kind], kind)
            files.close()
            dialog.close()
        }
    }

    function test_the_paper_goes_to_the_plan_exports() {
        const dialog = make()

        dialog.request("pdf")
        tryCompare(dialog, "opened", true)
        findChild(dialog.contentItem, "paper").select(1)
        findChild(dialog.contentItem, "orientation").select(0)
        dialog.save("file:///work/a.pdf")
        dialog.kind = "png"
        dialog.save("file:///work/a.png")
        dialog.kind = "svg"
        dialog.save("file:///work/a.svg")

        compare(fake.calls, [["exportTo", "pdf", "cut", "a3-portrait", "file:///work/a.pdf"],
                             ["exportTo", "png", "cut", "a3-portrait", "file:///work/a.png"],
                             ["exportTo", "svg", "cut", "a3-portrait", "file:///work/a.svg"]])
    }

    function test_the_shot_list_follows_the_order() {
        const dialog = make()
        dialog.order = "shooting"
        dialog.kind = "csv"

        dialog.save("file:///work/shots.csv")

        compare(fake.calls, [["exportTo", "csv", "shooting", "a4-landscape", "file:///work/shots.csv"]])
    }

    function test_the_next_suggestion_is_in_the_folder_of_the_last_export() {
        const dialog = make()
        dialog.kind = "pdf"
        compare(dialog.suggestion(), "file:///work/Wartburg-Plan-A.pdf")

        dialog.save("file:///elsewhere/out.pdf")

        compare(dialog.suggestion(), "file:///elsewhere/Wartburg-Plan-A.pdf")
    }

    function test_cancelling_the_paper_writes_nothing() {
        const dialog = make()

        dialog.request("pdf")
        tryCompare(dialog, "opened", true)
        dialog.reject()
        tryCompare(dialog, "opened", false)

        compare(fake.calls, [])
    }
}
