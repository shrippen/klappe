import QtQuick
import QtTest
import "../../qml"

// Every page that reads the project data keeps following it: a new `projectJson` reaches the
// derived list of each, and without a project each shows an empty one.
TestCase {
    id: tc
    name: "ProjectReads"
    width: 900
    height: 600
    when: windowShown
    visible: true

    QtObject {
        id: fake
        property bool loaded: true
        property string projectJson: ""
        property string sceneId: ""
        property string sceneKind: ""
        property bool backupOpen: false
        property bool busy: false
        function usesOf() { return "" }
    }

    TestTexts { id: de; language: "de" }

    Component { id: sceneList; SceneList { texts: de; controller: fake } }
    Component { id: viewCommands; ViewCommands { texts: de; controller: fake } }
    Component { id: objectCard; ObjectCard { texts: de; controller: fake } }
    Component { id: planMenu; PlanMenu { texts: de; controller: fake } }
    Component { id: bookNotice; BookNotice { texts: de; controller: fake } }
    Component { id: teamList; TeamList { texts: de; controller: fake } }
    Component { id: castingList; CastingList { texts: de; controller: fake } }
    Component { id: lookList; LookList { texts: de; controller: fake } }
    Component { id: elementList; ElementList { texts: de; controller: fake } }

    function init() {
        fake.projectJson = ""
    }

    function make(component) {
        const page = createTemporaryObject(component, tc)
        verify(page)
        return page
    }

    function test_scenes_follow_the_project() {
        const page = make(sceneList)
        compare(page.scenes.length, 0)

        fake.projectJson = JSON.stringify({ scenes: [{ id: "a" }, { id: "b" }] })
        compare(page.scenes.length, 2)

        fake.projectJson = JSON.stringify({ scenes: [{ id: "c" }] })
        compare(page.scenes[0].id, "c")

        fake.projectJson = ""
        compare(page.scenes.length, 0)
    }

    function test_saved_views_follow_the_project() {
        const page = make(viewCommands)
        compare(page.savedViews.length, 0)

        fake.projectJson = JSON.stringify({ views: [{ id: "v" }] })
        compare(page.savedViews.length, 1)

        fake.projectJson = JSON.stringify({})
        compare(page.savedViews.length, 0)
    }

    function test_object_elements_follow_the_project() {
        const page = make(objectCard)
        compare(page.elements.length, 0)

        fake.projectJson = JSON.stringify({ elements: [{ id: "e" }] })
        compare(page.elements.length, 1)
    }

    function test_plans_follow_the_project() {
        const page = make(planMenu)
        compare(page.plans.length, 0)

        fake.projectJson = JSON.stringify({ plans: [{ id: "p", name: "P" }], plan: "p" })
        compare(page.plans.length, 1)
        compare(page.index, 0)
    }

    function test_questions_follow_the_project() {
        const page = make(bookNotice)
        compare(page.questions.length, 0)

        fake.projectJson = JSON.stringify({ questions: [{ kind: "scene", key: "a" }] })
        compare(page.questions.length, 1)
    }

    function test_departments_follow_the_project() {
        const page = make(teamList)
        compare(page.departments.length, 0)

        fake.projectJson = JSON.stringify({ departments: [{ id: "d" }] })
        compare(page.departments.length, 1)
    }

    function test_characters_follow_the_project() {
        const casting = make(castingList)
        const looks = make(lookList)
        compare(casting.characters.length, 0)
        compare(looks.characters.length, 0)

        fake.projectJson = JSON.stringify({ characters: [{ id: "c", name: "C" }] })
        compare(casting.characters.length, 1)
        compare(looks.characters.length, 1)
    }

    function test_categories_follow_the_project() {
        const page = make(elementList)
        compare(page.categories.length, 0)

        fake.projectJson = JSON.stringify({ element_categories: [{ id: "k" }] })
        compare(page.categories.length, 1)
    }
}
