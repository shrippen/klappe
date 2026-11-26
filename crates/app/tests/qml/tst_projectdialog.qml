import QtQuick
import QtTest
import "../../qml"

// ProjectDialog (LH-01 to LH-03, D-08): what its lists show and what its forms beside them
// send to the controller; a change that alters the script says so first, a delete asks.
TestCase {
    id: tc
    name: "ProjectDialog"
    width: 1000
    height: 800
    when: windowShown
    visible: true

    QtObject {
        id: fake
        property string status: ""
        property string sceneId: "m4djbtj6"
        property bool accept: true
        property var calls: []
        // What a rename would change in the script text (`sceneChanges`, `characterChanges`).
        property var changes: []
        property string projectJson: fixture
        readonly property string fixture: JSON.stringify({
            scenes: [
                { id: "m4djbtj6", number: "12", heading: "INT. SÄNGERSAAL – TAG", location: "saengersaal", locationName: "Sängersaal", ownLocation: false, place: "Sängersaal" },
                { id: "szene-013", number: "13", heading: "EXT. HOF", location: "hof", locationName: "Burghof", ownLocation: true, place: "Burg" }
            ],
            places: [
                { id: "saal", name: "Sängersaal", variants: ["SAAL"], location: "saengersaal", location_name: "Sängersaal", scenes: ["12"] },
                { id: "burg", name: "Burg", variants: [], location: "", location_name: "", scenes: ["13", "14"] }
            ],
            locations: [
                { id: "hof", name: "Burghof", width: 30, depth: 20.5, height: 3.2, scenes: 1 },
                { id: "saengersaal", name: "Sängersaal", width: 20, depth: 13, height: 2.6, scenes: 1 }
            ],
            numbering: "scene",
            papers: [{ id: "a4", label: "A4" }, { id: "a3", label: "A3" }, { id: "letter", label: "Letter" }],
            script_papers: ["a4", "letter"],
            script: { language: "de", words: "local", paper: "letter", contd_after_action: false, scene_numbering: "locked", first_scene: 1 },
            numberings: ["scene", "camera", "group"],
            characters: [
                { id: "elisabeth", name: "Elisabeth", short: "E", color: "purple", group: 0, size: null, aliases: [] },
                { id: "burgvogt", name: "Burgvogt", short: "B", color: "blue", group: 0, size: 2.4, aliases: [] },
                { id: "gaeste", name: "Gäste", short: "G", color: "green", group: 12, size: null, aliases: [] }
            ]
        })

        // Every call is noted; a refusal sets the message the way the controller does.
        function call(name, args) {
            calls.push([name].concat(args))
            if (!accept) {
                status = "Refused"
            }
            return accept
        }
        // The session's rules, as far as the tests need them.
        function sameFigureName(a, b) { return a.trim().toUpperCase() === b.trim().toUpperCase() }
        function samePlaceName(a, b) { return a.split(/\s+/).join(" ").toUpperCase() === b.split(/\s+/).join(" ").toUpperCase() }
        function clearStatus() { status = "" }
        function sceneChanges(id, n, h) { return JSON.stringify(changes) }
        function characterChanges(id, n, a) { return JSON.stringify(changes) }
        function selectScene(id) { return call("selectScene", [id]) }
        function addScene(n, h, l) { return call("addScene", [n, h, l]) }
        function editScene(id, n, h, l, sun) { return call("editScene", [id, n, h, l, sun]) }
        function removeScene(id) { return call("removeScene", [id]) }
        function addLocation(n, w, d, h, site) { return call("addLocation", [n, w, d, h, site]) }
        function editLocation(id, n, w, d, h, site) { return call("editLocation", [id, n, w, d, h, site]) }
        function removeLocation(id) { return call("removeLocation", [id]) }
        function addCharacter(n, s, c, g, z) { return call("addCharacter", [n, s, c, g, z]) }
        function editCharacter(id, n, s, c, g, z, a) { return call("editCharacter", [id, n, s, c, g, z, a]) }
        function removeCharacter(id) { return call("removeCharacter", [id]) }
        function setNumbering(name) { return call("setNumbering", [name]) }
        function setScriptSetup(l, w, p, c, n, f) { return call("setScriptSetup", [l, w, p, c, n, f]) }
    }

    TestTexts { id: de; language: "de" }

    Component {
        id: dialogComponent
        ProjectDialog {
            controller: fake
            texts: de
        }
    }

    function init() {
        fake.status = ""
        fake.accept = true
        fake.calls = []
        fake.changes = []
        fake.projectJson = fake.fixture
    }

    function make() {
        const pd = createTemporaryObject(dialogComponent, tc)
        verify(pd)
        return pd
    }

    function opened(pd) {
        pd.open()
        const dialog = findChild(pd, "dialog")
        tryCompare(dialog, "opened", true)
        return dialog
    }

    // The tab `index` of the dialog `pd`: its list.
    function listOf(pd, name) {
        return findChild(pd, name)
    }

    function showTab(pd, index) {
        findChild(pd, "tabs").activated(index)
        wait(0)
    }

    // Button `name` of the form `form`.
    function press(form, name) {
        findChild(form, name).clicked()
    }

    // Selects the entry `id` of the list `list`, which shows its form.
    function select(pd, list, form, id) {
        listOf(pd, list).currentId = id
        const f = findChild(pd, form)
        verify(f.visible)
        return f
    }

    function test_the_lists_show_their_rows() {
        const pd = make()
        opened(pd)

        compare(findChild(listOf(pd, "sceneList"), "rows").count, 2)
        compare(findChild(listOf(pd, "locationList"), "rows").count, 2)
        compare(findChild(listOf(pd, "characterList"), "rows").count, 3)
        verify(!findChild(listOf(pd, "sceneList"), "empty").visible)
    }

    function test_the_scene_on_view_is_selected_and_shown_on_request() {
        const pd = make()
        const dialog = opened(pd)
        const form = findChild(pd, "sceneForm")

        compare(listOf(pd, "sceneList").currentId, "m4djbtj6")
        verify(form.visible)
        compare(findChild(form, "numberField").text, "12")

        select(pd, "sceneList", "sceneForm", "szene-013")
        press(form, "show")

        compare(fake.calls, [["selectScene", "szene-013"]])
        tryCompare(dialog, "opened", false)
    }

    function test_the_form_waits_for_a_selection() {
        const pd = make()
        opened(pd)
        const form = findChild(pd, "locationForm")
        showTab(pd, 2)

        verify(!form.visible)
        verify(findChild(pd, "pick").visible)
        listOf(pd, "locationList").currentId = "hof"
        verify(form.visible)
        verify(!findChild(pd, "pick").visible)
        verify(findChild(form, "remove").visible)
    }

    function test_the_list_and_the_form_stand_side_by_side() {
        const pd = make()
        opened(pd)
        const list = listOf(pd, "sceneList")
        const form = findChild(pd, "sceneForm")

        const left = list.mapToItem(null, 0, 0)
        const right = form.mapToItem(null, 0, 0)
        verify(left.x + list.width <= right.x)
        // No dialog opens over this one for the form.
        verify(form.parent !== null)
        verify(!findChild(pd, "confirm"))
    }

    function test_a_new_scene_needs_a_number_and_sends_its_fields() {
        const pd = make()
        opened(pd)
        const form = findChild(pd, "sceneForm")

        press(listOf(pd, "sceneList"), "add")
        verify(form.visible)
        compare(form.title, "Neue Szene")
        verify(!findChild(form, "apply").enabled)
        verify(!findChild(form, "remove").visible)

        findChild(form, "numberField").text = "14"
        findChild(form, "headingField").text = "INT. TURM"
        verify(findChild(form, "apply").enabled)
        press(form, "apply")

        compare(fake.calls, [["addScene", "14", "INT. TURM", ""]])
        // Done: the form for a new entry closes.
        verify(!form.visible)
    }

    function test_a_second_new_entry_starts_empty() {
        const pd = make()
        opened(pd)
        const form = findChild(pd, "sceneForm")

        press(listOf(pd, "sceneList"), "add")
        findChild(form, "numberField").text = "14"
        press(form, "apply")
        press(listOf(pd, "sceneList"), "add")

        compare(findChild(form, "numberField").text, "")
    }

    function test_changing_only_the_heading_edits_without_moving() {
        const pd = make()
        opened(pd)
        const form = findChild(pd, "sceneForm")

        compare(findChild(form, "numberField").text, "12")
        // Follows its scripted location.
        compare(findChild(form, "locationBox").currentIndex, 0)

        findChild(form, "headingField").text = "INT. SÄNGERSAAL – NACHT"
        press(form, "apply")

        compare(fake.calls, [["editScene", "m4djbtj6", "12", "INT. SÄNGERSAAL – NACHT", "", '["","",""]']])
    }

    function test_another_location_is_one_edit() {
        const pd = make()
        opened(pd)
        const form = findChild(pd, "sceneForm")

        findChild(form, "locationBox").currentIndex = 1
        press(form, "apply")

        compare(fake.calls, [["editScene", "m4djbtj6", "12", "INT. SÄNGERSAAL – TAG", "hof", '["","",""]']])
    }

    // Another location keeps the plans on their set with their object ways (DB-51): the form
    // says nothing of dropped ways, and needs no location to exist first.
    function test_another_location_drops_no_object_ways() {
        const pd = make()
        opened(pd)
        const form = findChild(pd, "sceneForm")

        findChild(form, "locationBox").currentIndex = 1
        compare(findChild(form, "waysDropped"), null)
        compare(findChild(form, "needLocation"), null)
    }

    function test_a_refusal_keeps_the_form_with_the_message() {
        const pd = make()
        opened(pd)
        const form = findChild(pd, "sceneForm")
        fake.accept = false

        press(listOf(pd, "sceneList"), "add")
        findChild(form, "numberField").text = "12"
        press(form, "apply")

        verify(form.visible)
        verify(findChild(form, "error").visible)
        compare(findChild(form, "error").text, "Refused")
        compare(findChild(form, "numberField").text, "12")
    }

    function test_a_form_opens_without_the_last_message() {
        const pd = make()
        opened(pd)
        const form = findChild(pd, "sceneForm")
        fake.status = "Old"

        press(listOf(pd, "sceneList"), "add")

        verify(!findChild(form, "error").visible)
    }

    function test_a_location_form_sends_the_text_of_its_sizes() {
        const pd = make()
        opened(pd)
        const form = findChild(pd, "locationForm")
        showTab(pd, 2)

        press(listOf(pd, "locationList"), "add")
        verify(form.visible)
        compare(findChild(form, "widthField").text, "10")
        compare(findChild(form, "depthField").text, "8")
        compare(findChild(form, "roomHeightField").text, "2,6")
        findChild(form, "nameField").text = "Keller"
        findChild(form, "widthField").text = "6,5"
        findChild(form, "roomHeightField").text = "3"
        press(form, "apply")

        compare(fake.calls, [["addLocation", "Keller", "6,5", "8", "3", '["","",""]']])
    }

    function test_a_location_edit_starts_with_its_values() {
        const pd = make()
        opened(pd)
        showTab(pd, 2)

        const form = select(pd, "locationList", "locationForm", "hof")

        compare(findChild(form, "nameField").text, "Burghof")
        compare(findChild(form, "widthField").text, "30")
        compare(findChild(form, "depthField").text, "20,5")
        compare(findChild(form, "roomHeightField").text, "3,2")
        press(form, "apply")
        compare(fake.calls, [["editLocation", "hof", "Burghof", "30", "20,5", "3,2", '["","",""]']])
    }

    // LH-52: the place of a location and its north are text; blank is no place.
    function test_a_location_form_sends_its_place_and_north() {
        const project = JSON.parse(fake.fixture)
        project.locations[0].latitude = 52.52
        project.locations[0].longitude = 13.405
        project.locations[0].north = 90
        fake.projectJson = JSON.stringify(project)
        const pd = make()
        opened(pd)
        showTab(pd, 2)

        const form = select(pd, "locationList", "locationForm", "hof")
        compare(findChild(form, "latitudeField").text, "52,52")
        compare(findChild(form, "longitudeField").text, "13,405")
        compare(findChild(form, "northField").text, "90")

        findChild(form, "northField").text = "-30"
        press(form, "apply")
        compare(fake.calls, [["editLocation", "hof", "Burghof", "30", "20,5", "3,2", '["52,52","13,405","-30"]']])

        const other = select(pd, "locationList", "locationForm", "saengersaal")
        compare(findChild(other, "latitudeField").text, "")
        compare(findChild(other, "northField").text, "")
    }

    // LH-52: the form shows the stored place without loss, so a rename writes it back unchanged.
    function test_renaming_a_location_keeps_its_place_exactly() {
        const project = JSON.parse(fake.fixture)
        project.locations[0].latitude = 52.520008
        project.locations[0].longitude = 12.34
        project.locations[0].north = 33.25
        fake.projectJson = JSON.stringify(project)
        const pd = make()
        opened(pd)
        showTab(pd, 2)

        const form = select(pd, "locationList", "locationForm", "hof")
        findChild(form, "nameField").text = "Burghof 2"
        press(form, "apply")
        compare(fake.calls, [["editLocation", "hof", "Burghof 2", "30", "20,5", "3,2", '["52,520008","12,34","33,25"]']])
    }

    // LH-52: date, time and offset of a scene are sent as typed.
    function test_a_scene_form_sends_its_date_time_and_offset() {
        const project = JSON.parse(fake.fixture)
        project.scenes[0].sunDate = "2026-06-21"
        project.scenes[0].sunTime = "14:30"
        project.scenes[0].sunOffset = "+02:00"
        fake.projectJson = JSON.stringify(project)
        const pd = make()
        opened(pd)
        const form = findChild(pd, "sceneForm")

        compare(findChild(form, "sunDateField").text, "2026-06-21")
        compare(findChild(form, "sunTimeField").text, "14:30")
        compare(findChild(form, "sunOffsetField").text, "+02:00")

        findChild(form, "sunTimeField").text = "09:15"
        press(form, "apply")
        compare(fake.calls, [["editScene", "m4djbtj6", "12", "INT. SÄNGERSAAL – TAG", "", '["2026-06-21","09:15","+02:00"]']])
    }

    function test_a_new_character_gets_a_free_colour_and_a_label() {
        const pd = make()
        opened(pd)
        const form = findChild(pd, "characterForm")
        showTab(pd, 3)

        press(listOf(pd, "characterList"), "add")
        verify(form.visible)
        // Purple, blue and green are taken; red is the first label colour left.
        compare(findChild(form, "colorPicker").currentIndex, 0)

        findChild(form, "nameField").text = "graf"
        findChild(form, "nameField").textEdited()
        compare(findChild(form, "shortField").text, "G")
        press(form, "apply")

        compare(fake.calls, [["addCharacter", "graf", "G", "red", 0, ""]])
    }

    function test_a_character_edit_keeps_its_colour_and_picks_another() {
        const pd = make()
        opened(pd)
        showTab(pd, 3)

        const form = select(pd, "characterList", "characterForm", "elisabeth")
        compare(findChild(form, "colorPicker").currentIndex, 6)

        findChild(form, "colorPicker").picked(3)
        press(form, "apply")

        compare(fake.calls, [["editCharacter", "elisabeth", "Elisabeth", "E", "green", 0, "", "[]"]])
    }

    function test_a_new_group_sends_its_head_count() {
        const pd = make()
        opened(pd)
        const form = findChild(pd, "characterForm")
        showTab(pd, 3)

        press(listOf(pd, "characterList"), "add")
        compare(findChild(form, "groupSwitch").checked, false)
        compare(findChild(form, "groupSize").visible, false)

        findChild(form, "nameField").text = "Wachen"
        findChild(form, "groupSwitch").checked = true
        compare(findChild(form, "groupSize").visible, true)
        findChild(form, "groupSize").value = 6
        press(form, "apply")

        compare(fake.calls[0][4], 6)
    }

    function test_a_group_edit_starts_from_its_count_and_can_become_single() {
        const pd = make()
        opened(pd)
        showTab(pd, 3)

        const form = select(pd, "characterList", "characterForm", "gaeste")
        compare(findChild(form, "groupSwitch").checked, true)
        compare(findChild(form, "groupSize").value, 12)

        findChild(form, "groupSwitch").checked = false
        press(form, "apply")

        compare(fake.calls, [["editCharacter", "gaeste", "Gäste", "G", "green", 0, "", "[]"]])
    }

    // LH-25: the size is typed in metres; empty is the standard.
    function test_a_new_figure_sends_the_typed_size() {
        const pd = make()
        opened(pd)
        const form = findChild(pd, "characterForm")
        showTab(pd, 3)

        press(listOf(pd, "characterList"), "add")
        compare(findChild(form, "sizeField").text, "")

        findChild(form, "nameField").text = "Kutsche"
        findChild(form, "sizeField").text = "4,5"
        press(form, "apply")

        compare(fake.calls[0][5], "4,5")
    }

    function test_a_figure_edit_starts_from_its_size_and_can_return_to_standard() {
        const pd = make()
        opened(pd)
        showTab(pd, 3)

        const form = select(pd, "characterList", "characterForm", "burgvogt")
        compare(findChild(form, "sizeField").text, de.num(2.4, 2))

        findChild(form, "sizeField").text = ""
        press(form, "apply")

        compare(fake.calls[0].slice(0, 2), ["editCharacter", "burgvogt"])
        compare(fake.calls[0][6], "")
    }

    function test_the_group_size_is_limited_to_two_to_999() {
        const pd = make()
        opened(pd)
        const form = findChild(pd, "characterForm")
        showTab(pd, 3)

        press(listOf(pd, "characterList"), "add")

        compare(findChild(form, "groupSize").from, 2)
        compare(findChild(form, "groupSize").to, 999)
    }

    function test_the_short_label_has_room_for_three_letters() {
        const pd = make()
        opened(pd)
        const form = findChild(pd, "characterForm")
        showTab(pd, 3)

        press(listOf(pd, "characterList"), "add")

        compare(findChild(form, "shortField").maximumLength, 3)
    }

    // D-08: a change that alters the script text says what changes there, and waits.
    function test_a_rename_warns_before_it_changes_the_script() {
        const pd = make()
        opened(pd)
        showTab(pd, 3)
        const form = select(pd, "characterList", "characterForm", "elisabeth")
        fake.changes = [{ kind: "cue", from: "ELISABETH", to: "ELISE", count: 3 }]

        findChild(form, "nameField").text = "Elise"
        press(form, "apply")

        const warning = findChild(form, "warning")
        verify(warning.visible)
        verify(warning.text.indexOf("3 Stellen: ELISABETH → ELISE") >= 0)
        verify(warning.text.indexOf("Rückgängig") >= 0)
        compare(fake.calls, [])
        verify(!findChild(form, "apply").enabled)

        press(form, "cancelChange")
        verify(!warning.visible)
        compare(fake.calls, [])

        press(form, "apply")
        press(form, "confirmChange")
        compare(fake.calls, [["editCharacter", "elisabeth", "Elise", "E", "purple", 0, "", "[]"]])
        verify(!warning.visible)
    }

    // The warning describes the fields as they were: typing more would change what is applied.
    function test_the_fields_wait_while_the_warning_stands() {
        const pd = make()
        opened(pd)
        showTab(pd, 3)
        const form = select(pd, "characterList", "characterForm", "elisabeth")
        fake.changes = [{ kind: "cue", from: "ELISABETH", to: "ELISE", count: 3 }]
        const field = findChild(form, "nameField")
        verify(field.enabled)

        field.text = "Elise"
        press(form, "apply")
        verify(findChild(form, "warning").visible)
        verify(!field.enabled)

        press(form, "cancelChange")
        verify(field.enabled)
    }

    function test_a_new_scene_head_names_heading_and_number() {
        const pd = make()
        opened(pd)
        const form = findChild(pd, "sceneForm")
        fake.changes = [
            { kind: "heading", from: "INT. SÄNGERSAAL – TAG", to: "INT. SÄNGERSAAL – NACHT", count: 1 },
            { kind: "number", from: "#12#", to: "#12A#", count: 1 }
        ]

        findChild(form, "numberField").text = "12A"
        findChild(form, "headingField").text = "INT. SÄNGERSAAL – NACHT"
        press(form, "apply")

        const text = findChild(form, "warning").text
        verify(text.indexOf("Szenenkopf: INT. SÄNGERSAAL – TAG → INT. SÄNGERSAAL – NACHT") >= 0)
        verify(text.indexOf("Szenennummer: #12# → #12A#") >= 0)
        compare(fake.calls, [])

        press(form, "confirmChange")
        compare(fake.calls.length, 1)
        compare(fake.calls[0][0], "editScene")
    }

    function test_one_place_reads_in_the_singular() {
        const pd = make()
        opened(pd)
        showTab(pd, 3)
        const form = select(pd, "characterList", "characterForm", "burgvogt")
        fake.changes = [{ kind: "cue", from: "BURGVOGT", to: "VOGT", count: 1 }]

        findChild(form, "nameField").text = "Vogt"
        press(form, "apply")

        verify(findChild(form, "warning").text.indexOf("1 Stelle: BURGVOGT → VOGT") >= 0)
    }

    function test_no_warning_without_a_change_in_the_text() {
        const pd = make()
        opened(pd)
        showTab(pd, 3)
        const form = select(pd, "characterList", "characterForm", "elisabeth")

        findChild(form, "colorPicker").picked(3)
        press(form, "apply")

        verify(!findChild(form, "warning").visible)
        compare(fake.calls.length, 1)
    }

    function test_deleting_asks_first() {
        const pd = make()
        opened(pd)
        const form = select(pd, "sceneList", "sceneForm", "szene-013")
        const ask = findChild(form, "removeAsk")

        press(form, "remove")
        verify(ask.visible)
        compare(fake.calls, [])
        compare(ask.text,
                "Szene 13 EXT. HOF löschen? Ihre Pläne gehen verloren; Rückgängig stellt sie wieder her.")

        press(form, "cancelRemove")
        verify(!ask.visible)
        compare(fake.calls, [])

        press(form, "remove")
        press(form, "confirmRemove")
        compare(fake.calls, [["removeScene", "szene-013"]])
    }

    function test_each_list_deletes_its_own_kind() {
        const pd = make()
        opened(pd)

        showTab(pd, 2)
        const place = select(pd, "locationList", "locationForm", "hof")
        press(place, "remove")
        compare(findChild(place, "removeAsk").text, "Drehort Burghof löschen?")
        press(place, "confirmRemove")

        showTab(pd, 3)
        const person = select(pd, "characterList", "characterForm", "burgvogt")
        press(person, "remove")
        compare(findChild(person, "removeAsk").text, "Figur Burgvogt löschen?")
        press(person, "confirmRemove")

        compare(fake.calls, [["removeLocation", "hof"], ["removeCharacter", "burgvogt"]])
    }

    function test_a_refused_delete_shows_the_message_in_the_form() {
        const pd = make()
        opened(pd)
        fake.accept = false
        showTab(pd, 2)

        const form = select(pd, "locationList", "locationForm", "hof")
        press(form, "remove")
        press(form, "confirmRemove")

        verify(findChild(pd, "dialog").opened)
        verify(form.visible)
        compare(findChild(form, "error").text, "Refused")
    }

    // The script's cards hand an entry on: the dialog opens on it.
    function test_it_opens_on_the_entry_of_a_card() {
        const pd = make()

        pd.openAt("character", "burgvogt")
        tryCompare(findChild(pd, "dialog"), "opened", true)

        compare(findChild(pd, "tabs").currentIndex, 3)
        compare(listOf(pd, "characterList").currentId, "burgvogt")
        const form = findChild(pd, "characterForm")
        verify(form.visible)
        compare(findChild(form, "nameField").text, "Burgvogt")

        pd.openAt("scene", "szene-013")
        compare(findChild(pd, "tabs").currentIndex, 0)
        compare(findChild(findChild(pd, "sceneForm"), "headingField").text, "EXT. HOF")
    }

    function test_the_lists_follow_the_project() {
        const pd = make()
        opened(pd)

        fake.projectJson = JSON.stringify({ scenes: [], locations: [], characters: [] })

        verify(findChild(listOf(pd, "sceneList"), "empty").visible)
        compare(findChild(listOf(pd, "sceneList"), "rows").count, 0)
        compare(findChild(listOf(pd, "characterList"), "empty").text, "Noch keine Figuren.")
    }

    // LH-76: the scheme of the shot numbers is a project setting, named with its example.
    function test_the_numbering_scheme_is_chosen_in_the_project() {
        const pd = make()
        opened(pd)
        const box = findChild(pd, "numberingBox")

        compare(box.count, 3)
        compare(box.currentIndex, 0)
        verify(box.textAt(1).indexOf("12A.3") >= 0)

        box.currentIndex = 2
        box.activated(2)

        compare(fake.calls, [["setNumbering", "group"]])
    }

    // DN-05, DB-31, DB-33: the project form has the screenplay's setup, in the papers it offers.
    function test_the_screenplay_setup_is_in_the_project_form() {
        const pd = make()
        opened(pd)
        const form = findChild(pd, "scriptSetup")

        compare(findChild(form, "paperBox").model, ["A4", "Letter"])
        compare(findChild(form, "paperBox").currentIndex, 1)

        findChild(form, "paperBox").activated(0)
        compare(fake.calls, [["setScriptSetup", "de", "local", "a4", "off", "locked", 1]])
    }
}
