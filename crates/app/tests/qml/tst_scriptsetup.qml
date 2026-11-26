import QtQuick
import QtTest
import "../../qml"

// ScriptSetupForm in the project dialog (DN-05, DB-31, DB-33): what it shows of the setup and
// what each change sends the controller: always the whole setup.
TestCase {
    id: tc
    name: "ScriptSetupForm"
    width: 800
    height: 600
    when: windowShown
    visible: true

    QtObject {
        id: fake
        property string status: ""
        property var calls: []
        // The project view takes the setup the controller was given, as the session does.
        property var script: ({ language: "de", words: "local", paper: "a4", contd_after_action: false,
                               scene_numbering: "locked", first_scene: 1 })

        function setScriptSetup(language, words, paper, contd, numbering, first) {
            calls.push([language, words, paper, contd, numbering, first])
            script = { language: language, words: words, paper: paper, contd_after_action: contd === "on",
                       scene_numbering: numbering, first_scene: first }
            return true
        }
    }

    TestTexts { id: de; language: "de" }
    TestTexts { id: en; language: "en" }

    Component {
        id: formComponent
        ScriptSetupForm {
            texts: de
            controller: fake
            script: fake.script
            papers: [{ id: "a4", label: "A4" }, { id: "letter", label: "Letter" }]
        }
    }

    function init() {
        fake.calls = []
        fake.script = { language: "de", words: "local", paper: "a4", contd_after_action: false,
                        scene_numbering: "locked", first_scene: 1 }
    }

    function make(props) {
        const form = createTemporaryObject(formComponent, tc, props || {})
        verify(form)
        return form
    }

    function test_it_shows_the_setup() {
        fake.script = { language: "en", words: "local", paper: "letter", contd_after_action: true,
                        scene_numbering: "automatic", first_scene: 5 }
        const form = make()

        compare(findChild(form, "languageBox").currentIndex, 1)
        compare(findChild(form, "paperBox").currentIndex, 1)
        verify(findChild(form, "contdSwitch").checked)
        compare(findChild(form, "sceneNumberingBox").currentIndex, 0)
        compare(findChild(form, "firstScene").value, 5)
    }

    function test_each_change_sends_the_whole_setup() {
        const form = make()

        findChild(form, "paperBox").activated(1)
        findChild(form, "wordsBox").activated(1)
        findChild(form, "languageBox").activated(1)

        compare(fake.calls, [
            ["de", "local", "letter", "off", "locked", 1],
            ["de", "international", "letter", "off", "locked", 1],
            ["en", "international", "letter", "off", "locked", 1]
        ])
    }

    function test_contd_after_action_is_a_switch() {
        const form = make()
        const box = findChild(form, "contdSwitch")

        box.toggle()
        box.toggled()

        compare(fake.calls, [["de", "local", "a4", "on", "locked", 1]])
    }

    // The words are German ones: only a German project chooses.
    function test_the_words_are_for_german_projects() {
        const form = make()
        verify(findChild(form, "wordsBox").enabled)

        fake.script = { language: "en", words: "local", paper: "a4", contd_after_action: false,
                        scene_numbering: "locked", first_scene: 1 }
        verify(!findChild(form, "wordsBox").enabled)
    }

    // The first number counts only while Klappe numbers.
    function test_the_first_number_is_for_automatic_numbers() {
        const form = make()
        const first = findChild(form, "firstScene")
        verify(!first.enabled)

        findChild(form, "sceneNumberingBox").activated(0)
        verify(first.enabled)
        first.value = 12
        first.valueModified()

        compare(fake.calls, [
            ["de", "local", "a4", "off", "automatic", 1],
            ["de", "local", "a4", "off", "automatic", 12]
        ])
    }

    function test_it_is_english_too() {
        const form = make({ texts: en })

        compare(findChild(form, "contdSwitch").text, "(CONT'D) also after action")
    }
}
