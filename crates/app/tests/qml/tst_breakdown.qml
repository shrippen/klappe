import QtQuick
import QtTest
import "../../qml"

// D-22: the breakdown module. The scene sheet (fields that send the sheet when editing ends, the
// figures, the elements of the categories that have some, the suggestions of the set) and the
// elements view (list by category, form that applies or deletes, scenes that jump).
TestCase {
    id: tc
    name: "Breakdown"
    width: 1200
    height: 900
    when: windowShown
    visible: true

    QtObject {
        id: fake
        property string status: ""
        property string sceneId: "s1"
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
            persons: [
                { id: "p1", name: "Jo Berg", department: "camera" },
                { id: "p2", name: "Mia Rot", department: "props" },
                { id: "p3", name: "Ole Blau", department: null }
            ],
            elements: [
                { id: "e1", category: "prop", name: "Brief", description: "", status: "open", scenes: [{ id: "s1", number: "1" }], responsible: null },
                { id: "e2", category: "prop", name: "Kerze", description: "Wachs", status: "working", scenes: [], responsible: "p3" },
                { id: "e3", category: "costume", name: "Ballkleid", description: "", status: "ready", scenes: [{ id: "s1", number: "1" }, { id: "s2", number: "2" }] }
            ],
            sheet: {
                story_day: "3", prestop_text: "1:30", planned_text: "2:00", continuity: "Brief links", note: "Nachtdreh",
                speaking: [{ id: "anna", name: "Anna", color: "purple", group: 0 }],
                present: [{ id: "fischer", name: "Fischer", color: "green", group: 3 }],
                categories: [{ category: "prop", elements: [{ id: "e1", name: "Brief", status: "open", lost: true }] }],
                text: [{ kind: "action", runs: [{ text: "Anna liest den Brief.", element: null, category: null }] }],
                suggestions: [
                    { element: "e2", name: "Kerze", category: "prop", object: "Kerzenleuchter" },
                    { element: "e3", name: "Ballkleid", category: "costume", object: null }
                ]
            }
        })
        property string projectJson: fixture

        function call(name, args) {
            calls.push([name].concat(args))
            return true
        }
        function clearStatus() { status = "" }
        function selectScene(id) { sceneId = id; return call("selectScene", [id]) }
        function setSheet(s, d, p, m, c, n) {
            call("setSheet", [s, d, p, m, c, n])
            // Like the bridge: the name of the refused field, empty when accepted.
            if (p === "1:99") {
                return "prestop"
            }
            return m === "1:99" ? "planned" : ""
        }
        function addElementToScene(s, c, n) { return call("addElementToScene", [s, c, n]) }
        function removeSceneElement(s, e) { return call("removeSceneElement", [s, e]) }
        function takeSuggestion(s, e) { return call("takeSuggestion", [s, e]) }
        function excludeSuggestion(s, e) { return call("excludeSuggestion", [s, e]) }
        function addSceneCharacter(s, c) { return call("addSceneCharacter", [s, c]) }
        function removeSceneCharacter(s, c) { return call("removeSceneCharacter", [s, c]) }
        // The page's columns as the bridge gives them (`scriptLooks`), for the kinds used here.
        function scriptLooks() {
            return JSON.stringify({ columns: 60,
                looks: { a: { kind: "action", x: 0, width: 60, align: "left" },
                         d: { kind: "dialogue", x: 10, width: 35, align: "left" },
                         r: { kind: "transition", x: 40, width: 20, align: "right" } },
                letters: { action: "a", dialogue: "d", transition: "r" } })
        }
        function forgetLost(s, e) { return call("forgetLost", [s, e]) }
        // Like the bridge: the persons, those of the department of the category first (only
        // the props department is known here).
        function personsFor(category) {
            const all = JSON.parse(projectJson).persons
            const own = all.filter(p => category === "prop" && p.department === "props")
            return JSON.stringify(own.concat(all.filter(p => !own.includes(p))))
        }
        // Makes the element as the project then lists it, under an id the form cannot guess
        // from the name; answers the id.
        function addElement(c, n) {
            const project = JSON.parse(projectJson)
            project.elements.push({ id: "e9", category: c, name: "Neu", description: "", status: "open", scenes: [] })
            projectJson = JSON.stringify(project)

            return call("addElement", [c, n]) ? "e9" : ""
        }
        function editElement(i, c, n, d, s, r) { return call("editElement", [i, c, n, d, s, r]) }
        function removeElement(i) { return call("removeElement", [i]) }
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

    // Types `text` into `field` and ends the editing with Enter.
    function type(field, text) {
        field.forceActiveFocus()
        field.text = text
        keyClick(Qt.Key_Return)
    }

    function test_the_sheet_shows_the_scene_and_its_fields() {
        const s = sheet()

        compare(findChild(s, "sheetTitle").text, "1  INT. SAAL – TAG")
        compare(findChild(s, "dayField").text, "3")
        compare(findChild(s, "prestopField").text, "1:30")
        compare(findChild(s, "plannedField").text, "2:00")
        compare(findChild(s, "continuityField").text, "Brief links")
        compare(findChild(s, "noteField").text, "Nachtdreh")
    }

    function test_the_pre_stop_goes_in_as_typed() {
        const s = sheet()

        type(findChild(s, "prestopField"), "2:05")

        compare(fake.calls, [["setSheet", "s1", "3", "2:05", "2:00", "Brief links", "Nachtdreh"]])
    }

    function test_the_planned_time_goes_in_as_typed() {
        const s = sheet()

        type(findChild(s, "plannedField"), "1:45")
        type(findChild(s, "plannedField"), "45")

        compare(fake.calls[0][4], "1:45")
        compare(fake.calls[1][4], "45")
    }

    function test_an_emptied_time_is_none() {
        const s = sheet()

        type(findChild(s, "prestopField"), "")

        compare(fake.calls, [["setSheet", "s1", "3", "", "2:00", "Brief links", "Nachtdreh"]])
    }

    function test_a_text_field_sends_the_whole_sheet() {
        const s = sheet()

        type(findChild(s, "noteField"), "Früh")

        compare(fake.calls, [["setSheet", "s1", "3", "1:30", "2:00", "Brief links", "Früh"]])
    }

    function test_a_time_the_session_refuses_is_marked() {
        const s = sheet()

        type(findChild(s, "prestopField"), "1:99")

        verify(findChild(s, "prestopField").invalid)
        verify(!findChild(s, "plannedField").invalid)
    }

    function test_a_refused_planned_time_marks_only_itself() {
        const s = sheet()

        type(findChild(s, "plannedField"), "1:99")

        verify(findChild(s, "plannedField").invalid)
        verify(!findChild(s, "prestopField").invalid)
    }

    function test_nothing_changed_sends_nothing() {
        const s = sheet()

        type(findChild(s, "dayField"), "3")
        type(findChild(s, "prestopField"), "1:30")

        compare(fake.calls.length, 0)
    }

    function test_speaking_figures_stay_and_the_others_can_leave() {
        const s = sheet()
        const anna = findChild(s, "speaking.anna")
        const fischer = findChild(s, "present.fischer")

        verify(!anna.removable)
        verify(fischer.removable)
        compare(fischer.text, "Fischer ×3")

        fischer.removeRequested()
        compare(fake.calls, [["removeSceneCharacter", "s1", "fischer"]])
    }

    function test_the_figure_box_offers_the_figures_not_in_the_scene() {
        const s = sheet()
        const box = findChild(s, "figureBox")

        compare(box.model, ["Bert"])
        box.activated(0)

        compare(fake.calls, [["addSceneCharacter", "s1", "bert"]])
    }

    function test_only_categories_with_elements_show() {
        const s = sheet()

        verify(findChild(s, "category.prop") !== null)
        verify(findChild(s, "category.costume") === null)
        compare(findChild(s, "element.e1").text, "Brief")
    }

    function test_the_cross_of_an_element_takes_it_out_of_the_scene() {
        const s = sheet()

        findChild(s, "element.e1").removeRequested()

        compare(fake.calls, [["removeSceneElement", "s1", "e1"]])
    }

    // Enter makes the typed element: no suggestion fits, so the "make" row is the one chosen.
    function test_enter_in_a_category_field_adds_the_typed_element() {
        const s = sheet()
        const field = findChild(s, "add.prop")

        field.forceActiveFocus()
        for (const letter of "Zeitung") {
            keyClick(letter)
        }
        keyClick(Qt.Key_Return)

        compare(fake.calls, [["addElementToScene", "s1", "prop", "Zeitung"]])
    }

    function test_the_field_offers_the_elements_of_its_category_the_scene_lacks() {
        const s = sheet()

        compare(findChild(s, "add.prop").model, ["Kerze"])
    }

    function test_a_suggestion_of_the_field_is_added_by_its_name() {
        const s = sheet()

        findChild(s, "add.prop").activated(0)

        compare(fake.calls, [["addElementToScene", "s1", "prop", "Kerze"]])
    }

    function test_add_category_opens_an_empty_row() {
        const s = sheet()
        const box = findChild(s, "categoryBox")

        compare(box.model[0], "Kostüm")
        box.activated(0)

        verify(findChild(s, "category.costume") !== null)
        verify(findChild(s, "add.costume") !== null)
        compare(fake.calls.length, 0)
        verify(!box.model.includes("Kostüm"))
    }

    function test_a_suggestion_is_taken_or_turned_down() {
        const s = sheet()
        const row = findChild(s, "suggestion.e2")

        compare(findChild(row, "suggestionChip").text, "Kerze")
        verify(findChild(row, "suggestionChip").toolTip.includes("aus dem Set: Kerzenleuchter"))

        findChild(row, "take").clicked()
        findChild(row, "suggestionChip").removeRequested()

        compare(fake.calls, [["takeSuggestion", "s1", "e2"], ["excludeSuggestion", "s1", "e2"]])
    }

    // D-23: an element whose spot the text lost says so; × drops the hint, not the element.
    function test_a_lost_spot_is_shown_and_can_be_dropped() {
        const s = sheet()

        verify(findChild(s, "lost.e1").visible)
        compare(findChild(s, "lost.e1").text, "Stelle nicht mehr im Text")
        findChild(s, "forget.e1").clicked()

        compare(fake.calls, [["forgetLost", "s1", "e1"]])
    }

    // D-23: the module shows the scene's text beside the sheet.
    function test_the_scene_text_stands_beside_the_sheet() {
        const v = view()

        verify(findChild(v, "sceneText").visible)
        compare(findChild(v, "textLine.0").getText(0, 100), "Anna liest den Brief.")
    }

    // D-23: a suggestion without an object comes from the scene's text.
    function test_a_suggestion_from_the_text_says_so() {
        const row = findChild(sheet(), "suggestion.e3")

        verify(findChild(row, "suggestionChip").toolTip.includes("steht im Text der Szene"))
    }

    function test_without_a_scene_the_sheet_says_so() {
        fake.projectJson = JSON.stringify({ scenes: [], element_categories: [], elements: [], sheet: null })
        const s = sheet()

        verify(findChild(s, "noScene").visible)
        verify(!findChild(s, "sheetTitle").visible)
    }

    function showElements(v) {
        findChild(v, "modeSwitch").activated(1)
        verify(findChild(v, "elementList").visible)
    }

    function row(v, id) {
        const rows = findChild(v, "elementRows")
        tryVerify(() => findChild(rows, "element." + id) !== null)
        return findChild(rows, "element." + id)
    }

    function test_the_switch_shows_the_elements_by_category() {
        const v = view()
        verify(findChild(v, "sceneList").visible)
        verify(!findChild(v, "elementList").visible)

        showElements(v)

        verify(!findChild(v, "sceneList").visible)
        verify(!findChild(v, "sheet").visible)
        compare(findChild(findChild(v, "elementRows"), "element.e3").parent !== null, true)
        // Kostüm before Requisite, the names in order.
        compare(findChild(v, "elementList").rows.map(e => e.id), ["e3", "e1", "e2"])
    }

    function test_without_a_chosen_element_the_form_says_what_to_do() {
        const v = view()
        showElements(v)

        const hint = findChild(v, "elementHint")
        verify(hint.visible)
        compare(hint.title, "Links ein Element wählen oder mit „Neu…“ anlegen.")
        verify(!findChild(v, "elementForm").visible)

        row(v, "e2").clicked()
        verify(!hint.visible)
    }

    function test_an_element_shows_in_the_form_and_applies() {
        const v = view()
        showElements(v)

        row(v, "e2").clicked()
        const form = findChild(v, "elementForm")
        compare(findChild(form, "nameField").text, "Kerze")
        compare(findChild(form, "descriptionField").text, "Wachs")
        compare(findChild(form, "statusBox").currentIndex, 1)
        compare(findChild(form, "categoryBox").currentIndex, 4)

        findChild(form, "nameField").text = "Kerzen"
        findChild(form, "statusBox").select(2)
        findChild(form, "apply").clicked()

        compare(fake.calls, [["editElement", "e2", "prop", "Kerzen", "Wachs", "ready", "p3"]])
    }

    function test_an_element_is_deleted_after_asking() {
        const v = view()
        showElements(v)

        row(v, "e1").clicked()
        const form = findChild(v, "elementForm")
        findChild(form, "remove").clicked()
        compare(fake.calls.length, 0)
        findChild(form, "confirmRemove").clicked()

        compare(fake.calls, [["removeElement", "e1"]])
    }

    function test_the_scenes_of_an_element_show_the_scene() {
        const v = view()
        showElements(v)

        row(v, "e3").clicked()
        findChild(findChild(v, "elementForm"), "scene.s2").clicked()

        compare(fake.calls, [["selectScene", "s2"]])
        compare(v.mode, "scenes")
        verify(findChild(v, "sheet").visible)
    }

    // Two scenes may carry one number (DB-33): the chip jumps to the scene by its id.
    function test_a_scene_chip_jumps_to_its_scene_when_numbers_repeat() {
        const project = JSON.parse(fake.fixture)
        project.scenes.push({ id: "s3", number: "2", heading: "EXT. HOF – TAG", inBook: true,
            plans: 0, shots: 0, length: "" })
        project.elements[2].scenes = [{ id: "s3", number: "2" }]
        fake.projectJson = JSON.stringify(project)
        const v = view()
        showElements(v)

        row(v, "e3").clicked()
        findChild(findChild(v, "elementForm"), "scene.s3").clicked()

        compare(fake.calls, [["selectScene", "s3"]])
    }

    function test_new_makes_an_element_with_a_category() {
        const v = view()
        showElements(v)

        findChild(v, "newElement").clicked()
        const form = findChild(v, "elementForm")
        compare(form.title, "Element anlegen")
        verify(!findChild(form, "remove").visible)

        findChild(form, "nameField").text = "Zeitung"
        findChild(form, "categoryBox").currentIndex = 4
        findChild(form, "apply").clicked()

        compare(fake.calls, [["addElement", "prop", "Zeitung"]])
        compare(v.elementId, "e9")
        verify(!v.creating)
    }

    // D-26: the responsible person. The persons of the department of the category come first.
    function test_the_responsible_box_lists_the_matching_department_first() {
        const v = view()
        showElements(v)

        row(v, "e2").clicked()
        const box = findChild(findChild(v, "elementForm"), "responsibleBox")

        compare(box.model, ["keine", "Mia Rot", "Jo Berg", "Ole Blau"])
        compare(box.currentIndex, 3)
    }

    function test_an_element_without_a_responsible_person_shows_none() {
        const v = view()
        showElements(v)

        row(v, "e1").clicked()

        compare(findChild(findChild(v, "elementForm"), "responsibleBox").currentIndex, 0)
    }

    function test_choosing_a_person_sends_the_responsible_with_the_element() {
        const v = view()
        showElements(v)

        row(v, "e1").clicked()
        const form = findChild(v, "elementForm")
        findChild(form, "responsibleBox").currentIndex = 1
        compare(fake.calls.length, 0)
        findChild(form, "apply").clicked()

        compare(fake.calls, [["editElement", "e1", "prop", "Brief", "", "open", "p2"]])
    }

    function test_none_takes_the_responsible_person_away() {
        const v = view()
        showElements(v)

        row(v, "e2").clicked()
        const form = findChild(v, "elementForm")
        findChild(form, "responsibleBox").currentIndex = 0
        findChild(form, "apply").clicked()

        compare(fake.calls, [["editElement", "e2", "prop", "Kerze", "Wachs", "working", ""]])
    }

    function test_an_unchanged_responsible_person_is_sent_as_it_is() {
        const v = view()
        showElements(v)

        row(v, "e2").clicked()
        findChild(findChild(v, "elementForm"), "apply").clicked()

        compare(fake.calls, [["editElement", "e2", "prop", "Kerze", "Wachs", "working", "p3"]])
    }

    function test_without_persons_the_form_says_where_they_are_made() {
        const project = JSON.parse(fake.fixture)
        project.persons = []
        project.elements[1].responsible = null
        fake.projectJson = JSON.stringify(project)
        const v = view()
        showElements(v)

        row(v, "e2").clicked()
        const form = findChild(v, "elementForm")

        verify(findChild(form, "noPersons").visible)
        compare(findChild(form, "noPersons").text, "Noch keine Personen; sie entstehen im Modul Team.")
    }
}
