import QtQuick
import QtTest
import "../../qml"

// The door and window section of the selection card (LH-10, LH-90, D-04): width, sill and head
// as fields.
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
        property var heights: []
        property bool heightOk: true

        function describe(key) {
            if (key === "opening:2:0") {
                return JSON.stringify({ kind: "opening", type: "door", width: 0.9, sill: 0, head: 2.1, sillDefault: true, headDefault: true, side: "left", hinge: "end" })
            }
            if (key === "opening:0:1") {
                return JSON.stringify({ kind: "opening", type: "window", width: 1.2, sill: 0.9, head: 2.4, sillDefault: true, headDefault: false, side: "right", hinge: "start" })
            }
            return JSON.stringify({ kind: "object", label: "", library: "table", width: 1.6, depth: 0.9, height: null })
        }
        function setOpeningWidth(text) {
            widths.push(text)
            return widthOk
        }
        function setSelectedField(field, text) {
            heights.push([field, text])
            return heightOk
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
        fake.heights = []
        fake.heightOk = true
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

    // LH-90: sill and head show the opening's own value; without one the field is empty and
    // shows its kind's greyed.
    function test_sill_and_head_show_own_values_and_defaults_greyed() {
        const card = make()
        const sill = findChild(card, "openingSillField")
        const head = findChild(card, "openingHeadField")

        fake.selectedKey = "opening:2:0"
        compare([sill.text, sill.placeholderText], ["", "0"])
        compare([head.text, head.placeholderText], ["", "2,1"])

        fake.selectedKey = "opening:0:1"
        compare([sill.text, sill.placeholderText], ["", "0,9"])
        compare(head.text, "2,4")
    }

    // Enter or the button on untouched fields keeps the kind's values as defaults.
    function test_untouched_defaults_are_not_pinned() {
        const card = make()
        fake.selectedKey = "opening:2:0"
        const sill = findChild(card, "openingSillField")

        sill.forceActiveFocus()
        keyClick(Qt.Key_Return)
        findChild(card, "openingApply").clicked()

        compare(fake.heights, [])
    }

    // Every field has a label of its own, not only a placeholder.
    function test_sill_and_head_have_a_visible_label() {
        const card = make()
        fake.selectedKey = "opening:2:0"

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

        verify(labels.indexOf("Brüstung (m)") >= 0, labels)
        verify(labels.indexOf("Sturz (m)") >= 0, labels)
    }

    function test_enter_gives_sill_and_head_to_the_controller() {
        const card = make()
        fake.selectedKey = "opening:2:0"
        const sill = findChild(card, "openingSillField")
        const head = findChild(card, "openingHeadField")

        sill.forceActiveFocus()
        verify(card.typing)
        sill.text = "0,2"
        keyClick(Qt.Key_Return)
        head.forceActiveFocus()
        head.text = "2,2"
        keyClick(Qt.Key_Return)

        compare(fake.heights, [["sill", "0,2"], ["head", "2,2"]])
        compare(fake.widths, [])
        verify(!sill.invalid)
        verify(!head.invalid)
    }

    // The button sets the width and what was changed of sill and head; an emptied field sets
    // the kind's value again.
    function test_the_button_gives_a_changed_sill_and_head_too() {
        const card = make()
        fake.selectedKey = "opening:0:1"

        findChild(card, "openingApply").clicked()
        compare(fake.widths.length, 1)
        compare(fake.heights, [])

        findChild(card, "openingSillField").text = "0.3"
        findChild(card, "openingHeadField").text = ""
        findChild(card, "openingApply").clicked()

        compare(fake.heights, [["sill", "0.3"], ["head", ""]])
    }

    function test_a_refused_sill_or_head_marks_the_field() {
        const card = make()
        fake.selectedKey = "opening:2:0"
        fake.heightOk = false
        const sill = findChild(card, "openingSillField")
        const head = findChild(card, "openingHeadField")

        sill.text = "-1"
        head.text = "hoch"
        findChild(card, "openingApply").clicked()

        verify(sill.invalid)
        verify(head.invalid)
        sill.text = "1"
        verify(!sill.invalid)
    }

    function test_a_refused_width_sets_no_sill_or_head() {
        const card = make()
        fake.selectedKey = "opening:2:0"
        fake.widthOk = false

        findChild(card, "openingWidthField").text = "breit"
        findChild(card, "openingSillField").text = "0.3"
        findChild(card, "openingApply").clicked()

        compare(fake.heights, [])
    }

    // LH-10: a door shows its swing and sends a pick as `side` or `hinge`; a window has none.
    function test_a_door_shows_and_sets_its_swing() {
        const card = make()
        const side = findChild(card, "sideChoice")
        const hinge = findChild(card, "hingeChoice")

        fake.selectedKey = "opening:2:0"
        verify(findChild(card, "swingRow").visible)
        compare([side.currentIndex, hinge.currentIndex], [0, 1])

        side.select(1)
        hinge.select(0)

        compare(fake.heights, [["side", "right"], ["hinge", "start"]])

        fake.selectedKey = "opening:0:1"
        verify(!findChild(card, "swingRow").visible)
    }
}
