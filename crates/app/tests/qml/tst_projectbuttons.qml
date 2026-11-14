import QtQuick
import QtTest
import "../../qml"

// ProjectButtons: New, Open and Project say what was asked; while the backup question is
// open (`locked`) none of them can be used (review of PR 23).
TestCase {
    id: tc
    name: "ProjectButtons"
    width: 600
    height: 100
    when: windowShown
    visible: true

    TestTexts { id: de; language: "de" }

    Component {
        id: buttonsComponent
        ProjectButtons {
            texts: de
        }
    }

    SignalSpy { id: newSpy; signalName: "newRequested" }
    SignalSpy { id: openSpy; signalName: "openRequested" }
    SignalSpy { id: projectSpy; signalName: "projectRequested" }

    function make() {
        const buttons = createTemporaryObject(buttonsComponent, tc)
        verify(buttons)
        for (const spy of [newSpy, openSpy, projectSpy]) {
            spy.target = buttons
            spy.clear()
        }
        return buttons
    }

    function test_each_button_says_what_was_asked() {
        const buttons = make()

        findChild(buttons, "newProjectButton").clicked()
        findChild(buttons, "openProjectButton").clicked()
        findChild(buttons, "projectButton").clicked()

        compare(newSpy.count, 1)
        compare(openSpy.count, 1)
        compare(projectSpy.count, 1)
    }

    function test_an_open_backup_question_locks_all_three() {
        const buttons = make()
        for (const name of ["newProjectButton", "openProjectButton", "projectButton"]) {
            verify(findChild(buttons, name).enabled, name)
        }

        buttons.locked = true

        for (const name of ["newProjectButton", "openProjectButton", "projectButton"]) {
            verify(!findChild(buttons, name).enabled, name)
        }
    }
}
