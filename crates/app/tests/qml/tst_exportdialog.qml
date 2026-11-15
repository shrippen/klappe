import QtQuick
import QtTest
import "../../qml"

// ExportDialog (LH-80, LH-81): what it sends the controller, and that paper and folder stay for
// the session. The file dialog is the platform's; the tests hand it the chosen file with `save`.
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
        compare(fake.calls, [])
    }

    function test_the_shot_list_asks_for_the_file_only() {
        const dialog = make()

        dialog.request("csv")

        verify(!dialog.opened)
        compare(dialog.kind, "csv")
    }

    function test_the_paper_goes_to_the_plan_exports_and_stays() {
        const dialog = make()

        dialog.request("pdf")
        tryCompare(dialog, "opened", true)
        findChild(dialog.contentItem, "paper").select(1)
        dialog.save("file:///work/a.pdf")
        dialog.kind = "png"
        dialog.save("file:///work/a.png")
        dialog.kind = "svg"
        dialog.save("file:///work/a.svg")

        compare(fake.calls, [["exportTo", "pdf", "cut", "a3", "file:///work/a.pdf"],
                             ["exportTo", "png", "cut", "a3", "file:///work/a.png"],
                             ["exportTo", "svg", "cut", "a3", "file:///work/a.svg"]])
    }

    function test_the_shot_list_follows_the_order() {
        const dialog = make()
        dialog.order = "shooting"
        dialog.kind = "csv"

        dialog.save("file:///work/shots.csv")

        compare(fake.calls, [["exportTo", "csv", "shooting", "a4", "file:///work/shots.csv"]])
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
