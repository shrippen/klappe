import QtQuick
import QtTest
import "../../qml"

// The other names of a character and the names that are no character (DB-50) in the project
// dialog: chips with a cross, a field that adds one, and a warning before the book asks again.
TestCase {
    id: tc
    name: "FigureNames"
    width: 1000
    height: 800
    when: windowShown
    visible: true

    QtObject {
        id: fake
        property string status: ""
        property string sceneId: ""
        property bool accept: true
        property var calls: []
        // What the preview returns: [{ kind, from, to, count }].
        property var changes: []
        // The second argument of the last `characterChanges`.
        property string previewed: ""
        readonly property string start: JSON.stringify({
            scenes: [],
            locations: [],
            numbering: "scene",
            numberings: ["scene"],
            characters: [
                { id: "ilva", name: "Ilva Brandes", short: "I", color: "purple", group: 0, size: null, aliases: ["ILVA", "FRAU BRANDES"] },
                { id: "bert", name: "Bert", short: "B", color: "blue", group: 0, size: null, aliases: [] }
            ],
            not_figures: ["ALLE", "STIMME"]
        })
        property string projectJson: start

        function call(name, args) {
            calls.push([name].concat(args))
            if (!accept) {
                status = "Refused"
            }
            return accept
        }
        function clearStatus() { status = "" }
        function characterChanges(id, n, a) { previewed = a; return JSON.stringify(changes) }
        function notFigureChanges(n) { return JSON.stringify(changes) }
        function addCharacter(n, s, c, g, z) { return call("addCharacter", [n, s, c, g, z]) }
        function editCharacter(id, n, s, c, g, z, a) { return call("editCharacter", [id, a]) }
        function addNotFigure(n) { return call("addNotFigure", [n]) }
        function removeNotFigure(n) { return call("removeNotFigure", [n]) }
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
        fake.changes = []
        fake.previewed = ""
        fake.projectJson = fake.start
    }

    // The dialog open on the characters, `id` selected ("" for none).
    function make(id) {
        const pd = createTemporaryObject(dialogComponent, tc)
        verify(pd)
        pd.open()
        tryCompare(findChild(pd, "dialog"), "opened", true)
        findChild(pd, "tabs").activated(2)
        findChild(pd, "characterList").currentId = id
        wait(0)
        return pd
    }

    // The texts of the chips named `name` under `parent`.
    function chips(parent, name) {
        const out = []
        for (let i = 0; i < parent.children.length; ++i) {
            const c = parent.children[i]
            if (c.objectName === name) {
                out.push(c.text)
            }
        }
        return out
    }

    function enter(field, text) {
        field.text = text
        field.accepted()
    }

    function test_the_form_shows_the_other_names_as_chips() {
        const form = findChild(make("ilva"), "characterForm")

        compare(chips(findChild(form, "aliasChips"), "alias"), ["ILVA", "FRAU BRANDES"])
        // KanteChip fills `%1` with the chip's text itself.
        compare(findChild(findChild(form, "aliasChips"), "alias").removeText, "%1 entfernen")
        verify(findChild(form, "aliasField").visible)
    }

    function test_enter_adds_a_name_once_and_apply_sends_the_list() {
        const form = findChild(make("ilva"), "characterForm")
        const field = findChild(form, "aliasField")

        enter(field, " Brandes ")
        enter(field, "ilva")
        compare(field.text, "")
        compare(chips(findChild(form, "aliasChips"), "alias"), ["ILVA", "FRAU BRANDES", "Brandes"])

        findChild(form, "apply").clicked()
        compare(fake.calls, [["editCharacter", "ilva", JSON.stringify(["ILVA", "FRAU BRANDES", "Brandes"])]])
    }

    function test_a_name_typed_but_not_entered_counts_on_apply() {
        const form = findChild(make("bert"), "characterForm")

        findChild(form, "aliasField").text = "BERTI"
        findChild(form, "apply").clicked()

        compare(fake.calls, [["editCharacter", "bert", JSON.stringify(["BERTI"])]])
    }

    function test_removing_a_name_the_book_uses_warns_first() {
        const form = findChild(make("ilva"), "characterForm")
        const chip = findChild(findChild(form, "aliasChips"), "alias")
        fake.changes = [{ kind: "unknown", from: "ILVA", to: "", count: 3 }]

        chip.removeRequested()
        findChild(form, "apply").clicked()

        compare(fake.previewed, JSON.stringify(["FRAU BRANDES"]))
        const warning = findChild(form, "warning")
        verify(warning.visible)
        compare(warning.title, "Das Buch fragt wieder nach diesen Namen")
        verify(warning.text.indexOf("3 Stellen: ILVA gehört dann zu keiner Figur.") >= 0, warning.text)
        compare(findChild(form, "confirmChange").text, "Übernehmen")
        compare(fake.calls, [])

        findChild(form, "confirmChange").clicked()
        compare(fake.calls, [["editCharacter", "ilva", JSON.stringify(["FRAU BRANDES"])]])
    }

    function test_a_new_character_has_no_other_names_yet() {
        const pd = make("")
        findChild(findChild(pd, "characterList"), "add").clicked()
        const form = findChild(pd, "characterForm")

        verify(form.visible)
        verify(!findChild(form, "aliasField").visible)
        verify(!findChild(form, "aliasChips").visible)
    }

    function test_the_names_that_are_no_character_stand_under_the_list() {
        const section = findChild(make(""), "notFigures")

        verify(section.visible)
        compare(chips(findChild(section, "notFigureChips"), "notFigure"), ["ALLE", "STIMME"])
        compare(findChild(findChild(section, "notFigureChips"), "notFigure").removeText,
            "%1 entfernen")
    }

    function test_enter_marks_a_name_and_a_refusal_shows_here() {
        const section = findChild(make(""), "notFigures")
        const field = findChild(section, "notFigureField")

        enter(field, "ERZÄHLER")
        compare(fake.calls, [["addNotFigure", "ERZÄHLER"]])
        compare(field.text, "")

        fake.accept = false
        enter(field, "BERT")
        compare(field.text, "BERT", "nothing typed is lost")
        const error = findChild(section, "notFigureError")
        verify(error.visible)
        compare(error.text, "Refused")
        compare(fake.status, "", "the form beside does not show it twice")
    }

    function test_taking_away_a_name_the_book_uses_asks_first() {
        const section = findChild(make(""), "notFigures")
        const chip = findChild(findChild(section, "notFigureChips"), "notFigure")
        fake.changes = [{ kind: "unknown", from: "ALLE", to: "", count: 1 }]

        chip.removeRequested()

        const ask = findChild(section, "notFigureAsk")
        verify(ask.visible)
        verify(ask.text.indexOf("1 Stelle: ALLE gehört dann zu keiner Figur.") >= 0, ask.text)
        compare(fake.calls, [])

        findChild(section, "confirmNotFigure").clicked()
        compare(fake.calls, [["removeNotFigure", "ALLE"]])
        verify(!ask.visible)
    }

    function test_a_name_the_book_does_not_use_goes_at_once() {
        const section = findChild(make(""), "notFigures")
        const chip = findChild(findChild(section, "notFigureChips"), "notFigure")

        chip.removeRequested()

        verify(!findChild(section, "notFigureAsk").visible)
        compare(fake.calls, [["removeNotFigure", "ALLE"]])
    }

    // Many names and the question open: the section grows the dialog, its field stays above
    // the numbering below (seen offscreen: the field lay over "Shotnummern").
    function test_many_names_do_not_overlap_the_numbering() {
        const project = JSON.parse(fake.projectJson)
        const many = project.not_figures.concat(["ERZÄHLER", "CHOR", "MENGE", "RADIO", "FERNSEHER",
            "LAUTSPRECHER", "TELEFON", "KIND 1", "KIND 2", "GAST", "BOTE", "WACHE"])
        fake.projectJson = JSON.stringify(Object.assign(project, { not_figures: many }))
        const pd = make("")
        const section = findChild(pd, "notFigures")
        fake.changes = [{ kind: "unknown", from: "ALLE", to: "", count: 1 }]
        findChild(findChild(section, "notFigureChips"), "notFigure").removeRequested()
        wait(0)

        const field = findChild(section, "notFigureField")
        const box = findChild(pd, "numberingBox")
        const fieldBottom = field.mapToItem(null, 0, field.height).y
        const boxTop = box.mapToItem(null, 0, 0).y
        verify(fieldBottom < boxTop, "field ends at " + fieldBottom + ", numbering starts at " + boxTop)
    }
}
