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
        property int pictureRevision: 0
        property string selectedKey: ""
        // Like the bridge's `describe`: reading it tells QML of no change.
        property var wall: ({ height: 1.1 })
        property var heights: []
        property bool heightOk: true

        function describe(key) {
            if (key === "wall:0") {
                return JSON.stringify({ kind: "wall", height: 2.6, heightDefault: true })
            }
            if (key === "wall:1") {
                return JSON.stringify({ kind: "wall", height: wall.height, heightDefault: false })
            }
            return JSON.stringify({ kind: "object", label: "", library: "table", width: 1.6, depth: 0.9, height: null })
        }
        function setSelectedField(field, text) {
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
        fake.wall = { height: 1.1 }
        fake.pictureRevision = 0
        fake.heights = []
        fake.heightOk = true
    }

    // Types `text` into `field` and leaves it (D-46): a changed value applies, as on Enter.
    function typeAndLeave(card, field, text) {
        field.forceActiveFocus()
        field.text = text
        card.forceActiveFocus()
    }

    function make() {
        const card = createTemporaryObject(cardComponent, tc)
        waitForRendering(card)
        return card
    }

    // The field shows the wall's own height; without one it is empty and shows the room's
    // greyed, so the room's is not taken for the wall's own.
    function test_only_a_selected_wall_shows_the_card() {
        const card = make()
        verify(!card.visible)

        fake.selectedKey = "object:o1"
        verify(!card.visible)

        fake.selectedKey = "wall:0"
        verify(card.visible)
        compare(card.title, "Wand")
        compare(findChild(card, "wallHeightField").text, "")
        compare(findChild(card, "wallHeightField").placeholderText, "2,6")

        fake.selectedKey = "wall:1"
        compare(findChild(card, "wallHeightField").text, "1,1")

        fake.selectedKey = ""
        verify(!card.visible)
    }

    // A drag shows its preview through the plan, and the edit at its end gives the same plan:
    // the card reads the wall again on the picture revision, which every edit raises.
    function test_the_card_follows_an_edit_that_leaves_the_plan_as_previewed() {
        const card = make()
        fake.selectedKey = "wall:1"
        fake.planJson = JSON.stringify({ preview: 1 })

        fake.wall.height = 2.2
        fake.pictureRevision++

        compare(findChild(card, "wallHeightField").text, "2,2")
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

    // D-46: leaving the field applies what was typed; no button.
    function test_leaving_the_field_gives_the_height_to_the_controller() {
        const card = make()
        fake.selectedKey = "wall:0"

        tc.typeAndLeave(card, findChild(card, "wallHeightField"), "2.2")

        compare(fake.heights, [["wall", "2.2"]])
        verify(!findChild(card, "wallApply"))
    }

    // Applying an untouched field changes nothing: the room's height stays the wall's default
    // and follows the room.
    function test_an_untouched_default_is_not_pinned() {
        const card = make()
        fake.selectedKey = "wall:0"
        const field = findChild(card, "wallHeightField")

        field.forceActiveFocus()
        card.forceActiveFocus()
        compare(fake.heights, [])
        field.forceActiveFocus()
        keyClick(Qt.Key_Return)

        compare(fake.heights, [])
    }

    // An emptied field takes the wall's own height off: the room's again.
    function test_an_emptied_field_goes_back_to_the_room_height() {
        const card = make()
        fake.selectedKey = "wall:1"

        tc.typeAndLeave(card, findChild(card, "wallHeightField"), "")

        compare(fake.heights, [["wall", ""]])
    }

    function test_a_refused_height_marks_the_field() {
        const card = make()
        fake.selectedKey = "wall:0"
        fake.heightOk = false
        const field = findChild(card, "wallHeightField")

        tc.typeAndLeave(card, field, "hoch")

        verify(field.invalid)
        field.text = "1"
        verify(!field.invalid)
    }
}
