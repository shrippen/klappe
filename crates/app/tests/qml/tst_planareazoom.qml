import QtQuick
import QtTest
import "../../qml"

// PlanArea zoom (NF-06): the keys of the window zoom the plan about its middle, or fit it.
TestCase {
    id: tc
    name: "PlanAreaZoom"
    width: 600
    height: 400
    when: windowShown
    visible: true

    QtObject {
        id: fake
        property bool loaded: true
        property string planJson: JSON.stringify({ width: 10, depth: 8, shapes: [] })
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

    function test_zoom_steps_and_fit() {
        const area = createTemporaryObject(areaComponent, tc)
        const canvas = findChild(area, "planCanvas")
        const fitted = canvas.zoom

        area.zoomBy("in")
        fuzzyCompare(canvas.zoom, fitted * area.zoomStep, 1e-6)

        area.zoomBy("out")
        area.zoomBy("out")
        fuzzyCompare(canvas.zoom, fitted / area.zoomStep, 1e-6)

        area.zoomBy("fit")
        fuzzyCompare(canvas.zoom, fitted, 1e-6)
    }
}
