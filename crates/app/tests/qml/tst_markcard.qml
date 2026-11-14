import QtQuick
import QtTest
import "../../qml"

// The mark fields (LH-22, LH-63): they follow the selection and pass what is typed or
// chosen to the controller.
TestCase {
    id: tc
    name: "MarkCard"
    width: 900
    height: 200
    when: windowShown
    visible: true

    QtObject {
        id: fake
        property string planJson: ""
        property string selectedKey: ""
        property var info: ({})
        property var poses: []
        property var holds: []
        property var paces: []
        property var arrivals: []
        property var departures: []
        property bool poseOk: true
        property bool holdOk: true

        function markInfo() { return JSON.stringify(info) }
        function setMarkPose(pose) {
            poses.push(pose)
            return poseOk
        }
        function setMarkHold(text) {
            holds.push(text)
            return holdOk
        }
        function setMarkArrival(name) {
            arrivals.push(name)
            return true
        }
        function setMarkDeparture(name) {
            departures.push(name)
            return true
        }
        function setMarkPace(name) {
            paces.push(name)
            return true
        }
    }

    TestTexts { id: de; language: "de" }
    TestTexts { id: en; language: "en" }

    Component {
        id: cardComponent
        MarkCard {
            controller: fake
            texts: de
        }
    }

    function init() {
        fake.info = {}
        fake.selectedKey = ""
        fake.poses = []
        fake.holds = []
        fake.paces = []
        fake.arrivals = []
        fake.departures = []
        fake.poseOk = true
        fake.holdOk = true
    }

    function figureMark(extra) {
        fake.info = Object.assign({ kind: "figure", pose: "sitting", arrival: "walk", departure: "stay", hold: 2.5, hasWay: true, pace: 1.4 }, extra || {})
        fake.selectedKey = "mark:elisabeth:1"
    }

    function test_no_mark_selected_no_card() {
        const card = createTemporaryObject(cardComponent, tc)

        verify(!card.visible)
    }

    function test_a_figure_mark_shows_pose_pause_and_pace() {
        const card = createTemporaryObject(cardComponent, tc)

        figureMark()

        verify(card.visible)
        compare(findChild(card, "poseChoice").currentIndex, 1)
        compare(findChild(card, "holdField").text, "2,5")
        compare(findChild(card, "paceValue").text, "Tempo: 1,4 m/s")
        verify(findChild(card, "paceWalk").enabled)
    }

    function test_english_uses_the_point() {
        const card = createTemporaryObject(cardComponent, tc, { texts: en })

        figureMark()

        compare(findChild(card, "holdField").text, "2.5")
        compare(findChild(card, "paceValue").text, "Pace: 1.4 m/s")
    }

    function test_choosing_a_pose_goes_to_the_controller() {
        const card = createTemporaryObject(cardComponent, tc)
        figureMark()

        findChild(card, "poseChoice").select(2)

        compare(fake.poses, ["lying"])
    }

    function test_a_refused_pose_shows_the_plan_again() {
        const card = createTemporaryObject(cardComponent, tc)
        figureMark()
        fake.poseOk = false

        findChild(card, "poseChoice").select(0)

        compare(findChild(card, "poseChoice").currentIndex, 1)
    }

    function test_the_pause_goes_to_the_controller() {
        const card = createTemporaryObject(cardComponent, tc)
        figureMark()
        const field = findChild(card, "holdField")

        field.text = "4,5"
        findChild(card, "holdApply").clicked()
        field.forceActiveFocus()
        keyClick(Qt.Key_Return)

        compare(fake.holds, ["4,5", "4,5"])
        verify(!field.invalid)
    }

    function test_a_refused_pause_marks_the_field() {
        const card = createTemporaryObject(cardComponent, tc)
        figureMark()
        fake.holdOk = false
        const field = findChild(card, "holdField")

        field.text = "-1"
        findChild(card, "holdApply").clicked()

        verify(field.invalid)
        field.text = "1"
        verify(!field.invalid)
    }

    function test_the_buttons_set_the_pace() {
        const card = createTemporaryObject(cardComponent, tc)
        figureMark()

        findChild(card, "paceWalk").clicked()
        findChild(card, "paceRun").clicked()

        compare(fake.paces, ["walk", "run"])
    }

    function test_a_figure_mark_shows_arrival_and_departure() {
        const card = createTemporaryObject(cardComponent, tc)

        figureMark({ arrival: "appear", departure: "leave" })

        verify(findChild(card, "cueRow").visible)
        compare(findChild(card, "arrivalChoice").currentIndex, 1)
        compare(findChild(card, "departureChoice").currentIndex, 1)
    }

    function test_choosing_arrival_and_departure_goes_to_the_controller() {
        const card = createTemporaryObject(cardComponent, tc)
        figureMark()

        findChild(card, "arrivalChoice").select(1)
        findChild(card, "departureChoice").select(1)

        compare(fake.arrivals, ["appear"])
        compare(fake.departures, ["leave"])
    }

    function test_a_camera_mark_has_no_arrival_or_departure() {
        const card = createTemporaryObject(cardComponent, tc)

        fake.info = { kind: "camera", pose: "", arrival: "", departure: "", hold: 6, hasWay: true, pace: 0.5 }
        fake.selectedKey = "camera:a"

        verify(!findChild(card, "cueRow").visible)
    }

    function test_the_first_mark_has_no_way_to_time() {
        const card = createTemporaryObject(cardComponent, tc)

        figureMark({ hasWay: false, pace: null })

        verify(!findChild(card, "paceValue").visible)
        verify(!findChild(card, "paceWalk").enabled)
        verify(!findChild(card, "paceRun").enabled)
    }

    function test_a_way_without_time_has_no_pace_number() {
        const card = createTemporaryObject(cardComponent, tc)

        figureMark({ pace: null })

        compare(findChild(card, "paceValue").text, "Tempo: –")
    }

    function test_a_camera_mark_has_a_pause_and_a_pace_to_read() {
        const card = createTemporaryObject(cardComponent, tc)

        fake.info = { kind: "camera", pose: "", hold: 6, hasWay: true, pace: 0.5 }
        fake.selectedKey = "camera:a"

        verify(card.visible)
        verify(!findChild(card, "poseChoice").visible)
        verify(!findChild(card, "paceWalk").visible)
        verify(!findChild(card, "paceRun").visible)
        compare(findChild(card, "holdField").text, "6")
        compare(findChild(card, "paceValue").text, "Tempo: 0,5 m/s")
    }

    function test_the_card_goes_with_the_selection() {
        const card = createTemporaryObject(cardComponent, tc)
        figureMark()
        verify(card.visible)

        fake.info = {}
        fake.selectedKey = "wall:0"

        verify(!card.visible)
    }
}
