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
        function exportFile(extension) { return "file:///work/Wartburg." + extension }
        function exportTo(kind, order, paper, path) { return call("exportTo", [kind, order, paper, path]) }
        function selectScene(id) { return call("selectScene", [id]) }
        function selectPlan(id) { return call("selectPlan", [id]) }
        function clearStatus() {}
        function stepMark(direction) { call("stepMark", [direction]) }
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
    SignalSpy { id: zoomSpy; signalName: "zoomRequested" }
    SignalSpy { id: playSpy; signalName: "playRequested" }

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
        for (const spy of [newSpy, openSpy, projectSpy, backgroundSpy, viewSpy, orderSpy, zoomSpy, playSpy]) {
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

    function test_the_exports_ask_for_paper_and_file() {
        const header = make()
        const dialog = findChild(header, "exportDialog")

        // The plan exports ask for the paper first, the shot list for the file only.
        for (const kind of ["pdf", "png", "svg"]) {
            pick(header, "exportMenu", kind)
            compare(dialog.kind, kind)
            tryCompare(dialog, "opened", true)
            dialog.close()
            tryCompare(dialog, "opened", false)
        }
        pick(header, "exportMenu", "csv")
        compare(dialog.kind, "csv")
        verify(!dialog.opened)

        // Nothing is written before a file is chosen.
        compare(fake.calls, [])
    }

    function test_the_exports_use_the_shot_order() {
        const header = make()
        const dialog = findChild(header, "exportDialog")

        dialog.kind = "pdf"
        dialog.save("file:///work/a.pdf")
        header.shotOrder = "shooting"
        dialog.save("file:///work/b.pdf")
        dialog.kind = "csv"
        dialog.save("file:///work/b.csv")

        compare(fake.calls, [["exportTo", "pdf", "cut", "a4", "file:///work/a.pdf"],
                             ["exportTo", "pdf", "shooting", "a4", "file:///work/b.pdf"],
                             ["exportTo", "csv", "shooting", "a4", "file:///work/b.csv"]])
    }

    function test_ctrl_p_is_the_key_of_the_pdf_export() {
        const header = make()
        const pdf = header.commandPalette.commands.find(c => c.id === "export.pdf")

        compare(pdf.keys, "Strg+P")
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

    function test_ctrl_n_and_ctrl_o_ask_for_a_project() {
        make()

        keyClick(Qt.Key_N, Qt.ControlModifier)
        compare(newSpy.count, 1)
        keyClick(Qt.Key_O, Qt.ControlModifier)
        compare(openSpy.count, 1)
    }

    function test_ctrl_n_and_ctrl_o_work_without_a_project() {
        fake.loaded = false
        make()

        keyClick(Qt.Key_N, Qt.ControlModifier)
        keyClick(Qt.Key_O, Qt.ControlModifier)

        compare(newSpy.count, 1)
        compare(openSpy.count, 1)
    }

    function test_ctrl_plus_minus_and_zero_zoom_the_plan() {
        make()

        keyClick(Qt.Key_Plus, Qt.ControlModifier)
        keyClick(Qt.Key_Equal, Qt.ControlModifier)
        keyClick(Qt.Key_Minus, Qt.ControlModifier)
        keyClick(Qt.Key_0, Qt.ControlModifier)

        compare(zoomSpy.count, 4)
        compare(zoomSpy.signalArguments.map(a => a[0]), ["in", "in", "out", "fit"])
    }

    function test_space_and_k_play_and_pause() {
        make()

        keyClick(Qt.Key_Space)
        keyClick(Qt.Key_K)

        compare(playSpy.count, 2)
    }

    function test_space_stays_with_the_plan_which_pans_with_it() {
        make()
        fakeCanvas.forceActiveFocus()
        verify(fakeCanvas.activeFocus)

        keyClick(Qt.Key_Space)
        compare(playSpy.count, 0)

        keyClick(Qt.Key_K)
        compare(playSpy.count, 1)
        fakeCanvas.focus = false
    }

    function test_space_presses_a_focused_button_instead_of_playing() {
        const header = make()
        const button = findChild(header, "undo")
        button.forceActiveFocus()
        verify(button.activeFocus)

        keyClick(Qt.Key_Space)

        compare(playSpy.count, 0)
        button.focus = false
    }

    function test_comma_and_period_step_the_marks() {
        make()

        keyClick(Qt.Key_Comma)
        keyClick(Qt.Key_Period)

        compare(fake.calls, [["stepMark", -1], ["stepMark", 1]])
    }

    function test_an_open_backup_question_locks_zoom_play_and_marks() {
        make().locked = true

        keyClick(Qt.Key_Plus, Qt.ControlModifier)
        keyClick(Qt.Key_Space)
        keyClick(Qt.Key_Comma)
        keyClick(Qt.Key_N, Qt.ControlModifier)

        compare(zoomSpy.count, 0)
        compare(playSpy.count, 0)
        compare(fake.calls, [])
        compare(newSpy.count, 0)
    }

    function test_the_palette_names_the_keys_of_the_new_commands() {
        const header = make()
        const keys = id => header.commandPalette.commands.find(c => c.id === id).keys

        compare(keys("project.new"), "Strg+N")
        compare(keys("project.open"), "Strg+O")
        compare(keys("view.zoomIn"), "Strg++")
        compare(keys("view.zoomOut"), "Strg+-")
        compare(keys("view.zoomFit"), "Strg+0")
        compare(keys("playback.toggle"), "Leertaste / K")
        compare(keys("playback.markPrev"), ",")
        compare(keys("playback.markNext"), ".")

        header.texts = en
        compare(keys("playback.toggle"), "Space / K")
    }

    function test_the_view_menu_zooms() {
        const header = make()

        pick(header, "viewMenu", "zoomIn")
        pick(header, "viewMenu", "zoomOut")
        pick(header, "viewMenu", "zoomFit")

        compare(zoomSpy.signalArguments.map(a => a[0]), ["in", "out", "fit"])
    }

    function test_the_texts_are_in_both_languages() {
        for (const key of ["menu.file", "menu.export", "menu.view", "menu.viewAreas", "menu.viewOrder",
                           "menu.commands", "palette.placeholder", "palette.empty", "palette.name",
                           "palette.group.tools", "palette.group.modes", "palette.mode", "mode.build", "palette.group.file", "palette.group.export",
                           "palette.group.view", "palette.group.edit", "palette.group.scenes",
                           "palette.group.plans", "key.ctrl", "key.shift", "key.space", "menu.viewZoom", "menu.zoomIn",
                           "menu.zoomOut", "menu.zoomFit", "menu.play", "menu.markPrev", "menu.markNext",
                           "palette.group.playback"]) {
            verify(de.t(key) !== key, "de " + key)
            verify(en.t(key) !== key, "en " + key)
        }
    }
}
