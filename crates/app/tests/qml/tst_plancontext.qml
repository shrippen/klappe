import QtQuick
import QtTest
import "../../qml"

// PlanArea, LH-100 and LH-101: the context menu (right click, Menu key, Shift+F10) offers what
// the selection takes, with the key beside it; Escape ends the operation, then the tool, then the
// selection; the hint line shows with a tool; the grips have tooltips (LH-102).
TestCase {
    id: tc
    name: "PlanContext"
    width: 900
    height: 400
    when: windowShown
    visible: true

    QtObject {
        id: fake
        property bool loaded: true
        property string planJson: ""
        property string selectedKey: ""
        property int selectedCount: 0
        property string choicesJson: ""
        property string layersJson: "{}"
        property string locksJson: "{}"
        property bool canUndo: true
        property bool canRedo: false
        property bool extendSelection: false
        property bool lassoSelection: false
        property int scalePoints: 0
        property bool measuring: false
        property bool asksNote: false
        property string noteText: ""
        property int areaPoints: 0
        property var calls: []
        // What a right click selects: the key and count the fake then shows.
        property string pickKey: ""

        function describe(key) { return "{}" }
        function markInfo() { return "{}" }
        // What the controller says of the selection: the table stands for its rules.
        readonly property var infos: ({
            "object": { layer: "set", remove: true, duplicate: true, turn: true },
            "camera": { layer: "cameras", remove: true, duplicate: true, turn: true, straighten: true, shot: true },
            "figure": { layer: "figures", remove: true, turn: true, straighten: true },
            "note": { layer: "notes", remove: true, duplicate: true, note: true },
            "opening": { layer: "set", remove: true },
            "many": { layer: "", remove: true, duplicate: true, turn: true }
        })
        function menuInfo() {
            if (selectedCount === 0) {
                return "{}"
            }
            const kind = selectedCount > 1 ? "many" : selectedKey.split(":")[0]
            return JSON.stringify(infos[kind] || { layer: "set", remove: true })
        }
        function selectAt(x, y) {
            calls.push(["selectAt"])
            selectedKey = pickKey
            selectedCount = pickKey === "" ? 0 : 1
            return pickKey !== ""
        }
        function removeSelected() { calls.push(["remove"]) }
        function duplicateSelected() { calls.push(["duplicate"]) }
        function turnSelected(step) { calls.push(["turn", step]) }
        function straightenSelected() { calls.push(["straighten"]) }
        function editSelectedNote() { calls.push(["editNote"]) }
        function addShot() { calls.push(["addShot"]) }
        function clearSelection() { calls.push(["clear"]) }
        function setLock(layer, state) { calls.push(["lock", layer, state]) }
        function setLayer(layer, state) { calls.push(["layer", layer, state]) }
        function undo() { calls.push(["undo"]) }
        function redo() { calls.push(["redo"]) }
        function cancelScale() { calls.push(["cancelScale"]); scalePoints = 0 }
        function cancelMeasure() { calls.push(["cancelMeasure"]); measuring = false }
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

    Component {
        id: spyComponent
        SignalSpy {}
    }

    function init() {
        fake.planJson = JSON.stringify({ width: 20, depth: 13, shapes: [] })
        fake.selectedKey = ""
        fake.selectedCount = 0
        fake.layersJson = "{}"
        fake.locksJson = "{}"
        fake.canUndo = true
        fake.canRedo = false
        fake.scalePoints = 0
        fake.measuring = false
        fake.calls = []
        fake.pickKey = ""
    }

    function made(tool) {
        const area = createTemporaryObject(areaComponent, tc, { tool: tool || "select" })
        waitForRendering(area)
        return area
    }

    function menuOf(area) {
        return findChild(area, "contextMenu")
    }

    function shown(area) {
        const names = ["ctxDelete", "ctxDuplicate", "ctxTurnRight", "ctxTurnLeft", "ctxStraighten", "ctxEditNote", "ctxAddShot",
                       "ctxLockLayer", "ctxHideLayer", "ctxDeselect", "ctxUndo", "ctxRedo", "ctxFit"]
        return names.filter(n => findChild(area, n).visible)
    }

    function open(area, key) {
        fake.selectedKey = key
        fake.selectedCount = key === "" ? 0 : 1
        area.openMenu(Qt.point(100, 100))
        tryVerify(() => menuOf(area).opened)
    }

    function test_a_right_click_selects_under_the_pointer_and_opens_the_menu() {
        const area = made()
        fake.pickKey = "object:o1"

        mouseClick(area, 100, 100, Qt.RightButton)

        compare(fake.calls[0], ["selectAt"])
        tryVerify(() => menuOf(area).opened)
        verify(findChild(area, "ctxDelete").visible)
    }

    function test_a_right_click_works_with_a_placing_tool_too() {
        const area = made("wall")

        mouseClick(area, 100, 100, Qt.RightButton)

        tryVerify(() => menuOf(area).opened)
        compare(fake.calls.filter(c => c[0] === "selectAt").length, 1)
    }

    function test_the_menu_key_and_shift_f10_open_it() {
        const area = made()
        const canvas = findChild(area, "planCanvas")

        canvas.forceActiveFocus()
        keyClick(Qt.Key_Menu)
        tryVerify(() => menuOf(area).opened)
        menuOf(area).close()
        tryVerify(() => !menuOf(area).opened)

        canvas.forceActiveFocus()
        keyClick(Qt.Key_F10, Qt.ShiftModifier)
        tryVerify(() => menuOf(area).opened)
        // The keys take the selection as it is.
        compare(fake.calls.filter(c => c[0] === "selectAt").length, 0)
    }

    function test_each_kind_gets_the_commands_it_takes() {
        const area = made()
        const kinds = {
            "object:o1": ["ctxDelete", "ctxDuplicate", "ctxTurnRight", "ctxTurnLeft", "ctxLockLayer", "ctxHideLayer", "ctxDeselect"],
            "camera:a": ["ctxDelete", "ctxDuplicate", "ctxTurnRight", "ctxTurnLeft", "ctxStraighten", "ctxAddShot", "ctxLockLayer", "ctxHideLayer", "ctxDeselect"],
            "figure:e": ["ctxDelete", "ctxTurnRight", "ctxTurnLeft", "ctxStraighten", "ctxLockLayer", "ctxHideLayer", "ctxDeselect"],
            "note:n1": ["ctxDelete", "ctxDuplicate", "ctxEditNote", "ctxLockLayer", "ctxHideLayer", "ctxDeselect"],
            "opening:3:0": ["ctxDelete", "ctxLockLayer", "ctxHideLayer", "ctxDeselect"]
        }

        open(area, "object:o1")
        for (const key in kinds) {
            fake.selectedKey = key
            fake.selectedCount = 1
            compare(shown(area), kinds[key], key)
        }
    }

    function test_several_elements_get_the_group_commands_and_no_layer() {
        const area = made()
        open(area, "object:o1")
        fake.selectedKey = ""
        fake.selectedCount = 3

        compare(shown(area), ["ctxDelete", "ctxDuplicate", "ctxTurnRight", "ctxTurnLeft", "ctxDeselect"])
    }

    function test_empty_ground_offers_undo_redo_and_fit() {
        const area = made()
        open(area, "")

        compare(shown(area), ["ctxUndo", "ctxRedo", "ctxFit"])
        verify(findChild(area, "ctxUndo").enabled)
        verify(!findChild(area, "ctxRedo").enabled)
    }

    function test_the_entries_show_their_keys() {
        const area = made()
        open(area, "object:o1")

        compare(findChild(area, "ctxDelete").shownShortcut, "Entf")
        compare(findChild(area, "ctxDuplicate").shownShortcut, "Strg+D")
        compare(findChild(area, "ctxTurnRight").shownShortcut, "R")
        compare(findChild(area, "ctxTurnLeft").shownShortcut, "Umschalt+R")
        compare(findChild(area, "ctxDeselect").shownShortcut, "Esc")
    }

    function test_the_entries_run_the_commands_of_the_keys() {
        const area = made()
        open(area, "camera:a")

        findChild(area, "ctxDuplicate").triggered()
        findChild(area, "ctxTurnRight").triggered()
        findChild(area, "ctxTurnLeft").triggered()
        findChild(area, "ctxStraighten").triggered()
        findChild(area, "ctxAddShot").triggered()
        findChild(area, "ctxDelete").triggered()
        findChild(area, "ctxLockLayer").triggered()
        findChild(area, "ctxHideLayer").triggered()

        compare(fake.calls, [["duplicate"], ["turn", 15], ["turn", -15], ["straighten"], ["addShot"], ["remove"],
                             ["lock", "cameras", "locked"], ["layer", "cameras", "hidden"]])
    }

    function test_a_note_is_edited_from_its_menu() {
        const area = made()
        open(area, "note:n1")

        findChild(area, "ctxEditNote").triggered()

        compare(fake.calls, [["editNote"]])
    }

    // A locked or hidden layer cannot be picked: empty ground brings it back.
    function test_empty_ground_brings_a_locked_or_hidden_layer_back() {
        fake.locksJson = JSON.stringify({ set: false, notes: true })
        fake.layersJson = JSON.stringify({ lights: false, notes: true })
        const area = made()
        open(area, "")

        const unlock = findChild(area, "ctxBack.notes.unlock")
        const show = findChild(area, "ctxBack.lights.show")
        verify(unlock)
        verify(show)
        compare(unlock.text, "Ebene Notizen entsperren")
        compare(show.text, "Ebene Licht einblenden")
        verify(!findChild(area, "ctxBack.set.unlock"))

        unlock.triggered()
        show.triggered()

        compare(fake.calls, [["lock", "notes", "unlocked"], ["layer", "lights", "shown"]])
    }

    // LH-101: Escape ends the operation, then goes to select, then clears the selection.
    function test_escape_goes_to_select_when_nothing_runs() {
        const area = made("wall")
        const spy = createTemporaryObject(spyComponent, tc, { target: area, signalName: "selectToolRequested" })

        keyClick(Qt.Key_Escape)

        compare(spy.count, 1)
    }

    function test_escape_first_ends_the_operation() {
        const area = made("scale")
        const spy = createTemporaryObject(spyComponent, tc, { target: area, signalName: "selectToolRequested" })
        fake.scalePoints = 1

        keyClick(Qt.Key_Escape)
        compare(fake.calls, [["cancelScale"]])
        compare(spy.count, 0)

        keyClick(Qt.Key_Escape)
        compare(spy.count, 1)
    }

    function test_escape_clears_the_selection_after_the_tool_is_select() {
        const area = made("wall")
        const spy = createTemporaryObject(spyComponent, tc, { target: area, signalName: "selectToolRequested" })
        fake.selectedKey = "object:o1"
        fake.selectedCount = 1

        keyClick(Qt.Key_Escape)
        compare(spy.count, 1)
        compare(fake.calls, [])

        area.tool = "select"
        keyClick(Qt.Key_Escape)
        compare(spy.count, 1)
        compare(fake.calls, [["clear"]])
    }

    function test_the_hint_line_shows_with_a_tool() {
        const area = made("wall")
        const line = findChild(area, "hintLine")

        verify(!line.visible)
        area.toolHint = "Ziehen zeichnet eine Wand."
        verify(line.visible)
        compare(findChild(area, "hintLabel").text, "Ziehen zeichnet eine Wand.")

        area.tool = "select"
        verify(!line.visible)
    }

    // The hint line stands clear of the cards at the bottom, which belong to the same tools.
    function test_the_hint_line_does_not_cover_the_cards_of_the_tool() {
        const area = made("note")
        area.toolHint = "Klick setzt eine Notiz. Das Werkzeug bleibt für weitere aktiv, Esc wählt die Auswahl."
        fake.asksNote = true
        waitForRendering(area)
        const line = findChild(area, "hintLine")
        const card = findChild(area, "noteCard")
        verify(line.visible)
        verify(card.visible)

        const hint = line.mapToItem(area, 0, 0)
        const note = card.mapToItem(area, 0, 0)
        const apart = hint.y + line.height <= note.y || note.y + card.height <= hint.y
            || hint.x + line.width <= note.x || note.x + card.width <= hint.x
        verify(apart, "hint " + JSON.stringify(hint) + " " + line.width + "x" + line.height
               + ", card " + JSON.stringify(note) + " " + card.width + "x" + card.height)

        fake.asksNote = false
        const bottom = line.mapToItem(area, 0, 0).y + line.height
        verify(bottom > note.y, "the hint comes down again without a card")
    }

    // LH-102: the grips say what dragging them does.
    function test_grips_have_tooltips() {
        const area = made()

        compare(area.tipOf("objectgrip:o1:width"), ["Breite: ziehen"])
        compare(area.tipOf("objectgrip:o1:depth"), ["Tiefe: ziehen"])
        compare(area.tipOf("objectgrip:o1:point2"), ["Punkt: ziehen, Entf nimmt ihn heraus"])
        compare(area.tipOf("objectgrip:o1:side1"), ["Seite: ziehen"])
        compare(area.tipOf("lightgrip:l1:direction"), ["Richtung der Lampe: ziehen"])
        compare(area.tipOf("lightgrip:l1:beam"), ["Öffnungswinkel: ziehen"])
        compare(area.tipOf("openingedge:4:1:end"), ["Öffnung verbreitern oder verschmälern: ziehen"])
        compare(area.tipOf("notewidth:n1"), ["Breite der Notiz: ziehen"])
        compare(area.tipOf("imagesize:i1"), ["Größe des Bildes: ziehen"])
        compare(area.tipOf("drawingpoint:d1:2"), ["Punkt der Zeichnung: ziehen"])
        compare(area.tipOf("drawingedge:d1:1"), ["Kante der Zeichnung: ziehen"])
    }
}
