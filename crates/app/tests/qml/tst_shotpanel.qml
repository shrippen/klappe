import QtQuick
import QtTest
import "../../qml"

// ShotPanel: what leaving or editing a field sends to the controller (review gui-r1).
TestCase {
    id: tc
    name: "ShotPanel"
    width: 480
    height: 700
    visible: true
    when: windowShown

    QtObject {
        id: fake
        property string tracksJson: JSON.stringify([
            { kind: "cut", id: "cut", name: "Cut" },
            { kind: "camera", id: "a", name: "A" },
            { kind: "camera", id: "b", name: "B" },
            { kind: "figure", id: "elisabeth", name: "Elisabeth" },
            { kind: "figure", id: "burgvogt", name: "Burgvogt" }
        ])
        property string choicesJson: JSON.stringify({ movement: ["static", "pan", "tilt", "dolly", "crane", "handheld", "gimbal", "zoom"], axis: { a: "elisabeth", b: "burgvogt", own: false } })
        property string viewJson: ""
        property var calls: []
        property var follows: []
        property var axisCalls: []
        property bool accept: true
        property var shotCalls: []
        property bool loaded: true

        function addShot() {
            shotCalls.push(["add"])
            return accept
        }
        function copyShot(id) {
            shotCalls.push(["copy", id])
            return accept
        }
        function deleteShot(id) {
            shotCalls.push(["delete", id])
            return accept
        }
        function setAxis(a, b) {
            axisCalls.push([a, b])
            return accept
        }
        function setFollow(camera, figure) {
            follows.push([camera, figure])
            return accept
        }
        function editShot(id, field, value) {
            calls.push([id, field, value])
            return accept
        }
    }

    TestTexts { id: catalogue }

    function row(own, follows) {
        return {
            follows: follows === undefined ? "" : follows,
            id: "s12-3", number: "12.3", camera: "b", camera_label: "B", size: "medium", angle: "eye-level",
            movement: "static", lens_mm: own === null ? 50 : own, own_lens_mm: own, lens_to_mm: null, lens_from_marks: false,
            description: "", props: "", gear: "", crew: "", note: "", axis_jump: false, axis_against: ""
        }
    }

    Component {
        id: panelComponent
        ShotPanel {
            width: 480
            height: 700
            texts: catalogue
            controller: fake
        }
    }

    function make(own, follows, hint) {
        fake.calls = []
        fake.follows = []
        fake.axisCalls = []
        fake.shotCalls = []
        fake.accept = true
        const panel = createTemporaryObject(panelComponent, tc, { shot: Object.assign(row(own, follows), { movement_hint: hint || "" }) })
        verify(panel)
        return panel
    }

    // Typing keeps the text binding alive, unlike `field.text = ...`.
    function type(field, text) {
        field.remove(0, field.length)
        field.insert(0, text)
    }

    function field(panel, name) {
        const item = findChild(panel, name)
        verify(item, name)
        return item
    }

    function test_leaving_the_lens_unchanged_edits_nothing() {
        const panel = make(null)
        const lens = field(panel, "lensField")

        // Leaving a field fires editingFinished, whether or not the text changed.
        lens.editingFinished()

        compare(fake.calls.length, 0)
    }

    function test_a_shot_without_own_lens_shows_the_camera_lens_as_hint() {
        const lens = field(make(null), "lensField")

        compare(lens.text, "")
        compare(lens.placeholderText, "50")
    }

    function test_marks_with_a_lens_make_the_field_read_only() {
        const panel = make(40.5)
        panel.shot = Object.assign({}, panel.shot, { lens_mm: 35, lens_to_mm: 85, lens_from_marks: true })
        const lens = field(panel, "lensField")

        verify(lens.readOnly)
        compare(lens.text, "35–85 mm")
        verify(panel.lensMarked)

        panel.shot = Object.assign({}, panel.shot, { lens_to_mm: null })
        compare(lens.text, "35 mm")

        panel.shot = Object.assign({}, panel.shot, { lens_from_marks: false })
        verify(!lens.readOnly)
        compare(lens.text, "40,5")
    }

    function test_an_own_lens_is_the_text() {
        const lens = field(make(40.5), "lensField")

        compare(lens.text, "40,5")
    }

    function test_a_changed_lens_is_sent() {
        const lens = field(make(null), "lensField")

        type(lens, "85")
        lens.editingFinished()

        compare(fake.calls, [["s12-3", "lens", "85"]])
    }

    function test_a_refused_value_is_replaced_by_the_model_value() {
        const lens = field(make(40.5), "lensField")
        fake.accept = false

        type(lens, "5000")
        lens.editingFinished()

        compare(fake.calls.length, 1)
        compare(lens.text, "40,5")
    }

    function test_the_camera_chooser_lists_the_cameras() {
        const box = field(make(null), "cameraBox")

        compare(box.count, 2)
        compare(box.currentIndex, 1)

        box.activated(0)

        compare(fake.calls, [["s12-3", "camera", "a"]])
    }

    function test_the_follow_chooser_lists_none_and_the_figures() {
        const box = field(make(null, "burgvogt"), "followBox")

        compare(box.count, 3)
        compare(box.currentIndex, 2)
        compare(box.model[0], "keine")

        box.activated(1)

        compare(fake.follows, [["b", "elisabeth"]])
    }

    function test_choosing_none_lets_the_camera_go() {
        const box = field(make(null, "burgvogt"), "followBox")

        box.activated(0)

        compare(fake.follows, [["b", ""]])
    }

    function test_an_uncoupled_camera_shows_none_and_the_same_choice_sends_nothing() {
        const box = field(make(null), "followBox")

        compare(box.currentIndex, 0)

        box.activated(0)

        compare(fake.follows.length, 0)
    }

    // LH-40: the axis pair is chosen per end; the other end stays.
    function test_the_axis_choosers_show_the_pair() {
        const panel = make(null)

        compare(field(panel, "axisBox_a").currentIndex, 0)
        compare(field(panel, "axisBox_b").currentIndex, 1)
    }

    function test_choosing_an_axis_end_sends_the_pair() {
        const panel = make(null)

        field(panel, "axisBox_b").activated(0)

        compare(fake.axisCalls, [["elisabeth", "elisabeth"]])
    }

    function test_the_axis_button_resets_only_a_chosen_pair() {
        const panel = make(null)
        const button = field(panel, "axisAuto")

        verify(!button.enabled)

        const choices = fake.choicesJson
        fake.choicesJson = JSON.stringify({ axis: { a: "burgvogt", b: "elisabeth", own: true } })
        verify(button.enabled)

        button.clicked()

        compare(fake.axisCalls, [["", ""]])

        fake.choicesJson = choices
    }

    // LH-41: the warning chip names the shots on the other side.
    function test_the_warning_names_the_other_shots() {
        const panel = make(null)

        panel.shot = Object.assign({}, panel.shot, { axis_jump: true, axis_against: "12.1, 12.2" })
        const chip = findChild(panel, "axisChip")

        verify(chip.visible)
        compare(chip.toolTip, "Achsensprung gegen 12.1, 12.2")
    }

    // LH-33: a shot without a movement shows what the camera marks do, and a button takes it.
    function test_a_movement_hint_is_shown_and_taken() {
        const panel = make(null, "", "crane")

        verify(field(panel, "movementHintRow").visible)
        compare(field(panel, "movementHint").text, "Aus den Kamera-Marks: Kran")

        field(panel, "movementTake").clicked()

        compare(fake.calls, [["s12-3", "movement", "crane"]])
    }

    // The movements come from the controller's lists, not from the panel.
    function test_the_movement_chooser_lists_what_the_controller_offers() {
        const box = field(make(null), "movementBox")

        compare(box.count, 8)
        compare(box.currentText, "Statisch")

        box.activated(4)

        compare(fake.calls, [["s12-3", "movement", "crane"]])
    }

    function test_no_hint_no_row() {
        const panel = make(null)

        verify(!field(panel, "movementHintRow").visible)
    }

    function test_the_perspective_chooser_lists_the_angles() {
        const box = field(make(null), "angleBox")

        compare(box.count, 6)
        compare(box.currentIndex, 0)
        compare(box.currentText, "Normalsicht")

        box.activated(3)

        compare(fake.calls, [["s12-3", "angle", "birds-eye"]])
    }

    function test_the_current_perspective_is_not_sent_again() {
        const box = field(make(null), "angleBox")

        box.activated(0)

        compare(fake.calls.length, 0)
    }

    function test_the_current_camera_is_not_sent_again() {
        const box = field(make(null), "cameraBox")

        box.activated(1)

        compare(fake.calls.length, 0)
    }

    function test_new_shot_works_without_a_selected_shot() {
        const panel = createTemporaryObject(panelComponent, tc, { shot: null })
        fake.shotCalls = []

        const button = field(panel, "newShotButton")
        verify(button.enabled)
        button.clicked()

        compare(fake.shotCalls, [["add"]])
        verify(!field(panel, "copyShotButton").enabled)
        verify(!field(panel, "deleteShotButton").enabled)
    }

    function test_copy_sends_the_selected_shot() {
        const panel = make(null)

        field(panel, "copyShotButton").clicked()

        compare(fake.shotCalls, [["copy", "s12-3"]])
    }

    function test_delete_asks_first_and_sends_only_when_confirmed() {
        const panel = make(null)
        const dialog = field(panel, "confirmDelete")

        field(panel, "deleteShotButton").clicked()

        verify(dialog.opened || dialog.visible)
        compare(fake.shotCalls.length, 0)

        dialog.reject()
        compare(fake.shotCalls.length, 0)

        field(panel, "deleteShotButton").clicked()
        dialog.accept()
        compare(fake.shotCalls, [["delete", "s12-3"]])
    }

    // Review of PR 29: S adds a shot, except while a field takes the letter.
    function test_s_adds_a_shot() {
        const panel = make(null)

        keyClick(Qt.Key_S)

        compare(fake.shotCalls, [["add"]])
    }

    function test_s_in_a_field_adds_no_shot() {
        const panel = make(null)
        const number = field(panel, "numberField")
        number.forceActiveFocus()
        verify(number.activeFocus)

        keyClick(Qt.Key_S)

        compare(fake.shotCalls.length, 0)
    }
}
