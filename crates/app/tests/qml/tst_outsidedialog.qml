import QtQuick
import QtTest
import "../../qml"

// OutsideDialog: the question a save asks when a file changed outside and in the window;
// overwrite saves over it, reload takes the folder, cancel changes nothing.
TestCase {
    id: tc
    name: "OutsideDialog"
    width: 900
    height: 400
    when: windowShown
    visible: true

    QtObject {
        id: fake
        property string outsideFiles: "Drehbuch"
        property var calls: []

        function reloadProject() { calls.push("reload") }
        function saveOverwrite() { calls.push("overwrite"); return true }
    }

    TestTexts { id: de; language: "de" }

    Component {
        id: askComponent
        OutsideDialog {
            controller: fake
            texts: de
        }
    }

    function init() {
        fake.calls = []
    }

    function make() {
        const ask = createTemporaryObject(askComponent, tc)
        verify(ask)
        return ask
    }

    function test_it_opens_and_names_the_files() {
        const ask = make()
        verify(!ask.dialog.opened)

        ask.open()

        tryVerify(() => ask.dialog.opened)
        compare(ask.dialog.title, "Außerhalb geändert")
        compare(ask.dialog.acceptText, "Überschreiben")
        compare(ask.dialog.rejectText, "Abbrechen")
        ask.dialog.close()
    }

    function test_accepting_overwrites() {
        const ask = make()
        ask.open()
        tryVerify(() => ask.dialog.opened)

        ask.dialog.accept()

        compare(fake.calls, ["overwrite"])
    }

    function test_cancelling_changes_nothing() {
        const ask = make()
        ask.open()
        tryVerify(() => ask.dialog.opened)

        ask.dialog.reject()

        compare(fake.calls, [])
        verify(!ask.dialog.opened)
    }

    function test_reload_takes_the_folder() {
        const ask = make()
        ask.open()
        tryVerify(() => ask.dialog.opened)

        const button = findChild(ask.dialog.footer, "reload") || findChild(ask.dialog, "reload")
        verify(button, "the reload button is in the footer")
        button.clicked()

        compare(fake.calls, ["reload"])
        tryVerify(() => !ask.dialog.opened)
    }
}
