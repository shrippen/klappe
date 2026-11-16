import QtQuick
import QtTest
import "../../qml"

// AppHeader in 3D (LH-94, LH-98, LH-99): J switches to 3D, Y cuts the walls, the View menu looks
// through a camera, and "Fit" is off there. Split from tst_appheader.qml, which holds the rest.
TestCase {
    id: tc
    name: "AppHeader3D"
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
        fake.plan = "plan-a"
        fake.projectJson = JSON.stringify({ scenes: fake.scenes, plans: fake.plans, plan: fake.plan })
        fakeTools.tool = "select"
        fakeTools.mode = "build"
        fakeTools.enabled = true
    }

    function make() {
        const header = createTemporaryObject(headerComponent, tc)
        verify(header)
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

    // LH-94: J, the View menu's entry and the palette's all switch between plan and 3D.
    function test_j_switches_to_3d_and_back() {
        const header = make()
        const free = findChild(header, "viewMenu")
        verify(!entry(header, "viewMenu", "free").checked)

        keyClick(Qt.Key_J)
        compare(fake.calls, [["toggleView"]])
        verify(entry(header, "viewMenu", "free").checked)

        pick(header, "viewMenu", "free")
        compare(fake.calls, [["toggleView"], ["toggleView"]])
        verify(!entry(header, "viewMenu", "free").checked)
    }

    // LH-98: Y cuts the walls in 3D only; the View menu and the palette do the same.
    function test_y_cuts_the_walls_in_3d() {
        const header = make()
        verify(!entry(header, "viewMenu", "wallCut").enabled)
        keyClick(Qt.Key_Y)
        compare(fake.calls, [])

        keyClick(Qt.Key_J)
        verify(entry(header, "viewMenu", "wallCut").enabled)
        keyClick(Qt.Key_Y)
        compare(fake.calls, [["toggleView"], ["freeLook", "cut"]])
        verify(entry(header, "viewMenu", "wallCut").checked)

        pick(header, "viewMenu", "wallCut")
        verify(!entry(header, "viewMenu", "wallCut").checked)
        header.run("view.wallCut")
        verify(entry(header, "viewMenu", "wallCut").checked)

        const cmd = header.commandPalette.commands.find(c => c.id === "view.wallCut")
        compare(cmd.keys, "Y")
        compare(cmd.text, "Wände kappen")
        header.texts = en
        compare(header.commandPalette.commands.find(c => c.id === "view.wallCut").text, "Cut walls")
    }

    // LH-99: looking through the camera is a command of the View menu and the palette, in 3D only.
    function test_the_view_looks_through_a_camera() {
        const header = make()
        verify(!entry(header, "viewMenu", "through").enabled)

        keyClick(Qt.Key_J)
        verify(entry(header, "viewMenu", "through").enabled)
        pick(header, "viewMenu", "through")
        compare(fake.calls, [["toggleView"], ["freeLook", "camera"]])
        verify(entry(header, "viewMenu", "through").checked)

        header.run("view.through")
        verify(!entry(header, "viewMenu", "through").checked)
        compare(header.commandPalette.commands.find(c => c.id === "view.through").text, "Durch Kamera sehen")
    }

    // A plan change ends the look without a 3D command: the entry follows `pictureRevision`.
    function test_the_entry_unchecks_when_the_plan_changes() {
        const header = make()
        keyClick(Qt.Key_J)
        pick(header, "viewMenu", "through")
        verify(entry(header, "viewMenu", "through").checked)
        fake.state.look = ""
        fake.pictureRevision++
        verify(!entry(header, "viewMenu", "through").checked)
    }

    // No shot, no camera or none with marks: the look stays off, so the entry does too.
    function test_a_refused_look_leaves_the_entry_unchecked() {
        const header = make()
        keyClick(Qt.Key_J)
        fake.refuseCamera = true

        pick(header, "viewMenu", "through")
        verify(!entry(header, "viewMenu", "through").checked)

        header.run("view.through")
        verify(!entry(header, "viewMenu", "through").checked)
        fake.refuseCamera = false
    }

    function test_the_palette_lists_the_3d_view_with_its_key() {
        const header = make()

        const free = header.commandPalette.commands.find(c => c.id === "view.free")
        compare(free.text, "3D-Ansicht")
        compare(free.keys, "J")
        compare(free.group, "Ansicht")

        header.run("view.free")
        compare(fake.calls, [["toggleView"]])

        header.texts = en
        compare(header.commandPalette.commands.find(c => c.id === "view.free").text, "3D view")
    }

    // The 3D view has no plan to fit: "Fit" is off there (LH-94).
    function test_fit_is_off_in_3d() {
        const header = make()
        const fit = findChild(header, "zoomFit")
        verify(fit.enabled)

        fake.viewFree = true
        verify(!fit.enabled)
    }
}
