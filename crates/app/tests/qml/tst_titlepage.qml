import QtQuick
import QtQuick.Controls as QQC2
import QtTest
import "../../qml"

// TitlePageDialog (DB-06): the form shows the fields the controller reads from the text, Apply
// puts only the changed part of the text in place (one `putText`), Cancel puts nothing, and
// nothing changed puts nothing.
TestCase {
    id: tc
    name: "TitlePageDialog"
    width: 800
    height: 600
    when: windowShown
    visible: true

    property var puts: []
    property var fields: ({ "Title": "Heimat", "Credit": "", "Author": "", "Source": "",
                            "Draft date": "", "Contact": "Eva\nWeg 1" })
    // What the controller makes of the text: the title line replaced, the rest kept.
    property string result: ""

    QtObject {
        id: fake
        property string status: ""
        function clearStatus() {}
        function titleFields(text) { return JSON.stringify(tc.fields) }
        function titleText(text, json) { return tc.result !== "" ? tc.result : text }
    }

    TextEdit { id: editor; text: "Title: Heimat\nContact:\n  Eva\n  Weg 1\n\nINT. SAAL\n" }
    TestTexts { id: de; language: "de" }
    TestTexts { id: en; language: "en" }

    Component {
        id: formComponent
        TitlePageDialog {
            texts: de
            controller: fake
            area: editor
            putText: (from, to, text) => tc.puts.push({ from: from, to: to, text: text })
        }
    }

    function init() {
        puts = []
        result = ""
    }

    function make() {
        const form = createTemporaryObject(formComponent, tc)
        verify(form)
        return form
    }

    function test_the_form_shows_the_fields() {
        const form = make()

        form.show()

        compare(form.title, "Titelseite")
        compare(findChild(form, "field.Title").text, "Heimat")
        compare(findChild(form, "field.Author").text, "")
        compare(form.contactField.text, "Eva\nWeg 1")
        compare(form.values()["Title"], "Heimat")
    }

    function test_it_is_english_too() {
        const form = createTemporaryObject(formComponent, tc, { texts: en })

        compare(form.title, "Title page")
    }

    function test_apply_puts_only_the_changed_part() {
        const form = make()
        form.show()
        result = "Title: Neu\nContact:\n  Eva\n  Weg 1\n\nINT. SAAL\n"

        form.accept()

        compare(puts.length, 1)
        compare(puts[0].from, "Title: ".length)
        compare(puts[0].to, "Title: Heimat".length)
        compare(puts[0].text, "Neu")
    }

    function test_cancel_puts_nothing() {
        const form = make()
        form.show()
        result = "Title: Neu\n"

        form.reject()

        compare(puts.length, 0)
    }

    function test_unchanged_fields_put_nothing() {
        const form = make()
        form.show()

        form.accept()

        compare(puts.length, 0)
    }

    function test_change_finds_the_replaced_part() {
        const form = make()

        compare(JSON.stringify(form.change("abcd", "abXd")), JSON.stringify({ from: 2, to: 3, text: "X" }))
        compare(JSON.stringify(form.change("abc", "abcd")), JSON.stringify({ from: 3, to: 3, text: "d" }))
        compare(JSON.stringify(form.change("aXa", "aa")), JSON.stringify({ from: 1, to: 2, text: "" }))
        compare(JSON.stringify(form.change("", "x")), JSON.stringify({ from: 0, to: 0, text: "x" }))
    }

    function test_the_fields_are_named_for_the_screen_reader() {
        const form = make()

        compare(findChild(form, "field.Title").Accessible.name, "Titel")
        compare(form.contactField.Accessible.name, "Kontakt")
    }
}
