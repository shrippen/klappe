import QtQuick
import QtQuick.Controls as QQC2
import QtTest
import "../../qml"

// TitleCard (DB-06): folded by default with the title and the author; unfolded it shows the fields
// the controller reads from the text; Apply puts only the changed part of the text in place (one
// `putText`) and folds, Discard sets the fields back, nothing changed puts nothing; the card
// follows the text, except in a field typed in it; Enter and Space on the head fold it.
TestCase {
    id: tc
    name: "TitleCard"
    width: 800
    height: 600
    when: windowShown
    visible: true

    property var puts: []
    property int appliedCount: 0
    property int dismissedCount: 0
    property var fields: ({ "Title": "Heimat", "Credit": "", "Author": "", "Source": "",
                            "Draft date": "", "Contact": "Eva\nWeg 1" })
    // What the controller makes of the text: the title line replaced, the rest kept.
    property string result: ""
    // The fields the last Apply sent, as JSON.
    property string sent: ""

    QtObject {
        id: fake
        function titleFields(text) { return JSON.stringify(tc.fields) }
        function titleText(text, json) { tc.sent = json; return tc.result !== "" ? tc.result : text }
    }

    TextEdit { id: editor; text: "Title: Heimat\nContact:\n  Eva\n  Weg 1\n\nINT. SAAL\n" }
    TestTexts { id: de; language: "de" }
    TestTexts { id: en; language: "en" }

    Component {
        id: formComponent
        TitleCard {
            texts: de
            controller: fake
            area: editor
            putText: (from, to, text) => tc.puts.push({ from: from, to: to, text: text })
            onApplied: tc.appliedCount++
            onDismissed: tc.dismissedCount++
        }
    }

    function init() {
        puts = []
        appliedCount = 0
        dismissedCount = 0
        result = ""
        sent = ""
        fields = { "Title": "Heimat", "Credit": "", "Author": "Eva", "Source": "",
                   "Draft date": "", "Contact": "Eva\nWeg 1" }
    }

    function make() {
        const form = createTemporaryObject(formComponent, tc)
        verify(form)
        form.follow()
        return form
    }

    function head(form) {
        return form.children.find(c => c.activeFocusOnTab)
    }

    function test_it_starts_folded_with_title_and_author() {
        const form = make()

        verify(form.collapsed)
        compare(form.title, "Titelseite · Heimat · Eva")
    }

    function test_without_a_title_page_the_folded_line_says_empty() {
        fields = { "Title": "", "Credit": "", "Author": "", "Source": "", "Draft date": "", "Contact": "" }
        const form = make()

        compare(form.title, "Titelseite · leer")
    }

    function test_show_unfolds_and_focuses_the_first_field() {
        const form = make()

        form.show()

        verify(!form.collapsed)
        compare(form.title, "Titelseite")
        verify(findChild(form, "field.Title").activeFocus)
        compare(findChild(form, "field.Title").text, "Heimat")
        compare(findChild(form, "field.Author").text, "Eva")
        compare(form.contactField.text, "Eva\nWeg 1")
    }

    function test_it_is_english_too() {
        const form = createTemporaryObject(formComponent, tc, { texts: en })

        compare(form.collapseText, "Fold the title page")
        compare(form.texts.t("titlepage.title"), "Title page")
    }

    function test_enter_and_space_on_the_head_fold_it() {
        const form = make()
        head(form).forceActiveFocus()

        keyClick(Qt.Key_Space)
        verify(!form.collapsed)
        keyClick(Qt.Key_Return)
        verify(form.collapsed)
    }

    function test_apply_puts_only_the_changed_part_and_folds() {
        const form = make()
        form.show()
        result = "Title: Neu\nContact:\n  Eva\n  Weg 1\n\nINT. SAAL\n"
        form.setField("Title", "Neu")

        form.apply()

        compare(puts.length, 1)
        compare(puts[0].from, "Title: ".length)
        compare(puts[0].to, "Title: Heimat".length)
        compare(puts[0].text, "Neu")
        verify(form.collapsed)
        compare(appliedCount, 1)
    }

    function test_the_apply_button_needs_a_change() {
        const form = make()
        form.show()
        const apply = findChild(form, "titleApply")

        verify(!apply.enabled)
        form.setField("Author", "Anna")
        verify(apply.enabled)
        verify(form.dirty)
    }

    function test_unchanged_fields_put_nothing() {
        const form = make()
        form.show()

        form.apply()

        compare(puts.length, 0)
    }

    function test_discard_sets_the_fields_back() {
        const form = make()
        form.show()
        form.setField("Title", "Neu")
        form.setField("Contact", "Anna")

        form.discard()

        compare(findChild(form, "field.Title").text, "Heimat")
        compare(form.contactField.text, "Eva\nWeg 1")
        verify(!form.dirty)
        verify(!form.collapsed)
        compare(puts.length, 0)
    }

    function test_it_follows_the_text_while_nothing_is_typed() {
        const form = make()
        form.show()

        fields = { "Title": "Neu", "Credit": "", "Author": "Eva", "Source": "",
                   "Draft date": "", "Contact": "" }
        form.follow()

        compare(findChild(form, "field.Title").text, "Neu")
        compare(form.contactField.text, "")
    }

    function test_it_keeps_what_is_typed_when_the_text_changes() {
        const form = make()
        form.show()
        form.setField("Author", "Anna")

        fields = { "Title": "Neu", "Credit": "", "Author": "Eva", "Source": "",
                   "Draft date": "", "Contact": "" }
        form.follow()

        compare(findChild(form, "field.Author").text, "Anna")
        verify(form.dirty)
    }

    // Review of PR 356: a field not typed in the card takes what the text says now, so Apply
    // never sets a title typed in the text back.
    function test_untyped_fields_follow_the_text_while_something_is_typed() {
        const form = make()
        form.show()
        form.setField("Author", "Anna")

        fields = { "Title": "Neu", "Credit": "", "Author": "Eva", "Source": "",
                   "Draft date": "", "Contact": "" }
        form.follow()

        compare(findChild(form, "field.Title").text, "Neu")
        compare(form.contactField.text, "")
    }

    function test_apply_keeps_what_the_text_got_since_the_card_read_it() {
        const form = make()
        form.show()
        form.setField("Author", "Anna")

        fields = { "Title": "Neu", "Credit": "", "Author": "Eva", "Source": "",
                   "Draft date": "", "Contact": "Eva\nWeg 1" }
        form.apply()

        const json = JSON.parse(sent)
        compare(json["Title"], "Neu")
        compare(json["Author"], "Anna")
    }

    function test_the_folded_line_follows_the_text_while_something_is_typed() {
        const form = make()
        form.setField("Author", "Anna")

        fields = { "Title": "Neu", "Credit": "", "Author": "Eva", "Source": "",
                   "Draft date": "", "Contact": "" }
        form.follow()

        compare(form.title, "Titelseite · Neu · Eva · geändert")
    }

    function test_the_folded_line_follows_the_text() {
        const form = make()

        fields = { "Title": "Neu", "Credit": "", "Author": "", "Source": "",
                   "Draft date": "", "Contact": "" }
        form.follow()

        compare(form.title, "Titelseite · Neu")
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

    function test_esc_in_a_field_folds_and_keeps_what_is_typed() {
        const form = make()
        form.show()
        form.setField("Author", "Anna")
        verify(findChild(form, "field.Title").activeFocus)

        keyClick(Qt.Key_Escape)

        verify(form.collapsed)
        compare(dismissedCount, 1)
        compare(form.values()["Author"], "Anna")
        verify(form.dirty)
        compare(puts.length, 0)
    }

    function test_esc_in_the_contact_field_folds_too() {
        const form = make()
        form.show()
        form.contactField.area.forceActiveFocus()

        keyClick(Qt.Key_Escape)

        verify(form.collapsed)
        compare(dismissedCount, 1)
    }

    function test_the_folded_line_says_changed_while_something_is_typed() {
        const form = make()
        form.show()
        form.setField("Author", "Anna")
        form.collapsed = true

        compare(form.title, "Titelseite · Heimat · Eva · geändert")
        form.discard()
        compare(form.title, "Titelseite · Heimat · Eva")
    }

    function test_the_changed_note_is_english_too() {
        const form = createTemporaryObject(formComponent, tc, { texts: en })
        form.follow()
        form.setField("Author", "Anna")

        compare(form.title, "Title page · Heimat · Eva · changed")
    }
}
