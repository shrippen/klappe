import QtQuick
import QtTest
import "../../qml"

// PrintView (D-15, D-36): the pages as they print; emphasis shows as bold, italic and an
// underline, never as marks.
TestCase {
    id: tc
    name: "PrintView"
    width: 900
    height: 600
    when: windowShown
    visible: true

    readonly property var sheet: ({
        fontPt: 12, ascentPt: 10,
        pages: [{ width: 595, height: 842, texts: [
            { x: 108, y: 82, bold: false, italic: false, underline: false, text: "Sie ist " },
            { x: 165.6, y: 82, bold: true, italic: false, underline: false, text: "sehr" },
            { x: 201.6, y: 82, bold: false, italic: true, underline: false, text: "müde" },
            { x: 266.4, y: 82, bold: true, italic: true, underline: true, text: "allein" }
        ] }]
    })

    QtObject {
        id: fake
        function scriptPrintJson() { return JSON.stringify(tc.sheet) }
    }

    TestTexts { id: de; language: "de" }

    Component {
        id: viewComponent
        PrintView { width: 700; height: 500; texts: de; controller: fake }
    }

    function line(view, index) {
        const page = view.itemAtIndex(0)
        verify(page)
        return findChild(page, "line." + index)
    }

    function test_emphasis_shows_as_face_and_underline() {
        const view = createTemporaryObject(viewComponent, tc)
        view.refresh()
        wait(50)

        const plain = line(view, 0)
        verify(plain)
        compare(plain.text, "Sie ist ")
        verify(!plain.font.bold && !plain.font.italic && !plain.font.underline)

        verify(line(view, 1).font.bold)
        verify(!line(view, 1).font.italic)
        verify(line(view, 2).font.italic)
        verify(!line(view, 2).font.underline)

        const both = line(view, 3)
        verify(both.font.bold && both.font.italic && both.font.underline)
        compare(both.text, "allein")
    }
}
