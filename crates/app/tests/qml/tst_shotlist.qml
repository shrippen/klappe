import QtQuick
import QtQuick.Controls as QQC2
import QtCore
import QtTest
import Kante
import "../../qml"

// The shot area's list (D-05, LH-70, LH-72): tiles and table, the switches between them, the
// selection and the edit in a cell. Each view also leaves a picture in the temp folder (TMPDIR).
TestCase {
    id: tc
    name: "ShotList"
    width: 480
    height: 760
    visible: true
    when: windowShown

    // Cut order 12.1 (A, setup 1), 12.2 (B, setup 2), 12.3 (A, setup 1); shooting order groups the setups.
    readonly property var rows: [
        row("s1", "12.1", "a", "A", "wide", "static", 35, 2.39, 1, 0, 6),
        row("s2", "12.2", "b", "B", "close-up", "dolly", 85, 1.78, 2, 6, 12, true),
        row("s3", "12.3", "a", "A", "medium", "pan", 35, 2.39, 1, 12, 20)
    ]

    function row(id, number, camera, label, size, movement, lens, aspect, setup, from, to, jump) {
        return {
            id: id, number: number, camera: camera, camera_label: label, aspect: aspect, follows: "",
            size: size, angle: "eye-level", movement: movement, movement_hint: "", lens_mm: lens,
            lens_to_mm: null, lens_from_marks: false, own_lens_mm: null, from_s: from, to_s: to,
            description: "Szene " + number, props: "", gear: "", crew: "", note: "", axis_jump: jump === true, setup: setup
        }
    }

    QtObject {
        id: fake
        property string tracksJson: JSON.stringify([
            { kind: "camera", id: "a", name: "A" },
            { kind: "camera", id: "b", name: "B" }
        ])
        property string choicesJson: JSON.stringify({ movement: ["static", "pan", "dolly"] })
        property string shotsJson: JSON.stringify(tc.rows)
        property string shotsShootingJson: JSON.stringify([tc.rows[0], tc.rows[2], tc.rows[1]])
        property bool loaded: true
        property bool accept: true
        property var calls: []
        property var selects: []

        function addShot() { return true }
        function copyShot() { return true }
        function deleteShot() { return true }
        function setFollow() { return true }
        function selectShot(id) { selects.push(id) }
        function editShot(id, field, value) {
            calls.push([id, field, value])
            return accept
        }
    }

    TestTexts { id: de; language: "de" }
    TestTexts { id: en; language: "en" }

    Component {
        id: panelComponent
        ShotPanel {
            width: 480
            height: 760
            texts: de
            controller: fake
        }
    }

    function make(view, order, selected) {
        fake.calls = []
        fake.selects = []
        fake.accept = true
        const panel = createTemporaryObject(panelComponent, tc, {
            view: view, order: order || "cut", shot: selected ? rows.find(r => r.id === selected) : null
        })
        verify(panel)
        return panel
    }

    function part(panel, name) {
        const item = findChild(panel, name)
        verify(item, name)
        return item
    }

    function shoot(panel, name) {
        const dir = StandardPaths.writableLocation(StandardPaths.TempLocation).toString().replace("file://", "")
        grabImage(panel).save(dir + "/klappe-shots-" + name + ".png")
    }

    function numbers(table) {
        return table.items.filter(i => !i.band).map(i => i.shot.number)
    }

    // Typing keeps the text binding alive, unlike `field.text = ...`.
    function type(field, text) {
        field.remove(0, field.length)
        field.insert(0, text)
    }

    function test_the_switch_chooses_the_view() {
        const panel = make("tiles")
        const picked = createTemporaryObject(spyComponent, tc, { target: panel, signalName: "viewPicked" })
        verify(part(panel, "shotTiles").visible)
        verify(!part(panel, "shotTable").visible)

        part(panel, "viewSwitch").select(1)

        compare(picked.count, 1)
        compare(picked.signalArguments[0][0], "table")
        panel.view = "table"
        verify(!part(panel, "shotTiles").visible)
        verify(part(panel, "shotTable").visible)
        compare(part(panel, "viewSwitch").currentIndex, 1)
    }

    Component {
        id: spyComponent
        SignalSpy {}
    }

    function test_the_switch_chooses_the_order() {
        const panel = make("table")
        const picked = createTemporaryObject(spyComponent, tc, { target: panel, signalName: "orderPicked" })

        part(panel, "orderSwitch").select(1)

        compare(picked.count, 1)
        compare(picked.signalArguments[0][0], "shooting")
    }

    function test_the_switch_is_labelled_in_both_languages() {
        compare(part(make("tiles"), "viewSwitch").model, ["Kacheln", "Tabelle"])

        const panel = createTemporaryObject(panelComponent, tc, { texts: en })

        compare(part(panel, "viewSwitch").model, ["Tiles", "Table"])
        compare(part(panel, "orderSwitch").model, ["Cut order", "Shooting order"])
    }

    function test_a_tile_has_the_frame_of_its_camera() {
        const panel = make("tiles")

        compare(part(panel, "tile-s1").stageAspect, 2.39)
        compare(part(panel, "tile-s2").stageAspect, 1.78)
        compare(part(panel, "tile-s1").name, "12.1 · Totale")
        compare(part(panel, "tile-s2").figure, "85 mm")
    }

    // The jump is a sign and a word on the tile, not only the tier colour.
    function test_a_tile_names_an_axis_jump() {
        const panel = make("tiles")
        const jump = part(panel, "axisJump-s2")

        verify(jump.visible)
        compare(jump.text, "Achsensprung")
        compare(jump.iconName, "dialog-warning")
        verify(!part(panel, "axisJump-s1").visible)
    }

    function test_a_tile_click_selects_the_shot() {
        const panel = make("tiles", "cut", "s3")

        verify(part(panel, "tile-s3").selected)
        verify(!part(panel, "tile-s1").selected)

        part(panel, "tile-s2").clicked()

        compare(fake.selects, ["s2"])
    }

    function test_tiles_follow_the_order() {
        const panel = make("tiles", "shooting", "s3")

        compare(part(panel, "shotTiles").shots.map(s => s.number), ["12.1", "12.3", "12.2"])
        shoot(panel, "tiles")
    }

    function test_the_table_lists_the_cut_order() {
        const panel = make("table", "cut", "s2")
        const table = part(panel, "shotTable")

        compare(numbers(table), ["12.1", "12.2", "12.3"])
        verify(!table.items.some(i => i.band))
        shoot(panel, "table-cut")
    }

    function test_the_shooting_order_groups_the_setups() {
        const panel = make("table", "shooting", "s3")
        const table = part(panel, "shotTable")
        shoot(panel, "table-shooting")

        compare(table.items.map(i => i.band ? "setup " + i.setup : i.shot.number),
                ["setup 1", "12.1", "12.3", "setup 2", "12.2"])
    }

    function test_the_table_has_the_columns_of_the_export() {
        const table = part(make("table"), "shotTable")

        compare(table.columns.map(c => de.t(c.title)), ["Nummer", "Kamera", "Objektiv (mm)", "Größe", "Perspektive",
            "Bewegung", "Von (s)", "Bis (s)", "Dauer (s)", "Inhalt", "Requisiten", "Technik", "Team", "Notiz"])
        compare(table.cellText(tc.rows[1], "camera"), "B")
        compare(table.cellText(tc.rows[1], "size"), "Groß")
        compare(table.cellText(tc.rows[1], "duration"), "6")
    }

    // The selected row wears Kante's selection role and edge, as `KanteListRow`.
    function test_the_selected_row_has_the_kante_selection() {
        const panel = make("table", "cut", "s2")
        const table = part(panel, "shotTable")
        const fill = findChild(table, "rowFill-s2")
        const edge = findChild(table, "rowEdge-s2")

        verify(fill)
        compare(fill.color, KanteStyle.selectionColor)
        compare(edge.width, KanteStyle.edgeWidth)
        verify(!findChild(table, "rowEdge-s1").visible)
    }

    function test_a_click_on_a_row_selects_the_shot() {
        const panel = make("table", "cut", "s1")
        const list = part(panel, "shotRows")
        waitForRendering(list)

        const line = list.itemAtIndex(1)
        verify(line)
        mouseClick(line, 10, line.height / 2)

        compare(fake.selects, ["s2"])
    }

    function test_the_arrow_keys_move_the_selection_over_the_bands() {
        const panel = make("table", "shooting", "s3")
        const table = part(panel, "shotTable")
        part(panel, "shotRows").forceActiveFocus()

        keyClick(Qt.Key_Down)
        keyClick(Qt.Key_Up)

        compare(fake.selects, ["s2", "s1"])
    }

    function test_a_text_cell_is_edited_and_sent() {
        const panel = make("table", "cut", "s2")
        const table = part(panel, "shotTable")

        table.edit("s2", 2)
        const editor = part(panel, "cellEditor")
        compare(editor.text, "")
        type(editor, "100")
        editor.editingFinished()

        compare(fake.calls, [["s2", "lens", "100"]])
        compare(table.editId, "")
    }

    function test_a_cell_left_unchanged_edits_nothing() {
        const panel = make("table", "cut", "s2")

        part(panel, "shotTable").edit("s2", 9)
        part(panel, "cellEditor").editingFinished()

        compare(fake.calls.length, 0)
    }

    function test_a_choice_cell_sends_the_choice() {
        const panel = make("table", "cut", "s2")
        const table = part(panel, "shotTable")

        table.edit("s2", 1)
        part(panel, "cellEditor").activated(0)

        compare(fake.calls, [["s2", "camera", "a"]])
        compare(table.editId, "")
    }

    function test_the_movement_cell_offers_the_movements_of_the_controller() {
        const panel = make("table", "cut", "s2")

        part(panel, "shotTable").edit("s2", 5)
        const box = part(panel, "cellEditor")
        compare(box.model, ["Statisch", "Schwenk", "Fahrt"])
        box.activated(1)

        compare(fake.calls, [["s2", "movement", "pan"]])
    }

    function test_escape_drops_the_edit() {
        const panel = make("table", "cut", "s2")
        const table = part(panel, "shotTable")

        table.edit("s2", 9)
        const editor = part(panel, "cellEditor")
        type(editor, "anderer Text")
        keyClick(Qt.Key_Escape)

        compare(table.editId, "")
        compare(fake.calls.length, 0)
    }

    function test_enter_edits_the_current_cell() {
        const panel = make("table", "cut", "s2")
        const table = part(panel, "shotTable")
        part(panel, "shotRows").forceActiveFocus()

        for (let i = 0; i < 9; i++) {
            keyClick(Qt.Key_Right)
        }
        keyClick(Qt.Key_Return)

        compare(table.editId, "s2")
        compare(table.editColumn, 9)
        verify(part(panel, "cellEditor").activeFocus)
    }

    // QtTest's clicks never add up to a double tap (it spaces them by the double-click time),
    // so the cell's `open`, which a double tap calls, is called.
    function test_a_double_tap_edits_the_cell() {
        const panel = make("table", "cut", "s1")
        const list = part(panel, "shotRows")
        waitForRendering(list)

        findChild(list.itemAtIndex(1), "cell-description").parent.open()

        compare(part(panel, "shotTable").editId, "s2")
        compare(part(panel, "shotTable").editColumn, 9)
        compare(fake.selects, ["s2"])
    }

    function test_the_computed_cells_are_not_edited() {
        const panel = make("table", "cut", "s1")
        const table = part(panel, "shotTable")

        table.edit("s1", 6)
        table.edit("s1", 8)

        compare(table.editId, "")
    }

    function test_a_lens_from_the_marks_is_not_edited() {
        const panel = make("table", "cut", "s1")
        const table = part(panel, "shotTable")
        const marked = Object.assign({}, tc.rows[0], { lens_from_marks: true })

        verify(!table.editable(marked, 2))
        verify(table.editable(tc.rows[0], 2))
    }

    function test_a_refused_cell_shows_the_model_value_again() {
        const panel = make("table", "cut", "s2")
        const table = part(panel, "shotTable")
        const list = part(panel, "shotRows")
        waitForRendering(list)
        fake.accept = false

        table.edit("s2", 9)
        type(part(panel, "cellEditor"), "anderer Text")
        part(panel, "cellEditor").editingFinished()

        compare(fake.calls.length, 1)
        compare(findChild(list.itemAtIndex(1), "cell-description").text, "Szene 12.2")
    }

    // The details of the selected shot stay under the list.
    function test_the_details_stay_under_the_list() {
        const panel = make("table", "cut", "s2")

        verify(part(panel, "numberField").visible)
        compare(part(panel, "numberField").text, "12.2")
        verify(part(panel, "numberField").mapToItem(panel, 0, 0).y > part(panel, "shotTable").mapToItem(panel, 0, 0).y)
    }
}
