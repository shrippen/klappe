import QtQuick
import QtTest
import "../../qml"

// The object section of the selection card (LH-25, LH-17, LH-12, D-04): size, height, filing.
TestCase {
    id: tc
    name: "ObjectCard"
    width: 600
    height: 400
    when: windowShown
    visible: true

    QtObject {
        id: fake
        property string planJson: "{}"
        property string selectedKey: ""
        property var sizes: []
        property bool sizeOk: true
        property var heights: []
        property bool heightOk: true
        property var bases: []
        property bool baseOk: true
        property var filed: []
        property bool fileOk: true

        function describe(key) {
            if (key === "object:o1") {
                return JSON.stringify({ kind: "object", label: "", library: "table", width: 1.6, depth: 0.9, height: null, base: 0, baseDefault: true })
            }
            if (key === "object:o2") {
                return JSON.stringify({ kind: "object", label: "Thron", library: "thron", width: 0.9, depth: 0.8, height: 0.8, base: 0.4, baseDefault: false })
            }
            if (key === "object:o3") {
                return JSON.stringify({ kind: "object", label: "", library: "zeppelin", width: 12, depth: 3, height: null, base: 0, baseDefault: true })
            }
            return JSON.stringify({ kind: "camera", label: "A", lens: 35, fov: 39.6, height: 1.6, shots: "12.1" })
        }
        function resizeSelected(width, depth) {
            sizes.push([width, depth])
            return sizeOk
        }
        function setObjectHeight(text) {
            heights.push(text)
            return heightOk
        }
        function setSelectedHeight(field, text) {
            bases.push([field, text])
            return baseOk
        }
        function saveToLibrary(name) {
            filed.push(name)
            return fileOk
        }
    }

    TestTexts { id: de; language: "de" }

    Component {
        id: cardComponent
        ObjectCard {
            texts: de
            controller: fake
        }
    }

    function init() {
        fake.selectedKey = ""
        fake.sizes = []
        fake.sizeOk = true
        fake.heights = []
        fake.heightOk = true
        fake.bases = []
        fake.baseOk = true
        fake.filed = []
        fake.fileOk = true
    }

    function make() {
        const card = createTemporaryObject(cardComponent, tc)
        waitForRendering(card)
        return card
    }

    // LH-25: an object selected shows its size in two fields, with the catalogue's separator.
    function test_the_object_card_shows_the_size_of_the_selected_object() {
        const card = make()
        verify(!card.visible)

        fake.selectedKey = "camera:a"
        verify(!card.visible)

        fake.selectedKey = "object:o1"
        verify(card.visible)
        compare(findChild(card, "widthField").text, "1,6")
        compare(findChild(card, "depthField").text, "0,9")

        fake.selectedKey = ""
        verify(!card.visible)
    }

    // LH-17: the height is its own field; an object without one leaves it empty.
    function test_the_object_card_shows_the_height_or_none() {
        const card = make()
        const field = findChild(card, "heightField")
        card.expanded = true

        fake.selectedKey = "object:o2"
        compare(field.text, "0,8")

        fake.selectedKey = "object:o1"
        compare(field.text, "")
    }

    function test_enter_gives_the_height_to_the_controller() {
        const card = make()
        card.expanded = true
        fake.selectedKey = "object:o1"
        const field = findChild(card, "heightField")

        field.forceActiveFocus()
        field.text = "1,2"
        keyClick(Qt.Key_Return)

        compare(fake.heights, ["1,2"])
        compare(fake.sizes, [])
        verify(!field.invalid)
    }

    // The button sets the size and, when it was changed, the height; the typed height
    // survives the refill the size edit causes.
    function test_the_apply_button_gives_a_changed_height_too() {
        const card = make()
        fake.selectedKey = "object:o1"

        findChild(card, "objectApply").clicked()
        compare(fake.heights, [])

        findChild(card, "heightField").text = "2"
        findChild(card, "objectApply").clicked()
        compare(fake.heights, ["2"])
        compare(fake.sizes.length, 2)

        // Emptying the field of an object that has a height takes the height off.
        fake.selectedKey = "object:o2"
        findChild(card, "heightField").text = ""
        findChild(card, "objectApply").clicked()
        compare(fake.heights, ["2", ""])
    }

    // Review of PR 37: a refused size still let Apply set the height.
    function test_a_refused_size_sets_no_height() {
        const card = make()
        fake.selectedKey = "object:o1"
        fake.sizeOk = false

        findChild(card, "widthField").text = "abc"
        findChild(card, "heightField").text = "2"
        findChild(card, "objectApply").clicked()

        compare(fake.heights, [])
        verify(findChild(card, "widthField").invalid)
    }

    function test_a_refused_height_marks_the_field() {
        const card = make()
        fake.selectedKey = "object:o1"
        fake.heightOk = false
        const field = findChild(card, "heightField")

        field.text = "-1"
        findChild(card, "objectApply").clicked()

        verify(field.invalid)
        field.text = "1"
        verify(!field.invalid)
    }

    function test_enter_gives_the_size_to_the_controller() {
        const card = make()
        fake.selectedKey = "object:o1"
        const width = findChild(card, "widthField")

        width.forceActiveFocus()
        width.text = "2,5"
        keyClick(Qt.Key_Return)

        compare(fake.sizes, [["2,5", "0,9"]])
        verify(!width.invalid)
        verify(!findChild(card, "depthField").invalid)
    }

    function test_the_apply_button_gives_the_size_to_the_controller() {
        const card = make()
        fake.selectedKey = "object:o1"

        findChild(card, "depthField").text = "0.5"
        findChild(card, "objectApply").clicked()

        compare(fake.sizes, [["1,6", "0.5"]])
    }

    function test_a_refused_size_marks_both_fields() {
        const card = make()
        fake.selectedKey = "object:o1"
        fake.sizeOk = false
        const width = findChild(card, "widthField")
        const depth = findChild(card, "depthField")

        width.text = "0"
        findChild(card, "objectApply").clicked()

        verify(width.invalid)
        verify(depth.invalid)
        width.text = "1"
        verify(!width.invalid)
    }

    // LH-12: the name of the entry starts with the object's own label, else its kind's name;
    // a kind without a name (an old project) starts empty.
    function test_the_library_name_starts_with_the_objects_name() {
        const card = make()
        const name = findChild(card, "libraryName")
        card.expanded = true

        fake.selectedKey = "object:o1"
        compare(name.text, "Tisch")
        fake.selectedKey = "object:o2"
        compare(name.text, "Thron")
        fake.selectedKey = "object:o3"
        compare(name.text, "")
    }

    function test_filing_gives_the_name_to_the_controller() {
        const card = make()
        card.expanded = true
        fake.selectedKey = "object:o1"
        const name = findChild(card, "libraryName")

        name.text = "Lange Tafel"
        findChild(card, "libraryFile").clicked()

        compare(fake.filed, ["Lange Tafel"])
        verify(!name.invalid)

        name.forceActiveFocus()
        keyClick(Qt.Key_Return)
        compare(fake.filed, ["Lange Tafel", "Lange Tafel"])
    }

    function test_a_refused_name_marks_the_field() {
        const card = make()
        fake.selectedKey = "object:o1"
        fake.fileOk = false
        const name = findChild(card, "libraryName")

        name.text = " "
        findChild(card, "libraryFile").clicked()

        verify(name.invalid)
        name.text = "x"
        verify(!name.invalid)
    }

    // D-04: width and depth are the common fields; height and library wait behind "More".
    function test_more_shows_the_height_and_the_library() {
        const card = make()
        fake.selectedKey = "object:o1"

        verify(findChild(card, "widthField").visible)
        verify(!findChild(card, "heightField").visible)
        verify(!findChild(card, "libraryName").visible)

        findChild(card, "objectMore").clicked()

        verify(findChild(card, "heightField").visible)
        verify(findChild(card, "libraryName").visible)
    }

    // LH-90: the underside shows the object's own value; without one the field is empty and
    // shows where it stands greyed, so stacking stays in force.
    function test_the_underside_shows_its_own_value_or_the_stacked_one_greyed() {
        const card = make()
        const field = findChild(card, "baseField")
        card.expanded = true

        fake.selectedKey = "object:o2"
        compare(field.text, "0,4")
        fake.selectedKey = "object:o1"
        compare([field.text, field.placeholderText], ["", "0"])
    }

    function test_an_untouched_stacked_underside_is_not_pinned() {
        const card = make()
        card.expanded = true
        fake.selectedKey = "object:o1"
        const field = findChild(card, "baseField")

        field.forceActiveFocus()
        keyClick(Qt.Key_Return)

        compare(fake.bases, [])
    }

    function test_the_underside_has_a_visible_label() {
        const card = make()
        card.expanded = true
        fake.selectedKey = "object:o1"

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

        verify(labels.indexOf("Unterkante (m)") >= 0, labels)
        verify(findChild(card, "baseField").visible)
    }

    function test_enter_gives_the_underside_to_the_controller() {
        const card = make()
        card.expanded = true
        fake.selectedKey = "object:o1"
        const field = findChild(card, "baseField")

        field.forceActiveFocus()
        verify(card.typing)
        field.text = "0,4"
        keyClick(Qt.Key_Return)

        compare(fake.bases, [["base", "0,4"]])
        compare(fake.sizes, [])
        verify(!field.invalid)
    }

    // The button sets the underside when it was changed; emptied it stands on what is below.
    function test_the_apply_button_gives_a_changed_underside_too() {
        const card = make()
        fake.selectedKey = "object:o1"

        findChild(card, "objectApply").clicked()
        compare(fake.bases, [])

        findChild(card, "baseField").text = "0.4"
        findChild(card, "objectApply").clicked()
        compare(fake.bases, [["base", "0.4"]])

        fake.selectedKey = "object:o2"
        findChild(card, "baseField").text = ""
        findChild(card, "objectApply").clicked()
        compare(fake.bases, [["base", "0.4"], ["base", ""]])
    }

    function test_a_refused_underside_marks_the_field() {
        const card = make()
        card.expanded = true
        fake.selectedKey = "object:o1"
        fake.baseOk = false
        const field = findChild(card, "baseField")

        field.text = "-1"
        findChild(card, "objectApply").clicked()

        verify(field.invalid)
        field.text = "1"
        verify(!field.invalid)
    }
}
