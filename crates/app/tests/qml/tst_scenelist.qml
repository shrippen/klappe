import QtQuick
import QtTest
import "../../qml"

// SceneList (D-13): every scene of the project with its markers (text, plans, shots), the scene
// on view highlighted, a click that shows the scene, and a filter by number or heading.
TestCase {
    id: tc
    name: "SceneList"
    width: 400
    height: 600
    when: windowShown
    visible: true

    readonly property var base: [
            { id: "a", number: "1", heading: "INT. SAAL - TAG", inBook: true, plans: 1, shots: 5, length: "1 3/8" },
            { id: "b", number: "2", heading: "EXT. HOF - NACHT", inBook: true, plans: 0, shots: 0 },
            { id: "c", number: "3", heading: "INT. KELLER", inBook: false, plans: 2, shots: 7 },
            { id: "d", number: "12A", heading: "", inBook: false, plans: 0, shots: 0 }
        ]

    QtObject {
        id: fake
        property string sceneId: "b"
        property string projectJson: JSON.stringify({ scenes: tc.base })
        property var selected: []
        function selectScene(id) {
            selected.push(id)
            return true
        }
    }

    TestTexts { id: de; language: "de" }

    SignalSpy { id: picks; signalName: "picked" }

    Component {
        id: listComponent
        SceneList {
            texts: de
            controller: fake
            width: tc.width
            height: tc.height
        }
    }

    function init() {
        fake.projectJson = JSON.stringify({ scenes: base })
        fake.sceneId = "b"
        fake.selected = []
    }

    function make() {
        const list = createTemporaryObject(listComponent, tc)
        verify(list)
        return list
    }

    function row(list, id) {
        const view = findChild(list, "sceneRows")
        tryVerify(() => findChild(view, "scene." + id) !== null)
        return findChild(view, "scene." + id)
    }

    function test_all_scenes_are_listed_with_number_and_heading() {
        const list = make()

        compare(findChild(list, "sceneRows").count, 4)
        compare(row(list, "a").leadingText, "1")
        compare(row(list, "a").text, "INT. SAAL - TAG")
    }

    function test_a_scene_not_in_the_book_is_dimmed_and_says_so() {
        const list = make()

        compare(row(list, "a").opacity, 1)
        verify(row(list, "c").opacity < 1)
        compare(row(list, "c").text, "INT. KELLER")
        compare(row(list, "c").subtitle, "nicht im Buch")
        // Without a heading the note takes its place.
        compare(row(list, "d").text, "nicht im Buch")
        compare(row(list, "d").subtitle, "")
    }

    function test_the_markers_show_text_plans_and_shots() {
        const list = make()

        compare(row(list, "a").meta, "T P1 S5")
        compare(row(list, "b").meta, "T")
        compare(row(list, "c").meta, "P2 S7")
        compare(row(list, "d").meta, "")
    }

    function test_the_scene_on_view_is_highlighted() {
        const list = make()

        verify(row(list, "b").selected)
        verify(!row(list, "a").selected)

        fake.sceneId = "a"
        verify(row(list, "a").selected)
        verify(!row(list, "b").selected)
    }

    function test_a_click_shows_the_scene() {
        const list = make()
        picks.target = list
        picks.clear()

        row(list, "c").clicked()

        compare(fake.selected, ["c"])
        compare(picks.count, 1)
        compare(picks.signalArguments[0][0].id, "c")
        compare(picks.signalArguments[0][0].number, "3")
    }

    function test_the_filter_takes_a_number_or_heading_text() {
        const list = make()
        const filter = findChild(list, "sceneFilter")
        const rows = findChild(list, "sceneRows")

        filter.text = "12"
        compare(rows.count, 1)
        compare(row(list, "d").leadingText, "12A")

        filter.text = "hof"
        compare(rows.count, 1)
        compare(row(list, "b").text, "EXT. HOF - NACHT")

        filter.text = ""
        compare(rows.count, 4)
    }

    function test_a_filter_without_a_match_says_so() {
        const list = make()

        findChild(list, "sceneFilter").text = "zzz"

        compare(findChild(list, "sceneRows").count, 0)
        verify(findChild(list, "noMatch").visible)
        compare(findChild(list, "noMatch").text, "Keine Szene passt.")
    }

    function test_without_scenes_the_list_says_so() {
        fake.projectJson = JSON.stringify({ scenes: [] })
        const list = make()

        compare(findChild(list, "sceneRows").count, 0)
        compare(findChild(list, "noScenes").text, "Noch keine Szene.")
        verify(!findChild(list, "noMatch").visible)
    }

    // Arrow keys: from the filter into the list, down the rows with the view following, and
    // back up to the filter.
    function test_the_arrow_keys_walk_the_list_and_scroll() {
        const many = []
        for (let i = 1; i <= 60; i++) {
            many.push({ id: "m" + i, number: String(i), heading: "INT. ORT " + i, inBook: true, plans: 0, shots: 0 })
        }
        fake.projectJson = JSON.stringify({ scenes: many })
        const list = make()
        const filter = findChild(list, "sceneFilter")
        const rows = findChild(list, "sceneRows")
        filter.forceActiveFocus()

        keyClick(Qt.Key_Down)
        tryVerify(() => findChild(rows, "scene.m1") !== null && findChild(rows, "scene.m1").activeFocus)

        for (let i = 0; i < 40; i++) {
            keyClick(Qt.Key_Down)
        }
        const item = findChild(rows, "scene.m41")
        verify(item !== null)
        verify(item.activeFocus, "row 41 has the focus")
        verify(item.y >= rows.contentY - 1 && item.y + item.height <= rows.contentY + rows.height + 1,
               "row 41 is in view: y " + item.y + ", contentY " + rows.contentY)

        for (let i = 0; i < 40; i++) {
            keyClick(Qt.Key_Up)
        }
        tryVerify(() => findChild(rows, "scene.m1").activeFocus)
        keyClick(Qt.Key_Up)
        verify(filter.activeFocus)
    }

    // A row says all of it to a screen reader: number, heading, the note and the markers.
    function test_a_row_describes_itself_whole() {
        const list = make()

        const text = row(list, "c").Accessible.description
        verify(text.includes("nicht im Buch"), text)
        verify(text.includes("Pläne: 2"), text)
        verify(text.includes("Shots: 7"), text)
        verify(row(list, "a").Accessible.description.includes("im Buch"), "text marker")
        verify(!row(list, "a").Accessible.description.includes("nicht im Buch"))
    }

    // DB-32, DN-02: the length stands beside the markers, pale while the layout is out of date.
    function test_the_length_is_pale_when_out_of_date() {
        const list = make()
        const length = findChild(row(list, "a"), "length.a")

        compare(length.text, "1 3/8")
        compare(length.opacity, 1)
        verify(!findChild(row(list, "b"), "length.b").visible)

        list.lengthsStale = true
        verify(length.opacity < 1)
    }

    // DB-32: a scene of the book says its length, to the eye and to a screen reader.
    function test_a_scene_says_its_length() {
        const list = make()

        verify(row(list, "a").Accessible.description.includes("Länge 1 3/8"))
        verify(!row(list, "b").Accessible.description.includes("Länge"))
    }

    // DB-21: with an outline the rows are a tree: Akt I holds a synopsis, scene 1 and the
    // sequence "Ankunft" with scene 2; scene 3 stands in no section; scene 4 is not in the text.
    readonly property var treeScenes: [
        { id: "a", number: "1", heading: "INT. SAAL - TAG", inBook: true, plans: 1, shots: 5, line: 4, length: "4/8" },
        { id: "b", number: "2", heading: "EXT. HOF - NACHT", inBook: true, plans: 0, shots: 0, line: 8, length: "5/8" },
        { id: "c", number: "3", heading: "INT. KELLER", inBook: true, plans: 0, shots: 0, line: 12, length: "1" },
        { id: "d", number: "4", heading: "INT. BODEN", inBook: false, plans: 0, shots: 0, line: null }
    ]
    readonly property var treeOutline: [
        { kind: "section", level: 1, depth: 0, line: 0, end: 12, text: "Akt I", number: null, length: "1 1/8" },
        { kind: "synopsis", level: 0, depth: 1, line: 2, end: 3, text: "Es beginnt.", number: null, length: "" },
        { kind: "scene", level: 0, depth: 1, line: 4, end: 6, text: "INT. SAAL - TAG", number: null, length: "4/8" },
        { kind: "section", level: 2, depth: 1, line: 6, end: 12, text: "Ankunft", number: null, length: "5/8" },
        { kind: "scene", level: 0, depth: 2, line: 8, end: 12, text: "EXT. HOF - NACHT", number: null, length: "5/8" },
        { kind: "scene", level: 0, depth: 0, line: 12, end: 14, text: "INT. KELLER", number: null, length: "1" }
    ]

    function makeTree() {
        fake.projectJson = JSON.stringify({ scenes: treeScenes })
        const list = make()
        list.outline = treeOutline
        return list
    }

    function section(list, line) {
        const view = findChild(list, "sceneRows")
        tryVerify(() => findChild(view, "section." + line) !== null)
        return findChild(view, "section." + line)
    }

    function test_an_outline_makes_a_tree_with_totals_and_synopses() {
        const list = makeTree()
        const rows = findChild(list, "sceneRows")

        compare(rows.count, 6)
        compare(section(list, 0).text, "Akt I")
        compare(section(list, 0).subtitle, "Es beginnt.")
        compare(findChild(section(list, 0), "length.section.0").text, "1 1/8")
        compare(section(list, 6).text, "Ankunft")
        compare(row(list, "a").text, "INT. SAAL - TAG")
        // The scene without text stays at the bottom, dimmed.
        compare(rows.model[5].scene.id, "d")
        verify(row(list, "d").opacity < 1)
        verify(section(list, 0).Accessible.description.includes("Abschnitt, aufgeklappt"))
    }

    function test_the_arrow_folds_and_unfolds_a_section() {
        const list = makeTree()
        const rows = findChild(list, "sceneRows")
        const hover = findChild(section(list, 6), "fold.6")

        mouseMove(hover, hover.width / 2, hover.height / 2)
        tryVerify(() => findChild(hover, "overArrow.6").hovered)
        section(list, 6).clicked()

        compare(rows.count, 5)
        tryCompare(findChild(rows, "section.6"), "objectName", "section.6")
        verify(section(list, 6).Accessible.description.includes("zugeklappt"))

        const again = findChild(section(list, 6), "fold.6")
        mouseMove(again, again.width / 2, again.height / 2)
        tryVerify(() => findChild(again, "overArrow.6").hovered)
        section(list, 6).clicked()
        compare(rows.count, 6)
    }

    function test_right_and_left_fold_a_section_and_left_goes_up() {
        const list = makeTree()
        const rows = findChild(list, "sceneRows")

        section(list, 0).forceActiveFocus()
        keyClick(Qt.Key_Left)
        compare(rows.count, 3)
        tryVerify(() => section(list, 0).activeFocus)

        keyClick(Qt.Key_Right)
        compare(rows.count, 6)
        tryVerify(() => section(list, 0).activeFocus)

        row(list, "b").forceActiveFocus()
        keyClick(Qt.Key_Left)
        tryVerify(() => section(list, 6).activeFocus)
    }

    function test_what_is_folded_is_remembered_by_title() {
        const list = makeTree()
        const rows = findChild(list, "sceneRows")

        section(list, 6).forceActiveFocus()
        keyClick(Qt.Key_Left)
        compare(rows.count, 5)

        // The outline moves down two lines: the section stays folded.
        const moved = treeOutline.map(e => Object.assign({}, e, { line: e.line + 2, end: e.end + 2 }))
        list.outline = moved
        compare(rows.count, 5)
        verify(section(list, 8).Accessible.description.includes("zugeklappt"))

        // Renamed, it is open again.
        list.outline = moved.map(e => e.text === "Ankunft" ? Object.assign({}, e, { text: "Ankommen" }) : e)
        compare(rows.count, 6)
    }

    function test_alt_up_and_down_ask_to_move_the_block() {
        const list = makeTree()
        const asked = createTemporaryObject(spyComponent, tc, { target: list, signalName: "moveRequested" })

        row(list, "a").forceActiveFocus()
        keyClick(Qt.Key_Down, Qt.AltModifier)
        compare(asked.count, 1)
        compare(asked.signalArguments[0][0], 4)
        compare(asked.signalArguments[0][1], "down")

        section(list, 6).forceActiveFocus()
        keyClick(Qt.Key_Up, Qt.AltModifier)
        compare(asked.signalArguments[1][0], 6)
        compare(asked.signalArguments[1][1], "up")

        // Without Alt the focus steps and nothing moves; a scene outside the text never moves.
        keyClick(Qt.Key_Down)
        compare(asked.count, 2)
        row(list, "d").forceActiveFocus()
        keyClick(Qt.Key_Up, Qt.AltModifier)
        compare(asked.count, 2)
    }

    function test_a_click_on_a_section_shows_its_heading() {
        const list = makeTree()
        picks.target = list
        picks.clear()

        section(list, 6).clicked()

        compare(picks.count, 1)
        compare(picks.signalArguments[0][0].line, 6)
        compare(fake.selected, [])
    }

    function test_a_filter_shows_the_flat_list() {
        const list = makeTree()
        const rows = findChild(list, "sceneRows")

        findChild(list, "sceneFilter").text = "hof"

        compare(rows.count, 1)
        compare(row(list, "b").text, "EXT. HOF - NACHT")
    }

    function test_focus_at_puts_the_focus_on_a_heading() {
        const list = makeTree()

        list.focusAt(8)

        tryVerify(() => row(list, "b").activeFocus)
    }

    Component {
        id: spyComponent
        SignalSpy {}
    }
}
