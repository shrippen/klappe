import QtQuick
import QtTest
import "../../qml"

// ShotPanel: what leaving or editing a field sends to the controller (review gui-r1).
TestCase {
    id: tc
    name: "ShotPanel"
    width: 480
    height: 700
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
        property var calls: []
        property var follows: []
        property bool accept: true

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
            movement: "static", lens_mm: own === null ? 50 : own, own_lens_mm: own,
            description: "", props: "", gear: "", crew: "", note: "", axis_jump: false
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

    function make(own, follows) {
        fake.calls = []
        fake.follows = []
        fake.accept = true
        const panel = createTemporaryObject(panelComponent, tc, { shot: row(own, follows) })
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
}
