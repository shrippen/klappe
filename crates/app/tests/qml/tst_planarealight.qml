import QtQuick
import QtTest
import "../../qml"

// PlanArea and lamps (LH-50, LH-51): the tooltip of a lamp, and which selection shows the lamp
// card.
TestCase {
    id: tc
    name: "PlanAreaLight"
    width: 600
    height: 400
    when: windowShown
    visible: true

    QtObject {
        id: fake
        property bool loaded: true
        property string planJson: ""
        property string choicesJson: ""
        property string selectedKey: ""
        property int selectedCount: 0

        function markInfo() { return "{}" }
        function describe(key) {
            const lamps = {
                "light:l1": { type: "fresnel", label: "L1", beam: 30, temp: 3200, power: 650, gives_light: true },
                "light:l3": { type: "reflector", label: "L3", beam: 90, temp: 5600, power: 650, gives_light: false },
                "light:l2": { type: "flag", label: "L2", beam: 20, temp: 5600, power: 0, gives_light: false },
            }
            const camera = { kind: "camera", label: "A", lens: 35, ownLens: 35, sensor: 24.89, aspect: 2.39, body: "", fov: 39.6, height: 1.6, shots: "" }

            return JSON.stringify(lamps[key] ? Object.assign({ kind: "light", facing: 0, tunable: false }, lamps[key]) : camera)
        }
    }

    TestTexts { id: de; language: "de" }

    Component {
        id: areaComponent
        PlanArea {
            width: 600
            height: 400
            controller: fake
            texts: de
        }
    }

    function init() {
        fake.selectedKey = ""
    }

    // The tooltip names kind, power, colour and beam; a flag has neither power nor colour, nor a
    // bounce, whatever power it carries.
    function test_a_lamp_tooltip_names_kind_power_colour_and_beam() {
        const area = createTemporaryObject(areaComponent, tc)

        compare(area.tipLines("light:l1"), ["L1 · Stufenlinse · 650 W · 3200 K · 30°"])
        compare(area.tipLines("light:l2"), ["L2 · Abschatter · 20°"])
        compare(area.tipLines("light:l3"), ["L3 · Reflektor · 90°"])
    }

    // A selected lamp shows the lamp card, and only the lamp card.
    function test_the_lamp_card_belongs_to_a_selected_lamp() {
        const area = createTemporaryObject(areaComponent, tc)
        const card = findChild(area, "lightCard")
        verify(!card.visible)

        fake.selectedKey = "light:l1"
        verify(card.visible)
        verify(!findChild(area, "cameraCard").visible)

        // D-04: a camera's card opens at the element, in the selection card (its own tests).
        fake.selectedKey = "camera:a"
        verify(!card.visible)
    }
}
