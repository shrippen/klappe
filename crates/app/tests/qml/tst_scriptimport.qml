import QtQuick
import QtTest
import QtQuick.Controls as QQC2
import "../../qml"

// ScriptImport (E-09): a project with a screenplay asks before the file dialog; an empty one
// goes straight to it.
TestCase {
    id: tc
    name: "ScriptImport"
    width: 600
    height: 400
    when: windowShown
    visible: true

    QtObject {
        id: fake
        property string scriptText: ""
        property var imported: []
        property string importReport: ""
        function importScript(path) { imported.push(path); return importReport !== "" }
    }

    TestTexts { id: de; language: "de" }

    Component {
        id: importComponent
        ScriptImport { texts: de; controller: fake }
    }

    function init() {
        fake.importReport = ""
        fake.scriptText = ""
        fake.imported = []
    }

    function test_an_empty_screenplay_goes_to_the_file_dialog() {
        const imp = createTemporaryObject(importComponent, tc)

        imp.start()

        verify(!imp.dialog.opened)
        verify(imp.picker.visible || imp.picker.title !== "")
    }

    function test_a_screenplay_asks_first() {
        fake.scriptText = "INT. HALLE - TAG\n"
        const imp = createTemporaryObject(importComponent, tc)

        imp.start()

        tryVerify(() => imp.dialog.opened)
        compare(imp.dialog.acceptText, "Ersetzen")
    }

    function test_a_blank_screenplay_does_not_ask() {
        fake.scriptText = "\n  \n"
        const imp = createTemporaryObject(importComponent, tc)

        imp.start()

        verify(!imp.dialog.opened)
    }

    function test_cancel_leaves_everything() {
        fake.scriptText = "INT. HALLE - TAG\n"
        const imp = createTemporaryObject(importComponent, tc)
        imp.start()
        tryVerify(() => imp.dialog.opened)

        imp.dialog.reject()

        compare(fake.imported.length, 0)
        verify(!imp.picker.visible)
    }

    function test_the_file_dialog_is_named_in_german() {
        const imp = createTemporaryObject(importComponent, tc)

        compare(imp.picker.title, "Drehbuch laden")
        compare(imp.picker.nameFilters[0], "Drehbuch (*.fountain *.txt *.fdx)")
    }

    function report(pending, notInBook) {
        fake.importReport = JSON.stringify({ scenes: 14, created: 3, pending: pending, notInBook: notInBook })
        const imp = createTemporaryObject(importComponent, tc)
        imp.load("file:///buch.fountain")
        return imp
    }

    // One scene and one question read in the singular, not "1 Szenen", "1 Fragen".
    function test_one_scene_and_one_question_are_singular() {
        fake.importReport = JSON.stringify({ scenes: 1, created: 1, pending: 1, notInBook: 0 })
        const imp = createTemporaryObject(importComponent, tc)
        imp.load("file:///buch.fountain")

        tryVerify(() => imp.report.opened)
        compare(findChild(imp.report.contentItem, "reportCounts").text, "1 Szene erkannt · 1 neu · 1 Frage")
    }

    function test_the_report_shows_the_counts() {
        const imp = report(2, 0)

        tryVerify(() => imp.report.opened)
        compare(findChild(imp.report.contentItem, "reportCounts").text, "14 Szenen erkannt · 3 neu · 2 Fragen")
        verify(!findChild(imp.report.contentItem, "reportGone").visible)
    }

    function test_the_report_names_emphasis_an_fdx_import_lost() {
        fake.importReport = JSON.stringify({ scenes: 1, created: 0, pending: 0, notInBook: 0, stylesLost: 2 })
        const imp = createTemporaryObject(importComponent, tc)
        imp.load("file:///buch.fdx")

        tryVerify(() => imp.report.opened)
        const lost = findChild(imp.report.contentItem, "reportStyles")
        verify(lost.visible)
        compare(lost.text, "2 Hervorhebungen nicht übernommen")
    }

    function test_the_report_hides_the_emphasis_line_without_loss() {
        const imp = report(0, 0)

        tryVerify(() => imp.report.opened)
        verify(!findChild(imp.report.contentItem, "reportStyles").visible)
    }

    function test_the_report_names_entries_not_in_the_book() {
        const imp = report(0, 4)

        tryVerify(() => imp.report.opened)
        compare(findChild(imp.report.contentItem, "reportGone").text, "4 nicht im Buch")
    }

    function test_questions_are_offered_and_open_the_book_dialog() {
        const imp = report(2, 0)
        const spy = createTemporaryObject(spyComponent, tc, { target: imp })
        tryVerify(() => imp.report.opened)
        compare(imp.report.rejectText, "Später")

        imp.report.accept()

        compare(spy.count, 1)
    }

    function test_without_questions_there_is_nothing_to_answer() {
        const imp = report(0, 0)

        tryVerify(() => imp.report.opened)
        compare(imp.report.standardButtons, QQC2.Dialog.Close)
        compare(imp.report.rejectText, "Schließen")
    }

    function test_a_refused_file_shows_no_report() {
        const imp = createTemporaryObject(importComponent, tc)

        imp.load("file:///kaputt.png")

        verify(!imp.report.opened)
    }

    Component {
        id: spyComponent
        SignalSpy { signalName: "questionsRequested" }
    }
}
