import QtQuick
import QtQuick.Controls as QQC2
import QtTest
import "../../qml"

// The command palette of AppHeader (D-03): it lists the tools, modes, menu commands, scenes and
// plans with their keys, runs them, and locks with the window.
TestCase {
    id: tc
    name: "AppHeaderPalette"
    width: 1000
    height: 600
    when: windowShown
    visible: true

    FakeHeaderController { id: fake }

    QtObject {
        id: fakeTools
        property string tool: "select"
        property string mode: "build"
        property bool enabled: true
        readonly property var names: ["select", "wall", "camera"]
        readonly property var keys: ["V", "W", "C"]
        readonly property var modes: ["build", "stage", "resolve", "draw"]
        readonly property var modeKeys: ["1", "2", "3", "4"]
    }

    TestTexts { id: de; language: "de" }
    TestTexts { id: en; language: "en" }

    Component {
        id: headerComponent
        AppHeader {
            controller: fake
            texts: de
            tools: fakeTools
            width: tc.width
        }
    }

    SignalSpy { id: newSpy; signalName: "newRequested" }
    SignalSpy { id: openSpy; signalName: "openRequested" }
    SignalSpy { id: closeSpy; signalName: "closeRequested" }
    SignalSpy { id: projectSpy; signalName: "projectRequested" }
    SignalSpy { id: backgroundSpy; signalName: "backgroundRequested" }
    SignalSpy { id: viewSpy; signalName: "viewToggled" }
    SignalSpy { id: orderSpy; signalName: "shotOrderPicked" }
    SignalSpy { id: zoomSpy; signalName: "zoomRequested" }
    SignalSpy { id: playSpy; signalName: "playRequested" }
    SignalSpy { id: heightSpy; signalName: "heightRequested" }

    // Stands for the plan's canvas, which takes Space itself (held, it pans).
    Item { id: fakeCanvas; objectName: "planCanvas"; width: 10; height: 10 }

    function init() {
        fake.loaded = true
        fake.title = "Wartburg"
        fake.sceneId = "s12"
        fake.status = ""
        fake.dirty = false
        fake.canUndo = true
        fake.canRedo = true
        fake.hasBackground = false
        fake.viewFree = false
        fake.missingHeights = 0
        fake.calls = []
        fake.selectedKey = ""
        fake.curve = ""
        fake.plan = "plan-a"
        fake.projectJson = JSON.stringify({ scenes: fake.scenes, plans: fake.plans, plan: fake.plan, formats: fake.formats })
        fakeTools.tool = "select"
        fakeTools.mode = "build"
        fakeTools.enabled = true
    }

    function make() {
        const header = createTemporaryObject(headerComponent, tc)
        verify(header)
        for (const spy of [newSpy, openSpy, closeSpy, projectSpy, backgroundSpy, viewSpy, orderSpy, zoomSpy, playSpy, heightSpy]) {
            spy.target = header
            spy.clear()
        }
        return header
    }

    // An entry of a menu: found by its object name, the menu opened so that it shows.
    function entry(header, menuName, name) {
        const button = findChild(header, menuName)
        verify(button, menuName)
        button.menu.open()
        const found = findChild(button.menu.contentItem, name) || findChild(button.menu, name)
        verify(found, name)
        return found
    }

    function pick(header, menuName, name) {
        const item = entry(header, menuName, name)
        verify(item.enabled, name)
        // Not a click: a theme with native menus takes none from a test.
        verify(item.action, name + " runs an action")
        item.action.trigger()
        header.children.forEach(c => { if (c.menu) { c.menu.close() } })
    }

    function names(header) {
        return header.commandPalette.entries.map(c => c.text)
    }

    function test_the_palette_lists_tools_with_their_keys() {
        const header = make()

        const camera = header.commandPalette.commands.find(c => c.id === "tool:camera")

        compare(camera.text, "Kamera")
        compare(camera.keys, "C")
        compare(camera.group, "Werkzeuge")
    }

    function test_the_palette_lists_the_modes_with_their_keys() {
        const header = make()

        const modes = header.commandPalette.commands.filter(c => c.id.startsWith("mode:"))

        compare(modes.map(c => c.text), ["Modus: Set bauen", "Modus: Inszenieren", "Modus: Auflösen", "Modus: Zeichnen"])
        compare(modes.map(c => c.keys), ["1", "2", "3", "4"])
        compare(modes[0].group, "Modus")
    }

    function test_a_mode_from_the_palette_is_picked() {
        const header = make()

        header.run("mode:draw")

        compare(fakeTools.mode, "draw")
    }

    function test_a_tool_from_the_palette_is_picked() {
        const header = make()

        header.run("tool:wall")

        compare(fakeTools.tool, "wall")
    }

    function test_the_palette_lists_the_menu_commands_and_runs_them() {
        const header = make()
        const wanted = ["PDF", "PNG", "SVG", "Shotliste CSV", "Speichern", "Neu…", "Öffnen…", "Projekt…",
                        "Hintergrund…", "Drehbuch", "Shot", "Zeitleiste", "Schnittfolge", "Drehreihenfolge",
                        "Rückgängig", "Wiederholen"]
        for (const text of wanted) {
            verify(header.commandPalette.commands.some(c => c.text === text), text)
        }

        header.commandPalette.open()
        tryCompare(header.commandPalette, "opened", true)
        keyClick("s")
        keyClick("v")
        keyClick("g")
        compare(header.commandPalette.entries.map(c => c.id), ["export.svg"])
        keyClick(Qt.Key_Return)

        const dialog = findChild(header, "exportDialog")
        compare(dialog.kind, "svg")
        tryCompare(dialog, "opened", true)
    }

    function test_the_palette_names_the_key_of_a_command() {
        const header = make()

        compare(header.commandPalette.commands.find(c => c.id === "project.save").keys, "Strg+S")
        compare(header.commandPalette.commands.find(c => c.id === "edit.undo").keys, "Strg+Z")

        header.texts = en
        compare(header.commandPalette.commands.find(c => c.id === "edit.redo").keys, "Ctrl+Shift+Z")
    }

    function test_the_palette_switches_scene_and_plan() {
        const header = make()
        const scene = header.commandPalette.commands.find(c => c.id === "scene:s13")
        const plan = header.commandPalette.commands.find(c => c.id === "plan:plan-b")

        compare(scene.text, "Szene 13 · EXT. HOF – NACHT")
        compare(scene.group, "Szenen")
        compare(plan.text, "Plan B")
        compare(plan.group, "Pläne")
        verify(!header.commandPalette.commands.find(c => c.id === "scene:s12").enabled)
        verify(!header.commandPalette.commands.find(c => c.id === "plan:plan-a").enabled)

        header.run("scene:s13")
        header.run("plan:plan-b")

        compare(fake.calls, [["selectScene", "s13"], ["selectPlan", "plan-b"]])
    }

    function test_the_palette_runs_the_plan_commands() {
        const header = make()
        const form = findChild(findChild(header, "planMenu"), "planForm")

        header.run("plan.new")

        tryCompare(form, "opened", true)
        compare(form.title, "Neuer Plan")
    }

    SignalSpy { id: templateImportSpy; signalName: "templateImportRequested" }

    function test_the_palette_asks_the_window_for_a_template_file() {
        const header = make()
        templateImportSpy.target = header
        templateImportSpy.clear()

        header.run("plan.templateImport")

        compare(templateImportSpy.count, 1)
    }

    function test_a_locked_window_locks_scenes_plans_and_tools_in_the_palette() {
        const header = make()
        header.locked = true
        fakeTools.enabled = false
        fake.hasBackground = true

        for (const id of ["scene:s13", "plan:plan-b", "tool:wall", "mode:stage", "project.new", "plan.snapshot",
                          "plan.new", "plan.rename", "plan.remove", "background.set", "background.remove"]) {
            verify(!header.commandPalette.commands.find(c => c.id === id).enabled, id)
        }
    }
}
