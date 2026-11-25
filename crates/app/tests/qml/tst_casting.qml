import QtQuick
import QtTest
import "../../qml"

// D-26: the casting module. The roles with number and who plays them, the role form (number
// shown and set by hand, lock and unlock, the proposals as chips, fix and drop, the person
// search with the cast department first and "New: <name>") and the empty states.
TestCase {
    id: tc
    name: "Casting"
    width: 1200
    height: 800
    when: windowShown
    visible: true

    QtObject {
        id: fake
        property string status: ""
        property var calls: []
        // The answer of `setCastNumber`; false marks the field.
        property bool accepts: true
        readonly property string fixture: JSON.stringify({
            characters: [
                { id: "fischer", name: "Fischer", color: "green", group: 3 },
                { id: "anna", name: "Anna", color: "purple", group: 0 },
                { id: "bert", name: "Bert", color: "blue", group: 0 },
                { id: "clara", name: "Clara", color: "red", group: 0 }
            ],
            persons: [
                { id: "p1", name: "Jo Berg", department: "camera" },
                { id: "p2", name: "Lena Weber", department: "cast" },
                { id: "p3", name: "Mia Rot", department: "cast" },
                { id: "p4", name: "Ole Blau", department: null }
            ],
            castings: [
                { character: "fischer", fixed: null, proposed: [], number: null, stored: false, scenes: 1 },
                { character: "anna", fixed: "p2", proposed: ["p2", "p3"], number: 1, stored: false, scenes: 7 },
                { character: "bert", fixed: null, proposed: ["p3", "p4"], number: 2, stored: true, scenes: 1 },
                { character: "clara", fixed: null, proposed: [], number: 3, stored: false, scenes: 0 }
            ],
            numbers_locked: false
        })
        property string projectJson: fixture

        function call(name, args) {
            calls.push([name].concat(args))
            return true
        }
        function clearStatus() { status = "" }
        function setCastNumber(c, n) { call("setCastNumber", [c, n]); return accepts }
        function lockNumbers() { return call("lockNumbers", []) }
        function unlockNumbers() { return call("unlockNumbers", []) }
        function proposeCast(c, p) { return call("proposeCast", [c, p]) }
        function fixCast(c, p) { return call("fixCast", [c, p]) }
        function dropProposal(c, p) { return call("dropProposal", [c, p]) }
        function addPerson(n, d) { call("addPerson", [n, d]); return "p9" }
    }

    TestTexts { id: de; language: "de" }

    Component {
        id: viewComponent
        CastingView { texts: de; controller: fake; width: 1200; height: 700 }
    }

    function init() {
        fake.calls = []
        fake.accepts = true
        fake.projectJson = fake.fixture
    }

    function view() {
        const v = createTemporaryObject(viewComponent, tc)
        verify(v)
        waitForRendering(v)
        return v
    }

    function pick(v, id) {
        const rows = findChild(v, "roleRows")
        tryVerify(() => findChild(rows, "role." + id) !== null)
        findChild(rows, "role." + id).clicked()
        return findChild(v, "roleForm")
    }

    function test_the_roles_list_the_groups_last() {
        const v = view()

        compare(findChild(v, "roleList").rows.map(r => r.role.id), ["anna", "bert", "clara", "fischer"])
    }

    function test_a_row_shows_the_number_and_who_plays_it() {
        const v = view()
        const rows = findChild(v, "roleRows")
        tryVerify(() => findChild(rows, "role.anna") !== null)

        compare(findChild(rows, "role.anna").leadingText, "1")
        compare(findChild(rows, "role.anna").meta, "Lena Weber")
        compare(findChild(rows, "role.bert").meta, "2 Vorschläge")
        compare(findChild(rows, "role.clara").meta, "offen")
        compare(findChild(rows, "role.fischer").leadingText, "–")
        compare(findChild(rows, "role.fischer").text, "Fischer ×3")
    }

    function test_without_a_chosen_role_the_form_says_what_to_do() {
        const v = view()

        verify(findChild(v, "roleHint").visible)
        verify(!findChild(v, "roleForm").visible)
        compare(findChild(v, "roleHint").title, "Wähle links eine Rolle.")

        pick(v, "anna")
        verify(!findChild(v, "roleHint").visible)
    }

    function test_without_characters_the_list_says_so() {
        const project = JSON.parse(fake.fixture)
        project.characters = []
        project.castings = []
        fake.projectJson = JSON.stringify(project)
        const v = view()

        verify(findChild(v, "noRoles").visible)
    }

    function test_the_form_shows_the_number_and_how_it_came() {
        const v = view()

        const auto = pick(v, "anna")
        compare(findChild(auto, "numberInfo").text, "1, automatisch")
        compare(findChild(auto, "numberField").text, "")
        compare(findChild(auto, "sceneCount").text, "In 7 Szenen")

        const set = pick(v, "bert")
        compare(findChild(set, "numberInfo").text, "2, festgelegt")
        compare(findChild(set, "numberField").text, "2")
        compare(findChild(set, "sceneCount").text, "In 1 Szene")

        compare(findChild(pick(v, "fischer"), "numberInfo").text, "keine Nummer (Gruppe)")
        compare(findChild(pick(v, "clara"), "sceneCount").text, "In keiner Szene")
    }

    function type(field, text) {
        field.forceActiveFocus()
        field.text = text
        keyClick(Qt.Key_Return)
    }

    function test_a_typed_number_is_sent() {
        const v = view()
        const form = pick(v, "anna")

        type(findChild(form, "numberField"), "5")

        compare(fake.calls, [["setCastNumber", "anna", "5"]])
        verify(!findChild(form, "numberField").invalid)
    }

    function test_an_emptied_number_goes_back_to_the_book() {
        const v = view()
        const form = pick(v, "bert")

        type(findChild(form, "numberField"), "")

        compare(fake.calls, [["setCastNumber", "bert", ""]])
    }

    function test_a_refused_number_is_marked() {
        fake.accepts = false
        const v = view()
        const form = pick(v, "anna")

        type(findChild(form, "numberField"), "2")

        verify(findChild(form, "numberField").invalid)
    }

    function test_an_unchanged_number_sends_nothing() {
        const v = view()
        const form = pick(v, "bert")

        type(findChild(form, "numberField"), "2")

        compare(fake.calls.length, 0)
    }

    function test_the_lock_button_locks_and_then_frees_the_numbers() {
        const v = view()
        const form = pick(v, "anna")
        verify(findChild(form, "lock").visible)
        verify(!findChild(form, "unlock").visible)

        findChild(form, "lock").clicked()
        compare(fake.calls, [["lockNumbers"]])

        const project = JSON.parse(fake.fixture)
        project.numbers_locked = true
        fake.projectJson = JSON.stringify(project)

        verify(!findChild(form, "lock").visible)
        verify(findChild(form, "unlock").visible)
        verify(findChild(form, "lockedHint").visible)
        findChild(form, "unlock").clicked()
        compare(fake.calls[1], ["unlockNumbers"])
    }

    function test_the_proposals_show_as_chips_with_the_fixed_one_marked() {
        const v = view()
        const form = pick(v, "anna")

        compare(findChild(findChild(form, "proposal.p2"), "proposalChip").text, "Lena Weber · fest")
        compare(findChild(findChild(form, "proposal.p3"), "proposalChip").text, "Mia Rot")
        verify(!findChild(findChild(form, "proposal.p2"), "fix").visible)
        verify(findChild(findChild(form, "proposal.p3"), "fix").visible)
    }

    function test_the_fix_action_has_a_text_and_names_the_person() {
        const v = view()
        const form = pick(v, "anna")
        const fix = findChild(findChild(form, "proposal.p3"), "fix")

        verify(fix.text.length > 1)
        verify(fix.Accessible.name.includes("Mia Rot"))
    }

    function test_the_check_fixes_a_proposal() {
        const v = view()
        const form = pick(v, "anna")

        findChild(findChild(form, "proposal.p3"), "fix").clicked()

        compare(fake.calls, [["fixCast", "anna", "p3"]])
    }

    function test_the_cross_drops_a_proposal() {
        const v = view()
        const form = pick(v, "anna")

        findChild(findChild(form, "proposal.p3"), "proposalChip").removeRequested()

        compare(fake.calls, [["dropProposal", "anna", "p3"]])
    }

    function test_a_role_without_proposals_says_so() {
        const v = view()

        verify(findChild(pick(v, "clara"), "nobody").visible)
        verify(!findChild(pick(v, "anna"), "nobody").visible)
    }

    // The persons the role has not yet, those of the cast department first.
    function test_the_search_offers_the_persons_not_proposed_with_the_cast_first() {
        const v = view()
        const form = pick(v, "clara")

        compare(findChild(form, "personSearch").model, ["Lena Weber", "Mia Rot", "Jo Berg", "Ole Blau"])

        const other = pick(v, "anna")
        compare(findChild(other, "personSearch").model, ["Jo Berg", "Ole Blau"])
    }

    function test_a_person_of_the_search_is_proposed() {
        const v = view()
        const form = pick(v, "clara")

        findChild(form, "personSearch").activated(1)

        compare(fake.calls, [["proposeCast", "clara", "p3"]])
    }

    function test_a_new_name_makes_a_cast_person_and_proposes_it() {
        const v = view()
        const form = pick(v, "clara")

        findChild(form, "personSearch").newEntered("Eva Grün")

        compare(fake.calls, [["addPerson", "Eva Grün", "cast"], ["proposeCast", "clara", "p9"]])
    }
}
