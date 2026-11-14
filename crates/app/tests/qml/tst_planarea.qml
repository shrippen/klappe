import QtQuick
import QtTest
import "../../qml"

// PlanArea tooltip: numbers use the catalogue's decimal separator (review gui-r1).
TestCase {
    id: tc
    name: "PlanArea"
    width: 600
    height: 400
    when: windowShown
    visible: true

    QtObject {
        id: fake
        property bool loaded: true
        property string planJson: ""
        property string selectedKey: ""
        property int selectedCount: 0
        property bool extendSelection: false
        property int duplicates: 0
        property var drags: []
        property var places: []
        property var turns: []
        property int removes: 0
        property int straightens: 0
        property int cancels: 0
        property int scalePoints: 0
        property var calibrations: []
        property int scaleCancels: 0
        property bool calibrateOk: true
        property bool asksNote: false
        property string noteText: ""
        property var notes: []
        property int noteCancels: 0
        property int noteEdits: 0
        property bool noteOk: true
        property int areaPoints: 0
        property int areaFinishes: 0
        property int areaCancels: 0
        property var sizes: []
        property bool sizeOk: true
        property var filed: []
        property bool fileOk: true

        function markInfo() { return "{}" }
        function describe(key) {
            if (key === "object:o1") {
                return JSON.stringify({ kind: "object", label: "", library: "table", width: 1.6, depth: 0.9 })
            }
            if (key === "object:o2") {
                return JSON.stringify({ kind: "object", label: "Thron", library: "thron", width: 0.9, depth: 0.8 })
            }
            if (key === "object:o3") {
                return JSON.stringify({ kind: "object", label: "", library: "zeppelin", width: 12, depth: 3 })
            }
            if (key === "note:n1") {
                return JSON.stringify({ kind: "note", text: "Vorhang schließen" })
            }
            if (key === "camera:b") {
                return JSON.stringify({ kind: "camera", label: "B", lens: 50, fov: 27.9, height: 1.6, shots: "", follows: "Elisabeth" })
            }
            return JSON.stringify({ kind: "camera", label: "A", lens: 35, fov: 39.6, height: 1.6, shots: "12.1" })
        }
        function drag(fromX, fromY, x, y, done) {
            drags.push({ from: [fromX, fromY], to: [x, y], done: done })
            return true
        }
        function place(x, y) {
            places.push([x, y])
            if (scalePoints < 2) {
                scalePoints++
            }
            return true
        }
        function calibrate(text) {
            calibrations.push(text)
            return calibrateOk
        }
        function commitNote(text) {
            notes.push(text)
            return noteOk
        }
        function cancelNote() {
            noteCancels++
            asksNote = false
        }
        function editSelectedNote() { noteEdits++ }
        function finishArea() {
            areaFinishes++
            return true
        }
        function cancelArea() {
            areaCancels++
            areaPoints = 0
        }
        function cancelScale() {
            scaleCancels++
            scalePoints = 0
        }
        function duplicateSelected() { duplicates++ }
        function removeSelected() { removes++ }
        function straightenSelected() { straightens++ }
        function turnSelected(step) { turns.push(step) }
        function cancelDrag() { cancels++ }
        function resizeSelected(width, depth) {
            sizes.push([width, depth])
            return sizeOk
        }
        function saveToLibrary(name) {
            filed.push(name)
            return fileOk
        }
    }

    TestTexts { id: de; language: "de" }
    TestTexts { id: en; language: "en" }

    Component {
        id: areaComponent
        PlanArea {
            width: 600
            height: 400
            controller: fake
        }
    }

    function test_german_tooltip_has_commas() {
        const area = createTemporaryObject(areaComponent, tc, { texts: de })

        compare(area.tipLines("camera:a"), ["Kamera A", "35 mm · 39,6°", "Höhe 1,6 m", "Shots 12.1"])
    }

    function test_a_following_camera_says_whom() {
        const area = createTemporaryObject(areaComponent, tc, { texts: de })

        verify(area.tipLines("camera:b").indexOf("folgt Elisabeth") >= 0)
    }

    function test_english_tooltip_has_points() {
        const area = createTemporaryObject(areaComponent, tc, { texts: en })
        const lines = area.tipLines("camera:a")

        verify(lines[1].indexOf("39.6") >= 0, lines[1])
        verify(lines[2].indexOf("1.6") >= 0, lines[2])
    }

    function init() {
        fake.planJson = JSON.stringify({ width: 20, depth: 13, shapes: [] })
        fake.selectedKey = ""
        fake.selectedCount = 0
        fake.extendSelection = false
        fake.duplicates = 0
        fake.drags = []
        fake.places = []
        fake.turns = []
        fake.removes = 0
        fake.straightens = 0
        fake.cancels = 0
        fake.scalePoints = 0
        fake.calibrations = []
        fake.scaleCancels = 0
        fake.calibrateOk = true
        fake.asksNote = false
        fake.noteText = ""
        fake.notes = []
        fake.noteCancels = 0
        fake.noteEdits = 0
        fake.noteOk = true
        fake.areaPoints = 0
        fake.areaFinishes = 0
        fake.areaCancels = 0
        fake.sizes = []
        fake.sizeOk = true
        fake.filed = []
        fake.fileOk = true
    }

    function drawn(tool) {
        const area = createTemporaryObject(areaComponent, tc, { texts: de, tool: tool })
        waitForRendering(area)
        return area
    }

    function drag(area, steps) {
        mousePress(area, 100, 100, Qt.LeftButton)
        for (let i = 1; i <= steps; i++) {
            mouseMove(area, 100 + i * 40, 100 + i * 20, 0, Qt.LeftButton)
        }
        mouseRelease(area, 100 + steps * 40, 100 + steps * 20, Qt.LeftButton)
    }

    function test_a_wall_drag_goes_to_the_controller() {
        const area = drawn("wall")

        drag(area, 3)

        verify(fake.drags.length >= 2, fake.drags.length)
        verify(!fake.drags[0].done)
        verify(fake.drags[fake.drags.length - 1].done)
        verify(fake.drags[0].to[0] > fake.drags[0].from[0])
    }

    // LH-19: a drag with a drawing tool draws; the area tool takes clicks.
    function test_a_drawing_tool_drag_goes_to_the_controller() {
        for (const tool of ["line", "arrow", "rect", "ellipse"]) {
            fake.drags = []
            const area = drawn(tool)

            drag(area, 3)

            verify(fake.drags.length >= 2, tool)
            verify(fake.drags[fake.drags.length - 1].done, tool)
        }
    }

    function test_enter_closes_the_area_and_escape_drops_it() {
        const area = drawn("area")
        fake.areaPoints = 3

        keyClick(Qt.Key_Return)
        compare(fake.areaFinishes, 1)
        keyClick(Qt.Key_Enter)
        compare(fake.areaFinishes, 2)

        keyClick(Qt.Key_Escape)
        compare(fake.areaCancels, 1)
        compare(fake.areaPoints, 0)

        // Nothing drawn: the keys stay free.
        keyClick(Qt.Key_Return)
        keyClick(Qt.Key_Escape)
        compare(fake.areaFinishes, 2)
        compare(fake.areaCancels, 1)
    }

    function test_the_area_keys_belong_to_the_area_tool() {
        const area = drawn("wall")
        fake.areaPoints = 3

        keyClick(Qt.Key_Return)
        keyClick(Qt.Key_Escape)

        compare(fake.areaFinishes, 0)
        compare(fake.areaCancels, 0)
    }

    // LH-08: with the select tool a drag on empty ground is reported (the controller pulls up
    // a box), not taken for a pan; the middle button still pans.
    function test_select_drags_on_empty_ground() {
        const area = drawn("select")

        drag(area, 3)

        verify(fake.drags.length >= 2, fake.drags.length)
        verify(fake.drags[fake.drags.length - 1].done)
    }

    function test_select_drags_what_the_pointer_is_over() {
        const area = drawn("select")
        fake.planJson = JSON.stringify({ width: 20, depth: 13, shapes: [
            { kind: "polygon", points: [0, 0, 20, 0, 20, 13, 0, 13], radius: 0, width: 0, label: "muted",
              stroke: "solid", fill: "solid", text: "", key: "object:t", selected: false }] })
        mouseMove(area, 90, 90)
        mouseMove(area, 100, 100)

        drag(area, 2)

        verify(fake.drags.length >= 1)
    }

    function test_the_middle_button_pans_in_the_select_tool() {
        const area = drawn("select")

        mousePress(area, 100, 100, Qt.MiddleButton)
        mouseMove(area, 160, 140, 0, Qt.MiddleButton)
        mouseRelease(area, 160, 140, Qt.MiddleButton)

        compare(fake.drags.length, 0)
    }

    // Shift at the press extends the selection; the canvas still gets the click and the drag.
    function test_shift_at_the_press_extends_the_selection() {
        const area = drawn("select")

        mouseClick(area, 120, 150, Qt.LeftButton, Qt.ShiftModifier)
        verify(fake.extendSelection)
        compare(fake.places.length, 1)

        mouseClick(area, 120, 150, Qt.LeftButton, Qt.NoModifier)
        verify(!fake.extendSelection)
        compare(fake.places.length, 2)

        mousePress(area, 100, 100, Qt.LeftButton, Qt.ShiftModifier)
        mouseMove(area, 160, 120, 0, Qt.LeftButton)
        mouseMove(area, 200, 140, 0, Qt.LeftButton)
        mouseRelease(area, 200, 140, Qt.LeftButton, Qt.ShiftModifier)
        verify(fake.extendSelection)
        verify(fake.drags.length >= 1)
    }

    // LH-08: several selected elements take the keys, though no single key names them.
    function test_the_keys_act_on_several_elements() {
        const area = drawn("select")
        fake.selectedKey = ""
        fake.selectedCount = 2

        keyClick(Qt.Key_R)
        keyClick(Qt.Key_Delete)
        keyClick(Qt.Key_D, Qt.ControlModifier)

        compare(fake.turns, [15])
        compare(fake.removes, 1)
        compare(fake.duplicates, 1)
    }

    function test_copying_needs_a_selection() {
        const area = drawn("select")

        keyClick(Qt.Key_D, Qt.ControlModifier)
        compare(fake.duplicates, 0)

        fake.selectedKey = "object:o1"
        keyClick(Qt.Key_D, Qt.ControlModifier)
        compare(fake.duplicates, 1)
    }

    function test_a_click_is_passed_on() {
        const area = drawn("camera")

        mouseClick(area, 120, 150)

        compare(fake.places.length, 1)
        compare(fake.drags.length, 0)
    }

    function test_escape_aborts_the_drag_until_release() {
        const area = drawn("wall")

        mousePress(area, 100, 100, Qt.LeftButton)
        mouseMove(area, 160, 120, 0, Qt.LeftButton)
        mouseMove(area, 200, 140, 0, Qt.LeftButton)
        const before = fake.drags.length
        verify(before >= 1)

        keyClick(Qt.Key_Escape)
        compare(fake.cancels, 1)

        mouseMove(area, 260, 160, 0, Qt.LeftButton)
        mouseRelease(area, 260, 160, Qt.LeftButton)
        compare(fake.drags.length, before)

        // The next drag is a new one.
        drag(area, 2)
        verify(fake.drags.length > before)
    }

    function test_delete_and_turn_keys_act_on_the_selection() {
        const area = drawn("select")

        keyClick(Qt.Key_R)
        compare(fake.turns.length, 0)

        fake.selectedKey = "camera:a"
        keyClick(Qt.Key_R)
        keyClick(Qt.Key_R, Qt.ShiftModifier)
        keyClick(Qt.Key_Delete)

        compare(fake.turns, [15, -15])
        compare(fake.removes, 1)
    }

    // G straightens the selected way, and only with something selected (LH-21).
    function test_g_straightens_the_selected_way() {
        drawn("select")

        keyClick(Qt.Key_G)
        compare(fake.straightens, 0)

        fake.selectedKey = "figurevia:e:1"
        keyClick(Qt.Key_G)
        compare(fake.straightens, 1)
    }

    function test_scale_asks_for_the_length_after_two_clicks() {
        const area = drawn("scale")
        const card = findChild(area, "lengthCard")

        verify(!card.visible)
        mouseClick(area, 120, 150)
        verify(!card.visible)
        mouseClick(area, 300, 150)

        compare(fake.places.length, 2)
        verify(card.visible)
        verify(findChild(area, "lengthField").activeFocus)
    }

    function test_enter_gives_the_length_to_the_controller() {
        const area = drawn("scale")
        mouseClick(area, 120, 150)
        mouseClick(area, 300, 150)

        const field = findChild(area, "lengthField")
        field.text = "4,5"
        keyClick(Qt.Key_Return)

        compare(fake.calibrations, ["4,5"])
        verify(!field.invalid)
    }

    function test_a_refused_length_marks_the_field() {
        const area = drawn("scale")
        mouseClick(area, 120, 150)
        mouseClick(area, 300, 150)
        fake.calibrateOk = false

        const field = findChild(area, "lengthField")
        field.text = "abc"
        keyClick(Qt.Key_Return)

        verify(field.invalid)
        field.text = "4"
        verify(!field.invalid)
    }

    function test_escape_drops_the_scale_points() {
        const area = drawn("scale")
        mouseClick(area, 120, 150)
        mouseClick(area, 300, 150)

        keyClick(Qt.Key_Escape)

        compare(fake.scaleCancels, 1)
        verify(!findChild(area, "lengthCard").visible)
    }

    function test_the_length_card_is_for_the_scale_tool_only() {
        const area = drawn("camera")
        mouseClick(area, 120, 150)
        mouseClick(area, 300, 150)

        verify(!findChild(area, "lengthCard").visible)
    }

    function test_scale_drags_go_to_the_controller() {
        const area = drawn("scale")

        drag(area, 3)

        verify(fake.drags.length >= 2)
        verify(fake.drags[fake.drags.length - 1].done)
    }

    function test_a_note_tooltip_shows_its_text() {
        const area = createTemporaryObject(areaComponent, tc, { texts: de })

        compare(area.tipLines("note:n1"), ["Vorhang schließen"])
    }

    function test_the_note_card_asks_for_the_text_with_its_start_value() {
        const area = drawn("note")
        const card = findChild(area, "noteCard")
        const field = findChild(area, "noteField")

        verify(!card.visible)
        fake.noteText = "Vorhang schließen"
        fake.asksNote = true

        verify(card.visible)
        verify(field.activeFocus)
        compare(field.text, "Vorhang schließen")
    }

    function test_enter_gives_the_note_text_to_the_controller() {
        const area = drawn("note")
        fake.asksNote = true
        const field = findChild(area, "noteField")

        field.text = "Tür quietscht"
        keyClick(Qt.Key_Return)

        compare(fake.notes, ["Tür quietscht"])
        verify(!field.invalid)
    }

    function test_a_refused_note_text_marks_the_field() {
        const area = drawn("note")
        fake.asksNote = true
        fake.noteOk = false
        const field = findChild(area, "noteField")

        field.text = " "
        keyClick(Qt.Key_Return)

        verify(field.invalid)
        field.text = "x"
        verify(!field.invalid)
    }

    function test_escape_drops_the_note_question() {
        const area = drawn("note")
        fake.asksNote = true

        keyClick(Qt.Key_Escape)

        compare(fake.noteCancels, 1)
        verify(!findChild(area, "noteCard").visible)
    }

    // Letters typed into the text are text: no turn, no delete on the selection.
    function test_typing_a_note_leaves_the_selection_alone() {
        const area = drawn("note")
        fake.selectedKey = "note:n1"
        fake.asksNote = true

        keyClick(Qt.Key_R)
        keyClick(Qt.Key_G)
        keyClick(Qt.Key_Delete)

        compare(fake.turns.length, 0)
        compare(fake.straightens, 0)
        compare(fake.removes, 0)
        compare(findChild(area, "noteField").text, "rg")
    }

    function test_f2_edits_a_selected_note_only() {
        const area = drawn("select")

        fake.selectedKey = "camera:a"
        keyClick(Qt.Key_F2)
        compare(fake.noteEdits, 0)

        fake.selectedKey = "note:n1"
        keyClick(Qt.Key_F2)
        compare(fake.noteEdits, 1)
    }

    // LH-25: an object selected shows its size in two fields, with the catalogue's separator.
    function test_the_object_card_shows_the_size_of_the_selected_object() {
        const area = drawn("select")
        const card = findChild(area, "objectCard")
        verify(!card.visible)

        fake.selectedKey = "camera:a"
        verify(!card.visible)

        fake.selectedKey = "object:o1"
        verify(card.visible)
        compare(findChild(area, "widthField").text, "1,6")
        compare(findChild(area, "depthField").text, "0,9")

        fake.selectedKey = ""
        verify(!card.visible)
    }

    // LH-32: a selected camera shows the camera card, and only the camera card.
    function test_the_camera_card_belongs_to_a_selected_camera() {
        const area = drawn("select")
        const card = findChild(area, "cameraCard")
        verify(!card.visible)

        fake.selectedKey = "camera:a"
        verify(card.visible)
        verify(!findChild(area, "objectCard").visible)

        fake.selectedKey = "object:o1"
        verify(!card.visible)
    }

    function test_enter_gives_the_size_to_the_controller() {
        const area = drawn("select")
        fake.selectedKey = "object:o1"
        const width = findChild(area, "widthField")

        width.forceActiveFocus()
        width.text = "2,5"
        keyClick(Qt.Key_Return)

        compare(fake.sizes, [["2,5", "0,9"]])
        verify(!width.invalid)
        verify(!findChild(area, "depthField").invalid)
    }

    function test_the_apply_button_gives_the_size_to_the_controller() {
        const area = drawn("select")
        fake.selectedKey = "object:o1"

        findChild(area, "depthField").text = "0.5"
        findChild(area, "objectApply").clicked()

        compare(fake.sizes, [["1,6", "0.5"]])
    }

    function test_a_refused_size_marks_both_fields() {
        const area = drawn("select")
        fake.selectedKey = "object:o1"
        fake.sizeOk = false
        const width = findChild(area, "widthField")
        const depth = findChild(area, "depthField")

        width.text = "0"
        findChild(area, "objectApply").clicked()

        verify(width.invalid)
        verify(depth.invalid)
        width.text = "1"
        verify(!width.invalid)
    }

    // Letters and Delete typed into a size or a name are text: no turn, no delete.
    function test_typing_a_size_leaves_the_selection_alone() {
        const area = drawn("select")
        fake.selectedKey = "object:o1"

        for (const name of ["widthField", "depthField", "libraryName"]) {
            findChild(area, name).forceActiveFocus()
            keyClick(Qt.Key_R)
            keyClick(Qt.Key_Delete)
        }

        compare(fake.turns.length, 0)
        compare(fake.removes, 0)
    }

    // LH-12: the name of the entry starts with the object's own label, else its kind's name;
    // a kind without a name (an old project) starts empty.
    function test_the_library_name_starts_with_the_objects_name() {
        const area = drawn("select")
        const name = findChild(area, "libraryName")

        fake.selectedKey = "object:o1"
        compare(name.text, "Tisch")
        fake.selectedKey = "object:o2"
        compare(name.text, "Thron")
        fake.selectedKey = "object:o3"
        compare(name.text, "")
    }

    function test_filing_gives_the_name_to_the_controller() {
        const area = drawn("select")
        fake.selectedKey = "object:o1"
        const name = findChild(area, "libraryName")

        name.text = "Lange Tafel"
        findChild(area, "libraryFile").clicked()

        compare(fake.filed, ["Lange Tafel"])
        verify(!name.invalid)

        name.forceActiveFocus()
        keyClick(Qt.Key_Return)
        compare(fake.filed, ["Lange Tafel", "Lange Tafel"])
    }

    function test_a_refused_name_marks_the_field() {
        const area = drawn("select")
        fake.selectedKey = "object:o1"
        fake.fileOk = false
        const name = findChild(area, "libraryName")

        name.text = " "
        findChild(area, "libraryFile").clicked()

        verify(name.invalid)
        name.text = "x"
        verify(!name.invalid)
    }
}
