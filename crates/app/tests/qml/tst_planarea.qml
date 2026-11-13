import QtQuick
import QtTest
import "../../qml"

// PlanArea tooltip: numbers use the catalogue's decimal separator (review gui-r1).
TestCase {
    id: tc
    name: "PlanArea"
    width: 600
    height: 400
    when: windowShown

    QtObject {
        id: fake
        property bool loaded: true
        property string planJson: ""

        function describe(key) {
            return JSON.stringify({ kind: "camera", label: "A", lens: 35, fov: 39.6, height: 1.6, shots: "12.1" })
        }
    }

    TestTexts { id: de; language: "de" }
    TestTexts { id: en; language: "en" }

    Component {
        id: areaComponent
        PlanArea {
            width: 600
            height: 400
            controller: fake
        }
    }

    function test_german_tooltip_has_commas() {
        const area = createTemporaryObject(areaComponent, tc, { texts: de })

        compare(area.tipLines("camera:a"), ["Kamera A", "35 mm · 39,6°", "Höhe 1,6 m", "Shots 12.1"])
    }

    function test_english_tooltip_has_points() {
        const area = createTemporaryObject(areaComponent, tc, { texts: en })
        const lines = area.tipLines("camera:a")

        verify(lines[1].indexOf("39.6") >= 0, lines[1])
        verify(lines[2].indexOf("1.6") >= 0, lines[2])
    }
}
