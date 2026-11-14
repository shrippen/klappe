import QtQuick
import QtTest
import "../../qml"

// PlanChooser (LH-04): what the box lists and shows, and what the buttons and their forms
// send to the controller.
TestCase {
    id: tc
    name: "PlanChooser"
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
    }

    TestTexts { id: de; language: "de" }
    TestTexts { id: en; language: "en" }

    Component {
        id: chooserComponent
        PlanChooser {
            controller: fake
            texts: de
            width: tc.width
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

    function test_the_box_lists_the_plans_and_shows_the_one_on_view() {
        const box = findChild(make(), "planBox")

        compare(box.count, 3)
        compare(box.currentIndex, 0)
        compare(box.textAt(1), "Plan A (2)")

        fake.plan = "plan-b"
        fake.reshow()
        compare(box.currentIndex, 2)
        compare(box.currentText, "Plan B")
    }

    function test_choosing_a_plan_sends_its_id() {
        const box = findChild(make(), "planBox")

        box.currentIndex = 1
        box.activated(1)

        compare(fake.calls, [["selectPlan", "plan-a-2"]])
    }

    function test_a_refused_choice_shows_the_plan_on_view_again() {
        const box = findChild(make(), "planBox")
        fake.accept = false

        box.currentIndex = 2
        box.activated(2)

        compare(box.currentIndex, 0)
    }

    function test_the_box_keeps_the_plan_on_view_when_a_name_changes() {
        const box = findChild(make(), "planBox")
        fake.plan = "plan-b"
        fake.reshow()

        fake.plans = [
            { id: "plan-a", name: "Plan A", shots: 5 },
            { id: "plan-a-2", name: "Plan A (2)", shots: 5 },
            { id: "plan-b", name: "Fassung 3", shots: 0 }
        ]
        fake.reshow()

        compare(box.currentIndex, 2)
        compare(box.currentText, "Fassung 3")
    }

    function test_a_new_plan_may_leave_its_name_empty() {
        const chooser = make()
        const form = findChild(chooser, "planForm")

        findChild(chooser, "newPlan").clicked()
        tryCompare(form, "opened", true)
        compare(form.title, "Neuer Plan")
        verify(form.acceptEnabled)
        form.accept()

        compare(fake.calls, [["addPlan", ""]])
    }

    function test_a_new_plan_sends_the_name_typed() {
        const chooser = make()
        const form = findChild(chooser, "planForm")

        findChild(chooser, "newPlan").clicked()
        tryCompare(form, "opened", true)
        findChild(form, "nameField").text = "Mit Pferd"
        form.accept()

        compare(fake.calls, [["addPlan", "Mit Pferd"]])
    }

    function test_a_snapshot_copies_under_the_name_typed() {
        const chooser = make()
        const form = findChild(chooser, "planForm")

        findChild(chooser, "snapshotPlan").clicked()
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

        findChild(chooser, "renamePlan").clicked()
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

        findChild(chooser, "newPlan").clicked()
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

        findChild(chooser, "removePlan").clicked()
        tryCompare(confirm, "opened", true)
        compare(findChild(confirm, "question").text,
                "Plan Plan B löschen? Rückgängig stellt ihn wieder her.")
        confirm.reject()
        compare(fake.calls, [])

        findChild(chooser, "removePlan").clicked()
        tryCompare(confirm, "opened", true)
        confirm.accept()
        compare(fake.calls, [["removePlan", "plan-b"]])
    }

    function test_the_last_plan_cannot_be_deleted() {
        const chooser = make()
        fake.plans = [{ id: "plan-a", name: "Plan A", shots: 5 }]
        fake.reshow()

        verify(!findChild(chooser, "removePlan").enabled)
        verify(findChild(chooser, "snapshotPlan").enabled)
    }

    function test_without_a_scene_nothing_can_be_done() {
        const chooser = make()
        fake.plans = []
        fake.plan = ""
        fake.reshow()

        verify(!findChild(chooser, "planBox").enabled)
        for (const name of ["newPlan", "snapshotPlan", "renamePlan", "removePlan"]) {
            verify(!findChild(chooser, name).enabled, name)
        }
    }

    function test_the_texts_are_in_both_languages() {
        for (const key of ["plan.choose", "plan.new", "plan.snapshot", "plan.rename",
                           "plan.remove", "plan.newTitle", "plan.snapshotTitle",
                           "plan.renameTitle", "plan.name", "plan.nameAuto",
                           "plan.snapshotHint", "plan.removeTitle", "remove.plan"]) {
            verify(de.t(key) !== key, "de " + key)
            verify(en.t(key) !== key, "en " + key)
        }
    }
}
