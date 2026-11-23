import QtQuick
import QtTest
import Kante
import "../../qml"

// What stands over the plan (D-21): the notice when the plan is drawn on another set than the
// scene is shot at, and the empty state of a scene that is shot nowhere. A click acts, nothing
// before it.
TestCase {
    id: tc
    name: "PlanNotices"
    width: 900
    height: 500
    when: windowShown
    visible: true

    QtObject {
        id: fake
        property bool loaded: true
        property string planJson: ""
        property string projectJson: ""
        property string sceneId: "s15"
        property var calls: []

        function nameFromCapitals(name) {
            return "named:" + name
        }
        function addPlan(name) { calls.push(["addPlan", name]); return true }
        function shootSceneAt(id) { calls.push(["shootSceneAt", id]); return true }
        function editPlace(id, name, variants, location) {
            calls.push(["editPlace", id, name, variants, location])
            return true
        }
    }

    TestTexts { id: de; language: "de" }
    TestTexts { id: en; language: "en" }

    Component {
        id: areaComponent
        PlanArea {
            width: 800
            height: 400
            controller: fake
        }
    }

    readonly property var locations: [
        { id: "hof", name: "Burghof" },
        { id: "saal", name: "Sängersaal" }
    ]

    function project(extra) {
        return JSON.stringify(Object.assign({
            scenes: [{ id: "s15", number: "15", heading: "EXT. HAFEN - TAG" }],
            places: [{ id: "hafen", name: "HAFEN", variants: ["KAI"], location: "", location_name: "", scenes: ["15"] }],
            locations: locations,
            plans: [],
            plan: ""
        }, extra))
    }

    function init() {
        fake.calls = []
        fake.planJson = ""
    }

    function area(json, texts) {
        fake.projectJson = json
        const a = createTemporaryObject(areaComponent, tc, { texts: texts || de })
        waitForRendering(a)
        return a
    }

    function elsewhere() {
        return project({
            plans: [{ id: "p1", name: "Plan A", shots: 0 }],
            plan: "p1",
            elsewhere: { drawn_on: "hof", drawn_on_name: "Burghof", shot_at: "saal", shot_at_name: "Sängersaal" }
        })
    }

    function test_the_notice_names_both_sets_and_acts_on_a_click() {
        const a = area(elsewhere())
        const notice = findChild(a, "planElsewhere")

        verify(notice.visible)
        compare(notice.kind, KanteCallout.Kind.Warn)
        compare(notice.text, "Plan A ist auf Burghof gezeichnet; die Szene wird in Sängersaal gedreht.")
        compare(findChild(notice, "newPlanThere").text, "Neuer Plan auf Sängersaal")
        compare(findChild(notice, "shootThere").text, "Szene in Burghof drehen")
        compare(fake.calls, [])

        findChild(notice, "newPlanThere").clicked()
        findChild(notice, "shootThere").clicked()
        compare(fake.calls, [["addPlan", ""], ["shootSceneAt", "hof"]])
    }

    function test_the_notice_fits_the_plan() {
        const a = area(elsewhere())
        const notice = findChild(a, "planElsewhere")

        verify(notice.width > 0 && notice.x >= 0 && notice.x + notice.width <= a.width)
        verify(notice.height > 0 && notice.y >= 0)
    }

    function test_no_notice_when_the_plan_lies_where_the_scene_is_shot() {
        const a = area(project({ plans: [{ id: "p1", name: "Plan A", shots: 0 }], plan: "p1" }))

        compare(findChild(a, "planElsewhere"), null)
        compare(findChild(a, "notShot"), null)
    }

    function test_a_scene_shot_nowhere_asks_for_a_location_for_its_place() {
        const a = area(project({ not_shot: { place: "hafen", place_name: "HAFEN" } }))
        const empty = findChild(a, "notShot")

        verify(empty.visible)
        compare(empty.title, "HAFEN hat noch keinen Drehort.")
        compare(empty.text, "Die Wahl gilt für alle Szenen an diesem Spielort.")
        // This replaces the empty state of a scene without a plan.
        compare(findChild(a, "noPlan"), null)
        tryVerify(() => empty.width > a.width / 4)
        compare(fake.calls, [])
    }

    function test_the_choice_goes_to_the_place_with_its_name_and_spellings() {
        const a = area(project({ not_shot: { place: "hafen", place_name: "HAFEN" } }))

        findChild(a, "pickLocation").clicked()
        const popover = findChild(a, "locationPopover")
        tryCompare(popover, "opened", true)
        findChild(popover.contentItem, "location.saal").clicked()

        compare(fake.calls, [["editPlace", "hafen", "HAFEN", JSON.stringify(["KAI"]), "saal"]])
    }

    function test_a_scene_without_a_place_takes_the_location_for_itself() {
        const a = area(project({ not_shot: { place: "", place_name: "" } }))
        const empty = findChild(a, "notShot")

        compare(empty.title, "Die Szene hat noch keinen Drehort.")
        compare(empty.text, "Die Wahl gilt nur für diese Szene.")
        findChild(a, "pickLocation").clicked()
        const popover = findChild(a, "locationPopover")
        tryCompare(popover, "opened", true)
        findChild(popover.contentItem, "location.hof").clicked()

        compare(fake.calls, [["shootSceneAt", "hof"]])
    }

    function test_a_new_location_is_asked_for_by_the_name_the_controller_makes() {
        const a = area(project({ not_shot: { place: "hafen", place_name: "HAFEN" } }))
        const asked = []
        a.newLocationRequested.connect((name, place) => asked.push([name, place]))

        findChild(a, "newLocation").clicked()
        compare(asked, [["named:HAFEN", "hafen"]])
    }

    function test_without_a_location_in_the_project_only_a_new_one_is_offered() {
        const a = area(project({ locations: [], not_shot: { place: "hafen", place_name: "HAFEN" } }))

        verify(!findChild(a, "pickLocation").visible)
        verify(findChild(a, "newLocation").visible)
    }

    function test_a_scene_shot_but_without_a_plan_keeps_its_empty_state() {
        const a = area(project({}))

        verify(findChild(a, "noPlan").visible)
        compare(findChild(a, "notShot"), null)
    }

    function test_english() {
        const a = area(elsewhere(), en)

        compare(findChild(a, "planElsewhere").text, "Plan A is drawn on Burghof; the scene is shot at Sängersaal.")
        compare(findChild(a, "shootThere").text, "Shoot the scene at Burghof")
    }
}
