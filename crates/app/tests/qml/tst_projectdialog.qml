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
        property string sceneId: "szene-012"
        property bool accept: true
        property var calls: []
        // What a rename would change in the script text (`sceneChanges`, `characterChanges`).
        property var changes: []
        property string projectJson: fixture
        readonly property string fixture: JSON.stringify({
            scenes: [
                { id: "szene-012", number: "12", heading: "INT. SÄNGERSAAL – TAG", location: "saengersaal", locationName: "Sängersaal" },
                { id: "szene-013", number: "13", heading: "EXT. HOF", location: "hof", locationName: "Burghof" }
            ],
            locations: [
                { id: "hof", name: "Burghof", width: 30, depth: 20.5, scenes: 1 },
                { id: "saengersaal", name: "Sängersaal", width: 20, depth: 13, scenes: 1 }
            ],
            characters: [
                { id: "elisabeth", name: "Elisabeth", short: "E", color: "purple", group: 0 },
                { id: "burgvogt", name: "Burgvogt", short: "B", color: "blue", group: 0 },
                { id: "gaeste", name: "Gäste", short: "G", color: "green", group: 12 }
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
        function clearStatus() { status = "" }
        function sceneChanges(id, n, h) { return JSON.stringify(changes) }
        function characterChanges(id, n) { return JSON.stringify(changes) }
        function selectScene(id) { return call("selectScene", [id]) }
        function addScene(n, h, l) { return call("addScene", [n, h, l]) }
        function editScene(id, n, h, l) { return call("editScene", [id, n, h, l]) }
        function removeScene(id) { return call("removeScene", [id]) }
        function addLocation(n, w, d) { return call("addLocation", [n, w, d]) }
        function editLocation(id, n, w, d) { return call("editLocation", [id, n, w, d]) }
        function removeLocation(id) { return call("removeLocation", [id]) }
        function addCharacter(n, s, c, g) { return call("addCharacter", [n, s, c, g]) }
        function editCharacter(id, n, s, c, g) { return call("editCharacter", [id, n, s, c, g]) }
        function removeCharacter(id) { return call("removeCharacter", [id]) }
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

        compare(listOf(pd, "sceneList").currentId, "szene-012")
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
        showTab(pd, 1)

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

        compare(fake.calls, [["addScene", "14", "INT. TURM", "hof"]])
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
        compare(findChild(form, "locationBox").currentIndex, 1)

        findChild(form, "headingField").text = "INT. SÄNGERSAAL – NACHT"
        press(form, "apply")

        compare(fake.calls, [["editScene", "szene-012", "12", "INT. SÄNGERSAAL – NACHT", "saengersaal"]])
    }

    function test_another_location_is_one_edit() {
        const pd = make()
        opened(pd)
        const form = findChild(pd, "sceneForm")

        findChild(form, "locationBox").currentIndex = 0
        press(form, "apply")

        compare(fake.calls, [["editScene", "szene-012", "12", "INT. SÄNGERSAAL – TAG", "hof"]])
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
        showTab(pd, 1)

        press(listOf(pd, "locationList"), "add")
        verify(form.visible)
        compare(findChild(form, "widthField").text, "10")
        compare(findChild(form, "depthField").text, "8")
        findChild(form, "nameField").text = "Keller"
        findChild(form, "widthField").text = "6,5"
        press(form, "apply")

        compare(fake.calls, [["addLocation", "Keller", "6,5", "8"]])
    }

    function test_a_location_edit_starts_with_its_values() {
        const pd = make()
        opened(pd)
        showTab(pd, 1)

        const form = select(pd, "locationList", "locationForm", "hof")

        compare(findChild(form, "nameField").text, "Burghof")
        compare(findChild(form, "widthField").text, "30")
        compare(findChild(form, "depthField").text, "20,5")
        press(form, "apply")
        compare(fake.calls, [["editLocation", "hof", "Burghof", "30", "20,5"]])
    }

    function test_a_new_character_gets_a_free_colour_and_a_label() {
        const pd = make()
        opened(pd)
        const form = findChild(pd, "characterForm")
        showTab(pd, 2)

        press(listOf(pd, "characterList"), "add")
        verify(form.visible)
        // Purple, blue and green are taken; red is the first label colour left.
        compare(findChild(form, "colorPicker").currentIndex, 0)

        findChild(form, "nameField").text = "graf"
        findChild(form, "nameField").textEdited()
        compare(findChild(form, "shortField").text, "G")
        press(form, "apply")

        compare(fake.calls, [["addCharacter", "graf", "G", "red", 0]])
    }

    function test_a_character_edit_keeps_its_colour_and_picks_another() {
        const pd = make()
        opened(pd)
        showTab(pd, 2)

        const form = select(pd, "characterList", "characterForm", "elisabeth")
        compare(findChild(form, "colorPicker").currentIndex, 6)

        findChild(form, "colorPicker").picked(3)
        press(form, "apply")

        compare(fake.calls, [["editCharacter", "elisabeth", "Elisabeth", "E", "green", 0]])
    }

    function test_a_new_group_sends_its_head_count() {
        const pd = make()
        opened(pd)
        const form = findChild(pd, "characterForm")
        showTab(pd, 2)

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
        showTab(pd, 2)

        const form = select(pd, "characterList", "characterForm", "gaeste")
        compare(findChild(form, "groupSwitch").checked, true)
        compare(findChild(form, "groupSize").value, 12)

        findChild(form, "groupSwitch").checked = false
        press(form, "apply")

        compare(fake.calls, [["editCharacter", "gaeste", "Gäste", "G", "green", 0]])
    }

    function test_the_group_size_is_limited_to_two_to_999() {
        const pd = make()
        opened(pd)
        const form = findChild(pd, "characterForm")
        showTab(pd, 2)

        press(listOf(pd, "characterList"), "add")

        compare(findChild(form, "groupSize").from, 2)
        compare(findChild(form, "groupSize").to, 999)
    }

    function test_the_short_label_has_room_for_three_letters() {
        const pd = make()
        opened(pd)
        const form = findChild(pd, "characterForm")
        showTab(pd, 2)

        press(listOf(pd, "characterList"), "add")

        compare(findChild(form, "shortField").maximumLength, 3)
    }

    // D-08: a change that alters the script text says what changes there, and waits.
    function test_a_rename_warns_before_it_changes_the_script() {
        const pd = make()
        opened(pd)
        showTab(pd, 2)
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
        compare(fake.calls, [["editCharacter", "elisabeth", "Elise", "E", "purple", 0]])
        verify(!warning.visible)
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
        showTab(pd, 2)
        const form = select(pd, "characterList", "characterForm", "burgvogt")
        fake.changes = [{ kind: "cue", from: "BURGVOGT", to: "VOGT", count: 1 }]

        findChild(form, "nameField").text = "Vogt"
        press(form, "apply")

        verify(findChild(form, "warning").text.indexOf("1 Stelle: BURGVOGT → VOGT") >= 0)
    }

    function test_no_warning_without_a_change_in_the_text() {
        const pd = make()
        opened(pd)
        showTab(pd, 2)
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

        showTab(pd, 1)
        const place = select(pd, "locationList", "locationForm", "hof")
        press(place, "remove")
        compare(findChild(place, "removeAsk").text, "Drehort Burghof löschen?")
        press(place, "confirmRemove")

        showTab(pd, 2)
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
        showTab(pd, 1)

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

        compare(findChild(pd, "tabs").currentIndex, 2)
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
}
