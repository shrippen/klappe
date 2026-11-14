import QtQuick
import QtTest
import "../../qml"

// StartPage (LH-01, D-07): the two ways in and the recent projects as tiles.
TestCase {
    id: tc
    name: "StartPage"
    width: 900
    height: 600
    when: windowShown
    visible: true

    QtObject {
        id: fake
        property string recentJson: "[]"
    }

    TestTexts { id: de; language: "de" }
    TestTexts { id: en; language: "en" }

    Component {
        id: pageComponent
        StartPage {
            anchors.fill: parent
            controller: fake
            texts: de
        }
    }

    function make() {
        const page = createTemporaryObject(pageComponent, tc)
        verify(page)
        return page
    }

    function test_without_recent_projects_it_says_so() {
        fake.recentJson = "[]"
        const page = make()

        verify(findChild(page, "noRecent").visible)
        compare(findChild(page, "noRecent").text, "Noch keine Projekte geöffnet.")
    }

    function test_both_ways_in_are_signalled() {
        const page = make()
        const asked = []
        page.newRequested.connect(() => asked.push("new"))
        page.openRequested.connect(() => asked.push("open"))

        findChild(page, "newButton").clicked()
        findChild(page, "openButton").clicked()

        compare(asked, ["new", "open"])
        compare(findChild(page, "newButton").text, "Neues Projekt…")
    }

    readonly property var projects: [
        { path: "/film/wartburg.klappe", name: "Wartburg", state: "ready", scenes: 3, modifiedS: 1700000000, preview: "" },
        { path: "/film/zwei.klappe", name: "Zwei", state: "ready", scenes: 1, modifiedS: 0, preview: "" },
        { path: "/film/weg.klappe", name: "Weg", state: "missing" }
    ]

    function tiles(page) {
        const found = []
        const grid = findChild(page, "tiles")
        waitForRendering(grid)
        tryVerify(() => grid.count === JSON.parse(fake.recentJson).length)
        for (let i = 0; i < grid.count; i++) {
            grid.currentIndex = i
            found.push(grid.currentItem)
        }
        grid.currentIndex = 0
        return found
    }

    function test_the_projects_are_tiles_with_name_scenes_and_date() {
        fake.recentJson = JSON.stringify(projects)
        const page = make()
        const cells = tiles(page)

        verify(!findChild(page, "noRecent").visible)
        compare(cells.length, 3)
        compare(cells[0].name, "Wartburg")
        compare(cells[0].state, "ready")
        const subs = []
        for (const cell of cells) {
            subs.push(findChild(cell, "recentSub").text)
        }
        verify(subs[0].startsWith("3 Szenen · "), subs[0])
        compare(subs[1], "1 Szene")
        compare(subs[2], "Ordner fehlt")
    }

    function test_enter_opens_the_tile_with_the_focus_and_arrows_move() {
        fake.recentJson = JSON.stringify(projects)
        const page = make()
        tiles(page)
        let opened = ""
        page.recentRequested.connect(path => { opened = path })

        tryVerify(() => page.Window.activeFocusItem !== null && page.Window.activeFocusItem.objectName === "recent")
        keyClick(Qt.Key_Right)
        keyClick(Qt.Key_Return)

        compare(opened, "/film/zwei.klappe")
    }

    function test_a_missing_folder_is_dimmed_cannot_open_and_can_be_removed() {
        fake.recentJson = JSON.stringify(projects)
        const page = make()
        const grid = findChild(page, "tiles")
        const cells = tiles(page)
        const gone = cells[2]
        const tile = findChild(gone, "recent")
        let opened = ""
        let forgotten = ""
        page.recentRequested.connect(path => { opened = path })
        page.forgetRequested.connect(path => { forgotten = path })

        verify(tile.stale, "dimmed by the tile itself")
        tile.clicked()
        compare(opened, "")
        const button = findChild(gone, "forgetButton")
        verify(button.visible)
        button.clicked()
        compare(forgotten, "/film/weg.klappe")

        forgotten = ""
        grid.currentIndex = 2
        keyClick(Qt.Key_Delete)
        compare(forgotten, "/film/weg.klappe")
    }

    function test_a_finished_preview_updates_the_tile_in_place() {
        fake.recentJson = JSON.stringify(projects)
        const page = make()
        const before = tiles(page)[0]
        const loading = JSON.parse(fake.recentJson)
        loading[0].preview = "/cache/a.png"

        fake.recentJson = JSON.stringify(loading)

        tryVerify(() => before.preview === "/cache/a.png")
        compare(findChild(page, "tiles").currentItem, before)
    }

    function test_the_english_text() {
        const page = make()
        page.texts = en

        compare(findChild(page, "openButton").text, "Open project…")
    }
}
