import QtQuick
import QtTest
import "../../qml"

// ScriptPdfDialog (DB-70): asks for paper, scene numbers and header line, then for the file;
// the project's paper is the start. The file dialog is the platform's; the tests hand it the
// chosen file with `save`.
TestCase {
    id: tc
    name: "ScriptPdfDialog"
    width: 800
    height: 600
    when: windowShown
    visible: true

    QtObject {
        id: fake
        property string status: ""
        property var calls: []

        function clearStatus() {}
        function scriptFile() { return "file:///work/Wartburg.pdf" }
        function exportScript(paper, numbers, header, revisions, path) {
            calls.push(["exportScript", paper, numbers, header, revisions, path])
            return true
        }
    }

    TestTexts { id: de; language: "de" }

    Component {
        id: dialogComponent
        ScriptPdfDialog {
            texts: de
            controller: fake
            papers: [{ id: "a4", label: "A4" }, { id: "letter", label: "Letter" }]
            projectPaper: "letter"
            versions: 1
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

    function test_it_asks_before_it_writes_and_starts_from_the_projects_paper() {
        const dialog = make()

        dialog.request()
        tryCompare(dialog, "opened", true)

        compare(dialog.title, "Drehbuch ausgeben")
        compare(findChild(dialog.contentItem, "paper").currentIndex, 1)
        verify(findChild(dialog.contentItem, "numbers").checked)
        verify(!findChild(dialog.contentItem, "header").checked)
        compare(fake.calls, [])
    }

    function test_the_choices_go_to_the_controller_with_the_file() {
        const dialog = make()
        dialog.request()
        tryCompare(dialog, "opened", true)

        findChild(dialog.contentItem, "paper").activated(0)
        dialog.numbers = false
        dialog.headerLine = true
        dialog.revisions = false
        dialog.save("file:///work/Mein Drehbuch.pdf")

        compare(fake.calls, [["exportScript", "a4", "hidden", "title", "off", "file:///work/Mein Drehbuch.pdf"]])
    }

    function test_the_folder_stays_for_the_next_suggestion() {
        const dialog = make()

        compare(dialog.suggestion(), "file:///work/Wartburg.pdf")
        dialog.save("file:///elsewhere/x.pdf")

        compare(dialog.suggestion(), "file:///elsewhere/Wartburg.pdf")
    }

    // Cancelling the file dialog writes nothing.
    function test_cancelling_writes_nothing() {
        const dialog = make()
        dialog.request()
        tryCompare(dialog, "opened", true)

        dialog.close()

        compare(fake.calls, [])
    }

    // DB-41: the switch is for a project with a locked version, on by default; without one
    // nothing is marked, whatever the switch was.
    function test_revisions_are_asked_only_with_a_locked_version() {
        const dialog = make()
        dialog.request()
        tryCompare(dialog, "opened", true)
        const marks = findChild(dialog.contentItem, "revisions")
        verify(marks.visible)
        verify(marks.checked)

        dialog.save("file:///work/A.pdf")
        compare(fake.calls[0][4], "on")

        dialog.versions = 0
        verify(!marks.visible)
        dialog.save("file:///work/B.pdf")
        compare(fake.calls[1][4], "off")
    }

    // Choosing another paper and cancelling must not leave it as the start of the next opening.
    function test_the_paper_starts_from_the_projects_again() {
        const dialog = make()
        dialog.request()
        tryCompare(dialog, "opened", true)
        const paper = findChild(dialog.contentItem, "paper")

        paper.select(0)
        compare(dialog.paper, "a4")
        dialog.close()
        tryCompare(dialog, "opened", false)

        dialog.request()
        tryCompare(dialog, "opened", true)
        compare(dialog.paper, "letter")
        compare(paper.currentIndex, 1)
    }
}
