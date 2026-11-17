import QtQuick
import QtTest
import org.kde.kirigami as Kirigami
import "../../qml"

// PlanArea, the scale, the measuring and the note: two clicks ask for a length, the pointer
// goes to the controller while measuring, and the note card asks for its text.
TestCase {
    id: tc
    name: "PlanAreaCards"
    width: 900
    height: 400
    when: windowShown
    visible: true

    QtObject {
        id: fake
        property bool loaded: true
        property string planJson: ""
        property string selectedKey: ""
        property string choicesJson: ""
        property int selectedCount: 0
        property bool extendSelection: false
        property bool lassoSelection: false
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
        property var images: []
        property int noteCancels: 0
        property int noteEdits: 0
        property bool noteOk: true
        property bool measuring: false
        property var hovers: []
        property int measureCancels: 0
        property int areaPoints: 0
        property int areaFinishes: 0
        property int areaCancels: 0
        property var sizes: []
        property string markJson: "{}"
        property bool sizeOk: true
        property var heights: []
        property bool heightOk: true
        property var filed: []
        property bool fileOk: true

        function markInfo() { return markJson }
        function describe(key) {
            if (key === "object:o1") {
                return JSON.stringify({ kind: "object", label: "", library: "table", width: 1.6, depth: 0.9, height: null, base: 0, baseDefault: true })
            }
            if (key === "object:o2") {
                return JSON.stringify({ kind: "object", label: "Thron", library: "thron", width: 0.9, depth: 0.8, height: 0.8, base: 0, baseDefault: true })
            }
            if (key === "object:o3") {
                return JSON.stringify({ kind: "object", label: "", library: "zeppelin", width: 12, depth: 3, height: null, base: 0, baseDefault: true })
            }
            if (key === "note:n1") {
                return JSON.stringify({ kind: "note", text: "Vorhang schließen" })
            }
            if (key === "image:i1") {
                return JSON.stringify({ kind: "image", file: "bilder/skizze.png", opacity: 1 })
            }
            if (key === "drawing:d1") {
                return JSON.stringify({ kind: "drawing", color: "green", dash: "dashed", fill: "outline", closed: true })
            }
            if (key.indexOf("figurevia:") === 0) {
                return JSON.stringify({ kind: "bend", index: 2 })
            }
            if (key === "camera:b") {
                return JSON.stringify({ kind: "camera", label: "B", lens: 50, fov: 27.9, height: 1.6, shots: "", follows: "Elisabeth", rig: "crane" })
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
        function addImage(file, x, y) {
            images.push([file, x, y])
            return true
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
        function hover(x, y) {
            hovers.push([x, y])
            return true
        }
        function cancelMeasure() {
            measureCancels++
            measuring = false
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
        function setObjectHeight(text) {
            heights.push(text)
            return heightOk
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

    // One box per selectable element, along the bottom of the plan, away from the drags.
    function box(key, x) {
        const y = 11
        return { kind: "polygon", points: [x, y, x + 1, y, x + 1, y + 1, x, y + 1], radius: 0, width: 0,
                 label: "muted", stroke: "solid", fill: "tint", text: "", key: key, selected: false }
    }

    function planWith(x) {
        const keys = ["object:o1", "object:o2", "object:o3", "camera:a", "camera:b", "note:n1"]
        return JSON.stringify({ width: 20, depth: 13, shapes: keys.map((k, i) => box(k, x + i * 2)) })
    }

    function init() {
        fake.planJson = planWith(1)
        fake.selectedKey = ""
        fake.selectedCount = 0
        fake.extendSelection = false
        fake.duplicates = 0
        fake.drags = []
        fake.places = []
        fake.images = []
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
        fake.measuring = false
        fake.hovers = []
        fake.measureCancels = 0
        fake.areaPoints = 0
        fake.areaFinishes = 0
        fake.areaCancels = 0
        fake.sizes = []
        fake.sizeOk = true
        fake.markJson = "{}"
        fake.heights = []
        fake.heightOk = true
        fake.filed = []
        fake.fileOk = true
    }

    function drawn(tool, width) {
        const area = createTemporaryObject(areaComponent, tc, { texts: de, tool: tool, width: width || 600 })
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

    function test_the_pointer_goes_to_the_controller_while_measuring() {
        const area = drawn("measure")
        mouseMove(area, 100, 100)
        compare(fake.hovers.length, 0)

        fake.measuring = true
        mouseMove(area, 150, 120)
        mouseMove(area, 200, 140)

        verify(fake.hovers.length > 0, fake.hovers.length)
    }

    function test_the_pointer_is_not_sent_for_other_tools() {
        const area = drawn("wall")
        fake.measuring = true

        mouseMove(area, 150, 120)
        mouseMove(area, 200, 140)

        compare(fake.hovers.length, 0)
    }

    function test_escape_drops_the_measurement() {
        const area = drawn("measure")
        fake.measuring = true

        keyClick(Qt.Key_Escape)

        compare(fake.measureCancels, 1)
    }

    function test_escape_leaves_an_idle_measure_tool_alone() {
        const area = drawn("measure")

        keyClick(Qt.Key_Escape)

        compare(fake.measureCancels, 0)
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

    function test_an_image_tooltip_names_its_file() {
        const area = createTemporaryObject(areaComponent, tc, { texts: de })

        compare(area.tipLines("image:i1"), ["Bild: bilder/skizze.png"])
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
}
