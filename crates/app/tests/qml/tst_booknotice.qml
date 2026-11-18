import QtQuick
import QtTest
import "../../qml"

// BookNotice: shows while entries of the register wait for an answer, with their count, and its
// button asks for the dialog.
TestCase {
    id: tc
    name: "BookNotice"
    width: 900
    height: 300
    when: windowShown
    visible: true

    QtObject {
        id: fake
        property bool loaded: true
        property string projectJson: ""
    }

    TestTexts { id: de; language: "de" }

    Component {
        id: noticeComponent
        BookNotice { width: 800; texts: de; controller: fake }
    }

    function init() {
        fake.loaded = true
        fake.projectJson = JSON.stringify({ pending: [] })
    }

    function make() {
        const notice = createTemporaryObject(noticeComponent, tc)
        verify(notice)
        return notice
    }

    function test_nothing_shows_without_questions() {
        verify(!make().visible)
    }

    function test_open_questions_show_with_their_count_and_the_button_asks() {
        fake.projectJson = JSON.stringify({ pending: [{ id: "a" }, { id: "b" }] })
        const notice = make()
        const asked = []
        notice.matchRequested.connect(() => asked.push(true))

        verify(notice.visible)
        compare(notice.text, "Szenen zuzuordnen: 2.")
        findChild(notice, "match").clicked()

        compare(asked.length, 1)
    }

    function test_open_places_show_their_count_alone_and_after_the_scenes() {
        fake.projectJson = JSON.stringify({ pending: [], pendingPlaces: [{ name: "HOF" }] })
        const notice = make()

        verify(notice.visible)
        compare(notice.text, "Spielorte offen: 1.")

        fake.projectJson = JSON.stringify({ pending: [{ id: "a" }], pendingPlaces: [{ name: "HOF" }, { name: "KELLER" }] })

        compare(notice.text, "Szenen zuzuordnen: 1. Spielorte offen: 2.")
    }
}
