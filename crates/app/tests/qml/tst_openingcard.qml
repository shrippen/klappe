import QtQuick
import QtTest
import "../../qml"

// The door and window section of the selection card (LH-10, D-04): its width as a field.
TestCase {
    id: tc
    name: "OpeningCard"
    width: 600
    height: 400
    when: windowShown
    visible: true

    QtObject {
        id: fake
        property string planJson: "{}"
        property string selectedKey: ""
        property var widths: []
        property bool widthOk: true

        function describe(key) {
            if (key === "opening:2:0") {
                return JSON.stringify({ kind: "opening", type: "door", width: 0.9 })
            }
            if (key === "opening:0:1") {
                return JSON.stringify({ kind: "opening", type: "window", width: 1.2 })
            }
            return JSON.stringify({ kind: "object", label: "", library: "table", width: 1.6, depth: 0.9, height: null })
        }
        function setOpeningWidth(text) {
            widths.push(text)
            return widthOk
        }
    }

    TestTexts { id: de; language: "de" }

    Component {
        id: cardComponent
        OpeningCard {
            texts: de
            controller: fake
        }
    }

    function init() {
        fake.selectedKey = ""
        fake.widths = []
        fake.widthOk = true
    }

    function make() {
        const card = createTemporaryObject(cardComponent, tc)
        waitForRendering(card)
        return card
    }

    function test_only_a_selected_opening_shows_the_card() {
        const card = make()
        verify(!card.visible)

        fake.selectedKey = "object:o1"
        verify(!card.visible)

        fake.selectedKey = "opening:2:0"
        verify(card.visible)
        compare(findChild(card, "openingTitle").text, "Tür")
        compare(findChild(card, "openingWidthField").text, "0,9")

        fake.selectedKey = "opening:0:1"
        compare(findChild(card, "openingTitle").text, "Fenster")
        compare(findChild(card, "openingWidthField").text, "1,2")

        fake.selectedKey = ""
        verify(!card.visible)
    }

    function test_enter_gives_the_width_to_the_controller() {
        const card = make()
        fake.selectedKey = "opening:2:0"
        const field = findChild(card, "openingWidthField")

        field.forceActiveFocus()
        verify(card.typing)
        field.text = "1,1"
        keyClick(Qt.Key_Return)

        compare(fake.widths, ["1,1"])
        verify(!field.invalid)
    }

    function test_the_button_gives_the_width_to_the_controller() {
        const card = make()
        fake.selectedKey = "opening:2:0"

        findChild(card, "openingWidthField").text = "0.8"
        findChild(card, "openingApply").clicked()

        compare(fake.widths, ["0.8"])
    }

    function test_a_refused_width_marks_the_field() {
        const card = make()
        fake.selectedKey = "opening:2:0"
        fake.widthOk = false
        const field = findChild(card, "openingWidthField")

        field.text = "breit"
        findChild(card, "openingApply").clicked()

        verify(field.invalid)
        field.text = "1"
        verify(!field.invalid)
    }
}
