import QtQuick
import QtTest
import "../../qml"
import "FieldLabels.js" as FieldLabels

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
        property int pictureRevision: 0
        property string selectedKey: ""
        property string choicesJson: JSON.stringify({
            pose: ["standing", "sitting", "lying"],
            arrival: ["walk", "appear"],
            departure: ["stay", "leave"],
            rig: ["tripod", "dolly", "track", "crane", "handheld", "gimbal"],
            curve: ["smooth", "straight"]
        })
        property var info: ({})
        property var poses: []
        property var holds: []
        property var paces: []
        property var aims: []
        property var rigs: []
        property var lenses: []
        property var curves: []
        property var arrivals: []
        property var departures: []
        property bool poseOk: true
        property bool holdOk: true
        property bool paceOk: true
        property bool aimOk: true
        property bool rigOk: true
        property bool lensOk: true

        function markInfo() { return JSON.stringify(info) }
        function setMarkPose(pose) {
            poses.push(pose)
            return poseOk
        }
        function setMarkHold(text) {
            holds.push(text)
            return holdOk
        }
        function setCameraAim(height, tilt) {
            aims.push([height, tilt])
            return aimOk
        }
        function setMarkRig(name) {
            rigs.push(name)
            return rigOk
        }
        function setMarkLens(text) {
            lenses.push(text)
            return lensOk
        }
        function setMarkArrival(name) {
            arrivals.push(name)
            return true
        }
        function setMarkDeparture(name) {
            departures.push(name)
            return true
        }
        function setWayCurve(name) {
            curves.push(name)
            return true
        }
        function setMarkPace(name) {
            paces.push(name)
            return paceOk
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
        fake.pictureRevision = 0
        fake.selectedKey = ""
        fake.poses = []
        fake.holds = []
        fake.paces = []
        fake.arrivals = []
        fake.departures = []
        fake.poseOk = true
        fake.holdOk = true
        fake.paceOk = true
        fake.aims = []
        fake.aimOk = true
        fake.rigs = []
        fake.lenses = []
        fake.curves = []
        fake.rigOk = true
        fake.lensOk = true
    }

    function cameraMark(extra) {
        fake.info = Object.assign({ kind: "camera", pose: "", arrival: "", departure: "", hold: 6, hasWay: true, pace: 0.5, height: 1.6, tilt: -12.5, rig: "crane", lens: 85 }, extra || {})
        fake.selectedKey = "camera:a"
    }

    function figureMark(extra) {
        fake.info = Object.assign({ kind: "figure", pose: "sitting", arrival: "walk", departure: "stay", hold: 2.5, hasWay: true, pace: 1.4 }, extra || {})
        fake.selectedKey = "mark:elisabeth:1"
    }

    function test_no_mark_selected_no_card() {
        const card = createTemporaryObject(cardComponent, tc)

        verify(!card.visible)
    }

    // A drag shows its preview through the plan, and the edit at its end gives the same plan:
    // the card reads the mark again on the picture revision, which every edit raises. `info`
    // is changed in place, which tells QML of no change, like the bridge's `markInfo`.
    function test_the_card_follows_an_edit_that_leaves_the_mark_as_previewed() {
        const card = createTemporaryObject(cardComponent, tc)
        figureMark()

        fake.info.hold = 4
        fake.pictureRevision++

        compare(findChild(card, "holdField").text, "4")
    }

    // Playing while a drag is held moves the playhead with only a preview: no edit, a new
    // plan. The card shows the mark at the playhead, so it reads it again on the plan too.
    function test_the_card_follows_the_playhead_during_a_drag() {
        const card = createTemporaryObject(cardComponent, tc)
        figureMark()

        fake.info.hold = 4
        fake.planJson = JSON.stringify({ preview: 1 })

        compare(findChild(card, "holdField").text, "4")
    }

    function test_a_figure_mark_shows_pose_pause_and_pace() {
        const card = createTemporaryObject(cardComponent, tc)

        figureMark()

        verify(card.visible)
        compare(findChild(card, "poseChoice").currentIndex, 1)
        compare(findChild(card, "holdField").text, "2,5")
        compare(findChild(card, "paceField").text, "1,4")
        verify(!findChild(card, "paceValue").visible)
        verify(findChild(card, "paceWalk").enabled)
    }

    function test_english_uses_the_point() {
        const card = createTemporaryObject(cardComponent, tc, { texts: en })

        figureMark()

        compare(findChild(card, "holdField").text, "2.5")
        compare(findChild(card, "paceField").text, "1.4")
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

    function test_the_typed_pace_goes_to_the_controller() {
        const card = createTemporaryObject(cardComponent, tc)
        figureMark()
        const field = findChild(card, "paceField")

        field.text = "2,5"
        findChild(card, "paceApply").clicked()
        field.forceActiveFocus()
        keyClick(Qt.Key_Return)

        compare(fake.paces, ["2,5", "2,5"])
        verify(!field.invalid)
    }

    // The field shows the pace rounded: applying it unchanged would move the later marks.
    function test_an_unchanged_pace_is_not_sent() {
        const card = createTemporaryObject(cardComponent, tc)
        figureMark()

        findChild(card, "paceApply").clicked()

        compare(fake.paces, [])
    }

    // Enter keeps the focus, so the field is not filled again; typing the old pace back must
    // still be sent once the plan has the new one.
    function test_the_old_pace_typed_back_is_sent() {
        const card = createTemporaryObject(cardComponent, tc)
        figureMark()
        const field = findChild(card, "paceField")
        field.forceActiveFocus()

        field.text = "2,5"
        keyClick(Qt.Key_Return)
        figureMark({ pace: 2.5 })
        field.text = "1,4"
        keyClick(Qt.Key_Return)

        compare(fake.paces, ["2,5", "1,4"])
    }

    function test_a_refused_pace_marks_the_field() {
        const card = createTemporaryObject(cardComponent, tc)
        figureMark()
        fake.paceOk = false
        const field = findChild(card, "paceField")

        field.text = "0"
        findChild(card, "paceApply").clicked()

        verify(field.invalid)
        field.text = "1"
        verify(!field.invalid)
    }

    function test_a_figure_mark_shows_arrival_and_departure() {
        const card = createTemporaryObject(cardComponent, tc)

        figureMark({ arrival: "appear", departure: "leave" })

        verify(!findChild(card, "cueRow").visible)
        findChild(card, "markMore").clicked()
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

    // LH-34: a camera's mark shows height and tilt, a figure's does not.
    function test_a_camera_mark_shows_height_and_tilt() {
        const card = createTemporaryObject(cardComponent, tc)

        cameraMark()

        verify(findChild(card, "aimRow").visible)
        compare(findChild(card, "heightField").text, "1,6")
        compare(findChild(card, "tiltField").text, "-12,5")
    }

    function test_english_uses_the_point_for_height_and_tilt() {
        const card = createTemporaryObject(cardComponent, tc, { texts: en })

        cameraMark()

        compare(findChild(card, "heightField").text, "1.6")
        compare(findChild(card, "tiltField").text, "-12.5")
    }

    // LH-33: a camera's mark shows its rig and lens (empty: none), a figure's does not.
    function test_a_camera_mark_shows_rig_and_lens() {
        const card = createTemporaryObject(cardComponent, tc)

        cameraMark()

        verify(findChild(card, "gearRow").visible)
        compare(findChild(card, "rigChoice").currentIndex, 3)
        compare(findChild(card, "lensField").text, "85")

        cameraMark({ rig: "dolly", lens: null })

        compare(findChild(card, "rigChoice").currentIndex, 1)
        compare(findChild(card, "lensField").text, "")
    }

    function test_a_figure_mark_has_no_rig_or_lens() {
        const card = createTemporaryObject(cardComponent, tc)

        figureMark()

        verify(!findChild(card, "gearRow").visible)
    }

    function test_choosing_a_rig_goes_to_the_controller() {
        const card = createTemporaryObject(cardComponent, tc)
        cameraMark()

        findChild(card, "rigChoice").activated(4)

        compare(fake.rigs, ["handheld"])
    }

    // A refused rig (a tripod that would have to move) shows the plan's rig again.
    function test_a_refused_rig_snaps_back() {
        const card = createTemporaryObject(cardComponent, tc)
        cameraMark()
        fake.rigOk = false

        findChild(card, "rigChoice").currentIndex = 0
        findChild(card, "rigChoice").activated(0)

        compare(findChild(card, "rigChoice").currentIndex, 3)
    }

    function test_a_lens_goes_to_the_controller_and_a_refusal_marks_the_field() {
        const card = createTemporaryObject(cardComponent, tc)
        cameraMark()
        const lens = findChild(card, "lensField")

        lens.text = "50"
        findChild(card, "lensApply").clicked()
        lens.text = ""
        lens.forceActiveFocus()
        keyClick(Qt.Key_Return)

        compare(fake.lenses, ["50", ""])
        verify(!lens.invalid)

        fake.lensOk = false
        lens.text = "0"
        findChild(card, "lensApply").clicked()

        verify(lens.invalid)
    }

    function test_the_rig_and_lens_tips_are_in_both_languages() {
        compare(de.t("rig.crane"), "Kran")
        compare(en.t("rig.crane"), "Crane")
        verify(de.t("mark.lensTip") !== en.t("mark.lensTip"))
    }

    function test_a_figure_mark_has_no_height_or_tilt() {
        const card = createTemporaryObject(cardComponent, tc)

        figureMark()

        verify(!findChild(card, "aimRow").visible)
    }

    function test_height_and_tilt_go_to_the_controller_together() {
        const card = createTemporaryObject(cardComponent, tc)
        cameraMark()
        const height = findChild(card, "heightField")
        const tilt = findChild(card, "tiltField")

        height.text = "2,4"
        tilt.text = "15"
        findChild(card, "aimApply").clicked()
        height.forceActiveFocus()
        keyClick(Qt.Key_Return)
        tilt.forceActiveFocus()
        keyClick(Qt.Key_Return)

        compare(fake.aims, [["2,4", "15"], ["2,4", "15"], ["2,4", "15"]])
        verify(!height.invalid)
        verify(!tilt.invalid)
    }

    function test_a_refused_height_or_tilt_marks_both_fields() {
        const card = createTemporaryObject(cardComponent, tc)
        cameraMark()
        fake.aimOk = false
        const height = findChild(card, "heightField")
        const tilt = findChild(card, "tiltField")

        height.text = "99"
        findChild(card, "aimApply").clicked()

        verify(height.invalid)
        verify(tilt.invalid)
        tilt.text = "1"
        verify(!tilt.invalid)
    }

    // The tilt's tooltip names the direction of positive values, in both languages.
    function test_the_tilt_tip_says_which_way_is_positive() {
        compare(de.t("mark.tiltTip"), "Neigung der Kamera: positiv blickt nach oben, negativ nach unten")
        compare(en.t("mark.tiltTip"), "Tilt of the camera: positive looks up, negative down")
    }

    function test_the_first_mark_has_no_way_to_time() {
        const card = createTemporaryObject(cardComponent, tc)

        figureMark({ hasWay: false, pace: null })

        verify(!findChild(card, "paceValue").visible)
        verify(!findChild(card, "paceField").enabled)
        verify(!findChild(card, "paceApply").enabled)
        verify(!findChild(card, "paceWalk").enabled)
        verify(!findChild(card, "paceRun").enabled)
    }

    function test_a_way_without_time_has_no_pace_number() {
        const card = createTemporaryObject(cardComponent, tc)

        figureMark({ pace: null })

        compare(findChild(card, "paceField").text, "")
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

    // D-04: pose, pause and pace are common; arrival and departure wait behind "More".
    function test_more_shows_the_other_fields_of_a_camera_mark() {
        const card = createTemporaryObject(cardComponent, tc)
        cameraMark()

        verify(findChild(card, "heightField").visible)
        verify(!findChild(card, "tiltRow").visible)

        findChild(card, "markMore").clicked()

        verify(findChild(card, "tiltRow").visible)
    }

    // Q1-03: each number field is named by a label above it, not by a placeholder.
    function test_every_figure_field_has_a_visible_label() {
        const card = createTemporaryObject(cardComponent, tc)
        figureMark()
        const labels = FieldLabels.of(card)

        verify(labels.indexOf("Pause (s)") >= 0, labels)
        verify(labels.indexOf("Tempo (m/s)") >= 0, labels)
        compare(findChild(card, "holdField").Accessible.name, "Pause (s)")
        compare(findChild(card, "paceField").Accessible.name, "Tempo (m/s)")
    }

    function test_every_camera_field_has_a_visible_label() {
        const card = createTemporaryObject(cardComponent, tc)
        cameraMark()
        card.expanded = true
        const labels = FieldLabels.of(card)
        const names = {
            holdField: "Pause (s)",
            heightField: "Höhe (m)",
            lensField: "Brennweite (mm)",
            tiltField: "Neigung (°)",
        }

        for (const objectName in names) {
            verify(labels.indexOf(names[objectName]) >= 0, names[objectName] + " in " + labels)
            compare(findChild(card, objectName).Accessible.name, names[objectName])
        }
    }

    // LH-108: the shape of the way that leads to the mark.
    function test_the_way_shape_shows_and_goes_to_the_controller() {
        const card = createTemporaryObject(cardComponent, tc)
        figureMark({ curve: "straight" })
        const choice = findChild(card, "curveChoice")

        verify(findChild(card, "curveRow").visible)
        compare(choice.currentIndex, 1)

        choice.select(0)

        compare(fake.curves, ["smooth"])
    }

    function test_the_first_mark_has_no_way_shape() {
        const card = createTemporaryObject(cardComponent, tc)

        figureMark({ hasWay: false, curve: "" })

        verify(!findChild(card, "curveRow").visible)
    }
}
