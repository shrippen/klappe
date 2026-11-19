import QtQuick
import QtTest
import "../../qml"

// ProjectGuard (D-07, Q1-02): an action that replaces or closes the open project runs at once
// when nothing is unsaved, otherwise after the user saved or not at all.
TestCase {
    id: tc
    name: "ProjectGuard"
    width: 600
    height: 400
    when: windowShown
    visible: true

    QtObject {
        id: fake
        property bool loaded: true
        property bool dirty: false
        property bool saveWorks: true
        property int saves: 0
        function save() { saves++; return saveWorks }
    }

    TestTexts { id: de; language: "de" }

    Component {
        id: guardComponent
        ProjectGuard { texts: de; controller: fake }
    }

    property int ran: 0
    function action() { ran++ }

    function init() {
        fake.loaded = true
        fake.dirty = false
        fake.saveWorks = true
        fake.saves = 0
        ran = 0
    }

    function make() {
        return createTemporaryObject(guardComponent, tc)
    }

    function test_a_saved_project_does_not_ask() {
        const guard = make()

        guard.run(tc.action, "project.switchSave")

        compare(tc.ran, 1)
        verify(!guard.asking)
    }

    function test_without_a_project_nothing_is_asked() {
        fake.loaded = false
        fake.dirty = true
        const guard = make()

        guard.run(tc.action, "project.closeSave")

        compare(tc.ran, 1)
    }

    function test_unsaved_changes_ask_first() {
        fake.dirty = true
        const guard = make()

        guard.run(tc.action, "project.closeSave")

        verify(guard.asking)
        compare(tc.ran, 0)
        compare(fake.saves, 0)
    }

    function test_saving_then_runs_the_action() {
        fake.dirty = true
        const guard = make()
        guard.run(tc.action, "project.closeSave")

        guard.dialog.accept()

        compare(fake.saves, 1)
        compare(tc.ran, 1)
    }

    function test_a_failed_save_keeps_the_project() {
        fake.dirty = true
        fake.saveWorks = false
        const guard = make()
        guard.run(tc.action, "project.closeSave")

        guard.dialog.accept()

        compare(tc.ran, 0)
    }

    function test_cancel_runs_nothing() {
        fake.dirty = true
        const guard = make()
        guard.run(tc.action, "project.closeSave")

        guard.dialog.reject()

        compare(tc.ran, 0)
        compare(fake.saves, 0)
    }

    function test_the_button_names_the_action() {
        fake.dirty = true
        const guard = make()

        guard.run(tc.action, "project.closeSave")

        compare(guard.dialog.acceptText, "Speichern und schließen")
    }

    // Typing that has not reached the controller yet counts: the guard asks for it first.
    function test_pending_typing_is_flushed_before_asking() {
        const guard = createTemporaryObject(guardComponent, tc, { flush: () => { fake.dirty = true } })

        guard.run(tc.action, "project.closeSave")

        verify(guard.asking)
        compare(tc.ran, 0)
    }
}
