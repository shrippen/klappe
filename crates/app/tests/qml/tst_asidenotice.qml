import QtQuick
import QtTest
import "../../qml"

// AsideNotice: after the example replaced an old copy, the note says where the old one went,
// whole, until the user closes it.
TestCase {
    id: tc
    name: "AsideNotice"
    width: 400
    height: 300
    when: windowShown
    visible: true

    readonly property string note: "Die alte Beispielkopie passt nicht mehr zum Format und liegt jetzt unter /home/anna/.local/share/klappe/beispiel-de.klappe.alt-2."

    QtObject {
        id: fake
        property string exampleAside: ""
    }

    Component {
        id: noticeComponent
        AsideNotice {
            width: 360
            controller: fake
        }
    }

    function init() {
        fake.exampleAside = ""
    }

    function make() {
        const notice = createTemporaryObject(noticeComponent, tc)
        verify(notice)
        return notice
    }

    // The callout's close cross: what a click or a screen reader runs.
    function cross(item) {
        for (let i = 0; i < item.children.length; ++i) {
            const child = item.children[i]
            if (child.text === "×") {
                return child
            }
            const found = cross(child)
            if (found) {
                return found
            }
        }
        return null
    }

    function test_nothing_shows_without_a_note() {
        verify(!make().visible)
    }

    function test_the_note_shows_whole_and_wraps() {
        const notice = make()

        fake.exampleAside = note

        verify(notice.visible)
        compare(notice.text, note)
        tryVerify(() => notice.implicitHeight > 60, 2000, "wraps onto more lines")
    }

    function test_closing_empties_the_note() {
        const notice = make()
        fake.exampleAside = note

        notice.dismissed()

        compare(fake.exampleAside, "")
        verify(!notice.visible)
    }

    function test_another_project_takes_the_note_away() {
        const notice = make()
        fake.exampleAside = note

        fake.exampleAside = ""

        verify(!notice.visible)
        fake.exampleAside = note
        verify(notice.visible, "a later note shows again")
    }

    function test_a_later_note_shows_after_closing_with_the_cross() {
        const notice = make()
        fake.exampleAside = note

        cross(notice).dismiss()
        fake.exampleAside = note

        verify(notice.visible, "closing must not cut the binding")
    }
}
