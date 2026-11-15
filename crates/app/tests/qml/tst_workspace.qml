import QtQuick
import QtQuick.Layouts
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
        property bool asksNote: false
        property string noteText: ""
        property bool measuring: false
        property int scalePoints: 0
        property int areaPoints: 0
        property bool snappingOn: true
        property string choicesJson: JSON.stringify({
            figure: [{ id: "elisabeth", name: "Elisabeth" }],
            object: [{ kind: "table", label: "", width_m: 1.6, depth_m: 0.9, shape: "rect", custom: false }],
            light: ["fresnel"],
            color: ["red", "orange", "yellow", "green", "cyan", "blue", "purple"],
            dash: ["solid", "dashed", "dotted"],
            fill: ["outline", "filled"],
            layer: ["set", "figures", "cameras", "lights", "notes"]
        })
        property string layersJson: ""
        property string locksJson: ""
        property string scriptJson: JSON.stringify([
            { kind: "scene", text: "INNEN. BURGHOF – TAG", current: false, chips: [] },
            { kind: "action", text: "Elisabeth steigt die Treppe hinab.", current: true,
              chips: [{ text: "12.1", kind: "shot", target: "s12-1", label: "", at_s: 0 }] },
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
        function setDrawStyle() { return true }
        function setSnapping() {}
        function selectShot(id) { calls.push(["shot", id]) }
        function seek(at) { calls.push(["seek", at]) }
        function advance() { return false }
        function stepMark() {}
        function setTrack() {}
        function setLayer() {}
        function setLock() {}
        function editShot() { return true }
        function setFollow() { return true }
        function addShot() { return true }
        function copyShot() { return true }
        function deleteShot() { return true }
    }

    readonly property var shotRow: ({
        follows: "", id: "s12-3", number: "12.3", camera: "a", camera_label: "A", size: "medium",
        angle: "eye-level", movement: "static", lens_mm: 50, own_lens_mm: null,
        description: "", props: "", gear: "", crew: "", note: "", axis_jump: false
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
        const setups = [["build", "object"], ["draw", "rect"], ["draw", "select"], ["resolve", "light"], ["stage", "figure"]]
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

    function test_a_script_chip_reaches_the_controller() {
        const work = make(1600)

        verify(part(work, "scriptSlot").item)
        compare(part(work, "shotSlot").item.shot.id, "s12-3")
    }
}
