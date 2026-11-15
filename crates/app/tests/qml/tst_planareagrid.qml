import QtQuick
import QtTest
import "../../qml"

// PlanArea grid (LH-14): the grid drawn is the one chosen.
TestCase {
    id: tc
    name: "PlanAreaGrid"
    width: 600
    height: 400
    when: windowShown
    visible: true

    QtObject {
        id: fake
        property bool loaded: true
        property string planJson: ""
        property string choicesJson: ""
        property string gridName: ""
        property string selectedKey: ""
        property int selectedCount: 0
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
        fake.choicesJson = ""
        fake.gridName = ""
    }

    // Without a choice, the old metre.
    function test_grid_follows_the_chosen_spacing() {
        const area = createTemporaryObject(areaComponent, tc)
        compare(area.gridStepM, 1)

        fake.choicesJson = JSON.stringify({ grid: [{ id: "cm10", step_m: 0.1 }, { id: "cm25", step_m: 0.25 }] })
        fake.gridName = "cm10"
        compare(area.gridStepM, 0.1)

        fake.gridName = "cm25"
        compare(area.gridStepM, 0.25)
    }

    // Review of PR 86: a grid too fine for the zoom is not drawn; the overview then shows 1 m.
    function test_a_too_fine_grid_falls_back_to_the_metre() {
        const area = createTemporaryObject(areaComponent, tc)
        fake.choicesJson = JSON.stringify({ grid: [{ id: "cm5", step_m: 0.05 }] })
        fake.gridName = "cm5"
        const canvas = findChild(area, "planCanvas")

        canvas.zoom = 1000
        compare(area.drawnGridM, 0.05)

        canvas.zoom = 20
        compare(area.drawnGridM, 1)
    }
}
