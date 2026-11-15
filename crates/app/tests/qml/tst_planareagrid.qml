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
}
