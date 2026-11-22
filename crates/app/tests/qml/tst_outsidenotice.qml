import QtQuick
import QtTest
import "../../qml"

// OutsideNotice: shows while a file is changed outside and in the window, and hands the
// answer to the controller; the words for the files come from the controller.
TestCase {
    id: tc
    name: "OutsideNotice"
    width: 900
    height: 300
    when: windowShown
    visible: true

    QtObject {
        id: fake
        property bool loaded: true
        property string outsideFiles: ""
        property var calls: []

        function reloadProject() {
            calls.push("reload")
            outsideFiles = ""
        }
        function saveOverwrite() {
            calls.push("overwrite")
            outsideFiles = ""
            return true
        }
    }

    TestTexts { id: de; language: "de" }
    TestTexts { id: en; language: "en" }

    Component {
        id: noticeComponent
        OutsideNotice {
            width: 800
            controller: fake
            texts: de
        }
    }

    function init() {
        fake.loaded = true
        fake.outsideFiles = ""
        fake.calls = []
    }

    function make() {
        const notice = createTemporaryObject(noticeComponent, tc)
        verify(notice)
        return notice
    }

    function test_nothing_shows_without_a_conflict() {
        verify(!make().visible)
    }

    function test_a_conflict_names_the_files_and_offers_both_answers() {
        fake.outsideFiles = "Drehbuch, Pläne"
        const notice = make()

        verify(notice.visible)
        compare(notice.text, "Außerhalb und hier geändert: Drehbuch, Pläne.")
        compare(findChild(notice, "reload").text, "Neu laden (eigene Änderungen verwerfen)")
        compare(findChild(notice, "overwrite").text, "Überschreiben")
    }

    function test_the_english_text() {
        fake.outsideFiles = "Screenplay"
        const notice = make()
        notice.texts = en

        compare(notice.text, "Changed outside and here: Screenplay.")
        compare(findChild(notice, "overwrite").text, "Overwrite")
    }

    function test_the_answers_call_the_controller_and_the_note_goes() {
        fake.outsideFiles = "Drehbuch"
        const notice = make()

        findChild(notice, "reload").clicked()
        compare(fake.calls, ["reload"])
        verify(!notice.visible)

        fake.outsideFiles = "Drehbuch"
        findChild(notice, "overwrite").clicked()
        compare(fake.calls, ["reload", "overwrite"])
        verify(!notice.visible)
    }

    function test_no_project_no_note() {
        fake.outsideFiles = "Drehbuch"
        fake.loaded = false

        verify(!make().visible)
    }
}
