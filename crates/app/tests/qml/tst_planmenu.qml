import QtQuick
import QtTest
import "../../qml"

// PlanMenu (LH-04, D-03): what the menu lists and shows, and what its entries and their forms
// send to the controller.
TestCase {
    id: tc
    name: "PlanMenu"
    width: 1000
    height: 600
    when: windowShown
    visible: true

    QtObject {
        id: fake
        property string status: ""
        property bool accept: true
        property var calls: []
        property var plans: [
            { id: "plan-a", name: "Plan A", shots: 5 },
            { id: "plan-a-2", name: "Plan A (2)", shots: 5 },
            { id: "plan-b", name: "Plan B", shots: 0 }
        ]
        property string plan: "plan-a"
        property string projectJson: JSON.stringify({ plans: plans, plan: plan })

        function reshow() { projectJson = JSON.stringify({ plans: plans, plan: plan }) }
        function call(name, args) {
            calls.push([name].concat(args))
            if (!accept) {
                status = "Refused"
            }
            return accept
        }
        function clearStatus() { status = "" }
        function selectPlan(id) { return call("selectPlan", [id]) }
        function addPlan(name) { return call("addPlan", [name]) }
        function copyPlan(name) { return call("copyPlan", [name]) }
        function renamePlan(id, name) { return call("renamePlan", [id, name]) }
        function removePlan(id) { return call("removePlan", [id]) }
        function removePlanWithShots(id) { return call("removePlanWithShots", [id]) }
    }

    TestTexts { id: de; language: "de" }
    TestTexts { id: en; language: "en" }

    Component {
        id: chooserComponent
        PlanMenu {
            controller: fake
            texts: de
        }
    }

    function init() {
        fake.status = ""
        fake.accept = true
        fake.calls = []
        fake.plans = [
            { id: "plan-a", name: "Plan A", shots: 5 },
            { id: "plan-a-2", name: "Plan A (2)", shots: 5 },
            { id: "plan-b", name: "Plan B", shots: 0 }
        ]
        fake.plan = "plan-a"
        fake.reshow()
    }

    function make() {
        const chooser = createTemporaryObject(chooserComponent, tc)
        verify(chooser)
        return chooser
    }

    // The entry of the menu, found by its object name; `open` makes the menu show it.
    function entry(chooser, name) {
        chooser.menu.open()
        const found = findChild(chooser.menu.contentItem, name) || findChild(chooser.menu, name)
        verify(found, name)
        return found
    }

    function pick(chooser, name) {
        const item = entry(chooser, name)
        verify(item.enabled, name)
        // Not a click: a theme with native menus takes none from a test.
        verify(item.action || item.onView !== undefined, name)
        if (item.action) {
            item.action.trigger()
            return
        }
        // A click toggles the tick, then triggers.
        item.toggle()
        item.triggered()
    }

    function test_the_button_shows_the_plan_on_view() {
        const chooser = make()
        compare(chooser.text, "Plan A")

        fake.plan = "plan-b"
        fake.reshow()
        compare(chooser.text, "Plan B")
    }

    function test_the_menu_lists_the_plans_with_a_tick_on_the_one_on_view() {
        const chooser = make()
        fake.plan = "plan-a-2"
        fake.reshow()

        compare(entry(chooser, "plan.plan-a").text, "Plan A")
        compare(entry(chooser, "plan.plan-a-2").text, "Plan A (2)")
        compare(entry(chooser, "plan.plan-b").text, "Plan B")
        verify(!entry(chooser, "plan.plan-a").checked)
        verify(entry(chooser, "plan.plan-a-2").checked)
        verify(!entry(chooser, "plan.plan-b").checked)
    }

    function test_choosing_a_plan_sends_its_id() {
        const chooser = make()

        pick(chooser, "plan.plan-a-2")

        compare(fake.calls, [["selectPlan", "plan-a-2"]])
    }

    function test_a_refused_choice_keeps_the_tick_on_the_plan_on_view() {
        const chooser = make()
        fake.accept = false

        pick(chooser, "plan.plan-b")

        verify(entry(chooser, "plan.plan-a").checked)
        verify(!entry(chooser, "plan.plan-b").checked)
    }

    function test_choosing_the_plan_on_view_keeps_its_tick() {
        const chooser = make()

        pick(chooser, "plan.plan-a")

        verify(entry(chooser, "plan.plan-a").checked)
    }

    function test_the_menu_follows_a_rename() {
        const chooser = make()
        fake.plan = "plan-b"
        fake.reshow()

        fake.plans = [
            { id: "plan-a", name: "Plan A", shots: 5 },
            { id: "plan-a-2", name: "Plan A (2)", shots: 5 },
            { id: "plan-b", name: "Fassung 3", shots: 0 }
        ]
        fake.reshow()

        compare(chooser.text, "Fassung 3")
        compare(entry(chooser, "plan.plan-b").text, "Fassung 3")
        verify(entry(chooser, "plan.plan-b").checked)
    }

    function test_a_new_plan_may_leave_its_name_empty() {
        const chooser = make()
        const form = findChild(chooser, "planForm")

        pick(chooser, "newPlan")
        tryCompare(form, "opened", true)
        compare(form.title, "Neuer Plan")
        verify(form.acceptEnabled)
        form.accept()

        compare(fake.calls, [["addPlan", ""]])
    }

    function test_a_new_plan_sends_the_name_typed() {
        const chooser = make()
        const form = findChild(chooser, "planForm")

        pick(chooser, "newPlan")
        tryCompare(form, "opened", true)
        findChild(form, "nameField").text = "Mit Pferd"
        form.accept()

        compare(fake.calls, [["addPlan", "Mit Pferd"]])
    }

    function test_a_snapshot_copies_under_the_name_typed() {
        const chooser = make()
        const form = findChild(chooser, "planForm")

        pick(chooser, "snapshotPlan")
        tryCompare(form, "opened", true)
        compare(form.title, "Schnappschuss")
        findChild(form, "nameField").text = "Vor dem Umbau"
        form.accept()

        compare(fake.calls, [["copyPlan", "Vor dem Umbau"]])
    }

    function test_renaming_starts_with_the_name_and_needs_one() {
        const chooser = make()
        const form = findChild(chooser, "planForm")
        fake.plan = "plan-a-2"
        fake.reshow()

        pick(chooser, "renamePlan")
        tryCompare(form, "opened", true)
        compare(findChild(form, "nameField").text, "Plan A (2)")
        findChild(form, "nameField").text = "  "
        verify(!form.acceptEnabled)
        findChild(form, "nameField").text = "Stand 1"
        verify(form.acceptEnabled)
        form.accept()

        compare(fake.calls, [["renamePlan", "plan-a-2", "Stand 1"]])
    }

    function test_a_refusal_keeps_the_form_open_with_the_message() {
        const chooser = make()
        const form = findChild(chooser, "planForm")
        fake.accept = false

        pick(chooser, "newPlan")
        tryCompare(form, "opened", true)
        findChild(form, "nameField").text = "Plan A"
        form.accept()

        tryCompare(form, "opened", true)
        compare(findChild(form, "error").text, "Refused")
        compare(findChild(form, "nameField").text, "Plan A")
    }

    function test_deleting_asks_first_and_names_the_plan() {
        const chooser = make()
        const confirm = findChild(chooser, "confirm")
        fake.plan = "plan-b"
        fake.reshow()

        pick(chooser, "removePlan")
        tryCompare(confirm, "opened", true)
        compare(findChild(confirm, "question").text,
                "Plan Plan B löschen? Rückgängig stellt ihn wieder her.")
        confirm.reject()
        compare(fake.calls, [])

        pick(chooser, "removePlan")
        tryCompare(confirm, "opened", true)
        confirm.accept()
        compare(fake.calls, [["removePlan", "plan-b"]])
    }

    function test_deleting_a_plan_with_shots_names_their_number_and_takes_them() {
        const chooser = make()
        const confirm = findChild(chooser, "confirm")

        pick(chooser, "removePlan")
        tryCompare(confirm, "opened", true)
        compare(findChild(confirm, "question").text,
                "Plan Plan A und seine 5 Shots löschen? Rückgängig stellt alles wieder her.")
        confirm.reject()
        compare(fake.calls, [])

        pick(chooser, "removePlan")
        tryCompare(confirm, "opened", true)
        confirm.accept()
        compare(fake.calls, [["removePlanWithShots", "plan-a"]])
    }

    function test_the_question_is_in_english_and_counts_one_shot() {
        const chooser = make()
        chooser.texts = en
        const confirm = findChild(chooser, "confirm")
        fake.plans = [
            { id: "plan-a", name: "Plan A", shots: 1 },
            { id: "plan-b", name: "Plan B", shots: 3 }
        ]
        fake.reshow()

        pick(chooser, "removePlan")
        tryCompare(confirm, "opened", true)
        compare(findChild(confirm, "question").text,
                "Delete plan Plan A and its 1 shot? Undo brings both back.")
        confirm.reject()

        fake.plan = "plan-b"
        fake.reshow()
        pick(chooser, "removePlan")
        tryCompare(confirm, "opened", true)
        compare(findChild(confirm, "question").text,
                "Delete plan Plan B and its 3 shots? Undo brings them all back.")
    }

    function test_deleting_stands_apart_and_is_destructive() {
        const chooser = make()

        verify(entry(chooser, "removePlan").destructive)
        verify(!entry(chooser, "newPlan").destructive)
    }

    function test_the_last_plan_cannot_be_deleted() {
        const chooser = make()
        fake.plans = [{ id: "plan-a", name: "Plan A", shots: 5 }]
        fake.reshow()

        verify(!entry(chooser, "removePlan").enabled)
        verify(entry(chooser, "snapshotPlan").enabled)
    }

    function test_without_a_scene_nothing_can_be_done() {
        const chooser = make()
        fake.plans = []
        fake.plan = ""
        fake.reshow()

        verify(!chooser.enabled)
        for (const name of ["newPlan", "snapshotPlan", "renamePlan", "removePlan"]) {
            verify(!entry(chooser, name).enabled, name)
        }
    }

    function test_the_texts_are_in_both_languages() {
        for (const key of ["plan.choose", "plan.new", "plan.snapshot", "plan.rename",
                           "plan.remove", "plan.newTitle", "plan.snapshotTitle",
                           "plan.renameTitle", "plan.name", "plan.nameAuto",
                           "plan.snapshotHint", "plan.removeTitle", "remove.plan",
                           "remove.planShot", "remove.planShots"]) {
            verify(de.t(key) !== key, "de " + key)
            verify(en.t(key) !== key, "en " + key)
        }
    }
}
