import QtQuick
import QtTest
import "../../qml"

// D-23: the scene's text beside its sheet. Marked words carry the category's short name; a
// choice maps the shown text back to the line's own words; the menu offers similar elements,
// then a new one per category, and marks through the controller.
TestCase {
    id: tc
    name: "SceneText"
    width: 800
    height: 600
    when: windowShown
    visible: true

    QtObject {
        id: fake
        property string sceneId: "s1"
        property var calls: []
        readonly property string fixture: JSON.stringify({
            element_categories: ["costume", "prop", "sfx"],
            elements: [
                { id: "e1", category: "prop", name: "Seite", description: "", status: "open", scenes: [] },
                { id: "e2", category: "sfx", name: "Nebel", description: "", status: "open", scenes: [] }
            ],
            sheet: {
                text: [
                    { kind: "action", runs: [
                        { text: "Ein ", element: null, category: null },
                        { text: "Nebel", element: "e2", category: "sfx" },
                        { text: "  und Seiten.", element: null, category: null }
                    ] },
                    { kind: "dialogue", runs: [{ text: "Wo <ist> das?", element: null, category: null }] }
                ]
            }
        })

        property string projectJson: fixture

        function markText(s, l, p, t, e, c) {
            calls.push(["markText", s, l, p, t, e, c])
            return true
        }
    }

    TestTexts { id: de; language: "de" }

    Component {
        id: textComponent
        SceneText { texts: de; controller: fake; width: 700; height: 500 }
    }

    function init() {
        fake.calls = []
        fake.projectJson = fake.fixture
    }

    function make() {
        const v = createTemporaryObject(textComponent, tc)
        verify(v)
        waitForRendering(v)
        return v
    }

    function test_a_marked_run_is_underlined_with_its_short_name() {
        const v = make()
        const html = v.html(v.lines[0].runs)

        verify(html.includes("text-decoration: underline"))
        verify(html.includes(" SFX"))
        verify(html.includes("white-space: pre-wrap"))
        verify(v.html(v.lines[1].runs).includes("Wo &lt;ist&gt; das?"))
        compare(findChild(v, "textLine.0").getText(0, 100), "Ein Nebel SFX  und Seiten.")
    }

    // Positions of the shown line (with " SFX") map back to the words of the line.
    function test_a_position_maps_to_the_line_without_short_names() {
        const v = make()
        const runs = v.lines[0].runs

        compare(v.sourceBefore(runs, 4), "Ein ")
        compare(v.sourceBefore(runs, 9), "Ein Nebel")
        compare(v.sourceBefore(runs, 12), "Ein Nebel")
        compare(v.sourceBefore(runs, 13), "Ein Nebel")
        compare(v.sourceBefore(runs, 16), "Ein Nebel  u")
        compare(v.sourceBefore(runs, 99), "Ein Nebel  und Seiten.")
    }

    function test_a_choice_drops_the_spaces_around_the_words() {
        const v = make()
        const runs = v.lines[0].runs

        compare(v.choice(0, runs, 18, 25), { line: 0, prefix: "Ein Nebel  und ", text: "Seiten" })
        compare(v.choice(0, runs, 13, 15), null)
    }

    // Words already marked are not offered again: a second mark there would not show.
    function test_a_choice_over_a_marked_run_is_none() {
        const v = make()
        const runs = v.lines[0].runs

        compare(v.choice(0, runs, 5, 8), null)
        compare(v.choice(0, runs, 2, 16), null)
        compare(v.choice(0, runs, 0, 3), { line: 0, prefix: "", text: "Ein" })
    }

    // A word cut by the selection is taken whole, the full stop after it is not.
    function test_a_choice_takes_whole_words() {
        const v = make()
        const runs = v.lines[0].runs

        compare(v.choice(0, runs, 21, 26), { line: 0, prefix: "Ein Nebel  und ", text: "Seiten" })
        compare(v.choice(0, runs, 16, 21), { line: 0, prefix: "Ein Nebel  ", text: "und Seiten" })
    }

    function test_the_menu_offers_similar_elements_then_new_ones() {
        const v = make()
        v.chosen = { line: 0, prefix: "Ein Nebel  und ", text: "Seiten" }
        v.entries = v.rowsFor("Seiten")

        compare(v.entries.map(e => e.element || "new:" + e.category), ["e1", "new:costume", "new:prop", "new:sfx"])
        compare(v.entries[0].text, "Seite · Requisite")
        compare(v.entries[2].text, "Neu: Requisite")

        v.mark(v.entries[2])
        compare(fake.calls, [["markText", "s1", 0, "Ein Nebel  und ", "Seiten", "", "prop"]])

        // A new element does not change the rows of the open menu.
        const project = JSON.parse(fake.projectJson)
        project.elements.push({ id: "e3", category: "prop", name: "Seiten", description: "", status: "open", scenes: [] })
        fake.projectJson = JSON.stringify(project)
        compare(v.entries.length, 4)
    }

    // A selection made with the keyboard opens the menu on Enter.
    function test_enter_on_a_selection_opens_the_menu() {
        const v = make()
        const line = findChild(v, "textLine.0")
        line.forceActiveFocus()
        line.select(19, 25)
        keyClick(Qt.Key_Return)

        const menu = findChild(v, "markMenu")
        tryVerify(() => menu.opened)
        compare(v.chosen.text, "Seiten")

        // Closing the menu leaves no selection behind to open it again.
        menu.close()
        tryVerify(() => !menu.visible)
        compare(line.selectedText, "")
    }
}
