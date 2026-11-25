import QtQuick
import QtTest
import "../../qml"

// D-26: the team module. The persons by department with a filter, the person form (new, change
// with contact, delete after asking) and the empty states.
TestCase {
    id: tc
    name: "Team"
    width: 1200
    height: 800
    when: windowShown
    visible: true

    QtObject {
        id: fake
        property string status: ""
        property var calls: []
        readonly property string fixture: JSON.stringify({
            departments: ["direction", "camera", "props", "cast"],
            persons: [
                { id: "p1", name: "Jo Berg", department: "camera", function: "Kamera", agency: "",
                  phone: "0301", mail: "jo@example.org", address: "Hauptstr. 1" },
                { id: "p2", name: "Lena Weber", department: "cast", function: "", agency: "Agentur Eins",
                  phone: "", mail: "", address: "" },
                { id: "p3", name: "Ole Blau", department: null, function: "", agency: "",
                  phone: "", mail: "", address: "" },
                { id: "p4", name: "Amy Gold", department: "camera", function: "", agency: "",
                  phone: "", mail: "", address: "" }
            ]
        })
        property string projectJson: fixture

        function call(name, args) {
            calls.push([name].concat(args))
            return true
        }
        function clearStatus() { status = "" }
        function addPerson(n, d) { call("addPerson", [n, d]); return "p9" }
        function editPerson(i, n, d, f, a, p, m, ad) { return call("editPerson", [i, n, d, f, a, p, m, ad]) }
        function removePerson(i) { return call("removePerson", [i]) }
    }

    TestTexts { id: de; language: "de" }

    Component {
        id: viewComponent
        TeamView { texts: de; controller: fake; width: 1200; height: 700 }
    }

    function init() {
        fake.calls = []
        fake.projectJson = fake.fixture
    }

    function view() {
        const v = createTemporaryObject(viewComponent, tc)
        verify(v)
        waitForRendering(v)
        return v
    }

    function pick(v, id) {
        const rows = findChild(v, "personRows")
        tryVerify(() => findChild(rows, "person." + id) !== null)
        findChild(rows, "person." + id).clicked()
        return findChild(v, "personForm")
    }

    function test_the_persons_come_by_department_and_those_without_last() {
        const v = view()

        compare(findChild(v, "personList").rows.map(r => r.person.id), ["p4", "p1", "p2", "p3"])
        compare(findChild(v, "personList").rows.map(r => r.group), ["camera", "camera", "cast", ""])
    }

    function test_the_filter_narrows_to_one_department() {
        const v = view()
        const box = findChild(v, "departmentFilter")

        compare(box.model, ["Alle", "Regie", "Kamera", "Requisite", "Darsteller"])
        box.activated(3)

        compare(findChild(v, "personList").rows.map(r => r.person.id), [])
        box.activated(4)
        compare(findChild(v, "personList").rows.map(r => r.person.id), ["p2"])
        box.activated(0)
        compare(findChild(v, "personList").rows.length, 4)
    }

    function test_without_persons_the_list_says_so() {
        const project = JSON.parse(fake.fixture)
        project.persons = []
        fake.projectJson = JSON.stringify(project)
        const v = view()

        verify(findChild(v, "noPersons").visible)
    }

    function test_without_a_chosen_person_the_form_says_what_to_do() {
        const v = view()

        verify(findChild(v, "personHint").visible)
        compare(findChild(v, "personHint").title, "Wähle links eine Person oder lege mit „Neu …“ eine an.")
        verify(!findChild(v, "personForm").visible)

        pick(v, "p1")
        verify(!findChild(v, "personHint").visible)
    }

    function test_a_person_shows_in_the_form() {
        const v = view()
        const form = pick(v, "p1")

        compare(findChild(form, "nameField").text, "Jo Berg")
        compare(findChild(form, "departmentBox").currentIndex, 2)
        compare(findChild(form, "functionField").text, "Kamera")
        compare(findChild(form, "phoneField").text, "0301")
        compare(findChild(form, "mailField").text, "jo@example.org")
        compare(findChild(form, "addressField").text, "Hauptstr. 1")

        compare(findChild(pick(v, "p3"), "departmentBox").currentIndex, 0)
    }

    function test_apply_sends_the_person_and_the_contact_as_one_edit() {
        const v = view()
        const form = pick(v, "p1")

        findChild(form, "nameField").text = "Jo Berger"
        findChild(form, "departmentBox").currentIndex = 4
        findChild(form, "mailField").text = "jo@kamera.example"
        findChild(form, "apply").clicked()

        compare(fake.calls, [["editPerson", "p1", "Jo Berger", "cast", "Kamera", "",
                              "0301", "jo@kamera.example", "Hauptstr. 1"]])
    }

    function test_the_contact_alone_is_sent_with_the_person_as_it_is() {
        const v = view()
        const form = pick(v, "p1")

        findChild(form, "phoneField").text = "0302"
        findChild(form, "apply").clicked()

        compare(fake.calls, [["editPerson", "p1", "Jo Berg", "camera", "Kamera", "",
                              "0302", "jo@example.org", "Hauptstr. 1"]])
    }

    function test_nothing_is_sent_when_nothing_changed() {
        const v = view()
        const form = pick(v, "p1")

        findChild(form, "apply").clicked()

        compare(fake.calls.length, 0)
    }

    function test_no_department_is_sent_as_empty() {
        const v = view()
        const form = pick(v, "p1")

        findChild(form, "departmentBox").currentIndex = 0
        findChild(form, "apply").clicked()

        compare(fake.calls[0].slice(0, 6), ["editPerson", "p1", "Jo Berg", "", "Kamera", ""])
    }

    function test_a_person_is_deleted_after_asking_and_the_question_names_the_losses() {
        const v = view()
        const form = pick(v, "p2")

        findChild(form, "remove").clicked()
        compare(fake.calls.length, 0)
        verify(findChild(form, "removeAsk").visible)
        verify(findChild(form, "removeAsk").text.includes("Besetzungen und Verantwortlichkeiten"))
        findChild(form, "confirmRemove").clicked()

        compare(fake.calls, [["removePerson", "p2"]])
    }

    function test_new_makes_a_person_with_a_department() {
        const v = view()

        findChild(v, "newPerson").clicked()
        const form = findChild(v, "personForm")
        compare(form.title, "Neue Person")
        verify(!findChild(form, "remove").visible)
        verify(!findChild(form, "phoneField").visible)

        findChild(form, "nameField").text = "Eva Grün"
        findChild(form, "departmentBox").currentIndex = 3
        findChild(form, "apply").clicked()

        compare(fake.calls, [["addPerson", "Eva Grün", "props"]])
        compare(v.personId, "p9")
        verify(!v.creating)
    }
}
