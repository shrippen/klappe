import QtQuick
import QtQuick.Controls as QQC2
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

    SignalSpy { id: speakSpy; signalName: "speakRequested" }
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
    SignalSpy { id: scriptSpy; signalName: "scriptRequested" }
    SignalSpy { id: planSpy; signalName: "planRequested" }

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
        for (const spy of [newSpy, openSpy, closeSpy, projectSpy, backgroundSpy, viewSpy, orderSpy, zoomSpy, playSpy, heightSpy, scriptSpy, planSpy]) {
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

    // The status line is short; what it cuts off shows whole as a tooltip.
    function test_a_cut_status_shows_whole_as_tooltip() {
        const header = make()
        fake.status = "Gespeichert: ein sehr langer Pfad zu einer Datei in einem tiefen Ordner"
        const status = findChild(header, "status")
        tryVerify(() => status.truncated)

        verify(!status.QQC2.ToolTip.visible)
        compare(status.QQC2.ToolTip.text, fake.status)
        mouseMove(status, status.width / 2, status.height / 2)

        tryVerify(() => status.QQC2.ToolTip.visible)
    }

    function test_a_short_status_has_no_tooltip() {
        const header = make()
        fake.status = "Gespeichert"
        const status = findChild(header, "status")
        verify(!status.truncated)

        mouseMove(status, status.width / 2, status.height / 2)
        wait(300)

        verify(!status.QQC2.ToolTip.visible)
    }

    // The start page has the buttons for New and Open, so the File menu stays away (and the
    // status line: the start page shows it whole).
    function test_without_a_project_only_the_mark_stays() {
        fake.loaded = false
        fake.status = "Projekt konnte nicht gelesen werden"
        const header = make()

        for (const name of ["fileMenu", "status", "path", "planMenu", "exportMenu", "viewMenu", "undo", "redo", "paletteButton"]) {
            verify(!findChild(header, name).visible, name)
        }
    }

    function test_new_and_open_say_what_was_asked() {
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

    function test_close_is_in_the_file_menu_and_asks_to_close() {
        const header = make()

        compare(entry(header, "fileMenu", "close").text, "Schließen")
        pick(header, "fileMenu", "close")

        compare(closeSpy.count, 1)
        compare(fake.calls, [])
    }

    function test_ctrl_w_closes_the_project() {
        make()

        keyClick(Qt.Key_W, Qt.ControlModifier)
        compare(closeSpy.count, 1)

        fake.loaded = false
        keyClick(Qt.Key_W, Qt.ControlModifier)
        compare(closeSpy.count, 1)
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

    Component { id: strangerComponent; Window { width: 100; height: 100 } }

    // Q1-19: the file dialog is a window of its own; without a window manager nobody hands the
    // keys back, so the dialog does it when it closes. A window of ours stands for the dialog's
    // here (offscreen has none), and there is no `back()`.
    function test_the_shortcuts_work_right_after_the_file_dialog() {
        const header = make()
        const dialog = findChild(header, "exportDialog")
        const files = findChild(dialog, "files")

        pick(header, "exportMenu", "csv")
        tryCompare(files, "visible", true)
        const stranger = createTemporaryObject(strangerComponent, null)
        stranger.show()
        stranger.requestActivate()
        tryCompare(tc.Window.window, "active", false)
        stranger.close()
        files.close()
        tryCompare(files, "visible", false)
        tryCompare(tc.Window.window, "active", true)

        fake.calls = []
        keyClick(Qt.Key_S, Qt.ControlModifier)
        compare(fake.calls, [["save"]])
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
        // The shot list asks for the file straight away (QA Q1-01).
        const files = findChild(dialog, "files")
        tryCompare(files, "visible", true)
        files.close()
        tryCompare(files, "visible", false)
        back()

        // Nothing is written before a file is chosen.
        compare(fake.calls, [])
    }

    function test_the_list_and_frame_exports_are_in_the_menu() {
        const header = make()
        const dialog = findChild(header, "exportDialog")

        // The lists ask for the file only, the frames (a folder) for the paper first.
        const files = findChild(dialog, "files")
        for (const kind of ["ods", "xlsx"]) {
            pick(header, "exportMenu", kind)
            compare(dialog.kind, kind)
            verify(!dialog.opened)
            tryCompare(files, "visible", true)
            files.close()
            tryCompare(files, "visible", false)
            back()
        }
        pick(header, "exportMenu", "frames")
        compare(dialog.kind, "frames")
        tryCompare(dialog, "opened", true)
        dialog.close()
        tryCompare(dialog, "opened", false)

        for (const id of ["export.ods", "export.xlsx", "export.frames"]) {
            verify(header.commandPalette.commands.some(c => c.id === id), id)
        }

        dialog.kind = "ods"
        dialog.save("file:///work/Liste.ods")
        dialog.kind = "xlsx"
        dialog.save("file:///work/Liste.xlsx")
        dialog.kind = "frames"
        dialog.save("file:///work/Wartburg")

        compare(fake.calls, [["exportTo", "ods", "cut", "a4-landscape", "file:///work/Liste.ods"],
                             ["exportTo", "xlsx", "cut", "a4-landscape", "file:///work/Liste.xlsx"],
                             ["exportTo", "frames", "cut", "a4-landscape", "file:///work/Wartburg"]])
    }

    // LH-86: the 3D view as it is on screen (so only in 3D) and the camera view, as PNG: both ask
    // for the file only and are in the palette.
    function test_the_view_exports_are_in_the_menu() {
        const header = make()
        const dialog = findChild(header, "exportDialog")

        verify(!entry(header, "exportMenu", "view3d").enabled)
        verify(entry(header, "exportMenu", "camera").enabled)
        pick(header, "exportMenu", "camera")
        compare(dialog.kind, "camera")
        verify(!dialog.opened)

        fake.viewFree = true
        verify(entry(header, "exportMenu", "view3d").enabled)
        pick(header, "exportMenu", "view3d")
        compare(dialog.kind, "view3d")
        verify(!dialog.opened)

        for (const id of ["export.view3d", "export.camera"]) {
            verify(header.commandPalette.commands.some(c => c.id === id), id)
        }

        dialog.kind = "view3d"
        dialog.save("file:///work/Wartburg-3d.png")
        dialog.kind = "camera"
        dialog.save("file:///work/Wartburg-camera.png")

        compare(fake.calls, [["exportTo", "view3d", "cut", "a4-landscape", "file:///work/Wartburg-3d.png"],
                             ["exportTo", "camera", "cut", "a4-landscape", "file:///work/Wartburg-camera.png"]])
    }

    // LH-84: the video asks for the file only and is in the palette.
    function test_the_video_export_is_in_the_menu() {
        const header = make()
        const dialog = findChild(header, "exportDialog")

        verify(entry(header, "exportMenu", "video").enabled)
        pick(header, "exportMenu", "video")
        compare(dialog.kind, "mp4")
        verify(!dialog.opened)
        verify(header.commandPalette.commands.some(c => c.id === "export.mp4"))

        dialog.save("file:///work/Wartburg.mp4")

        compare(fake.calls, [["exportTo", "mp4", "cut", "a4-landscape", "file:///work/Wartburg.mp4"]])
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

        compare(fake.calls, [["exportTo", "pdf", "cut", "a4-landscape", "file:///work/a.pdf"],
                             ["exportTo", "pdf", "shooting", "a4-landscape", "file:///work/b.pdf"],
                             ["exportTo", "csv", "shooting", "a4-landscape", "file:///work/b.csv"]])
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

    // LH-103: the tooltips name the step; without a text for it the plain name and key stay.
    function test_undo_and_redo_tooltips_name_the_step() {
        const header = make()
        const undo = findChild(header, "undo")
        const redo = findChild(header, "redo")
        compare(undo.QQC2.ToolTip.text, "Rückgängig (Strg+Z)")
        fake.undoStep = "Shift"
        fake.redoStep = "AddObject"
        compare(undo.QQC2.ToolTip.text, "Rückgängig: Verschieben (Strg+Z)")
        compare(redo.QQC2.ToolTip.text, "Wiederholen: Objekt setzen (Strg+Umschalt+Z)")
        fake.undoStep = "NoSuchStep"
        compare(undo.QQC2.ToolTip.text, "Rückgängig (Strg+Z)")
        fake.undoStep = fake.redoStep = ""
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

    // The file dialog is a window of its own; once it closes, a window manager hands the keys
    // back to the main window. The test does that itself.
    function back() {
        tc.Window.window.requestActivate()
        tryCompare(tc.Window.window, "active", true)
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

    // LH-64: reading the dialogue aloud has a key and a palette entry; both need a voice.
    function test_the_palette_switches_reading_aloud_when_there_is_a_voice() {
        const header = make()
        speakSpy.target = header
        speakSpy.clear()
        const command = () => header.commandPalette.commands.find(c => c.id === "playback.speak")

        verify(!command().enabled)
        header.run("playback.speak")
        compare(speakSpy.count, 0)

        header.speechAvailable = true
        verify(command().enabled)
        compare(command().keys, "Umschalt+V")
        header.run("playback.speak")
        compare(speakSpy.count, 1)

        header.texts = en
        compare(command().keys, "Shift+V")
        compare(command().text, "Read dialogue aloud on or off")
    }

    // LH-93: the header counts the objects without a height; Shift+H, the View menu, the palette
    // and a click on the count all go to the next one.
    function test_the_missing_heights_are_counted_and_stepped_through() {
        const header = make()
        const count = findChild(header, "missingHeights")
        verify(!count.visible)
        verify(!entry(header, "viewMenu", "heights").enabled)

        fake.missingHeights = 2
        verify(count.visible)
        compare(count.text, "2 ohne Höhe · ergänzen (Umschalt+H)")
        verify(entry(header, "viewMenu", "heights").enabled)

        keyClick(Qt.Key_H, Qt.ShiftModifier)
        compare(fake.calls, [["nextMissingHeight"]])
        compare(heightSpy.count, 1)

        count.clicked()
        pick(header, "viewMenu", "heights")
        header.run("view.heights")
        compare(fake.calls.length, 4)
        compare(heightSpy.count, 4)

        const cmd = header.commandPalette.commands.find(c => c.id === "view.heights")
        compare(cmd.keys, "Umschalt+H")
        header.texts = en
        compare(count.text, "2 without height · add (Shift+H)")
    }

    // DB-63: the selection's script line, and the plan, by menu, key and palette.
    function test_the_script_and_the_plan_are_one_command_away() {
        const header = make()
        fake.lineNow = 3

        pick(header, "viewMenu", "scriptShow")
        keyClick(Qt.Key_D, Qt.ShiftModifier)
        header.run("script.show")
        compare(scriptSpy.signalArguments.map(a => a[0]), [3, 3, 3])

        fake.lineNow = -1
        pick(header, "viewMenu", "scriptShow")
        compare(scriptSpy.signalArguments[3][0], -1)

        pick(header, "viewMenu", "planShow")
        keyClick(Qt.Key_E, Qt.ShiftModifier)
        header.run("script.plan")
        compare(planSpy.count, 3)

        const command = id => header.commandPalette.commands.find(c => c.id === id)
        compare(command("script.show").keys, "Umschalt+D")
        compare(command("script.plan").keys, "Umschalt+E")
        fake.loaded = false
        verify(!entry(header, "viewMenu", "scriptShow").enabled)
        verify(!entry(header, "viewMenu", "planShow").enabled)
    }

    function test_the_view_menu_zooms() {
        const header = make()

        pick(header, "viewMenu", "zoomIn")
        pick(header, "viewMenu", "zoomOut")
        pick(header, "viewMenu", "zoomFit")

        compare(zoomSpy.signalArguments.map(a => a[0]), ["in", "out", "fit"])
    }

    function test_the_texts_are_in_both_languages() {
        for (const key of ["menu.file", "menu.export", "menu.view", "menu.viewAreas", "menu.viewMode", "view.free", "view.wallCut", "view.through", "view.throughHint", "height.next", "height.missingHeader", "menu.viewOrder",
                           "menu.ods", "menu.xlsx", "menu.frames", "export.pick.frames",
                           "menu.view3d", "menu.camera", "export.pick.view3d", "export.pick.camera",
                           "export.filter.view3d", "export.filter.camera",
                           "menu.mp4", "export.pick.mp4", "export.filter.mp4",
                           "menu.commands", "palette.placeholder", "palette.empty", "palette.name",
                           "palette.group.tools", "palette.group.modes", "palette.mode", "mode.build", "palette.group.file", "palette.group.export",
                           "palette.group.view", "palette.group.edit", "palette.group.scenes",
                           "palette.group.plans", "key.ctrl", "key.shift", "key.space", "menu.viewZoom", "menu.zoomIn",
                           "menu.zoomOut", "menu.zoomFit", "menu.play", "menu.markPrev", "menu.markNext",
                           "palette.group.playback", "script.toPlan", "script.show", "script.plan"]) {
            verify(de.t(key) !== key, "de " + key)
            verify(en.t(key) !== key, "en " + key)
        }
    }

    // LH-104, LH-105, LH-109: a key arms the next click for the selected figure or camera.
    function test_the_palette_arms_the_next_click_for_the_selection() {
        const header = make()
        const command = id => header.commandPalette.commands.find(c => c.id === id)

        verify(!command("plan.nextMark").enabled)

        fake.selectedKey = "figure:elisabeth"
        verify(command("plan.nextMark").enabled)
        verify(command("plan.markBefore").enabled)
        verify(command("plan.axisTo").enabled)
        compare(command("plan.nextMark").keys, "Umschalt+F")
        compare(command("plan.markBefore").keys, "Umschalt+Z")
        compare(command("plan.axisTo").keys, "Umschalt+A")

        header.run("plan.nextMark")
        header.run("plan.markBefore")
        header.run("plan.axisTo")
        compare(fake.calls, [["arm", "nextMark"], ["arm", "markBefore"], ["arm", "axisTo"]])

        // The axis runs between figures.
        fake.selectedKey = "camera:a"
        verify(command("plan.nextMark").enabled)
        verify(!command("plan.axisTo").enabled)
    }

    // LH-108: the palette switches the selected way.
    function test_the_palette_switches_the_way_shape() {
        const header = make()
        const command = id => header.commandPalette.commands.find(c => c.id === id)

        fake.selectedKey = "figurevia:elisabeth:1:at0"
        fake.curve = "smooth"
        verify(!command("plan.waySmooth").enabled)
        verify(command("plan.wayStraight").enabled)

        header.run("plan.wayStraight")
        compare(fake.calls, [["setWayCurve", "straight"]])
        verify(command("plan.waySmooth").enabled)
        verify(!command("plan.wayStraight").enabled)

        fake.curve = ""
        verify(!command("plan.waySmooth").enabled)
    }
}
