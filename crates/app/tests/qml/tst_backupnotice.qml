import QtQuick
import QtTest
import "../../qml"

// BackupNotice: shows for a found or an unreadable backup, offers the right answers and calls
// the controller; with no backup nothing shows.
TestCase {
    id: tc
    name: "BackupNotice"
    width: 900
    height: 300
    when: windowShown
    visible: true

    QtObject {
        id: fake
        property string backupState: "none"
        property string backupMessage: ""
        property var calls: []

        function restoreBackup() {
            calls.push("restore")
            backupState = "none"
        }
        function discardBackup() {
            calls.push("discard")
            backupState = "none"
        }
    }

    TestTexts { id: de; language: "de" }
    TestTexts { id: en; language: "en" }

    Component {
        id: noticeComponent
        BackupNotice {
            width: 800
            controller: fake
            texts: de
        }
    }

    function init() {
        fake.backupState = "none"
        fake.backupMessage = ""
        fake.calls = []
    }

    function make() {
        const notice = createTemporaryObject(noticeComponent, tc)
        verify(notice)
        return notice
    }

    function test_nothing_shows_without_a_backup() {
        verify(!make().visible)
    }

    function test_a_found_backup_offers_restore_and_discard() {
        fake.backupState = "found"
        const notice = make()

        verify(notice.visible)
        compare(notice.text, "Ungespeicherte Änderungen aus einer früheren Sitzung gefunden. Wiederherstellen ersetzt auch das Drehbuch auf der Platte.")
        compare(findChild(notice, "restore").text, "Wiederherstellen")
        compare(findChild(notice, "discard").text, "Verwerfen")
        verify(findChild(notice, "restore").visible)
    }

    function test_the_english_text() {
        fake.backupState = "found"
        const notice = make()
        notice.texts = en

        compare(findChild(notice, "restore").text, "Restore")
        compare(findChild(notice, "discard").text, "Discard")
    }

    function test_an_unreadable_backup_names_the_reason_and_only_discards() {
        fake.backupState = "unreadable"
        fake.backupMessage = "projekt.toml fehlt"
        const notice = make()

        verify(notice.visible)
        compare(notice.text, "Eine Sicherung ist unlesbar: projekt.toml fehlt")
        verify(!findChild(notice, "restore").visible)
        verify(findChild(notice, "discard").visible)
    }

    function test_restore_calls_the_controller_and_the_note_goes() {
        fake.backupState = "found"
        const notice = make()

        findChild(notice, "restore").clicked()

        compare(fake.calls, ["restore"])
        verify(!notice.visible)
    }

    function test_discard_calls_the_controller_and_the_note_goes() {
        fake.backupState = "unreadable"
        const notice = make()

        findChild(notice, "discard").clicked()

        compare(fake.calls, ["discard"])
        verify(!notice.visible)
    }
}
