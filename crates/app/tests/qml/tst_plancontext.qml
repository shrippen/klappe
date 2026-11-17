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
        property string layersJson: "{}"
        property string fadesJson: "{}"
        property string choicesJson: JSON.stringify({ object: [{ kind: "table", custom: false }, { kind: "mine", custom: true, label: "Mein Ding" }], fade: ["100", "60", "30"], layer: ["set", "figures", "cameras", "lights", "notes", "drawings"] })
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
        property string selectedShot: "s1"
        // What the controller adds to the menu: the commands of objects and shots (LH-114 to LH-118, LH-74 to LH-110).
        property var extra: ({})
        // What a right click selects: the key and count the fake then shows.
        property string pickKey: ""
        property string armed: ""
        property bool viewFree: false

        function describe(key) { return "{}" }
        function markInfo() { return "{}" }
        // What the controller says of the selection: the table stands for its rules.
        readonly property var infos: ({
            "object": { layer: "set", remove: true, duplicate: true, turn: true },
            "camera": { layer: "cameras", remove: true, duplicate: true, turn: true, straighten: true, shot: true },
            "figure": { layer: "figures", remove: true, turn: true, straighten: true, nextMark: true, markBefore: true, axisTo: true, way: "smooth" },
            "note": { layer: "notes", remove: true, duplicate: true, note: true },
            "opening": { layer: "set", remove: true },
            "many": { layer: "", remove: true, duplicate: true, turn: true }
        })
        function menuInfo() {
            if (selectedCount === 0) {
                return "{}"
            }
            const kind = selectedCount > 1 ? "many" : selectedKey.split(":")[0]
            return JSON.stringify(Object.assign({}, infos[kind] || { layer: "set", remove: true }, extra))
        }
        function changeSelected(what, arg) { calls.push(["change", what, arg]); return true }
        function labelSelected() { calls.push(["label"]); return true }
        function versionShot(id) { calls.push(["version", id]); return true }
        function editShot(id, field, value) { calls.push(["edit", id, field, value]); return true }
        function setShotImage(id, path) { calls.push(["image", id, path]); return true }
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
        function arm(what) { calls.push(["arm", what]); armed = what; return true }
        function cancelArm() { calls.push(["cancelArm"]); armed = "" }
        function cycleFade(layer) { calls.push(["fade", layer]) }
        function clearSketches() { calls.push(["clearSketches"]) }
        function setWayCurve(name) { calls.push(["curve", name]); return true }
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
        fake.fadesJson = "{}"
        fake.locksJson = "{}"
        fake.canUndo = true
        fake.canRedo = false
        fake.scalePoints = 0
        fake.measuring = false
        fake.calls = []
        fake.pickKey = ""
        fake.armed = ""
        fake.viewFree = false
        fake.extra = ({})
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
                       "ctxNextMark", "ctxMarkBefore", "ctxAxisTo", "ctxWaySmooth", "ctxWayStraight", "ctxLockLayer", "ctxFadeLayer", "ctxHideLayer", "ctxDeselect", "ctxUndo", "ctxRedo", "ctxClearSketches", "ctxFit"]
        return names.filter(n => findChild(area, n).visible)
    }

    // Closes an open menu first: the menu asks what the selection takes as it opens.
    function open(area, key, count) {
        menuOf(area).close()
        tryVerify(() => !menuOf(area).opened)
        fake.selectedKey = key
        fake.selectedCount = count !== undefined ? count : (key === "" ? 0 : 1)
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
            "object:o1": ["ctxDelete", "ctxDuplicate", "ctxTurnRight", "ctxTurnLeft", "ctxLockLayer", "ctxFadeLayer", "ctxHideLayer", "ctxDeselect"],
            "camera:a": ["ctxDelete", "ctxDuplicate", "ctxTurnRight", "ctxTurnLeft", "ctxStraighten", "ctxAddShot", "ctxLockLayer", "ctxFadeLayer", "ctxHideLayer", "ctxDeselect"],
            "figure:e": ["ctxDelete", "ctxTurnRight", "ctxTurnLeft", "ctxStraighten", "ctxNextMark", "ctxMarkBefore", "ctxAxisTo", "ctxWayStraight", "ctxLockLayer", "ctxFadeLayer", "ctxHideLayer", "ctxDeselect"],
            "note:n1": ["ctxDelete", "ctxDuplicate", "ctxEditNote", "ctxLockLayer", "ctxFadeLayer", "ctxHideLayer", "ctxDeselect"],
            "opening:3:0": ["ctxDelete", "ctxLockLayer", "ctxFadeLayer", "ctxHideLayer", "ctxDeselect"]
        }

        for (const key in kinds) {
            open(area, key)
            compare(shown(area), kinds[key], key)
        }
    }

    function test_several_elements_get_the_group_commands_and_no_layer() {
        const area = made()
        open(area, "", 3)

        compare(shown(area), ["ctxDelete", "ctxDuplicate", "ctxTurnRight", "ctxTurnLeft", "ctxDeselect"])
    }

    function test_empty_ground_offers_undo_redo_and_fit() {
        const area = made()
        open(area, "")

        compare(shown(area), ["ctxUndo", "ctxRedo", "ctxClearSketches", "ctxFit"])
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
        findChild(area, "ctxFadeLayer").triggered()
        findChild(area, "ctxHideLayer").triggered()

        compare(fake.calls, [["duplicate"], ["turn", 15], ["turn", -15], ["straighten"], ["addShot"], ["remove"],
                             ["lock", "cameras", "locked"], ["fade", "cameras"], ["layer", "cameras", "hidden"]])
    }

    // The entry says the level of the layer and the one a click brings.
    function test_the_fade_entry_names_the_level_and_the_next() {
        const area = made()
        fake.fadesJson = JSON.stringify({ cameras: "60", set: "30" })
        open(area, "camera:a")
        compare(findChild(area, "ctxFadeLayer").text, "Ebene Kameras abblenden: 60 % → 30 %")

        open(area, "object:o1")
        compare(findChild(area, "ctxFadeLayer").text, "Ebene Set abblenden: 30 % → 100 %")

        fake.fadesJson = "{}"
        open(area, "object:o1")
        compare(findChild(area, "ctxFadeLayer").text, "Ebene Set abblenden: 100 % → 60 %")
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

    // One Escape order: an armed click, then the tool, then the selection.
    function test_escape_ends_an_armed_click_before_the_tool_and_the_selection() {
        const area = made("wall")
        const spy = createTemporaryObject(spyComponent, tc, { target: area, signalName: "selectToolRequested" })
        fake.selectedKey = "figure:e"
        fake.selectedCount = 1
        fake.armed = "nextMark"

        keyClick(Qt.Key_Escape)
        compare(fake.calls, [["cancelArm"]])
        compare(spy.count, 0)

        keyClick(Qt.Key_Escape)
        compare(spy.count, 1)
        compare(fake.calls, [["cancelArm"]])

        area.tool = "select"
        keyClick(Qt.Key_Escape)
        compare(fake.calls, [["cancelArm"], ["clear"]])
    }

    function test_the_hint_line_shows_what_is_armed() {
        const area = made("select")
        const line = findChild(area, "hintLine")

        verify(!line.visible)
        fake.armed = "axisTo"
        verify(line.visible)
        compare(findChild(area, "hintLabel").text, "Klick auf die zweite Figur setzt die Achse. Esc bricht ab.")

        // The armed click wins over the tool's hint.
        area.tool = "wall"
        area.toolHint = "Ziehen zeichnet eine Wand."
        compare(findChild(area, "hintLabel").text, "Klick auf die zweite Figur setzt die Achse. Esc bricht ab.")
        fake.armed = ""
        compare(findChild(area, "hintLabel").text, "Ziehen zeichnet eine Wand.")
    }

    // The arm keys work in 3D too: the hint says what the click does there as well.
    function apart(area, a, b) {
        const p = a.mapToItem(area, 0, 0)
        const q = b.mapToItem(area, 0, 0)
        return p.y + a.height <= q.y || q.y + b.height <= p.y || p.x + a.width <= q.x || q.x + b.width <= p.x
    }

    // The bottom of the plan as a whole, in 3D through a camera: the hint line, the label that
    // says how to leave the camera, and the card of the tool do not lie over one another.
    function test_the_hint_line_the_through_label_and_the_cards_keep_apart() {
        const area = made("note")
        area.toolHint = "Klick setzt eine Notiz. Das Werkzeug bleibt für weitere aktiv, Esc wählt die Auswahl."
        fake.viewFree = true
        fake.asksNote = true
        const through = findChild(area, "freeThrough")
        findChild(area, "freeView").look = "camera"
        waitForRendering(area)
        const line = findChild(area, "hintLine")
        const card = findChild(area, "noteCard")
        verify(line.visible && through.visible && card.visible)

        verify(apart(area, line, through), "hint over the label")
        verify(apart(area, line, card), "hint over the card")
        verify(apart(area, through, card), "label under the card")
    }

    // The tool hints that do nothing in 3D stay away there; an armed click works in both.
    function test_the_sketch_hint_is_not_shown_in_3d() {
        const area = made("sketch")
        area.toolHint = "Ziehen zeichnet mit dem Skizzenstift."
        const line = findChild(area, "hintLine")
        verify(line.visible)

        fake.viewFree = true
        verify(!line.visible)
        area.tool = "wall"
        area.toolHint = "Ziehen zeichnet eine Wand."
        verify(line.visible)
        area.tool = "sketch"
        fake.armed = "nextMark"
        verify(line.visible)
    }

    function test_the_hint_line_shows_in_the_free_view() {
        const area = made("select")
        fake.viewFree = true
        fake.armed = "nextMark"

        verify(findChild(area, "hintLine").visible)
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

    // The object commands come as the controller offers them; each entry is there or absent.
    function test_the_object_entries_follow_the_controller() {
        const area = made()
        const entries = { mirror: "ctxMirror", resetSize: "ctxResetSize", label: "ctxLabel", unbind: "ctxUnbind" }

        for (const flag in entries) {
            fake.extra = ({})
            open(area, "object:o1")
            verify(!findChild(area, entries[flag]).visible, flag + " absent")

            fake.extra = ({ [flag]: true })
            open(area, "object:o1")
            verify(findChild(area, entries[flag]).visible, flag + " offered")
        }
    }

    function test_the_object_entries_run_the_commands() {
        const area = made()
        fake.extra = ({ mirror: true, resetSize: true, label: true, unbind: true })
        open(area, "object:o1")

        findChild(area, "ctxMirror").triggered()
        findChild(area, "ctxResetSize").triggered()
        findChild(area, "ctxLabel").triggered()
        findChild(area, "ctxUnbind").triggered()

        compare(fake.calls, [["change", "mirror", ""], ["change", "resetSize", ""], ["label"], ["change", "unbind", ""]])
        compare(findChild(area, "ctxMirror").shownShortcut, "Umschalt+M")
        compare(findChild(area, "ctxResetSize").shownShortcut, "Umschalt+G")
        compare(findChild(area, "ctxLabel").shownShortcut, "Umschalt+L")
        compare(findChild(area, "ctxUnbind").shownShortcut, "Umschalt+U")
    }

    function test_the_kind_submenu_lists_the_library() {
        const area = made()
        open(area, "object:o1")
        compare(findChild(area, "ctxKind"), null)

        fake.extra = ({ kind: true })
        open(area, "object:o1")
        const sub = findChild(area, "ctxKind")
        verify(sub !== null)
        compare(sub.title, "Art ändern")
        compare(findChild(area, "ctxKind.table").text, "Tisch")
        compare(findChild(area, "ctxKind.mine").text, "Mein Ding")

        findChild(area, "ctxKind.mine").triggered()
        compare(fake.calls, [["change", "kind", "mine"]])

        fake.extra = ({})
        open(area, "object:o1")
        compare(findChild(area, "ctxKind"), null)
    }

    function test_the_shot_entries_follow_the_controller() {
        const area = made()
        const entries = { versionShot: "ctxVersionShot", shortName: "ctxShortName", shotImage: "ctxShotImage", shotImageRemove: "ctxShotImageRemove" }

        for (const flag in entries) {
            fake.extra = ({})
            open(area, "camera:a")
            verify(!findChild(area, entries[flag]).visible, flag + " absent")

            fake.extra = ({ [flag]: true })
            open(area, "camera:a")
            verify(findChild(area, entries[flag]).visible, flag + " offered")
        }
    }

    // By keyboard the menu is still open when the entry runs: the picture dialog opens after it closed.
    function test_the_picture_dialog_opens_after_the_menu_closed() {
        const area = made()
        fake.extra = ({ shotImage: true })
        open(area, "camera:a")

        findChild(area, "ctxShotImage").triggered()
        const dialog = findChild(area, "ctxImageDialog")
        verify(dialog !== null)
        verify(!dialog.visible, "not opened while the menu is open")

        menuOf(area).close()
        tryVerify(() => dialog.visible)
        dialog.close()
    }

    function test_the_shot_entries_run_the_commands_of_the_shot() {
        const area = made()
        fake.extra = ({ versionShot: true, shortName: true, shortNameNow: "Totale", shotImage: true, shotImageRemove: true })
        open(area, "camera:a")

        findChild(area, "ctxVersionShot").triggered()
        findChild(area, "ctxShotImageRemove").triggered()
        compare(fake.calls, [["version", "s1"], ["image", "s1", ""]])

        findChild(area, "ctxShortName").triggered()
        const dialog = findChild(area, "ctxNameDialog")
        tryVerify(() => dialog.opened)
        const field = findChild(dialog, "ctxShortNameField")
        compare(field.text, "Totale")
        field.text = "Halbtotale"
        dialog.accept()
        compare(fake.calls[2], ["edit", "s1", "short_name", "Halbtotale"])
    }
}
