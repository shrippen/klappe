import QtQuick
import QtTest
import "../../qml"

// The choices under the line being typed in the screenplay editor (DB-03, D-27): the list opens
// under the line, Up and Down mark, Enter and Tab take one and a choice that ends the element goes
// on to the next, Esc closes the list. The rules are `script/`'s (`assist.rs`); the controller here
// answers what the test sets.
TestCase {
    id: tc
    name: "ScriptSuggest"
    width: 1000
    height: 600
    when: windowShown
    visible: true

    QtObject {
        id: fake
        property string scriptText: ""
        property string sceneId: ""
        property string projectJson: JSON.stringify({ scenes: [] })
        property int enters: 0
        function selectScene(id) { return true }
        function editScript(text) { scriptText = text }
        function paginateScript() {}
        function scriptBreaksJson() { return JSON.stringify({ pages: 0, breaks: [], scenes: [] }) }
        function scriptKinds(text) {
            return text.split("\n").map(l => l.startsWith("INT.") ? "h" : "a").join("")
        }
        function scriptOutlineJson() { return "[]" }
        function versionsJson() { return JSON.stringify({ versions: [], next: null }) }
        function revisionJson() { return "[]" }
        function scriptPrintJson() { return JSON.stringify({ fontPt: 12, ascentPt: 10, pages: [] }) }
        function enterStep(line, col, kind, below) {
            enters += 1
            return JSON.stringify({ step: "break" })
        }
        function tabStep(line, kind) { return "" }
        function typedCaps(text, at, pending) { return "" }
        function scriptSpeakers(text) { return "[]" }
        function sameFigureName(a, b) { return a === b }
        function scriptLooks() {
            return JSON.stringify({ columns: 60, looks: {
                "h": { kind: "heading", x: 0, width: 60, align: "left" },
                "a": { kind: "action", x: 0, width: 60, align: "left" }
            }, letters: { blank: " ", heading: "h", action: "a" } })
        }
        // The choices to give by the text of a line.
        property var suggestAnswers: ({})
        function scriptSuggest(text, line, kind) {
            const row = text.split("\n")[line]
            return suggestAnswers[row] ? JSON.stringify(suggestAnswers[row]) : ""
        }
    }

    TestTexts { id: de; language: "de" }

    Component {
        id: viewComponent
        ScreenplayView {
            texts: de
            controller: fake
            width: tc.width
            height: tc.height
            debounceMs: 50
            toolsSource: Qt.resolvedUrl("StubTools.qml")
        }
    }

    function init() {
        fake.scriptText = ""
        fake.suggestAnswers = ({})
        fake.enters = 0
    }

    function make() {
        const view = createTemporaryObject(viewComponent, tc)
        verify(view)
        return view
    }

    // DB-03: the choices stand under the line, Down marks the next, Enter takes it and, being a
    // time of day, goes on to the action; the text below keeps its place.
    function test_the_choices_open_under_the_line_and_enter_takes_one() {
        fake.scriptText = "INT. SAAL - T"
        fake.suggestAnswers = { "INT. SAAL - T": { from: 12, options: ["TAG", "TEXT"], finishes: true, kind: "a", gap: "blank" } }
        const view = make()
        const popup = findChild(view.area, "scriptSuggest")
        view.area.forceActiveFocus()
        view.area.cursorPosition = view.area.length
        view.flow.updateSuggest()

        tryVerify(() => popup.visible)
        compare(popup.options, ["TAG", "TEXT"])
        compare(popup.contentItem.Accessible.name, "Vorschläge")
        compare(findChild(popup.contentItem, "suggestion.0").selected, true)
        verify(popup.y >= view.area.positionToRectangle(view.area.cursorPosition).y)

        keyClick(Qt.Key_Down)
        compare(view.flow.suggestSel, 1)
        keyClick(Qt.Key_Up)
        compare(view.flow.suggestSel, 0)
        keyClick(Qt.Key_Down)
        keyClick(Qt.Key_Down)
        compare(view.flow.suggestSel, 1)

        keyClick(Qt.Key_Return)
        compare(view.area.text, "INT. SAAL - TEXT\n\n")
        compare(view.area.cursorPosition, view.area.length)
        compare(view.flow.pendingLine, 2)
        compare(view.flow.suggestions, null)
        tryVerify(() => !popup.visible)
        // Enter took the choice: it was no plain break.
        compare(fake.enters, 0)
    }

    function test_escape_closes_the_choices_for_this_text() {
        fake.scriptText = "INT. SAAL - T"
        fake.suggestAnswers = { "INT. SAAL - T": { from: 12, options: ["TAG"], finishes: true, kind: "a", gap: "blank" } }
        const view = make()
        view.area.forceActiveFocus()
        view.area.cursorPosition = view.area.length
        view.flow.updateSuggest()
        verify(view.flow.suggestions !== null)

        keyClick(Qt.Key_Escape)

        compare(view.flow.suggestions, null)
        view.flow.updateSuggest()
        compare(view.flow.suggestions, null)
        // Another character brings them back.
        fake.suggestAnswers = { "INT. SAAL - TA": { from: 12, options: ["TAG"], finishes: true, kind: "a", gap: "blank" } }
        view.area.insert(view.area.length, "A")
        view.area.cursorPosition = view.area.length
        view.flow.updateSuggest()
        verify(view.flow.suggestions !== null)
    }

    // A click takes the choice, as Enter does, and the typing stays in the field without a
    // warning from the click handler.
    function test_a_click_takes_a_choice() {
        fake.scriptText = "INT. SAAL - T"
        fake.suggestAnswers = { "INT. SAAL - T": { from: 12, options: ["TAG", "TEXT"], finishes: false, kind: "a", gap: "none" } }
        const view = make()
        const popup = findChild(view.area, "scriptSuggest")
        view.area.forceActiveFocus()
        view.area.cursorPosition = view.area.length
        view.flow.updateSuggest()
        tryVerify(() => popup.visible)
        failOnWarning(/ScriptSuggest.qml.*not defined/)

        mouseClick(findChild(popup.contentItem, "suggestion.1"))

        compare(view.area.text, "INT. SAAL - TEXT")
        compare(view.flow.suggestions, null)
        verify(view.area.activeFocus)
    }

    // A choice that ends the element does not open the next one while text stands below.
    function test_a_choice_with_text_below_only_completes() {
        fake.scriptText = "ANNA\nHallo."
        fake.suggestAnswers = { "ANN": { from: 0, options: ["ANNA"], finishes: true, kind: "d", gap: "none" } }
        const view = make()
        view.area.text = "ANN\nHallo."
        view.area.forceActiveFocus()
        view.area.cursorPosition = 3
        view.flow.updateSuggest()

        keyClick(Qt.Key_Tab)

        compare(view.area.text, "ANNA\nHallo.")
        compare(view.area.cursorPosition, 4)
    }
}
