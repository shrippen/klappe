import QtQuick
import QtTest
import "../../qml"

// PlanTools: keys pick the tool, the modes (D-01) decide which tools the bar shows, the list shows
// only for the tool that needs it, and the controller is told tool and choice. The working mode
// is kept in the settings (scripts/check-qml.sh points XDG_CONFIG_HOME to a folder of its own).
TestCase {
    id: tc
    name: "PlanTools"
    width: 900
    height: 200
    when: windowShown
    visible: true

    QtObject {
        id: fake
        property bool loaded: true
        property bool snappingOn: true
        property string gridName: "cm25"
        property var grids: []
        readonly property var choices: ({
            figure: [{ id: "elisabeth", name: "Elisabeth" }, { id: "burgvogt", name: "Burgvogt" }],
            object: [
                { kind: "table", label: "", group: "furniture", width_m: 1.6, depth_m: 0.9, shape: "rect", custom: false, uses: 1 },
                { kind: "chair", label: "", group: "furniture", width_m: 0.45, depth_m: 0.45, shape: "rect", custom: false, uses: 0 },
                { kind: "dolly", label: "", group: "camera", width_m: 1.2, depth_m: 0.8, shape: "rect", custom: false, uses: 0 },
                { kind: "thron", label: "Thron", group: "", width_m: 0.9, depth_m: 0.8, shape: "rect", custom: true, uses: 3 },
                { kind: "ruine", label: "Ruine", group: "Requisiten", width_m: 5, depth_m: 4, shape: "rect", custom: true, uses: 0 },
                { kind: "kelch", label: "Kelch", group: "Möbel", width_m: 0.1, depth_m: 0.1, shape: "rect", custom: true, uses: 0 }
            ],
            cameras: [{ id: "a", label: "A" }, { id: "b", label: "B" }],
            light: ["fresnel-650", "led-panel"],
            color: ["red", "orange", "yellow", "green", "cyan", "blue", "purple"],
            dash: ["solid", "dashed", "dotted"],
            fill: ["outline", "filled"],
            grid: [{ id: "cm5", step_m: 0.05 }, { id: "cm25", step_m: 0.25 }, { id: "m1", step_m: 1 }]
        })
        property string choicesJson: JSON.stringify(choices)
        property var tools: []
        property var snaps: []
        property var edits: []
        property var removed: []
        property bool accept: true
        property var styles: []

        function setTool(tool, choice) {
            tools.push([tool, choice])
            return true
        }
        function setDrawStyle(color, dash, fill) {
            styles.push([color, dash, fill])
            return true
        }
        function editLibraryItem(kind, name, group) {
            edits.push([kind, name, group])
            return accept
        }
        function removeLibraryItem(kind) {
            removed.push(kind)
            return true
        }
        function setSnapping(on) {
            snaps.push(on)
        }
        function setGrid(name) {
            grids.push(name)
        }
    }

    TestTexts { id: de; language: "de" }

    Component {
        id: spyComponent
        SignalSpy {}
    }

    Component {
        id: toolsComponent
        PlanTools {
            controller: fake
            texts: de
        }
    }

    // The settings need the names the app sets; the runner has none.
    function initTestCase() {
        Qt.application.organization = "klappe-test"
        Qt.application.domain = "klappe.test"
    }

    function init() {
        fake.choicesJson = JSON.stringify(fake.choices)
        fake.tools = []
        fake.snaps = []
        fake.edits = []
        fake.removed = []
        fake.accept = true
        fake.grids = []
        fake.gridName = "cm25"
        fake.styles = []

        // The settings keep the mode over the test cases: start each from the default.
        const stale = createTemporaryObject(toolsComponent, tc)
        stale.mode = "build"
        wait(50)
        stale.destroy()
        wait(50)
    }

    function make() {
        const bar = createTemporaryObject(toolsComponent, tc)
        verify(bar)
        waitForRendering(bar)
        return bar
    }

    function shownNames(bar) {
        return bar.shown.map(e => e.tool.name)
    }

    function test_the_bar_starts_in_build_mode() {
        const bar = make()

        compare(bar.mode, "build")
        compare(shownNames(bar), ["select", "wall", "object", "scale", "measure"])
    }

    // D-01 (table "Werkzeuge je Modus"): each mode shows its tools; selecting and measuring are in all.
    function test_each_mode_shows_its_tools() {
        const bar = make()
        const wanted = {
            build: ["select", "wall", "object", "scale", "measure"],
            stage: ["select", "figure", "measure"],
            resolve: ["select", "camera", "light", "measure"],
            draw: ["select", "note", "line", "arrow", "rect", "ellipse", "area", "measure"]
        }

        for (const mode of Object.keys(wanted)) {
            bar.mode = mode
            compare(shownNames(bar).slice().sort(), wanted[mode].slice().sort(), mode)
            for (const name of bar.names) {
                const button = findChild(bar, "tool." + name)
                compare(!!button && button.visible, wanted[mode].indexOf(name) >= 0, mode + " " + name)
            }
        }
    }

    function test_every_tool_is_in_a_mode() {
        const bar = make()
        const all = []
        for (const mode of bar.modes) {
            bar.mode = mode
            shownNames(bar).forEach(n => all.push(n))
        }

        compare(bar.names.filter(n => all.indexOf(n) < 0), [])
    }

    function test_the_mode_buttons_switch_the_bar() {
        const bar = make()
        const box = findChild(bar, "modeBox")

        compare(box.model, ["Set bauen", "Inszenieren", "Auflösen", "Zeichnen"])
        compare(box.tooltips[2], "Auflösen (3)")
        box.select(3)

        compare(bar.mode, "draw")
        compare(box.currentIndex, 3)
        verify(findChild(bar, "tool.arrow").visible)
        verify(!findChild(bar, "tool.wall") || !findChild(bar, "tool.wall").visible)
    }

    function test_keys_1_to_4_pick_the_mode() {
        const bar = make()
        const keys = [[Qt.Key_2, "stage"], [Qt.Key_3, "resolve"], [Qt.Key_4, "draw"], [Qt.Key_1, "build"]]

        for (const [key, mode] of keys) {
            keyClick(key)
            compare(bar.mode, mode)
            compare(findChild(bar, "modeBox").currentIndex, bar.modes.indexOf(mode))
        }
    }

    function test_the_key_of_a_tool_of_another_mode_switches_the_mode() {
        const bar = make()

        keyClick(Qt.Key_C)
        compare(bar.mode, "resolve")
        compare(bar.tool, "camera")
        compare(fake.tools[fake.tools.length - 1], ["camera", ""])

        keyClick(Qt.Key_N)
        compare(bar.mode, "draw")
        compare(bar.tool, "note")

        keyClick(Qt.Key_W)
        compare(bar.mode, "build")
        compare(bar.tool, "wall")
    }

    function test_selecting_and_measuring_keep_the_mode() {
        const bar = make()

        for (const mode of bar.modes) {
            bar.mode = mode
            keyClick(Qt.Key_D)
            compare(bar.tool, "measure", mode)
            compare(bar.mode, mode)
            keyClick(Qt.Key_V)
            compare(bar.tool, "select", mode)
            compare(bar.mode, mode)
        }
    }

    function test_a_mode_without_the_tool_falls_back_to_select() {
        const bar = make()
        bar.tool = "wall"

        bar.mode = "stage"

        compare(bar.tool, "select")
        compare(fake.tools[fake.tools.length - 1], ["select", ""])
    }

    function test_a_mode_keeps_a_tool_it_shows() {
        const bar = make()
        keyClick(Qt.Key_D)

        bar.mode = "draw"

        compare(bar.tool, "measure")
    }

    function test_a_locked_bar_ignores_the_mode_keys() {
        const bar = make()
        bar.enabled = false

        keyClick(Qt.Key_3)
        compare(bar.mode, "build")
    }

    function test_the_mode_is_kept_over_a_restart() {
        const first = make()
        first.mode = "draw"
        // The settings store on the next turn of the event loop; a destroyed bar is gone after one.
        wait(50)
        first.destroy()
        wait(50)

        const second = make()

        compare(second.mode, "draw")
        compare(findChild(second, "modeBox").currentIndex, 3)
        compare(second.tool, "select")
    }

    function test_select_is_the_first_tool() {
        const bar = make()

        compare(bar.tool, "select")
        compare(bar.choice, "")
        compare(fake.tools[0], ["select", ""])
    }

    function test_keys_pick_the_tool() {
        const bar = make()
        const keys = [[Qt.Key_W, "wall"], [Qt.Key_O, "object"], [Qt.Key_F, "figure"],
                      [Qt.Key_C, "camera"], [Qt.Key_L, "light"], [Qt.Key_M, "scale"], [Qt.Key_D, "measure"],
                      [Qt.Key_N, "note"], [Qt.Key_I, "line"], [Qt.Key_A, "arrow"],
                      [Qt.Key_B, "rect"], [Qt.Key_E, "ellipse"], [Qt.Key_P, "area"],
                      [Qt.Key_V, "select"]]

        for (const [key, tool] of keys) {
            keyClick(key)
            compare(bar.tool, tool)
        }
    }

    function test_the_measure_button_picks_the_tool() {
        const bar = make()
        const button = findChild(bar, "tool.measure")

        verify(button)
        compare(button.text, "Messen")
        mouseClick(button)

        compare(bar.tool, "measure")
        verify(button.checked)
        compare(fake.tools[fake.tools.length - 1], ["measure", ""])
    }

    // Review of PR 20: while the backup question locks the bar, keys changed the tool.
    function test_a_locked_bar_ignores_keys() {
        const bar = make()
        bar.enabled = false

        keyClick(Qt.Key_W)
        compare(bar.tool, "select")
    }

    function test_the_list_shows_for_its_tool_only() {
        const bar = make()
        const shown = () => ["objectBox", "figureBox", "cameraBox", "lightBox"].filter(n => findChild(bar, n).visible)

        compare(shown(), [])
        bar.tool = "object"
        compare(shown(), ["objectBox"])
        bar.tool = "figure"
        compare(shown(), ["figureBox"])
        bar.tool = "camera"
        compare(shown(), ["cameraBox"])
        bar.tool = "light"
        compare(shown(), ["lightBox"])
        bar.tool = "wall"
        compare(shown(), [])
    }

    function test_the_choice_goes_to_the_controller() {
        const bar = make()

        bar.tool = "figure"
        compare(bar.choice, "elisabeth")
        compare(fake.tools[fake.tools.length - 1], ["figure", "elisabeth"])

        findChild(bar, "figureBox").currentIndex = 1
        compare(bar.choice, "burgvogt")
        compare(fake.tools[fake.tools.length - 1], ["figure", "burgvogt"])

        bar.tool = "light"
        compare(bar.choice, "fresnel-650")
    }

    /// No camera picked: a click sets a new camera; a picked one gets the next mark.
    function test_the_camera_tool_picks_a_camera_or_a_new_one() {
        const bar = make()
        const box = findChild(bar, "cameraBox")

        bar.tool = "camera"
        compare(box.model.map(c => c.text), ["Neue Kamera", "A", "B"])
        compare(bar.choice, "")
        compare(fake.tools[fake.tools.length - 1], ["camera", ""])

        box.currentIndex = 2
        compare(bar.choice, "b")
        compare(fake.tools[fake.tools.length - 1], ["camera", "b"])
    }

    function test_the_library_lists_built_in_then_own_objects_in_groups() {
        const bar = make()
        const box = findChild(bar, "objectBox")

        // Built-in groups by name, then the own groups; "Möbel" of an own entry is the heading
        // of the built-in group, so its entries sit together.
        compare(box.model.map(o => o.text), ["Tisch", "Stuhl", "Kelch", "Dolly", "Thron", "Ruine"])
        compare(box.model.map(o => o.group), ["Möbel", "Möbel", "Möbel", "Kamera-Technik", "Eigene", "Requisiten"])
        compare(box.sectionAt(4), "Eigene")
    }

    function test_the_picked_object_goes_to_the_controller() {
        const bar = make()
        const box = findChild(bar, "objectBox")

        bar.tool = "object"
        compare(bar.choice, "table")
        compare(fake.tools[fake.tools.length - 1], ["object", "table"])

        box.currentIndex = 4
        box.activated(4)
        compare(bar.choice, "thron")
        compare(fake.tools[fake.tools.length - 1], ["object", "thron"])
    }

    // An object filed or a file imported changes the list; the pick stays what it was.
    function test_a_new_list_keeps_the_picked_object() {
        const bar = make()
        const box = findChild(bar, "objectBox")
        bar.tool = "object"
        box.currentIndex = 1
        box.activated(1)

        const objects = JSON.parse(fake.choicesJson).object
        objects.unshift({ kind: "bed", label: "", group: "furniture", width_m: 1.4, depth_m: 2, shape: "rect", custom: false })
        fake.choicesJson = JSON.stringify({ figure: [], object: objects, light: [], color: [], dash: [], fill: [] })

        compare(box.currentIndex, 2)
        compare(bar.choice, "chair")
    }

    function test_typing_narrows_the_library() {
        const bar = make()
        const box = findChild(bar, "objectBox")
        bar.tool = "object"

        box.forceActiveFocus()
        keyClick(Qt.Key_T)
        keyClick(Qt.Key_H)

        // Letters typed into the search are text: no tool change.
        compare(bar.tool, "object")
        compare(box.matches.map(i => box.textAt(i)), ["Thron"])
    }

    // A list in focus takes letters and digits to pick its entry: no tool or mode change.
    function test_keys_in_a_list_do_not_pick_tool_or_mode() {
        const bar = make()
        const box = findChild(bar, "lightBox")
        bar.mode = "resolve"
        bar.tool = "light"
        const tools = fake.tools.length

        box.forceActiveFocus()
        keyClick(Qt.Key_V)
        keyClick(Qt.Key_2)

        compare(bar.tool, "light")
        compare(bar.mode, "resolve")
        compare(fake.tools.length, tools)
    }

    function test_the_library_files_are_asked_for_by_the_object_tool() {
        const bar = make()
        const importButton = findChild(bar, "libraryImport")
        const exportButton = findChild(bar, "libraryExport")
        const imports = createTemporaryObject(spyComponent, tc, { target: bar, signalName: "importRequested" })
        const exports = createTemporaryObject(spyComponent, tc, { target: bar, signalName: "exportRequested" })

        verify(!importButton.visible)
        verify(!exportButton.visible)
        bar.tool = "object"
        verify(importButton.visible)
        verify(exportButton.visible)
        verify(exportButton.enabled)

        importButton.clicked()
        exportButton.clicked()
        compare(imports.count, 1)
        compare(exports.count, 1)
    }

    // Nothing of the project's own to pass on: no export.
    function test_without_own_objects_there_is_nothing_to_export() {
        const bar = make()
        const objects = JSON.parse(fake.choicesJson).object.filter(o => !o.custom)
        fake.choicesJson = JSON.stringify({ figure: [], object: objects, light: [], color: [], dash: [], fill: [] })
        bar.tool = "object"

        verify(!findChild(bar, "libraryExport").enabled)
        verify(findChild(bar, "libraryImport").enabled)
    }

    // Picks the entry of `kind` in the object list.
    function pick(bar, kind) {
        const box = findChild(bar, "objectBox")
        const index = box.model.findIndex(o => o.value === kind)
        box.currentIndex = index
        box.activated(index)
    }

    // LH-12: rename, regroup and delete are for the own entry the list has picked.
    function test_only_an_own_object_can_be_changed_or_deleted() {
        const bar = make()
        bar.tool = "object"
        const names = ["ownName", "ownGroup", "libraryRename", "libraryDelete"]

        pick(bar, "table")
        compare(names.map(n => findChild(bar, n).visible), [false, false, false, false])

        pick(bar, "thron")
        compare(names.map(n => findChild(bar, n).visible), [true, true, true, true])
        compare(findChild(bar, "ownName").text, "Thron")
        compare(findChild(bar, "ownGroup").text, "")

        pick(bar, "ruine")
        compare(findChild(bar, "ownName").text, "Ruine")
        compare(findChild(bar, "ownGroup").text, "Requisiten")

        bar.tool = "select"
        compare(names.map(n => findChild(bar, n).visible), [false, false, false, false])
    }

    function test_the_typed_name_and_group_go_to_the_controller() {
        const bar = make()
        bar.tool = "object"
        pick(bar, "thron")

        findChild(bar, "ownName").text = "Königsthron"
        findChild(bar, "ownGroup").text = "Hof"
        findChild(bar, "libraryRename").clicked()

        compare(fake.edits, [["thron", "Königsthron", "Hof"]])
        verify(!findChild(bar, "ownName").invalid)
    }

    function test_a_refused_name_marks_the_field() {
        const bar = make()
        bar.tool = "object"
        pick(bar, "thron")
        fake.accept = false

        findChild(bar, "ownName").text = ""
        findChild(bar, "libraryRename").clicked()

        verify(findChild(bar, "ownName").invalid)
        findChild(bar, "ownName").text = "x"
        verify(!findChild(bar, "ownName").invalid)
    }

    // The first click asks and names the objects of the plan; the second deletes.
    function test_deleting_asks_first_and_names_the_objects_in_use() {
        const bar = make()
        bar.tool = "object"
        pick(bar, "thron")
        const button = findChild(bar, "libraryDelete")
        compare(button.text, "Löschen")

        button.clicked()
        compare(button.text, "Wirklich löschen? 3 im Plan")
        compare(fake.removed, [])

        button.clicked()
        compare(fake.removed, ["thron"])
        compare(button.text, "Löschen")
    }

    // A lamp or character id can equal the kind of an own object; only the object tool edits.
    function test_another_tool_with_the_id_of_an_own_kind_changes_nothing() {
        const bar = make()
        const choices = JSON.parse(fake.choicesJson)
        choices.light.push("thron")
        fake.choicesJson = JSON.stringify(choices)
        bar.tool = "light"
        const box = findChild(bar, "lightBox")
        box.currentIndex = box.model.findIndex(o => o.value === "thron")
        box.activated(box.currentIndex)

        compare(bar.choice, "thron")
        for (const n of ["ownName", "ownGroup", "libraryRename", "libraryDelete"]) {
            verify(!findChild(bar, n).visible, n)
        }
    }

    function test_deleting_an_unused_entry_does_not_count() {
        const bar = make()
        bar.tool = "object"
        pick(bar, "ruine")
        const button = findChild(bar, "libraryDelete")

        button.clicked()
        compare(button.text, "Wirklich löschen?")
    }

    function test_another_pick_cancels_the_question() {
        const bar = make()
        bar.tool = "object"
        pick(bar, "thron")
        const button = findChild(bar, "libraryDelete")

        button.clicked()
        pick(bar, "ruine")

        compare(button.text, "Löschen")
        button.clicked()
        compare(fake.removed, [])
    }

    function test_lists_have_translated_names() {
        const bar = make()
        bar.tool = "object"

        compare(findChild(bar, "objectBox").textAt(0), "Tisch")
        bar.tool = "light"
        compare(findChild(bar, "lightBox").textAt(1), "LED-Panel")
    }

    // LH-19: colour and line style for every drawing tool, the fill only where there is an inside.
    function test_the_style_shows_for_the_drawing_tools() {
        const bar = make()
        const shown = () => ["colorPicker", "dashBox", "fillBox"].filter(n => findChild(bar, n).visible)

        compare(shown(), [])
        for (const tool of ["line", "arrow"]) {
            bar.tool = tool
            compare(shown(), ["colorPicker", "dashBox"], tool)
        }
        for (const tool of ["rect", "ellipse", "area"]) {
            bar.tool = tool
            compare(shown(), ["colorPicker", "dashBox", "fillBox"], tool)
        }
        bar.tool = "wall"
        compare(shown(), [])
    }

    function test_the_style_goes_to_the_controller() {
        const bar = make()
        bar.tool = "rect"

        compare(fake.styles[fake.styles.length - 1], ["red", "solid", "outline"])

        findChild(bar, "colorPicker").picked(5)
        compare(fake.styles[fake.styles.length - 1], ["blue", "solid", "outline"])

        const dash = findChild(bar, "dashBox")
        dash.currentIndex = 2
        dash.activated(2)
        compare(fake.styles[fake.styles.length - 1], ["blue", "dotted", "outline"])

        const fill = findChild(bar, "fillBox")
        fill.currentIndex = 1
        fill.activated(1)
        compare(fake.styles[fake.styles.length - 1], ["blue", "dotted", "filled"])
    }

    function test_the_style_controls_have_translated_names() {
        const bar = make()
        bar.tool = "area"

        compare(findChild(bar, "dashBox").textAt(1), "Gestrichelt")
        compare(findChild(bar, "fillBox").textAt(1), "Gefüllt")
    }

    function test_snapping_switch_tells_the_controller() {
        const bar = make()
        const button = findChild(bar, "snapButton")

        verify(button.checked)
        button.toggle()
        button.toggled()

        compare(fake.snaps, [false])
    }

    // LH-14: the grid box lists the spacings, shows the current one and tells the controller.
    function test_grid_box_lists_and_sets_the_spacing() {
        const bar = make()
        const box = findChild(bar, "gridBox")

        compare(box.count, 3)
        compare(box.currentIndex, 1)
        compare(box.textAt(0), "5 cm")
        compare(box.textAt(2), "1 m")

        box.currentIndex = 2
        box.activated(2)

        compare(fake.grids, ["m1"])
    }
}
