import QtQuick
import QtTest
import "../../qml"

// ProjectDialog (LH-01 to LH-03): what its lists show and what its forms and questions send
// to the controller.
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
                { id: "elisabeth", name: "Elisabeth", short: "E", color: "purple" },
                { id: "burgvogt", name: "Burgvogt", short: "B", color: "blue" }
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
        function selectScene(id) { return call("selectScene", [id]) }
        function addScene(n, h, l) { return call("addScene", [n, h, l]) }
        function editScene(id, n, h) { return call("editScene", [id, n, h]) }
        function moveScene(id, l) { return call("moveScene", [id, l]) }
        function removeScene(id) { return call("removeScene", [id]) }
        function addLocation(n, w, d) { return call("addLocation", [n, w, d]) }
        function editLocation(id, n, w, d) { return call("editLocation", [id, n, w, d]) }
        function removeLocation(id) { return call("removeLocation", [id]) }
        function addCharacter(n, s, c) { return call("addCharacter", [n, s, c]) }
        function editCharacter(id, n, s, c) { return call("editCharacter", [id, n, s, c]) }
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

    function test_the_lists_show_their_rows() {
        const pd = make()
        opened(pd)

        compare(findChild(listOf(pd, "sceneList"), "rows").count, 2)
        compare(findChild(listOf(pd, "locationList"), "rows").count, 2)
        compare(findChild(listOf(pd, "characterList"), "rows").count, 2)
        verify(!findChild(listOf(pd, "sceneList"), "empty").visible)
    }

    function test_the_scene_on_view_is_selected_and_shown_on_request() {
        const pd = make()
        const dialog = opened(pd)
        const scenes = listOf(pd, "sceneList")

        compare(scenes.currentId, "szene-012")
        scenes.currentId = "szene-013"
        findChild(scenes, "show").clicked()

        compare(fake.calls, [["selectScene", "szene-013"]])
        tryCompare(dialog, "opened", false)
    }

    function test_buttons_wait_for_a_selection() {
        const pd = make()
        opened(pd)
        const locations = listOf(pd, "locationList")

        verify(!findChild(locations, "edit").enabled)
        verify(!findChild(locations, "remove").enabled)
        locations.currentId = "hof"
        verify(findChild(locations, "edit").enabled)
        verify(findChild(locations, "remove").enabled)
    }

    function test_a_new_scene_needs_a_number_and_sends_its_fields() {
        const pd = make()
        opened(pd)
        const form = findChild(pd, "sceneForm")

        findChild(listOf(pd, "sceneList"), "add").clicked()
        tryCompare(form, "opened", true)
        compare(form.title, "Neue Szene")
        verify(!form.acceptEnabled)

        findChild(form, "numberField").text = "14"
        findChild(form, "headingField").text = "INT. TURM"
        verify(form.acceptEnabled)
        form.accept()

        compare(fake.calls, [["addScene", "14", "INT. TURM", "hof"]])
    }

    function test_changing_only_the_heading_edits_without_moving() {
        const pd = make()
        opened(pd)
        const scenes = listOf(pd, "sceneList")
        const form = findChild(pd, "sceneForm")

        scenes.currentId = "szene-012"
        findChild(scenes, "edit").clicked()
        tryCompare(form, "opened", true)
        compare(findChild(form, "numberField").text, "12")
        compare(findChild(form, "locationBox").currentIndex, 1)

        findChild(form, "headingField").text = "INT. SÄNGERSAAL – NACHT"
        form.accept()

        compare(fake.calls, [["editScene", "szene-012", "12", "INT. SÄNGERSAAL – NACHT"]])
    }

    function test_another_location_moves_the_scene() {
        const pd = make()
        opened(pd)
        const scenes = listOf(pd, "sceneList")
        const form = findChild(pd, "sceneForm")

        scenes.currentId = "szene-012"
        findChild(scenes, "edit").clicked()
        tryCompare(form, "opened", true)
        findChild(form, "locationBox").currentIndex = 0
        form.accept()

        compare(fake.calls.map(c => c[0]), ["editScene", "moveScene"])
        compare(fake.calls[1], ["moveScene", "szene-012", "hof"])
    }

    function test_a_refusal_keeps_the_form_open_with_the_message() {
        const pd = make()
        opened(pd)
        const form = findChild(pd, "sceneForm")
        fake.accept = false

        findChild(listOf(pd, "sceneList"), "add").clicked()
        tryCompare(form, "opened", true)
        findChild(form, "numberField").text = "12"
        form.accept()

        tryCompare(form, "opened", true)
        verify(findChild(form, "error").visible)
        compare(findChild(form, "error").text, "Refused")
        compare(findChild(form, "numberField").text, "12")
    }

    function test_a_form_opens_without_the_last_message() {
        const pd = make()
        opened(pd)
        const form = findChild(pd, "sceneForm")
        fake.status = "Old"

        findChild(listOf(pd, "sceneList"), "add").clicked()
        tryCompare(form, "opened", true)

        verify(!findChild(form, "error").visible)
    }

    function test_a_location_form_sends_the_text_of_its_sizes() {
        const pd = make()
        opened(pd)
        const form = findChild(pd, "locationForm")
        showTab(pd, 1)

        findChild(listOf(pd, "locationList"), "add").clicked()
        tryCompare(form, "opened", true)
        compare(findChild(form, "widthField").text, "10")
        compare(findChild(form, "depthField").text, "8")
        findChild(form, "nameField").text = "Keller"
        findChild(form, "widthField").text = "6,5"
        form.accept()

        compare(fake.calls, [["addLocation", "Keller", "6,5", "8"]])
    }

    function test_a_location_edit_starts_with_its_values() {
        const pd = make()
        opened(pd)
        const locations = listOf(pd, "locationList")
        const form = findChild(pd, "locationForm")
        showTab(pd, 1)

        locations.currentId = "hof"
        findChild(locations, "edit").clicked()
        tryCompare(form, "opened", true)

        compare(findChild(form, "nameField").text, "Burghof")
        compare(findChild(form, "widthField").text, "30")
        compare(findChild(form, "depthField").text, "20,5")
        form.accept()
        compare(fake.calls, [["editLocation", "hof", "Burghof", "30", "20,5"]])
    }

    function test_a_new_character_gets_a_free_colour_and_a_label() {
        const pd = make()
        opened(pd)
        const form = findChild(pd, "characterForm")
        showTab(pd, 2)

        findChild(listOf(pd, "characterList"), "add").clicked()
        tryCompare(form, "opened", true)
        // Purple and blue are taken; red is the first label colour left.
        compare(findChild(form, "colorPicker").currentIndex, 0)

        findChild(form, "nameField").text = "graf"
        findChild(form, "nameField").textEdited()
        compare(findChild(form, "shortField").text, "G")
        form.accept()

        compare(fake.calls, [["addCharacter", "graf", "G", "red"]])
    }

    function test_a_character_edit_keeps_its_colour_and_picks_another() {
        const pd = make()
        opened(pd)
        const characters = listOf(pd, "characterList")
        const form = findChild(pd, "characterForm")
        showTab(pd, 2)

        characters.currentId = "elisabeth"
        findChild(characters, "edit").clicked()
        tryCompare(form, "opened", true)
        compare(findChild(form, "colorPicker").currentIndex, 6)

        findChild(form, "colorPicker").picked(3)
        form.accept()

        compare(fake.calls, [["editCharacter", "elisabeth", "Elisabeth", "E", "green"]])
    }

    function test_the_short_label_has_room_for_three_letters() {
        const pd = make()
        opened(pd)
        const form = findChild(pd, "characterForm")

        form.show(null)
        tryCompare(form, "opened", true)

        compare(findChild(form, "shortField").maximumLength, 3)
    }

    function test_deleting_asks_first() {
        const pd = make()
        opened(pd)
        const scenes = listOf(pd, "sceneList")
        const confirm = findChild(pd, "confirm")

        scenes.currentId = "szene-013"
        findChild(scenes, "remove").clicked()
        tryCompare(confirm, "opened", true)

        compare(fake.calls, [])
        compare(findChild(confirm, "question").text,
                "Szene 13 EXT. HOF löschen? Ihre Pläne gehen verloren; Rückgängig stellt sie wieder her.")

        confirm.reject()
        compare(fake.calls, [])

        findChild(scenes, "remove").clicked()
        tryCompare(confirm, "opened", true)
        confirm.accept()
        compare(fake.calls, [["removeScene", "szene-013"]])
    }

    function test_each_list_deletes_its_own_kind() {
        const pd = make()
        opened(pd)
        const confirm = findChild(pd, "confirm")

        showTab(pd, 1)
        const locations = listOf(pd, "locationList")
        locations.currentId = "hof"
        findChild(locations, "remove").clicked()
        tryCompare(confirm, "opened", true)
        compare(findChild(confirm, "question").text, "Drehort Burghof löschen?")
        confirm.accept()

        showTab(pd, 2)
        const characters = listOf(pd, "characterList")
        characters.currentId = "burgvogt"
        findChild(characters, "remove").clicked()
        tryCompare(confirm, "opened", true)
        compare(findChild(confirm, "question").text, "Figur Burgvogt löschen?")
        confirm.accept()

        compare(fake.calls, [["removeLocation", "hof"], ["removeCharacter", "burgvogt"]])
    }

    function test_a_refused_delete_shows_the_message_in_the_dialog() {
        const pd = make()
        opened(pd)
        const confirm = findChild(pd, "confirm")
        fake.accept = false
        showTab(pd, 1)

        const locations = listOf(pd, "locationList")
        locations.currentId = "hof"
        findChild(locations, "remove").clicked()
        tryCompare(confirm, "opened", true)
        confirm.accept()

        verify(findChild(pd, "dialog").opened)
        compare(findChild(pd, "error").text, "Refused")
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
