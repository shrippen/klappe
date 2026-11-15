import QtQuick
import QtTest
import "../../qml"

// The drawing card (LH-19): colour, line and fill of the selected drawing.
TestCase {
    id: tc
    name: "DrawingCard"
    width: 800
    height: 300
    when: windowShown
    visible: true

    QtObject {
        id: fake
        property string planJson: "{}"
        property string selectedKey: ""
        property var styles: []
        property string closed: "false"
        property string choicesJson: JSON.stringify({
            color: ["red", "orange", "yellow", "green", "cyan", "blue", "purple"],
            dash: ["solid", "dashed", "dotted"],
            fill: ["outline", "filled"],
        })

        function describe(key) {
            const closedShape = key.indexOf("d2") >= 0
            return JSON.stringify({ kind: "drawing", color: "green", dash: "dashed", fill: "outline", closed: closedShape })
        }
        function setDrawingStyle(color, dash, fill) {
            styles.push([color, dash, fill])
            return true
        }
    }

    TestTexts { id: de; language: "de" }

    Component {
        id: cardComponent
        DrawingCard {
            texts: de
            controller: fake
        }
    }

    function init() {
        fake.selectedKey = ""
        fake.styles = []
    }

    function shown(key) {
        fake.selectedKey = key
        const card = createTemporaryObject(cardComponent, tc)
        waitForRendering(card)
        return card
    }

    function test_only_a_selected_drawing_has_a_card() {
        const card = shown("")
        verify(!card.visible)

        fake.selectedKey = "object:o1"
        verify(!card.visible)

        fake.selectedKey = "drawing:d1"
        verify(card.visible)
        fake.selectedKey = "drawingpoint:d1:0"
        verify(card.visible)
    }

    function test_the_controls_show_the_style_of_the_drawing() {
        const card = shown("drawing:d1")

        compare(findChild(card, "drawingColor").currentIndex, 3)
        compare(findChild(card, "drawingDash").currentValue, "dashed")
        compare(findChild(card, "drawingFill").currentValue, "outline")
    }

    function test_the_fill_shows_for_a_shape_with_an_inside() {
        const line = shown("drawing:d1")
        verify(!findChild(line, "drawingFill").visible)

        const box = shown("drawing:d2")
        verify(findChild(box, "drawingFill").visible)
    }

    function test_a_pick_sets_the_style_with_that_part_changed() {
        const card = shown("drawing:d2")

        findChild(card, "drawingColor").picked(5)
        compare(fake.styles[0], ["blue", "dashed", "outline"])

        const dash = findChild(card, "drawingDash")
        dash.currentIndex = 2
        dash.activated(2)
        compare(fake.styles[1], ["green", "dotted", "outline"])

        const fill = findChild(card, "drawingFill")
        fill.currentIndex = 1
        fill.activated(1)
        compare(fake.styles[2], ["green", "dashed", "filled"])
    }
}
