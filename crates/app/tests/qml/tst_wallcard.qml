import QtQuick
import QtTest
import "../../qml"

// The wall section of the selection card (LH-90, D-04): its height as a field.
TestCase {
    id: tc
    name: "WallCard"
    width: 600
    height: 400
    when: windowShown
    visible: true

    QtObject {
        id: fake
        property string planJson: "{}"
        property string selectedKey: ""
        property var heights: []
        property bool heightOk: true

        function describe(key) {
            if (key === "wall:0") {
                return JSON.stringify({ kind: "wall", height: 2.6, heightDefault: true })
            }
            if (key === "wall:1") {
                return JSON.stringify({ kind: "wall", height: 1.1, heightDefault: false })
            }
            return JSON.stringify({ kind: "object", label: "", library: "table", width: 1.6, depth: 0.9, height: null })
        }
        function setSelectedHeight(field, text) {
            heights.push([field, text])
            return heightOk
        }
    }

    TestTexts { id: de; language: "de" }

    Component {
        id: cardComponent
        WallCard {
            texts: de
            controller: fake
        }
    }

    function init() {
        fake.selectedKey = ""
        fake.heights = []
        fake.heightOk = true
    }

    function make() {
        const card = createTemporaryObject(cardComponent, tc)
        waitForRendering(card)
        return card
    }

    // The field shows the height in force: the room's when the wall has none of its own.
    function test_only_a_selected_wall_shows_the_card() {
        const card = make()
        verify(!card.visible)

        fake.selectedKey = "object:o1"
        verify(!card.visible)

        fake.selectedKey = "wall:0"
        verify(card.visible)
        compare(findChild(card, "wallTitle").text, "Wand")
        compare(findChild(card, "wallHeightField").text, "2,6")

        fake.selectedKey = "wall:1"
        compare(findChild(card, "wallHeightField").text, "1,1")

        fake.selectedKey = ""
        verify(!card.visible)
    }

    function test_the_height_has_a_visible_label() {
        const card = make()
        fake.selectedKey = "wall:0"

        const labels = []
        const walk = item => {
            for (const child of item.children) {
                if (child.visible && child.text !== undefined && child.text !== "") {
                    labels.push(child.text)
                }
                walk(child)
            }
        }
        walk(card)

        verify(labels.indexOf("Höhe (m)") >= 0, labels)
    }

    function test_enter_gives_the_height_to_the_controller() {
        const card = make()
        fake.selectedKey = "wall:0"
        const field = findChild(card, "wallHeightField")

        field.forceActiveFocus()
        verify(card.typing)
        field.text = "3,1"
        keyClick(Qt.Key_Return)

        compare(fake.heights, [["wall", "3,1"]])
        verify(!field.invalid)
    }

    function test_the_button_gives_the_height_to_the_controller() {
        const card = make()
        fake.selectedKey = "wall:0"

        findChild(card, "wallHeightField").text = "2.2"
        findChild(card, "wallApply").clicked()

        compare(fake.heights, [["wall", "2.2"]])
    }

    // An emptied field takes the wall's own height off: the room's again.
    function test_an_emptied_field_goes_back_to_the_room_height() {
        const card = make()
        fake.selectedKey = "wall:1"

        findChild(card, "wallHeightField").text = ""
        findChild(card, "wallApply").clicked()

        compare(fake.heights, [["wall", ""]])
    }

    function test_a_refused_height_marks_the_field() {
        const card = make()
        fake.selectedKey = "wall:0"
        fake.heightOk = false
        const field = findChild(card, "wallHeightField")

        field.text = "hoch"
        findChild(card, "wallApply").clicked()

        verify(field.invalid)
        field.text = "1"
        verify(!field.invalid)
    }
}
