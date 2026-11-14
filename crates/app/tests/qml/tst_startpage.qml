import QtQuick
import QtTest
import "../../qml"

// StartPage (LH-01): the two ways in and the recent projects.
TestCase {
    id: tc
    name: "StartPage"
    width: 900
    height: 600
    when: windowShown
    visible: true

    QtObject {
        id: fake
        property string recentJson: "[]"
    }

    TestTexts { id: de; language: "de" }
    TestTexts { id: en; language: "en" }

    Component {
        id: pageComponent
        StartPage {
            anchors.fill: parent
            controller: fake
            texts: de
        }
    }

    function make() {
        const page = createTemporaryObject(pageComponent, tc)
        verify(page)
        return page
    }

    function test_without_recent_projects_it_says_so() {
        fake.recentJson = "[]"
        const page = make()

        verify(findChild(page, "noRecent").visible)
        compare(findChild(page, "noRecent").text, "Noch keine Projekte geöffnet.")
    }

    function test_both_ways_in_are_signalled() {
        const page = make()
        const asked = []
        page.newRequested.connect(() => asked.push("new"))
        page.openRequested.connect(() => asked.push("open"))

        findChild(page, "newButton").clicked()
        findChild(page, "openButton").clicked()

        compare(asked, ["new", "open"])
        compare(findChild(page, "newButton").text, "Neues Projekt…")
    }

    function test_a_recent_project_is_named_and_opens_by_its_path() {
        fake.recentJson = JSON.stringify([
            { path: "/film/wartburg.klappe", name: "wartburg.klappe" },
            { path: "/film/zwei.klappe", name: "zwei.klappe" }
        ])
        const page = make()
        let opened = ""
        page.recentRequested.connect(path => { opened = path })

        verify(!findChild(page, "noRecent").visible)
        const row = findChild(page, "recent")
        compare(row.text, "wartburg.klappe")
        compare(row.subtitle, "/film/wartburg.klappe")
        row.clicked()

        compare(opened, "/film/wartburg.klappe")
    }

    function test_the_english_text() {
        const page = make()
        page.texts = en

        compare(findChild(page, "openButton").text, "Open project…")
    }
}
