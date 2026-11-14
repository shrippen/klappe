import QtQuick
import QtTest
import "../../qml"

// The camera card (LH-32, LH-38): body, sensor, lens set, lens and frame of the selected camera.
TestCase {
    id: tc
    name: "CameraCard"
    width: 800
    height: 300
    when: windowShown
    visible: true

    QtObject {
        id: fake
        property string planJson: "{}"
        property string selectedKey: ""
        property bool fieldOk: true
        // A field the controller refuses without an edit, as it does a text that is no number.
        property string refused: ""
        property var sets: []
        // The sensor the camera has; a set sensor comes back from `describe` as a number.
        property real sensor: 24.89
        property string choicesJson: JSON.stringify({
            camera: {
                bodies: [
                    { id: "super35", name: "Super 35", width_mm: 24.89, aspect: 2.39 },
                    { id: "full-frame", name: "Full Frame", width_mm: 36, aspect: 1.78 },
                ],
                lens_sets: [
                    { id: "primes", stops_mm: [18, 25, 32, 50], zoom: false },
                    { id: "zoom-standard", stops_mm: [24, 70], zoom: true },
                    { id: "zoom-tele", stops_mm: [70, 200], zoom: true },
                ],
            },
        })

        function describe(key) {
            const lens = { "camera:a": 35, "camera:b": 50, "camera:c": 200 }[key]
            return JSON.stringify({
                kind: "camera", label: key.substring(7).toUpperCase(), lens: lens, ownLens: lens,
                sensor: sensor, aspect: 2.39, body: "super35", fov: 39.6, height: 1.6, shots: "",
            })
        }
        function setCameraField(field, value) {
            sets.push([field, value])
            if (field === refused) {
                return false
            }
            const mm = Number(String(value).replace(",", "."))
            if (fieldOk && field === "sensor" && mm > 0) {
                sensor = mm
            }
            // Like the bridge: every edit gives a new plan, so the card reads the camera again.
            planJson = JSON.stringify({ edits: sets.length })
            return fieldOk
        }
    }

    TestTexts { id: de; language: "de" }
    TestTexts { id: en; language: "en" }

    Component {
        id: cardComponent
        CameraCard {
            texts: de
            controller: fake
        }
    }

    function init() {
        fake.selectedKey = ""
        fake.planJson = "{}"
        fake.sets = []
        fake.fieldOk = true
        fake.sensor = 24.89
        fake.refused = ""
    }

    function shown(key, texts) {
        fake.selectedKey = key
        const card = createTemporaryObject(cardComponent, tc, texts ? { texts: texts } : {})
        waitForRendering(card)
        return card
    }

    function test_only_a_selected_camera_has_a_card() {
        const card = shown("")
        verify(!card.visible)

        fake.selectedKey = "object:o1"
        verify(!card.visible)
        fake.selectedKey = "cameravia:a:1"
        verify(!card.visible)

        fake.selectedKey = "camera:a"
        verify(card.visible)
    }

    function test_the_fields_show_the_camera_with_the_catalogues_separator() {
        const card = shown("camera:a")

        compare(findChild(card, "cameraSensorField").text, "24,89")
        compare(findChild(card, "cameraLensField").text, "35")
        compare(findChild(card, "cameraAspectField").text, "2,39")
        compare(findChild(card, "cameraBodyBox").currentText, "Super 35")
    }

    function test_english_fields_have_points() {
        const card = shown("camera:a", en)

        compare(findChild(card, "cameraSensorField").text, "24.89")
        compare(findChild(card, "cameraAspectField").text, "2.39")
    }

    function test_the_bodies_are_listed_after_the_custom_sensor() {
        const card = shown("camera:a")
        const box = findChild(card, "cameraBodyBox")

        compare(box.count, 3)
        compare(box.textAt(0), "Eigener Sensor")
        compare(box.textAt(2), "Full Frame")
    }

    function test_choosing_a_body_gives_its_id_to_the_controller() {
        const card = shown("camera:a")
        const box = findChild(card, "cameraBodyBox")

        box.currentIndex = 2
        box.activated(2)

        compare(fake.sets, [["body", "full-frame"]])
    }

    function test_the_custom_entry_sets_nothing() {
        const card = shown("camera:a")
        const box = findChild(card, "cameraBodyBox")

        box.currentIndex = 0
        box.activated(0)

        compare(fake.sets, [])
    }

    // The set that holds the camera's lens starts chosen; else the first one.
    function test_the_lens_set_starts_with_the_one_holding_the_lens() {
        const first = shown("camera:a")
        compare(findChild(first, "cameraLensSetBox").currentIndex, 0)

        const tele = shown("camera:c")
        compare(findChild(tele, "cameraLensSetBox").currentIndex, 2)
        compare(findChild(tele, "cameraLensBox").currentText, "200 mm")
    }

    function test_the_lens_box_lists_the_stops_of_the_chosen_set() {
        const card = shown("camera:a")
        const lenses = findChild(card, "cameraLensBox")
        const sets = findChild(card, "cameraLensSetBox")

        compare(lenses.count, 4)
        compare(sets.textAt(0), "Festbrennweiten")

        sets.currentIndex = 1
        sets.activated(1)

        compare(lenses.count, 2)
        compare(lenses.textAt(0), "24 mm")
        compare(lenses.textAt(1), "70 mm")
    }

    function test_choosing_a_lens_gives_millimetres_to_the_controller() {
        const card = shown("camera:a")
        const lenses = findChild(card, "cameraLensBox")

        lenses.currentIndex = 3
        lenses.activated(3)

        compare(fake.sets, [["lens", "50"]])
    }

    function test_enter_sets_the_field_that_was_typed_in() {
        const card = shown("camera:a")
        const field = findChild(card, "cameraSensorField")

        field.forceActiveFocus()
        field.text = "23,5"
        keyClick(Qt.Key_Return)

        compare(fake.sets, [["sensor", "23,5"]])
        verify(!field.invalid)
    }

    function test_enter_in_an_untouched_field_sets_nothing() {
        const card = shown("camera:a")
        const field = findChild(card, "cameraLensField")

        field.forceActiveFocus()
        keyClick(Qt.Key_Return)

        compare(fake.sets, [])
    }

    function test_apply_sets_only_the_typed_fields() {
        const card = shown("camera:a")

        findChild(card, "cameraLensField").text = "40"
        findChild(card, "cameraAspectField").text = "16:9"
        findChild(card, "cameraApply").clicked()

        compare(fake.sets, [["lens", "40"], ["aspect", "16:9"]])
    }

    // Review of PR 33: the first edit refreshed the plan and wrote the camera's old values
    // back into the other fields before they were applied.
    function test_apply_keeps_every_typed_field() {
        const card = shown("camera:a")

        findChild(card, "cameraSensorField").text = "30"
        findChild(card, "cameraLensField").text = "40"
        findChild(card, "cameraApply").clicked()

        compare(fake.sets, [["sensor", "30"], ["lens", "40"]])
    }

    // Review of PR 33: after Apply the field showed the rounded value but remembered the
    // typed one, so the next Apply sent the rounded value and changed the camera.
    function test_a_second_apply_sends_nothing_new() {
        const card = shown("camera:a")

        findChild(card, "cameraSensorField").text = "30.123"
        findChild(card, "cameraApply").clicked()
        findChild(card, "cameraApply").clicked()

        compare(fake.sets, [["sensor", "30.123"]])
    }

    // Review of PR 33: a later field of the same Apply refreshed the plan and wiped the
    // refused value and its mark.
    function test_a_refused_value_stays_beside_an_accepted_one() {
        const card = shown("camera:a")
        const sensor = findChild(card, "cameraSensorField")
        fake.refused = "sensor"

        sensor.text = "abc"
        findChild(card, "cameraLensField").text = "40"
        findChild(card, "cameraApply").clicked()

        compare(sensor.text, "abc")
        verify(sensor.invalid)
    }

    // Review of PR 33: Enter in one field wrote the camera's values over the others, even
    // with nothing sent.
    function test_enter_without_change_leaves_the_other_fields() {
        const card = shown("camera:a")
        const lens = findChild(card, "cameraLensField")
        const sensor = findChild(card, "cameraSensorField")

        lens.text = "40"
        sensor.forceActiveFocus()
        keyClick(Qt.Key_Return)

        compare(fake.sets, [])
        compare(lens.text, "40")
    }

    function test_a_refused_value_marks_the_field_until_typing() {
        const card = shown("camera:a")
        const field = findChild(card, "cameraAspectField")
        fake.fieldOk = false

        field.text = "0"
        findChild(card, "cameraApply").clicked()

        verify(field.invalid)
        verify(!findChild(card, "cameraLensField").invalid)
        field.text = "2"
        verify(!field.invalid)
    }

    // Another edit changes the plan: the fields show the camera again.
    function test_the_fields_follow_the_camera() {
        const card = shown("camera:a")
        const field = findChild(card, "cameraLensField")
        compare(field.text, "35")

        fake.selectedKey = "camera:b"

        compare(field.text, "50")
    }
}
