import QtQuick
import QtTest
import "../../qml"

// The object commands of AppHeader (LH-114 to LH-116, LH-118): mirror, size, label and unbind
// are commands of the palette with a key; each kind of the library is a command of its own.
TestCase {
    id: tc
    name: "AppHeaderObjects"
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
        readonly property var names: ["select"]
        readonly property var keys: ["V"]
        readonly property var modes: ["build"]
        readonly property var modeKeys: ["1"]
    }

    TestTexts { id: de; language: "de" }

    Component {
        id: headerComponent
        AppHeader {
            controller: fake
            texts: de
            tools: fakeTools
            width: tc.width
        }
    }

    function init() {
        fake.selectedKey = ""
        fake.noteBound = false
        fake.calls = []
    }

    function make() {
        const header = createTemporaryObject(headerComponent, tc)
        verify(header)
        return header
    }

    function command(header, id) {
        return header.commandPalette.commands.find(c => c.id === id)
    }

    // LH-114 to LH-116, LH-118: mirror, size, label and unbind are commands with a key; they
    // wait for a selection they fit; each kind of the library is a command of its own.
    function test_the_object_commands_wait_for_their_selection() {
        const header = make()
        const ids = ["object.mirror", "object.resetSize", "object.label", "note.unbind"]

        for (const id of ids) {
            verify(!command(header, id).enabled, id)
        }
        compare(header.commandPalette.commands.filter(c => c.id.startsWith("kind:")).length, 0)

        fake.selectedKey = "object:o1"
        for (const id of ids.slice(0, 3)) {
            verify(command(header, id).enabled, id)
        }
        verify(!command(header, "note.unbind").enabled)

        fake.selectedKey = "mark:elisabeth:1"
        verify(command(header, "object.label").enabled)
        verify(!command(header, "object.mirror").enabled)

        fake.selectedKey = "objectgrip:o1:w"
        verify(!command(header, "object.label").enabled)

        fake.selectedKey = "note:n1"
        verify(!command(header, "note.unbind").enabled)
        fake.noteBound = true
        fake.pictureRevision++
        verify(command(header, "note.unbind").enabled)
    }

    function test_the_object_commands_have_keys_and_reach_the_controller() {
        const header = make()
        fake.selectedKey = "object:o1"

        compare(command(header, "object.mirror").keys, "Umschalt+M")
        compare(command(header, "object.resetSize").keys, "Umschalt+S")
        compare(command(header, "object.label").keys, "Umschalt+L")
        compare(command(header, "note.unbind").keys, "Umschalt+U")

        header.run("object.mirror")
        header.run("object.resetSize")
        header.run("object.label")
        fake.selectedKey = "note:n1"
        fake.noteBound = true
        fake.pictureRevision++
        header.run("note.unbind")

        compare(fake.calls, [["changeSelected", "mirror", ""], ["changeSelected", "resetSize", ""],
                             ["labelSelected"], ["changeSelected", "unbind", ""]])
    }

    function test_every_library_kind_is_a_command_for_a_selected_object() {
        const header = make()
        fake.selectedKey = "object:o1"

        const kinds = header.commandPalette.commands.filter(c => c.id.startsWith("kind:"))
        compare(kinds.map(c => c.id), ["kind:table", "kind:sofa"])
        compare(kinds[1].text, "Art ändern: Sofa")

        header.run("kind:sofa")
        compare(fake.calls, [["changeSelected", "kind", "sofa"]])
    }

    function test_the_texts_are_in_both_languages() {
        const en = createTemporaryObject(textsComponent, tc)
        for (const key of ["object.kind", "object.kindTo", "object.mirror", "object.mirrorTip", "object.resetSize",
                           "object.resetSizeTip", "object.label", "object.labelTip", "note.unbind"]) {
            verify(de.t(key) !== key, "de " + key)
            verify(en.t(key) !== key, "en " + key)
        }
    }

    Component {
        id: textsComponent
        TestTexts { language: "en" }
    }
}
