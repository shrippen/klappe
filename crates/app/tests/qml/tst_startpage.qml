import QtQuick
import QtQuick.Controls as QQC2
import QtTest
import "../../qml"
import Kante

// StartPage (LH-01, D-07): the two ways in, the recent projects as tiles between the blank plan and the example.
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
        property string status: ""
        property var calls: []
        property bool accept: true

        function clearStatus() { status = "" }
        // A refusal says why, as the controller does; a success takes the message away.
        function answer() { status = accept ? "" : "Fehler"; return accept }
        function renameRecent(path, title) { calls.push(["rename", path, title]); return answer() }
        function archiveRecent(path) { calls.push(["archive", path]); return answer() }
        function trashRecent(path) { calls.push(["trash", path]); return answer() }
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

    function make(properties) {
        const page = createTemporaryObject(pageComponent, tc, properties || {})
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
        { path: "/home/anna/film/wartburg.klappe", shortPath: "~/film/wartburg.klappe", name: "Wartburg", state: "ready", scenes: 3, modifiedS: 1700000000, preview: "" },
        { path: "/film/zwei.klappe", shortPath: "/film/zwei.klappe", name: "Zwei", state: "ready", scenes: 1, modifiedS: 0, preview: "" },
        { path: "/film/weg.klappe", shortPath: "/film/weg.klappe", name: "Weg", state: "missing" }
    ]

    // All tiles: the blank tile, the projects, the example tile after them.
    function allTiles(page) {
        const found = []
        const grid = findChild(page, "tiles")
        waitForRendering(grid)
        tryVerify(() => grid.count === JSON.parse(fake.recentJson).length + 2)
        for (let i = 0; i < grid.count; i++) {
            grid.currentIndex = i
            found.push(grid.currentItem)
        }
        grid.currentIndex = 0
        return found
    }

    // The projects and the example tile, without the blank one.
    function tiles(page) {
        return allTiles(page).slice(1)
    }

    function test_the_projects_are_tiles_with_name_scenes_and_date() {
        fake.recentJson = JSON.stringify(projects)
        const page = make()
        const cells = tiles(page)

        verify(!findChild(page, "noRecent").visible)
        compare(cells.length, 4)
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

    function test_the_example_tile_is_last_and_there_without_projects() {
        fake.recentJson = JSON.stringify(projects)
        const page = make()
        const cells = tiles(page)
        const example = findChild(cells[3], "example")

        verify(example)
        compare(example.name, "Beispiel ansehen")
        compare(findChild(cells[0], "recent").name, "Wartburg")

        fake.recentJson = "[]"
        const alone = tiles(make())
        compare(alone.length, 1)
        compare(findChild(alone[0], "example").name, "Beispiel ansehen")
    }

    function test_enter_on_the_example_tile_asks_for_the_example() {
        fake.recentJson = "[]"
        const page = make()
        tiles(page)
        let asked = 0
        let opened = ""
        page.exampleRequested.connect(() => { asked++ })
        page.recentRequested.connect(path => { opened = path })

        tryVerify(() => page.Window.activeFocusItem !== null && page.Window.activeFocusItem.objectName === "blank")
        keyClick(Qt.Key_Right)
        tryVerify(() => page.Window.activeFocusItem.objectName === "example")
        keyClick(Qt.Key_Return)

        compare(asked, 1)
        compare(opened, "")
    }

    function test_the_english_example_text() {
        fake.recentJson = "[]"
        const page = make()
        page.texts = en
        tiles(page)

        compare(findChild(page, "example").name, "Open the example")
    }

    function test_the_first_tile_has_the_focus_when_the_page_shows_later() {
        fake.recentJson = JSON.stringify(projects)
        const page = make({ visible: false })
        const grid = findChild(page, "tiles")
        tryVerify(() => grid.count === 5)

        page.visible = true

        tryVerify(() => grid.itemAtIndex(0) !== null)
        const first = findChild(grid.itemAtIndex(0), "blank")
        tryVerify(() => first.activeFocus)
        let opened = ""
        page.recentRequested.connect(path => { opened = path })
        keyClick(Qt.Key_Right)
        keyClick(Qt.Key_Right)
        keyClick(Qt.Key_Return)
        compare(opened, "/film/zwei.klappe")
    }

    function test_a_project_tile_shows_its_path_with_a_tilde_as_tooltip() {
        fake.recentJson = JSON.stringify(projects)
        const page = make()
        const cells = tiles(page)
        const first = findChild(cells[0], "recent")
        const example = findChild(cells[3], "example")

        compare(first.QQC2.ToolTip.text, "~/film/wartburg.klappe")
        compare(findChild(cells[1], "recent").QQC2.ToolTip.text, "/film/zwei.klappe")
        verify(!first.QQC2.ToolTip.visible)

        mouseMove(first, first.width / 2, first.height / 2)
        tryVerify(() => first.QQC2.ToolTip.visible)
        mouseMove(example, example.width / 2, example.height / 2)
        tryVerify(() => !first.QQC2.ToolTip.visible)
        verify(!example.QQC2.ToolTip.visible)
    }

    function test_enter_opens_the_tile_with_the_focus_and_arrows_move() {
        fake.recentJson = JSON.stringify(projects)
        const page = make()
        tiles(page)
        let opened = ""
        page.recentRequested.connect(path => { opened = path })

        tryVerify(() => page.Window.activeFocusItem !== null && page.Window.activeFocusItem.objectName === "blank")
        keyClick(Qt.Key_Right)
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
        grid.currentIndex = 3
        keyClick(Qt.Key_Delete)
        compare(forgotten, "/film/weg.klappe")
    }

    function test_a_finished_preview_updates_the_tile_in_place() {
        fake.recentJson = JSON.stringify(projects)
        const page = make()
        const before = tiles(page)[0]
        const current = findChild(page, "tiles").currentItem
        const loading = JSON.parse(fake.recentJson)
        loading[0].preview = "file:///cache/a.png"

        fake.recentJson = JSON.stringify(loading)

        tryVerify(() => before.preview === "file:///cache/a.png")
        compare(findChild(page, "tiles").currentItem, current)
    }

    function test_the_english_text() {
        const page = make()
        page.texts = en

        compare(findChild(page, "openButton").text, "Open project…")
    }

    // The tile menu (right click, menu key, Shift+F10): rename, remove from the list, archive, trash.
    function openMenuOn(page, index) {
        const cells = tiles(page)
        const tile = findChild(cells[index], "recent")
        mouseClick(tile, tile.width / 2, tile.height / 2, Qt.RightButton)
        const menu = findChild(page, "tileMenu")
        tryVerify(() => menu.visible)
        return menu
    }

    function test_a_right_click_opens_the_menu_with_its_four_entries() {
        fake.recentJson = JSON.stringify(projects)
        const page = make()
        const menu = openMenuOn(page, 0)

        compare(findChild(menu, "renameItem").text, "Umbenennen…")
        compare(findChild(menu, "forgetItem").text, "Aus der Liste entfernen")
        compare(findChild(menu, "archiveItem").text, "Archivieren…")
        compare(findChild(menu, "trashItem").text, "In den Papierkorb verschieben…")
        verify(findChild(menu, "renameItem").enabled)
        verify(findChild(menu, "archiveItem").enabled)
    }

    function test_the_menu_texts_are_english_too() {
        fake.recentJson = JSON.stringify(projects)
        const page = make()
        page.texts = en
        const menu = openMenuOn(page, 0)

        compare(findChild(menu, "renameItem").text, "Rename…")
        compare(findChild(menu, "trashItem").text, "Move to trash…")
    }

    function test_the_menu_key_and_shift_f10_open_the_menu_of_the_focused_tile() {
        fake.recentJson = JSON.stringify(projects)
        const page = make()
        tiles(page)
        const menu = findChild(page, "tileMenu")
        tryVerify(() => page.Window.activeFocusItem !== null && page.Window.activeFocusItem.objectName === "blank")

        keyClick(Qt.Key_Right)
        keyClick(Qt.Key_Menu)
        tryVerify(() => menu.visible)
        compare(menu.target.path, "/home/anna/film/wartburg.klappe")
        keyClick(Qt.Key_Escape)
        tryVerify(() => !menu.visible)

        keyClick(Qt.Key_Right)
        keyClick(Qt.Key_F10, Qt.ShiftModifier)
        tryVerify(() => menu.visible)
        compare(menu.target.path, "/film/zwei.klappe")
    }

    function test_the_example_tile_has_no_menu() {
        fake.recentJson = "[]"
        const page = make()
        const example = findChild(tiles(page)[0], "example")

        mouseClick(example, example.width / 2, example.height / 2, Qt.RightButton)

        verify(!findChild(page, "tileMenu").visible)
    }

    function test_the_blank_tile_is_first_has_the_focus_and_asks_for_a_blank_plan() {
        fake.recentJson = JSON.stringify(projects)
        const page = make()
        const cells = allTiles(page)
        const blank = findChild(cells[0], "blank")
        let asked = 0
        let opened = ""
        page.blankRequested.connect(() => { asked++ })
        page.recentRequested.connect(path => { opened = path })

        verify(blank)
        compare(blank.name, "Leerer Plan")
        compare(findChild(cells[1], "recent").name, "Wartburg")
        tryVerify(() => blank.activeFocus)

        keyClick(Qt.Key_Return)
        compare(asked, 1)
        blank.clicked()
        compare(asked, 2)
        compare(opened, "")
    }

    function test_the_blank_tile_is_there_without_projects_and_in_english() {
        fake.recentJson = "[]"
        const page = make()
        page.texts = en

        compare(allTiles(page).length, 2)
        compare(findChild(page, "blank").name, "Blank plan")
    }

    function test_the_blank_tile_has_no_menu() {
        fake.recentJson = "[]"
        const page = make()
        const blank = findChild(allTiles(page)[0], "blank")

        mouseClick(blank, blank.width / 2, blank.height / 2, Qt.RightButton)

        verify(!findChild(page, "tileMenu").visible)
    }

    function test_a_missing_folder_can_only_be_removed_from_the_list() {
        fake.recentJson = JSON.stringify(projects)
        const page = make()
        const menu = openMenuOn(page, 2)
        let forgotten = ""
        page.forgetRequested.connect(path => { forgotten = path })

        verify(!findChild(menu, "renameItem").enabled)
        verify(!findChild(menu, "archiveItem").enabled)
        verify(!findChild(menu, "trashItem").enabled)
        findChild(menu, "forgetItem").triggered()
        compare(forgotten, "/film/weg.klappe")
    }

    function test_remove_from_the_list_signals_the_path() {
        fake.recentJson = JSON.stringify(projects)
        const page = make()
        const menu = openMenuOn(page, 1)
        let forgotten = ""
        page.forgetRequested.connect(path => { forgotten = path })

        findChild(menu, "forgetItem").triggered()

        compare(forgotten, "/film/zwei.klappe")
    }

    function test_rename_asks_for_the_name_and_sends_it_to_the_controller() {
        fake.recentJson = JSON.stringify(projects)
        fake.calls = []
        fake.accept = true
        const page = make()
        const menu = openMenuOn(page, 0)

        findChild(menu, "renameItem").triggered()

        const dialog = findChild(page, "tileDialog")
        tryVerify(() => dialog.opened)
        const field = findChild(dialog, "nameField")
        compare(field.text, "Wartburg")
        field.text = "Wartburg II"
        dialog.accept()

        compare(fake.calls, [["rename", "/home/anna/film/wartburg.klappe", "Wartburg II"]])
    }

    function test_a_refused_rename_keeps_the_dialog_with_the_message() {
        fake.recentJson = JSON.stringify(projects)
        fake.calls = []
        fake.accept = false
        fake.status = ""
        const page = make()
        const menu = openMenuOn(page, 0)

        findChild(menu, "renameItem").triggered()
        const dialog = findChild(page, "tileDialog")
        tryVerify(() => dialog.opened)
        dialog.accept()

        tryVerify(() => dialog.opened)
        const error = findChild(dialog, "error")
        verify(error.visible)
        compare(error.text, "Fehler")

        fake.accept = true
        dialog.accept()
        tryVerify(() => !dialog.opened)
        verify(!findChild(page, "status").visible)
    }

    function test_trash_asks_first_and_then_calls_the_controller() {
        fake.recentJson = JSON.stringify(projects)
        fake.calls = []
        fake.accept = true
        const page = make()
        const menu = openMenuOn(page, 1)

        findChild(menu, "trashItem").triggered()

        const dialog = findChild(page, "tileDialog")
        tryVerify(() => dialog.opened)
        compare(fake.calls.length, 0)
        verify(findChild(dialog, "question").text.indexOf("Zwei") >= 0)
        dialog.reject()
        compare(fake.calls.length, 0)

        findChild(menu, "trashItem").triggered()
        tryVerify(() => dialog.opened)
        dialog.accept()

        compare(fake.calls, [["trash", "/film/zwei.klappe"]])
    }

    function test_archive_asks_first_and_then_calls_the_controller() {
        fake.recentJson = JSON.stringify(projects)
        fake.calls = []
        fake.accept = true
        const page = make()
        const menu = openMenuOn(page, 0)

        findChild(menu, "archiveItem").triggered()

        const dialog = findChild(page, "tileDialog")
        tryVerify(() => dialog.opened)
        compare(fake.calls.length, 0)
        dialog.accept()

        compare(fake.calls, [["archive", "/home/anna/film/wartburg.klappe"]])
    }

    // A refused project says why in the status line; on this page there is no header line wide
    // enough, so the page shows it whole, as a callout.
    function test_a_long_status_is_shown_whole_in_a_callout() {
        fake.status = "Projekt konnte nicht gelesen werden: szenen/szene-014.json: invalid type: map, expected a sequence"
        const page = make()
        page.anchors.fill = null
        page.width = 360
        page.height = 500

        const status = findChild(page, "status")
        verify(status.visible)
        verify(status instanceof KanteCallout, "a Kante callout")
        compare(status.kind, KanteCallout.Kind.Danger)
        compare(status.text, fake.status)
        verify(status.implicitHeight > 60, "wraps onto more lines")
        verify(status.width <= page.width)
        fake.status = ""
    }

    function test_without_a_status_the_line_is_hidden() {
        fake.status = ""
        const page = make()

        verify(!findChild(page, "status").visible)
    }
}
