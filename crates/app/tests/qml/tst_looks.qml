import QtQuick
import QtTest
import "../../qml"

// D-25: the looks of the breakdown window: the chip of a figure in the scene sheet and its menu,
// the elements of the worn looks in the sheet's categories, and the tab "Looks" (list, form,
// new look). The controller is a stand-in that records the calls (tst_breakdown.qml).
TestCase {
    id: tc
    name: "Looks"
    width: 1200
    height: 900
    when: windowShown
    visible: true

    QtObject {
        id: fake
        property string status: ""
        property string sceneId: "s1"
        property bool refuse: false
        property var calls: []
        readonly property var categories: ["costume", "makeup", "construction", "set-dressing", "prop",
            "graphics", "vehicle", "animal", "vfx", "sfx", "sound", "music", "camera-light", "stunt", "other"]
        readonly property string fixture: JSON.stringify({
            scenes: [
                { id: "s1", number: "1", heading: "INT. SAAL – TAG", inBook: true, plans: 1, shots: 0, length: "" },
                { id: "s2", number: "2", heading: "EXT. HOF – NACHT", inBook: true, plans: 0, shots: 0, length: "" }
            ],
            characters: [
                { id: "anna", name: "Anna", color: "purple", group: 0 },
                { id: "bert", name: "Bert", color: "blue", group: 0 },
                { id: "fischer", name: "Fischer", color: "green", group: 3 }
            ],
            element_categories: categories,
            element_statuses: ["open", "working", "ready"],
            elements: [
                { id: "e1", category: "prop", name: "Brief", description: "", status: "open", scenes: [] },
                { id: "e3", category: "costume", name: "Ballkleid", description: "", status: "ready", scenes: [] }
            ],
            looks: [
                { id: "l1", character: "anna", name: "Ölzeug, nass", scenes: [{ id: "s1", number: "1" }],
                  elements: [{ id: "e3", name: "Ballkleid", category: "costume" }] },
                { id: "l2", character: "anna", name: "Trocken", scenes: [], elements: [] },
                { id: "l3", character: "bert", name: "Anzug", scenes: [{ id: "s1", number: "1" }, { id: "s2", number: "2" }], elements: [] }
            ],
            sheet: {
                story_day: "", prestop_text: "", planned_text: "", continuity: "", note: "",
                speaking: [{ id: "anna", name: "Anna", color: "purple", group: 0, look: null }],
                present: [{ id: "fischer", name: "Fischer", color: "green", group: 3, look: null }],
                categories: [{ category: "prop", from_looks: [], elements: [{ id: "e1", name: "Brief", status: "open", lost: false }] }],
                text: [{ kind: "action", runs: [{ text: "Anna liest.", element: null, category: null }] }],
                suggestions: []
            }
        })
        property string projectJson: fixture

        function call(name, args) {
            calls.push([name].concat(args))
            return !refuse
        }
        function clearStatus() { status = "" }
        function selectScene(id) { sceneId = id; return call("selectScene", [id]) }
        function scriptLooks() {
            return JSON.stringify({ columns: 60,
                looks: { a: { kind: "action", x: 0, width: 60, align: "left" } }, letters: { action: "a" } })
        }
        function addElement(c, n) {
            const project = JSON.parse(projectJson)
            project.elements.push({ id: "e9", category: c, name: "Neu", description: "", status: "open", scenes: [] })
            projectJson = JSON.stringify(project)

            return call("addElement", [c, n]) ? "e9" : ""
        }
        function addLook(c, n) { return call("addLook", [c, n]) ? "l9" : "" }
        function editLook(i, n) { return call("editLook", [i, n]) }
        function removeLook(i) { return call("removeLook", [i]) }
        function addLookElement(l, e) { return call("addLookElement", [l, e]) }
        function addElementToLook(l, c, n) { return call("addElementToLook", [l, c, n]) }
        function removeLookElement(l, e) { return call("removeLookElement", [l, e]) }
        function setSceneLook(s, c, l) { return call("setSceneLook", [s, c, l]) }
    }

    TestTexts { id: de; language: "de" }

    Component {
        id: sheetComponent
        SceneSheet { texts: de; controller: fake; width: 800; height: 800 }
    }
    Component {
        id: viewComponent
        BreakdownView { texts: de; controller: fake; width: 1200; height: 800 }
    }

    function init() {
        fake.sceneId = "s1"
        fake.refuse = false
        fake.calls = []
        fake.projectJson = fake.fixture
    }

    function sheet() {
        const s = createTemporaryObject(sheetComponent, tc)
        verify(s)
        waitForRendering(s)
        return s
    }

    function view() {
        const v = createTemporaryObject(viewComponent, tc)
        verify(v)
        waitForRendering(v)
        return v
    }

    // Anna wears "Ölzeug, nass" (l1) in the scene: its costume stands in the sheet's category.
    function wearLook() {
        const project = JSON.parse(fake.fixture)
        project.sheet.speaking[0].look = "l1"
        project.sheet.categories.unshift({ category: "costume", elements: [],
            from_looks: [{ id: "e3", name: "Ballkleid", status: "ready", figure: "Anna" }] })
        fake.projectJson = JSON.stringify(project)
    }

    // A menu entry as the user chooses it: not a click, a theme with native menus takes none.
    function choose(menu, name) {
        const item = findChild(menu.contentItem, name) || findChild(menu, name)
        verify(item, name)
        verify(item.enabled, name)
        if (item.checkable) {
            item.toggle()
        }
        item.triggered()
    }

    function test_a_figure_chip_shows_its_look() {
        wearLook()
        const s = sheet()

        compare(findChild(s, "speaking.anna").text, "Anna · Ölzeug, nass")
        compare(findChild(s, "present.fischer").text, "Fischer ×3")
    }

    function test_the_chip_menu_lists_the_looks_of_the_character_with_a_tick() {
        wearLook()
        const s = sheet()
        const menu = findChild(s, "lookMenu")

        findChild(s, "speaking.anna").clicked()

        tryCompare(menu, "visible", true)
        verify(findChild(menu.contentItem, "look.l1").checked)
        verify(!findChild(menu.contentItem, "look.l2").checked)
        verify(findChild(menu.contentItem, "look.l3") === null)
        verify(findChild(menu.contentItem, "noLook").enabled)
        compare(findChild(menu.contentItem, "noLook").text, "Kein Look")
        compare(findChild(menu.contentItem, "newLook").text, "Neuer Look …")
        compare(findChild(menu.contentItem, "editLooks").text, "Looks bearbeiten …")
        menu.close()
    }

    function test_a_look_chosen_in_the_menu_is_set_for_the_scene() {
        const s = sheet()
        const menu = findChild(s, "lookMenu")

        findChild(s, "speaking.anna").clicked()
        verify(!findChild(menu.contentItem, "noLook").enabled)
        choose(menu, "look.l2")

        compare(fake.calls, [["setSceneLook", "s1", "anna", "l2"]])
        menu.close()
    }

    function test_no_look_takes_the_look_off() {
        wearLook()
        const s = sheet()
        const menu = findChild(s, "lookMenu")

        findChild(s, "speaking.anna").clicked()
        choose(menu, "noLook")

        compare(fake.calls, [["setSceneLook", "s1", "anna", ""]])
        menu.close()
    }

    function test_the_chip_of_a_figure_without_cue_opens_the_menu_too() {
        const s = sheet()
        const menu = findChild(s, "lookMenu")

        findChild(s, "present.fischer").clicked()

        tryCompare(menu, "visible", true)
        verify(findChild(menu.contentItem, "look.l1") === null)
        verify(findChild(menu.contentItem, "newLook") !== null)
        menu.close()
    }

    function test_a_chip_is_reached_by_the_keyboard() {
        const s = sheet()
        const chip = findChild(s, "speaking.anna")

        verify(chip.activeFocusOnTab)
        verify(chip.Accessible.name !== "")
        verify(chip.toolTip.includes("Look von Anna wählen"))
    }

    function test_a_new_look_is_made_and_put_on_in_this_scene() {
        const s = sheet()
        const menu = findChild(s, "lookMenu")
        const form = findChild(s, "sheetLookForm")

        findChild(s, "speaking.anna").clicked()
        choose(menu, "newLook")
        tryCompare(form, "opened", true)
        verify(!findChild(form, "characterBox").visible)
        verify(!form.acceptEnabled)
        findChild(form, "nameField").text = "Mantel"
        verify(form.acceptEnabled)
        form.accept()

        compare(fake.calls, [["addLook", "anna", "Mantel"], ["setSceneLook", "s1", "anna", "l9"]])
    }

    function test_a_refused_new_look_is_not_put_on() {
        const s = sheet()
        const menu = findChild(s, "lookMenu")
        const form = findChild(s, "sheetLookForm")

        findChild(s, "speaking.anna").clicked()
        choose(menu, "newLook")
        tryCompare(form, "opened", true)
        findChild(form, "nameField").text = "Mantel"
        fake.refuse = true
        form.accept()

        compare(fake.calls, [["addLook", "anna", "Mantel"]])
        form.close()
    }

    function test_edit_looks_shows_the_tab_with_the_look() {
        wearLook()
        const v = view()

        findChild(v, "speaking.anna").clicked()
        choose(findChild(v, "lookMenu"), "editLooks")

        compare(v.mode, "looks")
        compare(v.lookId, "l1")
        verify(findChild(v, "lookList").visible)
    }

    // Without a look worn, the character's first look is chosen.
    function test_edit_looks_without_a_look_takes_the_first_of_the_character() {
        const v = view()

        findChild(v, "speaking.anna").clicked()
        choose(findChild(v, "lookMenu"), "editLooks")

        compare(v.mode, "looks")
        compare(v.lookId, "l1")
    }

    function test_edit_looks_of_a_character_without_looks_shows_the_tab_alone() {
        const v = view()

        findChild(v, "present.fischer").clicked()
        choose(findChild(v, "lookMenu"), "editLooks")

        compare(v.mode, "looks")
        compare(v.lookId, "")
        verify(findChild(v, "lookHint").visible)
    }

    function test_the_look_elements_stand_in_the_category_without_a_cross() {
        wearLook()
        const s = sheet()

        const chip = findChild(s, "lookElement.e3")
        verify(chip !== null)
        compare(chip.text, "Ballkleid · Look Anna")
        verify(!chip.removable)
        verify(findChild(s, "category.costume") !== null)
        // The category has nothing of the sheet's own, so no element chip of it.
        verify(findChild(s, "element.e3") === null)
        verify(findChild(s, "add.costume") !== null)
    }

    function showLooks(v) {
        findChild(v, "modeSwitch").activated(2)
        verify(findChild(v, "lookList").visible)
    }

    function look(v, id) {
        const rows = findChild(v, "lookRows")
        tryVerify(() => findChild(rows, "look." + id) !== null)
        return findChild(rows, "look." + id)
    }

    function test_the_tab_lists_the_looks_by_character_with_their_scenes() {
        const v = view()
        showLooks(v)

        compare(findChild(v, "lookList").rows.map(r => r.id), ["l1", "l2", "l3"])
        compare(findChild(v, "lookList").rows.map(r => r.figure), ["Anna", "Anna", "Bert"])
        compare(look(v, "l1").meta, "1 Szene")
        compare(look(v, "l2").meta, "0 Szenen")
        compare(look(v, "l3").meta, "2 Szenen")
        verify(!findChild(v, "sheet").visible)
    }

    function test_without_a_chosen_look_the_form_says_what_to_do() {
        const v = view()
        showLooks(v)

        const hint = findChild(v, "lookHint")
        verify(hint.visible)
        compare(hint.title, "Links einen Look wählen oder mit „Neu…“ anlegen.")
        verify(!findChild(v, "lookForm").visible)

        look(v, "l2").clicked()
        verify(!hint.visible)
    }

    function test_a_look_shows_in_the_form_and_the_name_applies() {
        const v = view()
        showLooks(v)

        look(v, "l1").clicked()
        const form = findChild(v, "lookForm")
        compare(findChild(form, "nameField").text, "Ölzeug, nass")
        compare(findChild(form, "figure").text, "Anna")
        verify(findChild(form, "element.e3") !== null)
        compare(findChild(form, "element.e3").text, "Ballkleid")

        findChild(form, "nameField").text = "Ölzeug"
        findChild(form, "apply").clicked()

        compare(fake.calls, [["editLook", "l1", "Ölzeug"]])
    }

    // The same look in a refreshed project keeps what is typed in the name.
    function test_a_refresh_keeps_the_typed_name() {
        const v = view()
        showLooks(v)
        look(v, "l1").clicked()
        const form = findChild(v, "lookForm")

        findChild(form, "nameField").text = "Halb getippt"
        fake.projectJson = fake.fixture.replace("Trocken", "Anders")

        compare(findChild(form, "nameField").text, "Halb getippt")
    }

    function test_an_element_leaves_the_look_by_its_cross() {
        const v = view()
        showLooks(v)

        look(v, "l1").clicked()
        findChild(findChild(v, "lookForm"), "element.e3").removeRequested()

        compare(fake.calls, [["removeLookElement", "l1", "e3"]])
    }

    function test_an_existing_element_goes_into_the_look_by_its_name() {
        const project = JSON.parse(fake.fixture)
        project.elements.push({ id: "e5", category: "makeup", name: "Schramme", description: "", status: "open", scenes: [] })
        fake.projectJson = JSON.stringify(project)
        const v = view()
        showLooks(v)

        look(v, "l2").clicked()
        const field = findChild(findChild(v, "lookForm"), "add.makeup")
        compare(field.model, ["Schramme"])
        field.activated(0)

        compare(fake.calls, [["addElementToLook", "l2", "makeup", "Schramme"]])
    }

    // A costume the look lacks is offered; the typed name of a new one is one step.
    function test_a_new_element_is_made_and_put_in_the_look_in_one_call() {
        const v = view()
        showLooks(v)

        look(v, "l2").clicked()
        const field = findChild(findChild(v, "lookForm"), "add.costume")
        compare(field.model, ["Ballkleid"])
        field.newEntered("Hut")

        compare(fake.calls, [["addElementToLook", "l2", "costume", "Hut"]])
    }

    function test_a_typed_name_of_an_existing_element_goes_to_the_session_trimmed() {
        const v = view()
        showLooks(v)

        look(v, "l2").clicked()
        findChild(findChild(v, "lookForm"), "add.costume").newEntered(" ballkleid ")

        compare(fake.calls, [["addElementToLook", "l2", "costume", "ballkleid"]])
    }

    // The name decides nothing in the form: the same name, another case and a new one all go
    // to the session in one call (it takes the element of that name, or makes one).
    function test_a_typed_name_in_any_case_is_one_call_for_the_session() {
        const v = view()
        showLooks(v)

        look(v, "l2").clicked()
        const field = findChild(findChild(v, "lookForm"), "add.costume")
        field.newEntered("Ballkleid")
        field.newEntered("BALLKLEID")
        field.newEntered("Hut")

        compare(fake.calls, [["addElementToLook", "l2", "costume", "Ballkleid"],
            ["addElementToLook", "l2", "costume", "BALLKLEID"], ["addElementToLook", "l2", "costume", "Hut"]])
    }

    function test_the_scenes_of_a_look_show_the_scene() {
        const v = view()
        showLooks(v)

        look(v, "l3").clicked()
        findChild(findChild(v, "lookForm"), "scene.s2").clicked()

        compare(fake.calls, [["selectScene", "s2"]])
        compare(v.mode, "scenes")
    }

    function test_a_look_in_no_scene_says_so() {
        const v = view()
        showLooks(v)

        look(v, "l2").clicked()

        verify(findChild(findChild(v, "lookForm"), "noScenes").visible)
    }

    function test_a_look_is_deleted_after_asking() {
        const v = view()
        showLooks(v)

        look(v, "l1").clicked()
        const form = findChild(v, "lookForm")
        findChild(form, "remove").clicked()
        compare(fake.calls.length, 0)
        findChild(form, "confirmRemove").clicked()

        compare(fake.calls, [["removeLook", "l1"]])
    }

    function test_new_asks_for_the_character_and_the_name() {
        const v = view()
        showLooks(v)

        findChild(v, "newLook").clicked()
        const form = findChild(v, "listLookForm")
        tryCompare(form, "opened", true)
        verify(findChild(form, "characterBox").visible)
        compare(findChild(form, "characterBox").model, ["Anna", "Bert", "Fischer"])

        findChild(form, "characterBox").currentIndex = 1
        findChild(form, "nameField").text = "Frack"
        form.accept()

        compare(fake.calls, [["addLook", "bert", "Frack"]])
        compare(v.lookId, "l9")
    }

    function test_the_tab_without_looks_says_so() {
        const project = JSON.parse(fake.fixture)
        project.looks = []
        fake.projectJson = JSON.stringify(project)
        const v = view()
        showLooks(v)

        verify(findChild(v, "noLooks").visible)
        compare(findChild(v, "noLooks").text, "Noch keine Looks.")
    }
}
