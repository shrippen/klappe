import QtQuick
import QtTest
import "../../qml"

// AppHeader and the open questions about the book: the palette offers "match scenes with the
// book" only while questions are open, and running it asks the window for the dialog. Split from
// tst_appheader.qml, which holds the rest.
TestCase {
    id: tc
    name: "AppHeaderBook"
    width: 1000
    height: 600
    when: windowShown
    visible: true

    FakeHeaderController { id: fake }

    QtObject {
        id: fakeTools
        property string tool: "select"
        property string mode: "build"
        property bool enabled: true
        readonly property var names: ["select", "wall", "camera"]
        readonly property var keys: ["V", "W", "C"]
        readonly property var modes: ["build", "stage", "resolve", "draw"]
        readonly property var modeKeys: ["1", "2", "3", "4"]
    }

    TestTexts { id: de; language: "de" }

    SignalSpy { id: bookSpy; signalName: "bookRequested" }

    Component {
        id: headerComponent
        AppHeader {
            controller: fake
            texts: de
            tools: fakeTools
            width: tc.width
        }
    }

    function test_the_palette_offers_the_book_questions_only_while_open() {
        const header = createTemporaryObject(headerComponent, tc)
        verify(header)
        bookSpy.target = header
        bookSpy.clear()
        const entry = () => header.commandPalette.commands.find(c => c.text === "Szenen mit dem Buch abgleichen …")

        verify(!entry().enabled)
        fake.projectJson = JSON.stringify({ scenes: fake.scenes, plans: fake.plans, plan: fake.plan, formats: fake.formats,
                                            questions: [{ kind: "scene", key: "a", title: "", ask: "", answers: [] }] })
        verify(entry().enabled)

        header.run("book.match")

        compare(bookSpy.count, 1)
    }

    function test_open_places_alone_enable_the_command() {
        const header = createTemporaryObject(headerComponent, tc)
        verify(header)
        const before = fake.projectJson
        fake.projectJson = JSON.stringify({ scenes: fake.scenes, plans: fake.plans, plan: fake.plan, formats: fake.formats,
                                            questions: [{ kind: "place", key: "HOF", title: "", ask: "", answers: [] }] })

        verify(header.commandPalette.commands.find(c => c.text === "Szenen mit dem Buch abgleichen …").enabled)
        fake.projectJson = before
    }
}
