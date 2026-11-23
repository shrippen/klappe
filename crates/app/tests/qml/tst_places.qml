import QtQuick
import QtTest
import "../../qml"

// The scripted locations in the project dialog (D-21): the list with its shooting location, the
// form with the spellings as chips and the location, and a scene that follows its scripted
// location or has a location of its own.
TestCase {
    id: tc
    name: "Places"
    width: 1000
    height: 800
    when: windowShown
    visible: true

    QtObject {
        id: fake
        property string status: ""
        property string sceneId: "s12"
        property bool accept: true
        property var calls: []
        readonly property string fixture: JSON.stringify({
            scenes: [
                { id: "s12", number: "12", heading: "INT. SAAL – TAG", location: "saal", locationName: "Sängersaal", ownLocation: false, place: "Sängersaal" },
                { id: "s13", number: "13", heading: "EXT. BURG – TAG", location: "", locationName: "", ownLocation: false, place: "Burg" },
                { id: "s14", number: "14", heading: "INT. BURG – NACHT", location: "hof", locationName: "Burghof", ownLocation: true, place: "Burg" },
                { id: "s15", number: "15", heading: "INT. RAUM – TAG", location: "", locationName: "", ownLocation: false, place: "" }
            ],
            places: [
                { id: "saal-ort", name: "Sängersaal", variants: ["SAAL"], location: "saal", location_name: "Sängersaal", scenes: ["12"] },
                { id: "burg-ort", name: "Burg", variants: [], location: "", location_name: "", scenes: ["13", "14"] }
            ],
            locations: [
                { id: "hof", name: "Burghof", width: 30, depth: 20, height: 3, scenes: 1 },
                { id: "saal", name: "Sängersaal", width: 20, depth: 13, height: 2.6, scenes: 1 }
            ],
            numbering: "scene",
            numberings: ["scene"],
            characters: []
        })
        property string projectJson: fixture

        function call(name, args) {
            calls.push([name].concat(args))
            if (!accept) {
                status = "Refused"
            }
            return accept
        }
        function samePlaceName(a, b) { return a.split(/\s+/).join(" ").toUpperCase() === b.split(/\s+/).join(" ").toUpperCase() }
        function clearStatus() { status = "" }
        function sceneChanges(id, n, h) { return "[]" }
        function addScene(n, h, l) { return call("addScene", [n, h, l]) }
        function editScene(id, n, h, l, sun) { return call("editScene", [id, l]) }
        function editPlace(id, n, v, l) { return call("editPlace", [id, n, v, l]) }
        function removePlace(id) { return call("removePlace", [id]) }
        function shootSceneAt(l) { return call("shootSceneAt", [l]) }
        // Makes the location, as the project then lists it.
        function addLocation(n, w, d, h, site) {
            const project = JSON.parse(projectJson)
            project.locations.push({ id: "neu", name: n, width: 10, depth: 8, height: 2.6, scenes: 0 })
            projectJson = JSON.stringify(project)
            return call("addLocation", [n])
        }
        function setNumbering(name) { return call("setNumbering", [name]) }
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

    // The dialog open on the tab of the scripted locations, `id` selected ("" for none).
    function make(id) {
        const pd = createTemporaryObject(dialogComponent, tc)
        verify(pd)
        pd.open()
        tryCompare(findChild(pd, "dialog"), "opened", true)
        findChild(pd, "tabs").activated(1)
        findChild(pd, "placeList").currentId = id
        wait(0)
        return pd
    }

    function rowOf(pd, index) {
        const rows = findChild(findChild(pd, "placeList"), "rows")
        rows.positionViewAtIndex(index, ListView.Beginning)
        return rows.itemAtIndex(index)
    }

    function chips(parent, name) {
        const out = []
        for (let i = 0; i < parent.children.length; ++i) {
            if (parent.children[i].objectName === name) {
                out.push(parent.children[i].text)
            }
        }
        return out
    }

    function test_the_tab_sits_between_the_scenes_and_the_locations() {
        const tabs = findChild(make(""), "tabs")

        compare(tabs.model, ["Szenen", "Spielorte", "Drehorte", "Figuren"])
        compare(tabs.counts, ["4", "2", "2", "0"])
    }

    // Every tab shows whole at the width of the window of the screenshots (1440 x 900).
    function test_the_four_tabs_fit_their_column() {
        const pd = make("")
        const tabs = findChild(pd, "tabs")

        verify(tabs.width >= tabs.implicitWidth, tabs.width + " < " + tabs.implicitWidth)
        verify(!tabs.moreAfter)
    }

    function test_a_row_names_its_location_and_its_scenes() {
        const pd = make("")

        compare(findChild(rowOf(pd, 0), "listRowText").text, "Sängersaal")
        compare(findChild(rowOf(pd, 0), "listRowSubtitle").text, "Sängersaal · 1 Szene")
        verify(!findChild(rowOf(pd, 0), "noLocation").visible)
        compare(findChild(rowOf(pd, 1), "listRowSubtitle").text, "2 Szenen")
        verify(findChild(rowOf(pd, 1), "noLocation").visible)
        compare(findChild(rowOf(pd, 1), "noLocation").text, "noch kein Drehort")
    }

    function test_there_is_no_new_button() {
        verify(!findChild(findChild(make(""), "placeList"), "add").visible)
    }

    function test_the_form_shows_the_spellings_the_location_and_the_scenes() {
        const form = findChild(make("saal-ort"), "placeForm")

        verify(form.visible)
        compare(findChild(form, "nameField").text, "Sängersaal")
        compare(chips(findChild(form, "variantChips"), "variant"), ["SAAL"])
        // KanteChip fills `%1` with the chip's text itself.
        compare(findChild(findChild(form, "variantChips"), "variant").removeText, "%1 entfernen")
        const box = findChild(form, "locationBox")
        compare(box.model, ["Noch kein Drehort", "Burghof", "Sängersaal"])
        compare(box.currentIndex, 2)
        compare(findChild(form, "sceneNumbers").text, "Szenen hier: 12")
    }

    function test_a_place_without_location_starts_on_the_first_entry() {
        const form = findChild(make("burg-ort"), "placeForm")

        compare(findChild(form, "locationBox").currentIndex, 0)
        compare(findChild(form, "sceneNumbers").text, "Szenen hier: 13, 14")
    }

    function test_apply_sends_the_spellings_and_the_location() {
        const form = findChild(make("saal-ort"), "placeForm")
        const field = findChild(form, "variantField")

        field.text = " Saal 2 "
        field.accepted()
        field.text = "saal"
        field.accepted()
        compare(chips(findChild(form, "variantChips"), "variant"), ["SAAL", "Saal 2"])

        findChild(form, "locationBox").currentIndex = 1
        findChild(form, "apply").clicked()

        compare(fake.calls, [["editPlace", "saal-ort", "Sängersaal", JSON.stringify(["SAAL", "Saal 2"]), "hof"]])
    }

    function test_a_spelling_typed_but_not_entered_counts_and_a_cross_takes_one_away() {
        const form = findChild(make("saal-ort"), "placeForm")

        findChild(findChild(form, "variantChips"), "variant").removeRequested()
        findChild(form, "variantField").text = "HALLE"
        findChild(form, "locationBox").currentIndex = 0
        findChild(form, "apply").clicked()

        compare(fake.calls, [["editPlace", "saal-ort", "Sängersaal", JSON.stringify(["HALLE"]), ""]])
    }

    function test_delete_asks_and_a_refusal_shows_here() {
        const form = findChild(make("burg-ort"), "placeForm")
        fake.accept = false

        findChild(form, "remove").clicked()
        compare(findChild(form, "removeAsk").visible, true)
        compare(findChild(form, "removeAsk").text, "Spielort Burg löschen?")
        findChild(form, "confirmRemove").clicked()

        compare(fake.calls, [["removePlace", "burg-ort"]])
        compare(findChild(form, "error").text, "Refused")
        verify(form.visible)
    }

    function sceneForm(id) {
        const pd = createTemporaryObject(dialogComponent, tc)
        pd.open()
        tryCompare(findChild(pd, "dialog"), "opened", true)
        findChild(pd, "sceneList").currentId = id
        wait(0)
        return findChild(pd, "sceneForm")
    }

    function test_the_scene_follows_its_scripted_location_first() {
        const form = sceneForm("s12")
        const box = findChild(form, "locationBox")

        compare(findChild(form, "placeName").text, "Sängersaal")
        compare(box.model, ["Wie Spielort: Sängersaal", "Burghof", "Sängersaal"])
        compare(box.currentIndex, 0)

        findChild(form, "apply").clicked()
        compare(fake.calls, [["editScene", "s12", ""]])
    }

    function test_the_first_entry_says_when_the_place_has_no_location() {
        const form = sceneForm("s13")

        compare(findChild(form, "locationBox").model[0], "Wie Spielort (noch kein Drehort)")
        verify(findChild(form, "apply").enabled)
        findChild(form, "apply").clicked()
        compare(fake.calls, [["editScene", "s13", ""]])
    }

    function test_a_scene_with_its_own_location_shows_and_keeps_it() {
        const form = sceneForm("s14")

        compare(findChild(form, "locationBox").currentIndex, 1)
        findChild(form, "apply").clicked()
        compare(fake.calls, [["editScene", "s14", "hof"]])
    }

    function test_a_scene_without_a_scripted_location_shows_a_dash() {
        const form = sceneForm("s15")

        compare(findChild(form, "placeName").text, "—")
        compare(findChild(form, "locationBox").model[0], "Wie Spielort (noch kein Drehort)")
    }

    function test_the_scene_list_marks_an_own_location() {
        const pd = make("")
        findChild(pd, "tabs").activated(0)
        const rows = findChild(findChild(pd, "sceneList"), "rows")

        compare(rows.itemAtIndex(0).meta, "Sängersaal")
        compare(rows.itemAtIndex(0).subtitle, "")
        compare(rows.itemAtIndex(2).subtitle, "Burghof · eigener Drehort")
    }

    // The empty state of a scene without a location makes one: the dialog opens on a form with
    // the name, and the new location goes to the scripted location.
    function test_a_new_location_from_the_plan_goes_to_the_place() {
        const pd = createTemporaryObject(dialogComponent, tc)
        pd.openNewLocation("Burg", "burg-ort")
        tryCompare(findChild(pd, "dialog"), "opened", true)
        const form = findChild(pd, "locationForm")

        compare(findChild(pd, "tabs").currentIndex, 2)
        verify(form.visible)
        compare(findChild(form, "nameField").text, "Burg")

        findChild(form, "apply").clicked()
        compare(fake.calls, [["addLocation", "Burg"], ["editPlace", "burg-ort", "Burg", "[]", "neu"]])
        tryCompare(findChild(pd, "dialog"), "opened", false)
    }

    function test_a_new_location_without_a_place_goes_to_the_scene() {
        const pd = createTemporaryObject(dialogComponent, tc)
        pd.openNewLocation("", "")
        tryCompare(findChild(pd, "dialog"), "opened", true)
        const form = findChild(pd, "locationForm")

        findChild(form, "nameField").text = "Keller"
        findChild(form, "apply").clicked()

        compare(fake.calls, [["addLocation", "Keller"], ["shootSceneAt", "neu"]])
    }
}
