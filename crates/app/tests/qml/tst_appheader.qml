import QtQuick
import QtTest
import "../../qml"

// AppHeader (D-02, D-03, D-07): the menus run the controller's functions, the palette lists the
// same commands (and the tools, scenes and plans), Ctrl+K opens it, and without a project only
// the mark and the File menu stay.
TestCase {
    id: tc
    name: "AppHeader"
    width: 1000
    height: 600
    when: windowShown
    visible: true

    QtObject {
        id: fake
        property bool loaded: true
        property string title: "Wartburg"
        property string sceneId: "s12"
        property string status: ""
        property bool dirty: false
        property bool canUndo: true
        property bool canRedo: true
        property bool hasBackground: false
        property var calls: []
        property var scenes: [
            { id: "s12", number: "12", heading: "INT. SÄNGERSAAL – TAG" },
            { id: "s13", number: "13", heading: "EXT. HOF – NACHT" }
        ]
        property var plans: [
            { id: "plan-a", name: "Plan A", shots: 5 },
            { id: "plan-b", name: "Plan B", shots: 0 }
        ]
        property string plan: "plan-a"
        property string projectJson: JSON.stringify({ scenes: scenes, plans: plans, plan: plan })

        function call(name, args) {
            calls.push([name].concat(args))
            return true
        }
        function undo() { call("undo", []) }
        function redo() { call("redo", []) }
        function save() { return call("save", []) }
        function removeBackground() { call("removeBackground", []) }
        function exportPdf(order) { call("exportPdf", [order]) }
        function exportPng() { call("exportPng", []) }
        function exportSvg() { call("exportSvg", []) }
        function exportCsv(order) { call("exportCsv", [order]) }
        function selectScene(id) { return call("selectScene", [id]) }
        function selectPlan(id) { return call("selectPlan", [id]) }
        function clearStatus() {}
    }

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
    SignalSpy { id: projectSpy; signalName: "projectRequested" }
    SignalSpy { id: backgroundSpy; signalName: "backgroundRequested" }
    SignalSpy { id: viewSpy; signalName: "viewToggled" }
    SignalSpy { id: orderSpy; signalName: "shotOrderPicked" }

    function init() {
        fake.loaded = true
        fake.title = "Wartburg"
        fake.sceneId = "s12"
        fake.status = ""
        fake.dirty = false
        fake.canUndo = true
        fake.canRedo = true
        fake.hasBackground = false
        fake.calls = []
        fake.plan = "plan-a"
        fake.projectJson = JSON.stringify({ scenes: fake.scenes, plans: fake.plans, plan: fake.plan })
        fakeTools.tool = "select"
        fakeTools.mode = "build"
        fakeTools.enabled = true
    }

    function make() {
        const header = createTemporaryObject(headerComponent, tc)
        verify(header)
        for (const spy of [newSpy, openSpy, projectSpy, backgroundSpy, viewSpy, orderSpy]) {
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

    function test_the_path_names_project_scene_and_plan() {
        const header = make()

        compare(findChild(header, "path").text, "Wartburg › Szene 12 · INT. SÄNGERSAAL – TAG ›")
        compare(findChild(header, "planMenu").text, "Plan A")
    }

    function test_the_path_elides_before_the_menus_are_cut_at_1000_px() {
        const header = make()
        fake.title = "Ein sehr langer Projektname für ein Filmprojekt mit vielen Worten"
        fake.status = "Gespeichert: ein langer Pfad zu einer Datei"
        header.width = 1000

        const path = findChild(header, "path")
        verify(path.truncated)
        for (const name of ["fileMenu", "exportMenu", "viewMenu", "undo", "redo", "paletteButton", "planMenu"]) {
            const item = findChild(header, name)
            verify(item.visible, name)
            verify(item.x >= 0 && item.x + item.width <= header.width, name + " is cut")
        }
        verify(path.x + path.width <= findChild(header, "planMenu").x)
    }

    function test_without_a_project_only_the_mark_and_the_file_menu_stay() {
        fake.loaded = false
        const header = make()

        verify(findChild(header, "fileMenu").visible)
        for (const name of ["path", "planMenu", "exportMenu", "viewMenu", "undo", "redo", "paletteButton"]) {
            verify(!findChild(header, name).visible, name)
        }
        verify(entry(header, "fileMenu", "new").visible)
        verify(entry(header, "fileMenu", "open").visible)
        for (const name of ["project", "save", "background", "backgroundRemove"]) {
            verify(!entry(header, "fileMenu", name).visible, name)
        }
    }

    function test_new_and_open_say_what_was_asked() {
        fake.loaded = false
        const header = make()

        pick(header, "fileMenu", "new")
        pick(header, "fileMenu", "open")

        compare(newSpy.count, 1)
        compare(openSpy.count, 1)
    }

    function test_project_background_and_save_reach_their_targets() {
        const header = make()

        pick(header, "fileMenu", "project")
        pick(header, "fileMenu", "background")
        pick(header, "fileMenu", "save")

        compare(projectSpy.count, 1)
        compare(backgroundSpy.count, 1)
        compare(fake.calls, [["save"]])
    }

    function test_removing_the_background_is_offered_only_with_one() {
        const header = make()
        verify(!entry(header, "fileMenu", "backgroundRemove").visible)

        fake.hasBackground = true
        verify(entry(header, "fileMenu", "backgroundRemove").visible)
        pick(header, "fileMenu", "backgroundRemove")

        compare(fake.calls, [["removeBackground"]])
    }

    function test_an_open_backup_question_locks_what_replaces_the_project() {
        const header = make()
        header.locked = true

        for (const name of ["new", "open", "project"]) {
            verify(!entry(header, "fileMenu", name).enabled, name)
        }
        verify(!findChild(header, "planMenu").enabled)
        verify(entry(header, "fileMenu", "save").enabled)
    }

    function test_the_exports_use_the_shot_order() {
        const header = make()

        pick(header, "exportMenu", "pdf")
        pick(header, "exportMenu", "png")
        pick(header, "exportMenu", "svg")
        pick(header, "exportMenu", "csv")
        header.shotOrder = "shooting"
        pick(header, "exportMenu", "pdf")
        pick(header, "exportMenu", "csv")

        compare(fake.calls, [["exportPdf", "cut"], ["exportPng"], ["exportSvg"], ["exportCsv", "cut"],
                             ["exportPdf", "shooting"], ["exportCsv", "shooting"]])
    }

    function test_the_view_menu_toggles_the_areas() {
        const header = make()
        verify(entry(header, "viewMenu", "script").checked)

        pick(header, "viewMenu", "script")
        pick(header, "viewMenu", "timeline")

        compare(viewSpy.count, 2)
        compare(Array.from(viewSpy.signalArguments[0]), ["script", false])
        compare(Array.from(viewSpy.signalArguments[1]), ["timeline", false])
    }

    function test_the_view_menu_ticks_the_areas_shown() {
        const header = make()
        header.showShot = false

        verify(entry(header, "viewMenu", "script").checked)
        verify(!entry(header, "viewMenu", "shot").checked)
        verify(entry(header, "viewMenu", "timeline").checked)
    }

    function test_the_view_menu_picks_the_shot_order() {
        const header = make()
        verify(entry(header, "viewMenu", "cut").checked)
        verify(!entry(header, "viewMenu", "shooting").checked)

        pick(header, "viewMenu", "shooting")

        compare(orderSpy.count, 1)
        compare(Array.from(orderSpy.signalArguments[0]), ["shooting"])
    }

    function test_undo_and_redo_are_icon_buttons_with_their_keys_in_the_tooltip() {
        const header = make()
        const undo = findChild(header, "undo")
        const redo = findChild(header, "redo")

        compare(undo.icon.name, "edit-undo")
        compare(redo.icon.name, "edit-redo")
        compare(undo.text, "Rückgängig")
        mouseClick(undo)
        mouseClick(redo)

        compare(fake.calls, [["undo"], ["redo"]])
    }

    function test_undo_and_redo_wait_for_something_to_undo() {
        const header = make()
        fake.canUndo = false
        fake.canRedo = false

        verify(!findChild(header, "undo").enabled)
        verify(!findChild(header, "redo").enabled)
    }

    function test_ctrl_z_undoes_and_ctrl_k_opens_the_palette() {
        const header = make()

        keyClick(Qt.Key_Z, Qt.ControlModifier)
        compare(fake.calls, [["undo"]])

        keyClick(Qt.Key_K, Qt.ControlModifier)
        tryCompare(header.commandPalette, "opened", true)
    }

    function test_the_palette_button_opens_it() {
        const header = make()

        mouseClick(findChild(header, "paletteButton"))

        tryCompare(header.commandPalette, "opened", true)
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

        compare(fake.calls, [["exportSvg"]])
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

    function test_the_texts_are_in_both_languages() {
        for (const key of ["menu.file", "menu.export", "menu.view", "menu.viewAreas", "menu.viewOrder",
                           "menu.commands", "palette.placeholder", "palette.empty", "palette.name",
                           "palette.group.tools", "palette.group.modes", "palette.mode", "mode.build", "palette.group.file", "palette.group.export",
                           "palette.group.view", "palette.group.edit", "palette.group.scenes",
                           "palette.group.plans", "key.ctrl", "key.shift"]) {
            verify(de.t(key) !== key, "de " + key)
            verify(en.t(key) !== key, "en " + key)
        }
    }
}
