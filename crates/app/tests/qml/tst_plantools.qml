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
        readonly property var choices: ({
            figure: [{ id: "elisabeth", name: "Elisabeth" }, { id: "burgvogt", name: "Burgvogt" }],
            object: [
                { kind: "table", label: "", width_m: 1.6, depth_m: 0.9, shape: "rect", custom: false },
                { kind: "chair", label: "", width_m: 0.45, depth_m: 0.45, shape: "rect", custom: false },
                { kind: "thron", label: "Thron", width_m: 0.9, depth_m: 0.8, shape: "rect", custom: true }
            ],
            light: ["fresnel", "led-panel"],
            color: ["red", "orange", "yellow", "green", "cyan", "blue", "purple"],
            dash: ["solid", "dashed", "dotted"],
            fill: ["outline", "filled"]
        })
        property string choicesJson: JSON.stringify(choices)
        property var tools: []
        property var snaps: []
        property var styles: []

        function setTool(tool, choice) {
            tools.push([tool, choice])
            return true
        }
        function setDrawStyle(color, dash, fill) {
            styles.push([color, dash, fill])
            return true
        }
        function setSnapping(on) {
            snaps.push(on)
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
        const shown = () => ["objectBox", "figureBox", "lightBox"].filter(n => findChild(bar, n).visible)

        compare(shown(), [])
        bar.tool = "object"
        compare(shown(), ["objectBox"])
        bar.tool = "figure"
        compare(shown(), ["figureBox"])
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
        compare(bar.choice, "fresnel")
    }

    function test_the_library_lists_built_in_then_own_objects_in_groups() {
        const bar = make()
        const box = findChild(bar, "objectBox")

        compare(box.model.map(o => o.text), ["Tisch", "Stuhl", "Thron"])
        compare(box.model.map(o => o.group), ["Eingebaut", "Eingebaut", "Eigene"])
        compare(box.sectionAt(2), "Eigene")
    }

    function test_the_picked_object_goes_to_the_controller() {
        const bar = make()
        const box = findChild(bar, "objectBox")

        bar.tool = "object"
        compare(bar.choice, "table")
        compare(fake.tools[fake.tools.length - 1], ["object", "table"])

        box.currentIndex = 2
        box.activated(2)
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
        objects.unshift({ kind: "bed", label: "", width_m: 1.4, depth_m: 2, shape: "rect", custom: false })
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
}
