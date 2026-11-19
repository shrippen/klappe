import QtQuick
import QtQuick.Controls as QQC2
import QtTest
import "../../qml"

// D-11, D-12: the module bar lists the modules in order, marks the open one, asks for another on
// a click; the header binds Ctrl+1 to Ctrl+N and lists "Go to" in the palette, and its path shows
// what the module has open. Ctrl+E is no key of the app any more.
TestCase {
    id: tc
    name: "ModuleBar"
    width: 1000
    height: 600
    when: windowShown
    visible: true

    FakeHeaderController { id: fake }
    TestTexts { id: de; language: "de" }
    TestTexts { id: en; language: "en" }

    readonly property var modules: [
        { id: "screenplay", name: "module.screenplay", icon: "document-edit" },
        { id: "stellprobe", name: "module.stellprobe", icon: "view-grid" }
    ]

    Component {
        id: barComponent
        ModuleBar { texts: de; modules: tc.modules; module: "stellprobe"; height: 300; keyName: s => s.replace("Ctrl", "Strg") }
    }
    Component {
        id: headerComponent
        AppHeader { controller: fake; texts: de; modules: tc.modules; width: tc.width }
    }

    SignalSpy { id: pickSpy; signalName: "picked" }
    SignalSpy { id: askSpy; signalName: "moduleRequested" }

    function init() {
        fake.loaded = true
        fake.calls = []
    }

    function makeBar(module) {
        const bar = createTemporaryObject(barComponent, tc, { module: module })
        verify(bar)
        pickSpy.target = bar
        pickSpy.clear()
        return bar
    }

    function makeHeader(module) {
        const header = createTemporaryObject(headerComponent, tc, { module: module })
        verify(header)
        askSpy.target = header
        askSpy.clear()
        return header
    }

    function test_the_bar_lists_the_modules_in_order_with_names() {
        const bar = makeBar("stellprobe")
        const first = findChild(bar, "module.screenplay")
        const second = findChild(bar, "module.stellprobe")

        compare(first.text, "Drehbuch")
        compare(second.text, "Stellprobe")
        compare(first.Accessible.name, "Drehbuch")
        verify(first.y < second.y)
        compare(first.QQC2.ToolTip.text, "Drehbuch (Strg+1)")
        compare(second.QQC2.ToolTip.text, "Stellprobe (Strg+2)")
    }

    function test_the_open_module_is_checked_and_has_an_edge() {
        const bar = makeBar("stellprobe")

        verify(findChild(bar, "module.stellprobe").checked)
        verify(!findChild(bar, "module.screenplay").checked)
        verify(findChild(findChild(bar, "module.stellprobe"), "edge").visible)
        verify(!findChild(findChild(bar, "module.screenplay"), "edge").visible)

        bar.module = "screenplay"

        verify(findChild(bar, "module.screenplay").checked)
        verify(!findChild(bar, "module.stellprobe").checked)
    }

    function test_a_click_asks_and_leaves_the_state_to_the_window() {
        const bar = makeBar("stellprobe")

        findChild(bar, "module.screenplay").clicked()
        compare(pickSpy.count, 1)
        compare(pickSpy.signalArguments[0][0], "screenplay")

        // The open one stays checked when it is clicked again.
        findChild(bar, "module.stellprobe").toggle()
        findChild(bar, "module.stellprobe").clicked()
        verify(findChild(bar, "module.stellprobe").checked)
    }

    function test_ctrl_1_and_ctrl_2_ask_for_the_modules() {
        const header = makeHeader("stellprobe")

        keyClick(Qt.Key_1, Qt.ControlModifier)
        keyClick(Qt.Key_2, Qt.ControlModifier)

        compare(askSpy.count, 2)
        compare(askSpy.signalArguments[0][0], "screenplay")
        compare(askSpy.signalArguments[1][0], "stellprobe")
    }

    function test_the_palette_goes_to_the_modules() {
        const header = makeHeader("stellprobe")
        const first = header.commands.find(c => c.id === "module.screenplay")
        const second = header.commands.find(c => c.id === "module.stellprobe")

        compare(first.text, "Gehe zu: Drehbuch")
        compare(first.keys, "Strg+1")
        compare(second.text, "Gehe zu: Stellprobe")
        compare(second.keys, "Strg+2")

        header.run("module.screenplay")
        compare(askSpy.count, 1)
        compare(askSpy.signalArguments[0][0], "screenplay")
    }

    function test_ctrl_e_is_no_key_of_the_app() {
        const header = makeHeader("stellprobe")

        verify(!header.commands.find(c => c.id === "view.screenplay"))
        verify(!header.commands.find(c => c.keys === "Strg+E"))
    }

    function test_the_path_shows_what_the_module_has_open() {
        const header = makeHeader("stellprobe")
        const path = findChild(header, "path")

        compare(path.text, "Wartburg › Szene 12 · INT. SÄNGERSAAL – TAG ›")
        verify(findChild(header, "planMenu").visible)

        header.module = "screenplay"

        compare(path.text, "Wartburg › Szene 12 · INT. SÄNGERSAAL – TAG")
        verify(!findChild(header, "planMenu").visible)
    }

    // The bar is as wide as its icon buttons, also where the theme lacks the icons.
    function test_the_width_does_not_depend_on_the_icons() {
        const named = createTemporaryObject(barComponent, tc, { module: "stellprobe",
            modules: [{ id: "screenplay", name: "module.screenplay", icon: "no-such-icon-xyz" },
                      { id: "stellprobe", name: "module.stellprobe", icon: "no-such-icon-abc" }] })
        const plain = makeBar("stellprobe")

        compare(findChild(named, "module.screenplay").width, findChild(plain, "module.screenplay").width)
        compare(findChild(named, "module.stellprobe").width, findChild(plain, "module.stellprobe").width)
        compare(named.width, plain.width)
        verify(named.width < 100, "bar is " + named.width)
    }
}
