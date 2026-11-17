import QtQuick
import QtTest
import "../../qml"

// PlanTools, the object library of the bar (LH-11, LH-12): the list groups built-in and own
// entries, narrows by typing, and renames, regroups, deletes and files the own ones.
TestCase {
    id: tc
    name: "PlanToolsLibrary"
    width: 900
    height: 200
    when: windowShown
    visible: true

    QtObject {
        id: fake
        property bool loaded: true
        property bool snappingOn: true
        property string gridName: "cm25"
        property var grids: []
        readonly property var choices: ({
            figure: [{ id: "elisabeth", name: "Elisabeth" }, { id: "burgvogt", name: "Burgvogt" }],
            object: [
                { kind: "table", label: "", group: "furniture", width_m: 1.6, depth_m: 0.9, shape: "rect", custom: false, uses: 1 },
                { kind: "chair", label: "", group: "furniture", width_m: 0.45, depth_m: 0.45, shape: "rect", custom: false, uses: 0 },
                { kind: "dolly", label: "", group: "camera", width_m: 1.2, depth_m: 0.8, shape: "rect", custom: false, uses: 0 },
                { kind: "thron", label: "Thron", group: "", width_m: 0.9, depth_m: 0.8, shape: "rect", custom: true, uses: 3 },
                { kind: "ruine", label: "Ruine", group: "Requisiten", width_m: 5, depth_m: 4, shape: "rect", custom: true, uses: 0 },
                { kind: "kelch", label: "Kelch", group: "Möbel", width_m: 0.1, depth_m: 0.1, shape: "rect", custom: true, uses: 0 }
            ],
            cameras: [{ id: "a", label: "A" }, { id: "b", label: "B" }],
            light: ["fresnel-650", "led-panel"],
            color: ["red", "orange", "yellow", "green", "cyan", "blue", "purple"],
            dash: ["solid", "dashed", "dotted"],
            fill: ["outline", "filled"],
            grid: [{ id: "cm5", step_m: 0.05 }, { id: "cm25", step_m: 0.25 }, { id: "m1", step_m: 1 }]
        })
        property string choicesJson: JSON.stringify(choices)
        property var tools: []
        property var snaps: []
        property var edits: []
        property var removed: []
        property bool accept: true
        property var styles: []
        property int cleared: 0

        function clearSelection() { cleared++ }
        function setTool(tool, choice) {
            tools.push([tool, choice])
            return true
        }
        function setDrawStyle(color, dash, fill) {
            styles.push([color, dash, fill])
            return true
        }
        function editLibraryItem(kind, name, group) {
            edits.push([kind, name, group])
            return accept
        }
        function removeLibraryItem(kind) {
            removed.push(kind)
            return true
        }
        function setSnapping(on) {
            snaps.push(on)
        }
        function setGrid(name) {
            grids.push(name)
        }
    }

    TestTexts { id: de; language: "de" }

    Component {
        id: spyComponent
        SignalSpy {}
    }

    Component {
        id: toolsComponent
        PlanTools {
            controller: fake
            texts: de
        }
    }

    // The settings need the names the app sets; the runner has none.
    function initTestCase() {
        Qt.application.organization = "klappe-test"
        Qt.application.domain = "klappe.test"
    }

    function init() {
        fake.choicesJson = JSON.stringify(fake.choices)
        fake.tools = []
        fake.snaps = []
        fake.edits = []
        fake.removed = []
        fake.accept = true
        fake.grids = []
        fake.gridName = "cm25"
        fake.styles = []
        fake.cleared = 0

        // The settings keep the mode over the test cases: start each from the default.
        const stale = createTemporaryObject(toolsComponent, tc)
        stale.mode = "build"
        for (const name of ["objectBox", "figureBox", "cameraBox", "lightBox"]) {
            const box = findChild(stale, name)
            box.currentIndex = 0
            box.activated(0)
        }
        wait(50)
        stale.destroy()
        wait(50)
        fake.cleared = 0
    }

    function make() {
        const bar = createTemporaryObject(toolsComponent, tc)
        verify(bar)
        waitForRendering(bar)
        return bar
    }

    function shownNames(bar) {
        return bar.shown.map(e => e.tool.name)
    }

    function test_the_library_lists_built_in_then_own_objects_in_groups() {
        const bar = make()
        const box = findChild(bar, "objectBox")

        // Built-in groups by name, then the own groups; "Möbel" of an own entry is the heading
        // of the built-in group, so its entries sit together.
        compare(box.model.map(o => o.text), ["Tisch", "Stuhl", "Kelch", "Dolly", "Thron", "Ruine"])
        compare(box.model.map(o => o.group), ["Möbel", "Möbel", "Möbel", "Kamera-Technik", "Eigene", "Requisiten"])
        compare(box.sectionAt(4), "Eigene")
    }

    function test_the_picked_object_goes_to_the_controller() {
        const bar = make()
        const box = findChild(bar, "objectBox")

        bar.tool = "object"
        compare(bar.choice, "table")
        compare(fake.tools[fake.tools.length - 1], ["object", "table"])

        box.currentIndex = 4
        box.activated(4)
        compare(bar.choice, "thron")
        compare(fake.tools[fake.tools.length - 1], ["object", "thron"])
    }

    // An object filed or a file imported changes the list; the pick stays what it was.
    function test_a_new_list_keeps_the_picked_object() {
        const bar = make()
        const box = findChild(bar, "objectBox")
        bar.tool = "object"
        box.currentIndex = 1
        box.activated(1)

        const objects = JSON.parse(fake.choicesJson).object
        objects.unshift({ kind: "bed", label: "", group: "furniture", width_m: 1.4, depth_m: 2, shape: "rect", custom: false })
        fake.choicesJson = JSON.stringify({ figure: [], object: objects, light: [], color: [], dash: [], fill: [] })

        compare(box.currentIndex, 2)
        compare(bar.choice, "chair")
    }

    function test_typing_narrows_the_library() {
        const bar = make()
        const box = findChild(bar, "objectBox")
        bar.tool = "object"

        box.forceActiveFocus()
        keyClick(Qt.Key_T)
        keyClick(Qt.Key_H)

        // Letters typed into the search are text: no tool change.
        compare(bar.tool, "object")
        compare(box.matches.map(i => box.textAt(i)), ["Thron"])
    }

    // A list in focus takes letters and digits to pick its entry: no tool or mode change.
    function test_keys_in_a_list_do_not_pick_tool_or_mode() {
        const bar = make()
        const box = findChild(bar, "lightBox")
        bar.mode = "resolve"
        bar.tool = "light"
        const tools = fake.tools.length

        box.forceActiveFocus()
        keyClick(Qt.Key_V)
        keyClick(Qt.Key_2)

        compare(bar.tool, "light")
        compare(bar.mode, "resolve")
        compare(fake.tools.length, tools)
    }

    function test_the_library_files_are_asked_for_by_the_object_tool() {
        const bar = make()
        const importButton = findChild(bar, "libraryImport")
        const exportButton = findChild(bar, "libraryExport")
        const imports = createTemporaryObject(spyComponent, tc, { target: bar, signalName: "importRequested" })
        const exports = createTemporaryObject(spyComponent, tc, { target: bar, signalName: "exportRequested" })

        verify(!importButton.visible)
        verify(!exportButton.visible)
        bar.tool = "object"
        verify(importButton.visible)
        verify(exportButton.visible)
        verify(exportButton.enabled)

        importButton.clicked()
        exportButton.clicked()
        compare(imports.count, 1)
        compare(exports.count, 1)
    }

    // Nothing of the project's own to pass on: no export.
    function test_without_own_objects_there_is_nothing_to_export() {
        const bar = make()
        const objects = JSON.parse(fake.choicesJson).object.filter(o => !o.custom)
        fake.choicesJson = JSON.stringify({ figure: [], object: objects, light: [], color: [], dash: [], fill: [] })
        bar.tool = "object"

        verify(!findChild(bar, "libraryExport").enabled)
        verify(findChild(bar, "libraryImport").enabled)
    }

    // Picks the entry of `kind` in the object list.
    function pick(bar, kind) {
        const box = findChild(bar, "objectBox")
        const index = box.model.findIndex(o => o.value === kind)
        box.currentIndex = index
        box.activated(index)
    }

    // LH-12: rename, regroup and delete are for the own entry the list has picked.
    function test_only_an_own_object_can_be_changed_or_deleted() {
        const bar = make()
        bar.tool = "object"
        const names = ["ownName", "ownGroup", "libraryRename", "libraryDelete"]

        pick(bar, "table")
        compare(names.map(n => findChild(bar, n).visible), [false, false, false, false])

        pick(bar, "thron")
        compare(names.map(n => findChild(bar, n).visible), [true, true, true, true])
        compare(findChild(bar, "ownName").text, "Thron")
        compare(findChild(bar, "ownGroup").text, "")

        pick(bar, "ruine")
        compare(findChild(bar, "ownName").text, "Ruine")
        compare(findChild(bar, "ownGroup").text, "Requisiten")

        bar.tool = "select"
        compare(names.map(n => findChild(bar, n).visible), [false, false, false, false])
    }

    function test_the_typed_name_and_group_go_to_the_controller() {
        const bar = make()
        bar.tool = "object"
        pick(bar, "thron")

        findChild(bar, "ownName").text = "Königsthron"
        findChild(bar, "ownGroup").text = "Hof"
        findChild(bar, "libraryRename").clicked()

        compare(fake.edits, [["thron", "Königsthron", "Hof"]])
        verify(!findChild(bar, "ownName").invalid)
    }

    function test_a_refused_name_marks_the_field() {
        const bar = make()
        bar.tool = "object"
        pick(bar, "thron")
        fake.accept = false

        findChild(bar, "ownName").text = ""
        findChild(bar, "libraryRename").clicked()

        verify(findChild(bar, "ownName").invalid)
        findChild(bar, "ownName").text = "x"
        verify(!findChild(bar, "ownName").invalid)
    }

    // The first click asks and names the objects of the plan; the second deletes.
    function test_deleting_asks_first_and_names_the_objects_in_use() {
        const bar = make()
        bar.tool = "object"
        pick(bar, "thron")
        const button = findChild(bar, "libraryDelete")
        compare(button.text, "Löschen")

        button.clicked()
        compare(button.text, "Wirklich löschen? 3 im Plan")
        compare(fake.removed, [])

        button.clicked()
        compare(fake.removed, ["thron"])
        compare(button.text, "Löschen")
    }

    // A lamp or character id can equal the kind of an own object; only the object tool edits.
    function test_another_tool_with_the_id_of_an_own_kind_changes_nothing() {
        const bar = make()
        const choices = JSON.parse(fake.choicesJson)
        choices.light.push("thron")
        fake.choicesJson = JSON.stringify(choices)
        bar.tool = "light"
        const box = findChild(bar, "lightBox")
        box.currentIndex = box.model.findIndex(o => o.value === "thron")
        box.activated(box.currentIndex)

        compare(bar.choice, "thron")
        for (const n of ["ownName", "ownGroup", "libraryRename", "libraryDelete"]) {
            verify(!findChild(bar, n).visible, n)
        }
    }

    function test_deleting_an_unused_entry_does_not_count() {
        const bar = make()
        bar.tool = "object"
        pick(bar, "ruine")
        const button = findChild(bar, "libraryDelete")

        button.clicked()
        compare(button.text, "Wirklich löschen?")
    }

    // Undo of a rename changes the entry under the same kind; the fields follow.
    function test_the_fields_follow_a_change_of_the_entry() {
        const bar = make()
        bar.tool = "object"
        pick(bar, "thron")
        const original = fake.choicesJson
        const renamed = JSON.parse(original)
        renamed.object.find(o => o.kind === "thron").label = "Sessel"

        fake.choicesJson = JSON.stringify(renamed)
        compare(findChild(bar, "ownName").text, "Sessel")

        fake.choicesJson = original
        compare(findChild(bar, "ownName").text, "Thron")
    }

    function test_another_pick_cancels_the_question() {
        const bar = make()
        bar.tool = "object"
        pick(bar, "thron")
        const button = findChild(bar, "libraryDelete")

        button.clicked()
        pick(bar, "ruine")

        compare(button.text, "Löschen")
        button.clicked()
        compare(fake.removed, [])
    }

    function test_lists_have_translated_names() {
        const bar = make()
        bar.tool = "object"

        compare(findChild(bar, "objectBox").textAt(0), "Tisch")
        bar.tool = "light"
        compare(findChild(bar, "lightBox").textAt(1), "LED-Panel")
    }
}
