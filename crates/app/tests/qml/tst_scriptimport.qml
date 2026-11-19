import QtQuick
import QtTest
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
        function importScript(path) { imported.push(path); return true }
    }

    TestTexts { id: de; language: "de" }

    Component {
        id: importComponent
        ScriptImport { texts: de; controller: fake }
    }

    function init() {
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
        compare(imp.picker.nameFilters[0], "Drehbuch (*.fountain *.txt)")
    }
}
