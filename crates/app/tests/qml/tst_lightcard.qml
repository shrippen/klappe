import QtQuick
import QtTest
import "../../qml"

// The lamp card (LH-50, LH-51): kind, library lamp, direction, beam, power and colour of the
// selected lamp.
TestCase {
    id: tc
    name: "LightCard"
    width: 800
    height: 300
    when: windowShown
    visible: true

    QtObject {
        id: fake
        property string planJson: "{}"
        property int pictureRevision: 0
        property string selectedKey: ""
        // Like the bridge's `describe`: reading it tells QML of no change.
        property var lamp: ({ temp: 5600, power: 200 })
        property bool fieldOk: true
        property var sets: []
        property string choicesJson: JSON.stringify({
            light: ["fresnel-650", "led-panel", "flag"],
            lightKind: ["fresnel", "led-panel", "flag"],
        })

        function describe(key) {
            const led = key === "light:led"
            const bounce = key === "light:bounce"
            return JSON.stringify({
                kind: "light", label: led ? "LED" : "L1",
                type: led ? "led-panel" : (bounce ? "reflector" : "fresnel"),
                facing: 330, beam: 22.5, temp: led ? lamp.temp : 3200, power: bounce ? 650 : lamp.power,
                tunable: led, gives_light: !bounce,
            })
        }
        function setLightField(field, value) {
            sets.push([field, value])
            if (field === "temp" || field === "power") {
                lamp[field] = Number(value)
            }
            // Like the bridge: every edit gives a new picture revision. The plan changes only
            // with what it draws; the colour and the power it does not draw.
            pictureRevision++
            if (field !== "temp" && field !== "power") {
                planJson = JSON.stringify({ edits: sets.length })
            }
            return fieldOk
        }
    }

    TestTexts { id: de; language: "de" }
    TestTexts { id: en; language: "en" }

    Component {
        id: cardComponent
        LightCard {
            texts: de
            controller: fake
        }
    }

    function init() {
        fake.selectedKey = ""
        fake.planJson = "{}"
        fake.pictureRevision = 0
        fake.lamp = { temp: 5600, power: 200 }
        fake.sets = []
        fake.fieldOk = true
    }

    function shown(key, texts) {
        fake.selectedKey = key
        const card = createTemporaryObject(cardComponent, tc, texts ? { texts: texts } : {})
        waitForRendering(card)
        return card
    }

    function test_only_a_selected_lamp_has_a_card() {
        const card = shown("")
        verify(!card.visible)

        fake.selectedKey = "camera:a"
        verify(!card.visible)

        fake.selectedKey = "light:l1"
        verify(card.visible)
    }

    function test_the_fields_show_the_lamp_with_the_catalogues_separator() {
        const card = shown("light:l1")

        compare(findChild(card, "lightFacingField").text, "330")
        compare(findChild(card, "lightBeamField").text, "22,5")
        compare(findChild(card, "lightPowerField").text, "200")
        compare(findChild(card, "lightTempField").text, "3200")
        compare(findChild(card, "lightKindBox").currentText, "Stufenlinse")
    }

    function test_english_fields_have_points() {
        const card = shown("light:l1", en)

        compare(findChild(card, "lightBeamField").text, "22.5")
    }

    // Only an LED takes another colour; a tungsten lamp keeps its own.
    function test_only_an_led_has_a_colour_to_set() {
        verify(!findChild(shown("light:l1"), "lightTempField").enabled)
        verify(findChild(shown("light:led"), "lightTempField").enabled)
    }

    // A bounce or flag makes no light: no power to set.
    function test_a_passive_lamp_has_no_power_to_set() {
        verify(findChild(shown("light:l1"), "lightPowerField").enabled)
        verify(!findChild(shown("light:bounce"), "lightPowerField").enabled)
    }

    // A lamp that gives no light shows no power, whatever an old file still holds.
    function test_a_passive_lamp_shows_no_power() {
        compare(findChild(shown("light:bounce"), "lightPowerField").text, "")
        compare(findChild(shown("light:l1"), "lightPowerField").text, "200")
    }

    function test_choosing_a_kind_gives_its_name_to_the_controller() {
        const card = shown("light:l1")
        const box = findChild(card, "lightKindBox")

        box.currentIndex = 1
        box.activated(1)

        compare(fake.sets, [["kind", "led-panel"]])
    }

    function test_choosing_a_library_lamp_gives_its_id_to_the_controller() {
        const card = shown("light:l1")
        const box = findChild(card, "lightPresetBox")

        compare(box.count, 4)
        compare(box.textAt(1), "Stufenlinse 650 W")
        box.currentIndex = 2
        box.activated(2)

        compare(fake.sets, [["preset", "led-panel"]])
    }

    function test_the_pick_entry_sets_nothing() {
        const card = shown("light:l1")
        const box = findChild(card, "lightPresetBox")

        box.currentIndex = 0
        box.activated(0)

        compare(fake.sets, [])
    }

    function test_enter_sets_the_field_that_was_typed_in() {
        const card = shown("light:l1")
        const field = findChild(card, "lightBeamField")

        field.forceActiveFocus()
        field.text = "45"
        keyClick(Qt.Key_Return)

        compare(fake.sets, [["beam", "45"]])
        verify(!field.invalid)
    }

    function test_apply_sets_only_the_typed_fields() {
        const card = shown("light:l1")

        findChild(card, "lightFacingField").text = "90"
        findChild(card, "lightPowerField").text = "1000"
        findChild(card, "lightApply").clicked()

        compare(fake.sets, [["facing", "90"], ["power", "1000"]])
    }

    function test_a_second_apply_sends_nothing_new() {
        const card = shown("light:l1")

        findChild(card, "lightBeamField").text = "45"
        findChild(card, "lightApply").clicked()
        findChild(card, "lightApply").clicked()

        compare(fake.sets, [["beam", "45"]])
    }

    function test_a_refused_value_marks_the_field_until_typing() {
        const card = shown("light:l1")
        const field = findChild(card, "lightPowerField")
        fake.fieldOk = false

        field.text = "-1"
        findChild(card, "lightApply").clicked()

        verify(field.invalid)
        verify(!findChild(card, "lightBeamField").invalid)
        field.text = "1"
        verify(!field.invalid)
    }

    function test_the_fields_follow_the_lamp() {
        const card = shown("light:l1")
        compare(findChild(card, "lightTempField").text, "3200")

        fake.selectedKey = "light:led"

        compare(findChild(card, "lightTempField").text, "5600")
    }

    // The plan draws no colour: the card reads the lamp again after every edit, not only
    // after one that changes the plan (QA round 2).
    function test_a_new_colour_stays_in_the_field() {
        const card = shown("light:led")
        const field = findChild(card, "lightTempField")

        field.text = "4000"
        findChild(card, "lightApply").clicked()

        compare(fake.sets, [["temp", "4000"]])
        compare(field.text, "4000")
    }

    function test_a_new_power_stays_in_the_field() {
        const card = shown("light:led")
        const field = findChild(card, "lightPowerField")

        field.text = "300"
        findChild(card, "lightApply").clicked()

        compare(fake.sets, [["power", "300"]])
        compare(field.text, "300")
    }
}
