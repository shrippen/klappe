import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// DB-06: the title page of the screenplay as a form. The six fields of the Fountain title page;
// Apply writes them into the text (the block at the top, an empty field is left out), Cancel
// changes nothing. The change reaches the text as one replacement of the editor (`putText`, as a
// replacement of the search), so Ctrl+Z in the text takes it back as one step.
//
//  ┌ Titelseite ───────────────┐
//  │ Titel        [ Heimat   ] │
//  │ Nennung      [          ] │
//  │ Autor        [ Eva      ] │
//  │ Kontakt      [ Eva      ] │
//  │              [ Weg 1    ] │
//  │         [Abbrechen][Übern.]│
//  └───────────────────────────┘
FormDialog {
    id: form

    /// The editor's text area, read for the text that stands now.
    required property var area
    /// Puts text in place of `from` to `to` as one undo step of the editor.
    required property var putText

    /// The single-line fields in the order of the form.
    readonly property var singles: [fTitle, fCredit, fAuthor, fSource, fDraft]
    readonly property string contactKey: "Contact"
    readonly property alias contactField: contact
    /// The height of the contact field in grid units: four lines.
    readonly property int contactGridUnits: 5

    title: texts.t("titlepage.title")
    acceptText: texts.t("form.apply")

    /// Fills the fields from the text and opens the form.
    function show() {
        // The fields exist once the dialog is open.
        open()
        const values = JSON.parse(controller.titleFields(area.text))
        for (let i = 0; i < singles.length; i++) {
            singles[i].text = values[singles[i].key]
        }
        contact.text = values[contactKey]
        fTitle.focusField()
    }

    /// What the fields hold, by key.
    function values() {
        const out = {}
        for (let i = 0; i < singles.length; i++) {
            out[singles[i].key] = singles[i].text
        }
        out[contactKey] = contact.text
        return out
    }

    /// Sets the field of `key` (a single-line one or the contact).
    function setField(key, value) {
        if (key === contactKey) {
            contact.text = value
            return
        }
        singles.find(f => f.key === key).text = value
    }

    /// The part of `before` that `after` replaces: `{ from, to, text }`, so only the title block
    /// is replaced and the cursor stays where it was.
    function change(before, after) {
        const room = Math.min(before.length, after.length)
        let from = 0
        while (from < room && before[from] === after[from]) {
            from++
        }
        let tail = 0
        while (tail < room - from && before[before.length - 1 - tail] === after[after.length - 1 - tail]) {
            tail++
        }
        return { from: from, to: before.length - tail, text: after.slice(from, after.length - tail) }
    }

    submit: function () {
        const before = area.text
        const after = controller.titleText(before, JSON.stringify(values()))
        if (after === before) {
            return true
        }
        const c = change(before, after)
        putText(c.from, c.to, c.text)
        return true
    }

    component Single: ColumnLayout {
        id: single
        required property string key
        required property string label
        property alias text: field.text
        function focusField() { field.forceActiveFocus() }

        Layout.fillWidth: true
        spacing: 0

        KanteSectionLabel { text: form.texts.t(single.label) }
        KanteTextField {
            id: field
            objectName: "field." + single.key
            Layout.fillWidth: true
            Accessible.name: form.texts.t(single.label)
            onAccepted: form.accept()
        }
    }

    Single { id: fTitle; key: "Title"; label: "titlepage.f.title" }
    Single { id: fCredit; key: "Credit"; label: "titlepage.f.credit" }
    Single { id: fAuthor; key: "Author"; label: "titlepage.f.author" }
    Single { id: fSource; key: "Source"; label: "titlepage.f.source" }
    Single { id: fDraft; key: "Draft date"; label: "titlepage.f.draft" }

    // Kante has no multi-line field: the text area sits in a Kante card with the field's colours.
    KanteSectionLabel { text: form.texts.t("titlepage.f.contact") }
    KanteCard {
        Layout.fillWidth: true
        Layout.preferredHeight: Kirigami.Units.gridUnit * form.contactGridUnits
        color: KanteStyle.sunkenColor
        borderColor: KanteStyle.frameColor
        chamfer: 0

        QQC2.ScrollView {
            anchors.fill: parent
            QQC2.TextArea {
                id: contact
                objectName: "field.Contact"
                wrapMode: TextEdit.Wrap
                color: KanteStyle.textColor
                selectionColor: KanteStyle.selectionColor
                selectedTextColor: KanteStyle.textColor
                background: null
                Accessible.name: form.texts.t("titlepage.f.contact")
                // Tab leaves the field like in the others (a tab character has no place in a contact).
                Keys.onTabPressed: nextItemInFocusChain(true).forceActiveFocus()
                Keys.onBacktabPressed: nextItemInFocusChain(false).forceActiveFocus()
            }
        }
    }
}
