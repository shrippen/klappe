import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import QtTest
import QtCore
import "../../qml"

// Workspace (D-09): three steps by width, split handles that keep their widths, and in every
// step no area is cut off by the window's edge or lies on another, the tool bar included (its
// options go into a popover below the wide step). Each step also leaves a picture in the temp
// folder (TMPDIR) to look at; the test checks the geometry, not the pixels.
TestCase {
    id: tc
    name: "Workspace"
    width: 1700
    height: 900
    when: windowShown
    visible: true

    readonly property real tolerance: 0.5

    QtObject {
        id: fake
        property bool loaded: true
        property string planJson: ""
        property string selectedKey: ""
        property int selectedCount: 0
        property bool extendSelection: false
        property bool lassoSelection: false
        property bool asksNote: false
        property string noteText: ""
        property bool measuring: false
        property int scalePoints: 0
        property int areaPoints: 0
        property bool snappingOn: true
        property string choicesJson: JSON.stringify({
            figure: [{ id: "elisabeth", name: "Elisabeth" }],
            object: [{ kind: "table", label: "", width_m: 1.6, depth_m: 0.9, shape: "rect", custom: false }],
            light: ["fresnel-650"],
            color: ["red", "orange", "yellow", "green", "cyan", "blue", "purple"],
            dash: ["solid", "dashed", "dotted"],
            fill: ["outline", "filled"],
            layer: ["set", "figures", "cameras", "lights", "notes"]
        })
        property string layersJson: ""
        property string projectJson: ""
        property string sceneId: ""
        property string scriptText: ""
        property string outlineJson: "[]"
        property string locksJson: ""
        property string scriptJson: JSON.stringify([
            { kind: "scene", text: "INNEN. BURGHOF – TAG", current: false, chips: [] },
            { kind: "action", text: "Elisabeth steigt die Treppe hinab.", current: true,
              chips: [{ text: "12.1", kind: "shot", target: "s12-1", index: 0, label: "", at_s: 0 }] },
            { kind: "character", text: "ELISABETH", current: false, chips: [] },
            { kind: "dialogue", text: "Wer hat die Tür offen gelassen?", current: false, chips: [] }
        ])
        property string tracksJson: JSON.stringify([
            { kind: "cut", id: "cut", name: "Cut", label: "", shown: true, markers: [],
              segments: [{ from_s: 0, to_s: 12, text: "12.1", selected: true, key: "s12-1" },
                         { from_s: 12, to_s: 20, text: "12.2", selected: false, key: "s12-2" }] },
            { kind: "camera", id: "a", name: "A", label: "", shown: true, markers: [], segments: [] },
            { kind: "figure", id: "elisabeth", name: "Elisabeth", label: "red", shown: true, markers: [], segments: [] }
        ])
        property real time: 6
        property real duration: 20
        property var calls: []
        signal switched()

        function markInfo() { return "{}" }
        function setTool() { return true }
        function clearSelection() {}
        function setDrawStyle() { return true }
        function setSnapping() {}
        function selectShot(id) { calls.push(["shot", id]) }
        function showLink(kind, target, index) { calls.push(["link", kind, target, index]); return true }
        function seek(at) { calls.push(["seek", at]) }
        function advance() { return false }
        function stepMark() {}
        function setTrack() {}
        function setLayer() {}
        function setLock() {}
        function editShot() { return true }
        function setupChoices() { return "[]" }
        function setFollow() { return true }
        function addShot() { return true }
        function copyShot() { return true }
        function deleteShot() { return true }
    }

    readonly property var shotRow: ({
        follows: "", id: "s12-3", number: "12.3", camera: "a", camera_label: "A", size: "medium",
        angle: "eye-level", movement: "static", lens_mm: 50, own_lens_mm: null,
        description: "", props: "", gear: "", crew: "", note: "", axis_jump: false, axis_against: ""
    })

    TestTexts { id: de; language: "de" }

    // The window below the header, as Main builds it: tool bar, then the work area.
    Component {
        id: windowComponent
        ColumnLayout {
            property alias bar: bar
            property alias workspace: work
            spacing: 0

            PlanTools {
                id: bar
                Layout.fillWidth: true
                Layout.minimumWidth: 0
                compact: work.tier !== Workspace.Tier.Wide
                texts: de
                controller: fake
            }
            Workspace {
                id: work
                Layout.fillWidth: true
                Layout.fillHeight: true
                texts: de
                controller: fake
                shot: tc.shotRow
            }
        }
    }

    property var win: null

    // The settings need the names the app sets; the runner has none.
    function initTestCase() {
        Qt.application.organization = "klappe-test"
        Qt.application.domain = "klappe.test"
    }

    function make(w, h) {
        win = createTemporaryObject(windowComponent, tc, { width: w, height: h || 800 })
        verify(win)
        const work = win.workspace
        work.scriptWidth = work.paneDefault
        work.shotWidth = work.paneDefault
        work.scriptFolded = false
        work.shotFolded = false
        work.shotView = "tiles"
        work.shotOrder = "cut"
        waitForRendering(work)
        return work
    }

    function part(work, name) {
        const item = findChild(work, name)
        verify(item, name)
        return item
    }

    // The part's box in the workspace; empty if it is not shown.
    function box(work, name) {
        const item = part(work, name)
        if (!item.visible) {
            return null
        }
        const at = item.mapToItem(work, 0, 0)
        return { name: name, x: at.x, y: at.y, w: item.width, h: item.height }
    }

    function inside(work, b) {
        verify(b.x >= -tolerance && b.y >= -tolerance, b.name + " starts outside")
        verify(b.x + b.w <= work.width + tolerance, b.name + " ends right of the window: " + (b.x + b.w))
        verify(b.y + b.h <= work.height + tolerance, b.name + " ends below the window: " + (b.y + b.h))
        verify(b.w > 0 && b.h > 0, b.name + " is empty")
    }

    function apart(a, b) {
        const overlap = a.x < b.x + b.w - tolerance && b.x < a.x + a.w - tolerance
                     && a.y < b.y + b.h - tolerance && b.y < a.y + a.h - tolerance
        verify(!overlap, a.name + " lies on " + b.name)
    }

    // Every shown part is whole inside the workspace and none lies on another.
    function whole(work, names) {
        const boxes = names.map(n => box(work, n)).filter(b => b !== null)
        boxes.forEach(b => inside(work, b))
        boxes.forEach((a, i) => boxes.slice(i + 1).forEach(b => apart(a, b)))
        return boxes
    }

    // Every shown part of the tool bar ends inside the bar.
    function barFits(step) {
        const bar = win.bar
        verify(bar.visible)
        for (const item of bar.children) {
            if (!item.visible || item.width <= 0) {
                continue
            }
            verify(item.x + item.width <= bar.width + tolerance, step + ": " + item + " ends at " + (item.x + item.width) + " of " + bar.width)
        }
    }

    function shoot(work, step) {
        const dir = StandardPaths.writableLocation(StandardPaths.TempLocation).toString().replace("file://", "")
        const image = grabImage(win)
        image.save(dir + "/klappe-workspace-" + step + ".png")
    }

    function test_the_steps_follow_the_width() {
        const steps = [[1700, Workspace.Tier.Wide], [1400, Workspace.Tier.Wide],
                       [1399, Workspace.Tier.Medium], [1000, Workspace.Tier.Medium],
                       [999, Workspace.Tier.Narrow], [700, Workspace.Tier.Narrow]]
        for (const [w, tier] of steps) {
            compare(make(w).tier, tier, "width " + w)
        }
    }

    function test_the_tool_bar_fits_every_step() {
        const setups = [["build", "object"], ["draw", "rect"], ["draw", "select"], ["resolve", "light"], ["stage", "figure"], ["stage", "sketch"]]
        for (const w of [1700, 1400, 1200, 1000, 800, 720]) {
            for (const [mode, tool] of setups) {
                make(w)
                win.bar.mode = mode
                win.bar.tool = tool
                waitForRendering(win)
                wait(50)
                barFits(w + " " + mode + " " + tool)
            }
        }
    }

    function test_the_options_move_into_a_popover_below_wide() {
        const wide = make(1600)
        wide.tool = "select"
        win.bar.mode = "draw"
        win.bar.tool = "rect"
        waitForRendering(win)
        compare(part(win.bar, "optionsButton").visible, false)
        verify(part(win.bar, "colorPicker").visible)

        make(1200)
        win.bar.mode = "draw"
        win.bar.tool = "rect"
        waitForRendering(win)
        const button = part(win.bar, "optionsButton")
        compare(button.visible, true)
        const popover = part(win.bar, "optionsPopover")
        compare(popover.opened, false)

        mouseClick(button)

        tryCompare(popover, "opened", true)
        waitForRendering(win)
        const swatches = part(popover.contentItem, "colorPicker")
        verify(swatches.visible)
        const at = swatches.mapToItem(win, 0, 0)
        verify(at.x >= 0 && at.x + swatches.width <= win.width)
        verify(part(popover.contentItem, "fillBox").visible)
        verify(part(popover.contentItem, "snapButton").visible)
        shoot(win.workspace, "medium-options")
        popover.close()
    }

    // Tools the bar has no room for are in the popover, where a click picks them.
    function test_tools_without_room_are_in_the_popover() {
        make(720)
        win.bar.mode = "draw"
        win.bar.tool = "select"
        waitForRendering(win)
        wait(50)
        verify(win.bar.fitCount < win.bar.shown.length, "all " + win.bar.fitCount + " tools fit")
        const hidden = win.bar.shown[win.bar.shown.length - 1].tool.name

        mouseClick(part(win.bar, "optionsButton"))

        const popover = part(win.bar, "optionsPopover")
        tryCompare(popover, "opened", true)
        waitForRendering(win)
        const entry = part(popover.contentItem, "overflow." + hidden)
        verify(entry.visible)
        shoot(win.workspace, "narrow-options")

        // LH-102: an icon alone says its name and key.
        verify(entry.QQC2.ToolTip.text.indexOf("(") > 0, entry.QQC2.ToolTip.text)

        mouseClick(entry)

        compare(win.bar.tool, hidden)
        tryCompare(popover, "opened", false)
    }

    // The bar takes the width of the window: below wide it must not stay at its own size.
    function test_the_bar_has_the_width_of_the_window() {
        make(1200)
        win.bar.mode = "build"
        win.bar.tool = "select"
        waitForRendering(win)
        wait(50)
        compare(win.bar.width, 1200)
        compare(win.bar.fitCount, win.bar.shown.length)

        make(720)
        win.bar.mode = "draw"
        win.bar.tool = "select"
        waitForRendering(win)
        wait(50)
        compare(win.bar.width, 720)
        verify(win.bar.fitCount > 0 && win.bar.fitCount < win.bar.shown.length, "fit " + win.bar.fitCount)
    }

    function test_remembered_widths_are_cut_to_the_window() {
        const work = make(1400)
        work.scriptWidth = work.paneMax
        work.shotWidth = work.paneMax
        waitForRendering(work)
        wait(50)

        whole(work, ["scriptSlot", "scriptHandle", "plan", "shotHandle", "shotSlot", "timeline"])
        verify(part(work, "plan").width >= work.planMin - tolerance, "plan " + part(work, "plan").width)
        // What was remembered stays for a wider window.
        compare(work.scriptWidth, work.paneMax)
    }

    function test_the_view_menu_unfolds_a_folded_pane() {
        const work = make(1600)
        work.scriptFolded = true
        compare(work.scriptShown, false)

        work.setArea("script", true)

        compare(work.scriptFolded, false)
        compare(work.scriptShown, true)
    }

    // D-17: the script's empty column reaches the window through the workspace.
    function test_the_empty_script_column_asks_the_window() {
        fake.projectJson = JSON.stringify({ scenes: [{ id: "s15", number: "15", heading: "INT. HOF - TAG" }],
                                            plans: [{ id: "plan-a", name: "Plan A", shots: 0 }], plan: "plan-a" })
        fake.sceneId = "s15"
        const keptScript = fake.scriptJson
        fake.scriptJson = "[]"
        const work = make(1600)
        const imports = createTemporaryObject(spyComponent, tc, { target: work, signalName: "scriptImportRequested" })
        const opens = createTemporaryObject(spyComponent, tc, { target: work, signalName: "screenplayRequested" })

        findChild(work, "loadScreenplay").clicked()
        compare(imports.count, 1)

        fake.scriptText = "INT. HOF - TAG #1#\n"
        fake.outlineJson = JSON.stringify([{ kind: "scene", depth: 1, line: 0, number: "1", text: "INT. HOF - TAG" }])
        tryVerify(() => findChild(work, "openScreenplay").visible)
        findChild(work, "openScreenplay").clicked()
        compare(opens.count, 1)

        fake.scriptJson = keptScript
        fake.projectJson = ""
        fake.sceneId = ""
        fake.scriptText = ""
        fake.outlineJson = "[]"
    }

    Component {
        id: spyComponent
        SignalSpy {}
    }

    function test_wide_shows_everything_beside_the_plan() {
        const work = make(1600)
        const parts = ["scriptSlot", "scriptHandle", "plan", "shotHandle", "shotSlot", "timeline"]

        const boxes = whole(work, parts)

        compare(boxes.length, parts.length)
        verify(part(work, "plan").width >= work.planMin)
        barFits("wide")
        shoot(work, "wide")
    }

    function test_medium_has_drawers_for_script_and_shot() {
        const work = make(1200)

        compare(part(work, "scriptSlot").active, false)
        compare(part(work, "shotHandle").visible, false)
        whole(work, ["drawerButtons", "plan", "timeline"])
        compare(part(work, "scriptDrawer").opened, false)
        barFits("medium")
        shoot(work, "medium")

        work.setArea("script", true)
        tryCompare(part(work, "scriptDrawer"), "opened", true)
        tryVerify(() => part(work, "scriptDrawer").visible)
        compare(work.scriptShown, true)
        const drawer = part(work, "scriptDrawer").contentItem
        // The buttons' row stays free: both buttons can be clicked to close.
        const row = part(work, "drawerButtons")
        const rowEnd = row.mapToItem(work, 0, row.height).y
        verify(drawer.mapToItem(work, 0, 0).y >= rowEnd - tolerance, "drawer covers the buttons")
        verify(drawer.mapToItem(work, 0, drawer.height).y <= work.height + tolerance)
        verify(drawer.mapToItem(work, 0, 0).x >= -tolerance)
        verify(drawer.mapToItem(work, drawer.width, 0).x <= work.width / 2)
        wait(300)
        shoot(work, "medium-drawer")

        mouseClick(part(work, "scriptButton"))
        tryCompare(part(work, "scriptDrawer"), "opened", false)
        compare(work.area, "plan")
    }

    function test_medium_buttons_open_the_drawers() {
        const work = make(1200)

        mouseClick(part(work, "shotButton"))

        tryCompare(part(work, "shotDrawer"), "opened", true)
        compare(work.area, "shot")
        work.setArea("shot", false)
    }

    function test_narrow_has_tabs_for_the_rest() {
        const work = make(800)
        const tabs = part(work, "tabs")

        whole(work, ["tabs", "plan", "timeline"])
        compare(tabs.moreBefore, false)
        compare(tabs.moreAfter, false)
        barFits("narrow")
        shoot(work, "narrow")

        tabs.select(1)
        compare(work.area, "script")
        tryVerify(() => part(work, "tabScript").visible)
        waitForRendering(work)
        whole(work, ["tabs", "tabScript", "timeline"])
        compare(part(work, "plan").visible, false)
        shoot(work, "narrow-script")

        tabs.select(2)
        tryVerify(() => part(work, "tabShot").visible)
        waitForRendering(work)
        whole(work, ["tabs", "tabShot", "timeline"])
        shoot(work, "narrow-shot")

        tabs.select(0)
        compare(part(work, "plan").visible, true)
    }

    function test_narrowest_window_keeps_the_tabs_whole() {
        const work = make(Math.round(tc.tolerance + 720))

        const tabs = part(work, "tabs")

        verify(tabs.width >= tabs.implicitWidth)
        whole(work, ["tabs", "plan", "timeline"])
    }

    function test_the_view_toggles_follow_the_step() {
        const wide = make(1600)
        wide.setArea("script", false)
        compare(wide.scriptShown, false)
        compare(part(wide, "scriptSlot").visible, false)
        compare(part(wide, "scriptHandle").visible, false)

        const narrow = make(800)
        narrow.setArea("shot", true)
        compare(narrow.shotShown, true)
        compare(narrow.area, "shot")
        narrow.setArea("shot", false)
        compare(narrow.area, "plan")
    }

    function test_a_handle_sets_the_width_and_keeps_the_plan() {
        const work = make(1600)
        const handle = part(work, "scriptHandle")

        handle.setValue(500)
        compare(work.scriptWidth, 500)

        handle.setValue(99999)
        verify(work.scriptWidth <= handle.to)
        waitForRendering(work)
        verify(part(work, "plan").width >= work.planMin - tolerance)
        whole(work, ["scriptSlot", "scriptHandle", "plan", "shotHandle", "shotSlot"])
    }

    function test_the_shot_handle_is_inverted() {
        const work = make(1600)
        const handle = part(work, "shotHandle")

        compare(handle.inverted, true)
        handle.setValue(420)
        compare(work.shotWidth, 420)
    }

    function test_a_double_click_folds_and_opens_the_pane() {
        const work = make(1600)
        const handle = part(work, "shotHandle")

        mouseDoubleClickSequence(handle, handle.width / 2, handle.height / 2)

        compare(work.shotFolded, true)
        compare(part(work, "shotSlot").visible, false)
        compare(handle.visible, true)
        whole(work, ["scriptSlot", "scriptHandle", "plan", "shotHandle", "timeline"])
        shoot(work, "wide-folded")

        mouseDoubleClickSequence(handle, handle.width / 2, handle.height / 2)

        compare(work.shotFolded, false)
        compare(part(work, "shotSlot").visible, true)
    }

    function test_the_widths_stay_for_the_next_window() {
        const work = make(1600)
        work.scriptWidth = 444
        work.shotFolded = true
        // The settings store on the next turn of the event loop; a destroyed window is gone after one.
        wait(50)
        win.destroy()
        wait(50)

        const next = createTemporaryObject(windowComponent, tc, { width: 1600, height: 800 }).workspace

        compare(next.scriptWidth, 444)
        compare(next.shotFolded, true)
    }

    // D-05: the shot list's view and order are kept like the widths.
    function test_the_shot_view_and_order_stay_for_the_next_window() {
        const work = make(1600)
        const panel = part(work, "shotSlot").item
        compare(work.shotView, "tiles")

        panel.viewPicked("table")
        panel.orderPicked("shooting")
        compare(panel.view, "table")
        compare(panel.order, "shooting")
        wait(50)
        win.destroy()
        wait(50)

        const next = createTemporaryObject(windowComponent, tc, { width: 1600, height: 800 }).workspace

        compare(next.shotView, "table")
        compare(next.shotOrder, "shooting")
    }

    function test_play_and_zoom_reach_the_timeline_and_the_plan() {
        const work = make(1600)
        const line = part(work, "timeline")
        const canvas = part(work, "planCanvas")
        const before = canvas.zoom

        work.togglePlay()
        verify(line.playing)
        work.togglePlay()
        verify(!line.playing)

        work.zoomPlan("in")
        verify(canvas.zoom > before)
    }

    function test_a_script_chip_reaches_the_controller() {
        const work = make(1600)

        verify(part(work, "scriptSlot").item)
        compare(part(work, "shotSlot").item.shot.id, "s12-3")
    }

    // DB-63: a chip shows its link on the plan; the plan and the script ask each other in
    // every step of the layout.
    function test_a_chip_shows_its_link() {
        const work = make(1600)
        fake.calls = []

        part(work, "chip").clicked()

        compare(fake.calls, [["link", "shot", "s12-1", 0]])
    }

    // D-14: the left column has the tabs "Szenen" and "Text"; "Text" is the default.
    function test_the_left_column_opens_on_the_text() {
        const work = make(1600)

        compare(work.leftTab, "text")
        compare(findChild(work, "leftTabs").currentIndex, 1)
        verify(part(work, "lineList") !== null)
        verify(!findChild(work, "sceneRows").visible)
    }

    function test_the_tabs_switch_between_the_scenes_and_the_text() {
        fake.projectJson = JSON.stringify({ scenes: [{ id: "s15", number: "15", heading: "INT. HOF - TAG",
                                                       inBook: true, plans: 1, shots: 0 }] })
        const work = make(1600)
        const tabs = findChild(work, "leftTabs")

        tabs.activated(0)

        compare(work.leftTab, "scenes")
        verify(findChild(work, "sceneRows").visible)
        compare(findChild(work, "sceneRows").count, 1)
        verify(!part(work, "lineList").visible)

        tabs.activated(1)

        compare(work.leftTab, "text")
        verify(part(work, "lineList").visible)
        verify(!findChild(work, "sceneRows").visible)
        fake.projectJson = ""
    }

    function test_the_scene_list_is_in_every_step() {
        for (const width of [1600, 1200, 800]) {
            const work = make(width)
            work.leftTab = "scenes"
            work.setArea("script", true)

            tryVerify(() => findChild(work, "sceneRows") !== null && findChild(work, "sceneRows").visible,
                      5000, "scene list at " + width)
        }
    }

    function test_the_tab_stays_for_the_next_window() {
        const work = make(1600)
        work.leftTab = "scenes"
        wait(50)
        win.destroy()
        wait(50)

        const next = createTemporaryObject(windowComponent, tc, { width: 1600, height: 800 }).workspace

        compare(next.leftTab, "scenes")
        next.leftTab = "text"
    }

    // A jump to a script line shows the text, whichever tab was open.
    function test_a_jump_to_the_script_opens_the_text_tab() {
        const work = make(1600)
        work.leftTab = "scenes"

        work.revealScript(1)

        compare(work.leftTab, "text")
        tryVerify(() => part(work, "lineList").activeFocus)
    }

    function test_the_script_is_shown_in_every_step() {
        for (const width of [1600, 1200, 800]) {
            const work = make(width)

            work.revealScript(1)
            tryVerify(() => work.scriptShown)
            tryVerify(() => part(work, "lineList").activeFocus, 5000, "line list focus at " + width)
            compare(part(work, "lineList").currentIndex, 1)

            work.revealScript(-1)
            tryVerify(() => part(work, "sceneHead").activeFocus, 5000, "head focus at " + width)
        }
    }

    function test_the_plan_is_shown_from_the_script() {
        const work = make(800)

        work.area = "script"
        tryVerify(() => part(work, "tabScript").visible)
        work.revealPlan()

        compare(work.area, "plan")
        verify(part(work, "plan").visible)
        tryVerify(() => part(work, "planCanvas").activeFocus)
    }
}
