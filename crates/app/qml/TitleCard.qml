import QtQuick
import QtQuick.Layouts
import Kante

// DB-06: the title page of the screenplay as the top card above the flowing text. Folded it is
// one line (the title and the author, or "empty"); unfolded it holds the six fields of the
// Fountain title page. Apply writes them into the text (the block at the top, an empty field is
// left out) as one replacement of the editor (`putText`, as one of the search), so Ctrl+Z in
// the text takes it back as one step; Discard sets the fields to what the text says. It follows
// the text, except in a field typed in the card.
//
//  ┌ TITELSEITE · Heimat · Eva ─ ▸ ┐     ┌ TITELSEITE ──────────────── ▾ ┐
//  └───────────────────────────────┘     │ Titel [ Heimat ] Nennung [   ] │
//                                        │ Autor [ Eva    ] Vorlage [   ] │
//                                        │ Fassung vom [  ]               │
//                                        │ Kontakt [ Eva ]                │
//                                        │          [Verwerfen][Übernehmen]│
//                                        └────────────────────────────────┘
KanteFloatingPanel {
    id: card

    required property var texts
    required property var controller
    /// The editor's text (ScriptDoc), read for the text that stands now.
    required property var doc
    /// Puts text in place of `from` to `to` as one undo step of the editor.
    required property var putText

    /// The single-line fields in the order of the form.
    readonly property var singles: [fTitle, fCredit, fAuthor, fSource, fDraft]
    readonly property string contactKey: "Contact"
    readonly property alias contactField: contact
    /// Lines of the contact field before it scrolls.
    readonly property int contactLines: 4
    /// What the text said when the fields were last read, as JSON.
    property string base: ""
    /// Something is typed in the card that the text does not have.
    readonly property bool dirty: base !== "" && JSON.stringify(values()) !== base
    /// The title and the author as the text has them, for the folded line.
    property string summary: ""
    /// Asks the app to give the keys back to the text: after Apply and after Esc.
    signal applied()
    signal dismissed()

    collapsed: true
    collapseText: texts.t("titlepage.collapse")
    expandText: texts.t("titlepage.expand")
    title: collapsed ? texts.t("titlepage.title") + " · " + (summary !== "" ? summary : texts.t("titlepage.empty"))
                       + (dirty ? " · " + texts.t("titlepage.changed") : "")
                     : texts.t("titlepage.title")

    // Esc in a field folds the card and gives the keys back to the text; what is typed stays.
    Keys.onEscapePressed: event => {
        if (collapsed) {
            return
        }
        collapsed = true
        dismissed()
        event.accepted = true
    }

    /// Reads the fields from the text, but keeps a field typed in the card: Apply then writes
    /// that field and leaves what the text got since (e.g. a title typed there) as it is.
    function follow() {
        if (!dirty) {
            discard()
            return
        }
        const was = JSON.parse(base)
        const typed = values()
        const now = read()
        for (const key in now) {
            if (typed[key] === was[key]) {
                setField(key, now[key])
            }
        }
        take(now)
    }

    /// Sets the fields to what the text says.
    function discard() {
        const now = read()
        for (const key in now) {
            setField(key, now[key])
        }
        take(now)
    }

    /// What the text says, by key in the order of `values()`.
    function read() {
        const fields = JSON.parse(controller.titleFields(doc.read()))
        const out = {}
        for (let i = 0; i < singles.length; i++) {
            out[singles[i].key] = fields[singles[i].key]
        }
        out[contactKey] = fields[contactKey]
        return out
    }

    /// Marks `now` (what the text says) as read: the base of `dirty` and the folded line.
    function take(now) {
        base = JSON.stringify(now)
        summary = [now["Title"], now["Author"]].filter(v => v !== "").join(" · ")
    }

    /// Unfolds the card and puts the cursor in the first field.
    function show() {
        follow()
        collapsed = false
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

    /// Writes the fields into the text, folds the card and gives the keys back to the text.
    function apply() {
        follow()
        const before = doc.read()
        const after = controller.titleText(before, JSON.stringify(values()))
        if (after !== before) {
            const c = change(before, after)
            putText(c.from, c.to, c.text)
        }
        discard()
        collapsed = true
        applied()
    }

    component Single: ColumnLayout {
        id: single
        required property string key
        required property string label
        property alias text: field.text
        function focusField() { field.forceActiveFocus() }

        Layout.fillWidth: true
        Layout.preferredWidth: 1
        spacing: 0

        KanteSectionLabel { text: card.texts.t(single.label) }
        KanteTextField {
            id: field
            objectName: "field." + single.key
            Layout.fillWidth: true
            Accessible.name: card.texts.t(single.label)
            onAccepted: card.apply()
        }
    }

    GridLayout {
        width: parent.width
        columns: 2
        columnSpacing: KanteStyle.unit(10)
        rowSpacing: KanteStyle.unit(8)

        Single { id: fTitle; key: "Title"; label: "titlepage.f.title" }
        Single { id: fCredit; key: "Credit"; label: "titlepage.f.credit" }
        Single { id: fAuthor; key: "Author"; label: "titlepage.f.author" }
        Single { id: fSource; key: "Source"; label: "titlepage.f.source" }
        Single { id: fDraft; key: "Draft date"; label: "titlepage.f.draft" }

        ColumnLayout {
            Layout.columnSpan: 2
            Layout.fillWidth: true
            spacing: 0

            KanteSectionLabel { text: card.texts.t("titlepage.f.contact") }
            KanteTextArea {
                id: contact
                objectName: "field.Contact"
                Layout.fillWidth: true
                maxLines: card.contactLines
                Accessible.name: card.texts.t("titlepage.f.contact")
            }
        }

        RowLayout {
            Layout.columnSpan: 2
            Layout.alignment: Qt.AlignRight
            spacing: KanteStyle.unit(8)

            KanteButton {
                objectName: "titleDiscard"
                emphasis: KanteButton.Emphasis.Quiet
                enabled: card.dirty
                text: card.texts.t("titlepage.discard")
                onClicked: card.discard()
            }
            KanteButton {
                objectName: "titleApply"
                enabled: card.dirty
                text: card.texts.t("form.apply")
                onClicked: card.apply()
            }
        }
    }
}
