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

        compare(row(list, "a").meta, "1 3/8 T P1 S5")
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

    // DB-32: a scene of the book says its length, to the eye and to a screen reader.
    function test_a_scene_says_its_length() {
        const list = make()

        verify(row(list, "a").Accessible.description.includes("1 3/8 Seiten"))
        verify(!row(list, "b").Accessible.description.includes("Seiten"))
    }
}
