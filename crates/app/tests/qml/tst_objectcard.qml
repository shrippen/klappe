import QtQuick
import QtTest
import QtQuick.Controls as QQC2
import "../../qml"
import "FieldLabels.js" as FieldLabels

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
        property int pictureRevision: 0
        property string selectedKey: ""
        // Like the bridge's `describe`: reading it tells QML of no change.
        property var table: ({ depth: 0.9 })
        property var sizes: []
        property bool sizeOk: true
        property var heights: []
        property bool heightOk: true
        property var bases: []
        property bool baseOk: true
        property var filed: []
        property bool fileOk: true
        property var changes: []
        property int labels: 0
        property var elementCalls: []
        property string projectJson: JSON.stringify({ elements: [
            { id: "e1", category: "prop", name: "Brief" },
            { id: "e2", category: "costume", name: "Ballkleid" }
        ] })
        property string choicesJson: JSON.stringify({ object: [
            { kind: "table", custom: false, group: "furniture" },
            { kind: "sofa", custom: false, group: "furniture" },
            { kind: "thron", custom: true, label: "Thron", group: "" }
        ] })

        function describe(key) {
            if (key === "object:o1") {
                return JSON.stringify({ kind: "object", label: "", library: "table", width: 1.6, depth: table.depth, height: null, base: 0, baseDefault: true, id: "o1", location: "saal", element: null })
            }
            if (key === "object:o2") {
                return JSON.stringify({ kind: "object", label: "Thron", library: "thron", width: 0.9, depth: 0.8, height: 0.8, base: 0.4, baseDefault: false, id: "o2", location: "saal", element: "e2" })
            }
            if (key === "object:o4") {
                return JSON.stringify({ kind: "object", label: "", library: "sofa", width: 2, depth: 0.9, height: 0.85, base: 0, baseDefault: true, mirrored: true })
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
        function setSelectedField(field, text) {
            bases.push([field, text])
            return baseOk
        }
        function changeSelected(what, arg) {
            changes.push([what, arg])
            return true
        }
        function setObjectElement(location, object, element) {
            elementCalls.push([location, object, element])
            return true
        }
        function labelSelected() {
            labels++
            return true
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
        fake.table = { depth: 0.9 }
        fake.sizes = []
        fake.sizeOk = true
        fake.heights = []
        fake.heightOk = true
        fake.bases = []
        fake.baseOk = true
        fake.filed = []
        fake.fileOk = true
        fake.changes = []
        fake.labels = 0
        fake.elementCalls = []
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

    // A drag shows its preview through the plan, and the edit at its end gives the same plan:
    // the card reads the object again on the picture revision, which every edit raises.
    function test_the_card_follows_an_edit_that_leaves_the_plan_as_previewed() {
        const card = make()
        fake.selectedKey = "object:o1"
        fake.planJson = JSON.stringify({ preview: 1 })

        fake.table.depth = 1.75
        fake.pictureRevision++

        compare(findChild(card, "depthField").text, "1,75")
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

    // D-46: leaving the height field sets the height alone; an untouched field sets nothing.
    function test_leaving_the_height_field_gives_the_height() {
        const card = make()
        card.expanded = true
        fake.selectedKey = "object:o1"

        tc.typeAndLeave(card, findChild(card, "heightField"), "")
        compare(fake.heights, [])

        tc.typeAndLeave(card, findChild(card, "heightField"), "2")
        compare(fake.heights, ["2"])
        compare(fake.sizes, [])

        // Emptying the field of an object that has a height takes the height off.
        fake.selectedKey = "object:o2"
        tc.typeAndLeave(card, findChild(card, "heightField"), "")
        compare(fake.heights, ["2", ""])
        verify(!findChild(card, "objectApply"))
    }

    function test_a_refused_height_marks_the_field() {
        const card = make()
        fake.selectedKey = "object:o1"
        fake.heightOk = false
        card.expanded = true
        const field = findChild(card, "heightField")

        tc.typeAndLeave(card, field, "-1")

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

    function test_leaving_a_size_field_gives_the_size_to_the_controller() {
        const card = make()
        fake.selectedKey = "object:o1"

        tc.typeAndLeave(card, findChild(card, "depthField"), "0.5")

        compare(fake.sizes, [["1,6", "0.5"]])
    }

    function test_a_refused_size_marks_both_fields() {
        const card = make()
        fake.selectedKey = "object:o1"
        fake.sizeOk = false
        const width = findChild(card, "widthField")
        const depth = findChild(card, "depthField")

        tc.typeAndLeave(card, width, "0")

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

    // Leaving the underside field sets it when it was changed; emptied it stands on what is below.
    function test_leaving_the_underside_field_gives_the_underside() {
        const card = make()
        card.expanded = true
        fake.selectedKey = "object:o1"

        tc.typeAndLeave(card, findChild(card, "baseField"), "")
        compare(fake.bases, [])

        tc.typeAndLeave(card, findChild(card, "baseField"), "0.4")
        compare(fake.bases, [["base", "0.4"]])

        fake.selectedKey = "object:o2"
        tc.typeAndLeave(card, findChild(card, "baseField"), "")
        compare(fake.bases, [["base", "0.4"], ["base", ""]])
    }

    function test_a_refused_underside_marks_the_field() {
        const card = make()
        card.expanded = true
        fake.selectedKey = "object:o1"
        fake.baseOk = false
        const field = findChild(card, "baseField")

        tc.typeAndLeave(card, field, "-1")

        verify(field.invalid)
        field.text = "1"
        verify(!field.invalid)
    }

    // Q1-03: each number field is named by a label above it, not by a placeholder.
    function test_every_number_field_has_a_visible_label() {
        const card = make()
        card.expanded = true
        fake.selectedKey = "object:o1"
        const labels = FieldLabels.of(card)
        const names = { widthField: "Breite (m)", depthField: "Tiefe (m)", heightField: "Höhe (m)" }

        for (const objectName in names) {
            verify(labels.indexOf(names[objectName]) >= 0, names[objectName] + " in " + labels)
            compare(findChild(card, objectName).Accessible.name, names[objectName])
        }
    }

    // LH-114: the kind is a list of the library under "More"; the object's own kind is picked, a
    // pick changes the kind, and a free form or an unknown kind picks none.
    function test_the_kind_list_shows_the_kind_and_a_pick_changes_it() {
        const card = make()
        const box = findChild(card, "kindBox")
        card.expanded = true

        fake.selectedKey = "object:o1"
        compare(box.currentText, "Tisch")
        fake.selectedKey = "object:o2"
        compare(box.currentText, "Thron")
        fake.selectedKey = "object:o3"
        compare(box.currentIndex, -1)

        fake.selectedKey = "object:o1"
        box.activated(1)
        compare(fake.changes, [["kind", "sofa"]])
    }

    // LH-115, LH-116, LH-118: buttons under "More" mirror, reset the size and label; the mirror
    // button is primary while the object is mirrored.
    function test_the_buttons_mirror_reset_the_size_and_label() {
        const card = make()
        card.expanded = true
        fake.selectedKey = "object:o1"

        findChild(card, "objectMirror").clicked()
        findChild(card, "objectResetSize").clicked()
        findChild(card, "objectLabel").clicked()

        compare(fake.changes, [["mirror", ""], ["resetSize", ""]])
        compare(fake.labels, 1)
    }

    // The mirror button shows its state to the eye and to a screen reader, not by colour alone.
    function test_the_mirror_button_is_checked_while_mirrored() {
        const card = make()
        const button = findChild(card, "objectMirror")

        fake.selectedKey = "object:o1"
        verify(!button.checked)
        fake.selectedKey = "object:o4"
        verify(button.checked)
        verify(button.Accessible.checkable)
    }

    // The tooltip names the key the command has (Shift+G, not Shift+S).
    function test_the_reset_tooltip_names_its_key() {
        const card = make()
        const tip = findChild(card, "objectResetSize").QQC2.ToolTip.text

        verify(tip.indexOf("+G") >= 0, tip)
        verify(tip.indexOf("+S") < 0, tip)
    }

    function test_the_texts_of_the_object_commands_are_in_both_languages() {
        for (const key of ["object.kind", "object.kindTo", "object.mirror", "object.mirrorTip", "object.resetSize",
                           "object.resetSizeTip", "object.label", "object.labelTip", "note.unbind"]) {
            verify(de.t(key) !== key, "de " + key)
        }
    }

    // D-22: the object stands for an element of the breakdown; none is the first entry.
    function test_the_element_box_binds_the_object_to_an_element() {
        const card = make()
        card.expanded = true
        const box = findChild(card, "elementBox")

        fake.selectedKey = "object:o1"
        compare(box.model, ["keins", "Requisite · Brief", "Kostüm · Ballkleid"])
        compare(box.currentIndex, 0)

        box.activated(1)
        compare(fake.elementCalls, [["saal", "o1", "e1"]])

        fake.selectedKey = "object:o2"
        compare(box.currentIndex, 2)
        box.activated(0)
        compare(fake.elementCalls[1], ["saal", "o2", ""])
    }
}
