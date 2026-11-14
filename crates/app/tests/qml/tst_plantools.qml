import QtQuick
import QtTest
import "../../qml"

// PlanTools: keys pick the tool, the list shows only for the tool that needs it, and the
// controller is told tool and choice.
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
        property string choicesJson: JSON.stringify({
            figure: [{ id: "elisabeth", name: "Elisabeth" }, { id: "burgvogt", name: "Burgvogt" }],
            object: ["table", "chair"],
            light: ["fresnel", "led-panel"],
            color: ["red", "orange", "yellow", "green", "cyan", "blue", "purple"],
            dash: ["solid", "dashed", "dotted"],
            fill: ["outline", "filled"]
        })
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
        id: toolsComponent
        PlanTools {
            controller: fake
            texts: de
        }
    }

    function init() {
        fake.tools = []
        fake.snaps = []
        fake.styles = []
    }

    function make() {
        const bar = createTemporaryObject(toolsComponent, tc)
        verify(bar)
        waitForRendering(bar)
        return bar
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
                      [Qt.Key_C, "camera"], [Qt.Key_L, "light"], [Qt.Key_M, "scale"],
                      [Qt.Key_N, "note"], [Qt.Key_I, "line"], [Qt.Key_A, "arrow"],
                      [Qt.Key_B, "rect"], [Qt.Key_E, "ellipse"], [Qt.Key_P, "area"],
                      [Qt.Key_V, "select"]]

        for (const [key, tool] of keys) {
            keyClick(key)
            compare(bar.tool, tool)
        }
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
