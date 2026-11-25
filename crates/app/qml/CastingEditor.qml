import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// Besetzung (D-26): one role. Its cast number (shown, and set by hand; empty gives it back to the
// book), the buttons that lock or free the numbers of all roles, and the casting: the persons
// proposed as chips, the fixed one marked, each with a button to fix it and a × to drop it; a search
// field with the persons the role does not have yet (the cast department first), where a new name
// makes a person and proposes it. Every change goes in at once, one undo step each.
//
//  ┌ Anna ───────────────────────────────────┐
//  │ CAST-NUMMER  1, automatisch  [ leer: automatisch ]│
//  │ [Nummern festschreiben]                        │
//  │ BESETZUNG  [Lena Weber · fest ×] [Jo Berg ×] [Festlegen] │
//  │ [ Person suchen oder neu anlegen ]             │
//  │ In 7 Szenen                                    │
//  └────────────────────────────────────────────────┘
ColumnLayout {
    id: form

    required property var texts
    required property var controller
    /// The role, `{ id, name, group }`.
    property var role: null
    /// Its casting, `{ character, fixed, proposed, number, stored, scenes }`.
    property var casting: null
    /// All persons of the project, `{ id, name, department }`.
    property var persons: []
    property bool locked: false

    readonly property string castDepartment: "cast"
    readonly property var proposed: casting ? casting.proposed : []
    /// The persons that are no proposal yet, those of the cast department first.
    readonly property var candidates: persons.filter(p => !proposed.includes(p.id) && p.department === castDepartment)
        .concat(persons.filter(p => !proposed.includes(p.id) && p.department !== castDepartment))

    spacing: Kirigami.Units.smallSpacing

    onCastingChanged: fill()
    onRoleChanged: fill()

    function personName(id) {
        const person = persons.find(p => p.id === id)
        return person ? person.name : ""
    }

    // The number the field shows: the stored one, empty for an automatic one.
    function storedText() {
        return casting && casting.stored ? String(casting.number) : ""
    }

    // The field shows it, except while it is typed in.
    function fill() {
        if (numberField.activeFocus) {
            return
        }
        numberField.text = storedText()
        numberField.invalid = false
    }

    // Sends the typed number; the session refuses a number another role has, and the field is marked.
    function commitNumber() {
        const typed = numberField.text.trim()
        if (!role || typed === storedText()) {
            return
        }
        numberField.invalid = !controller.setCastNumber(role.id, typed)
    }

    function numberInfo() {
        if (!casting || casting.number === null) {
            return texts.t("casting.numberNone")
        }
        return texts.t(casting.stored ? "casting.numberSet" : "casting.numberAuto", { number: casting.number })
    }

    // A person made here plays no other role yet: it is proposed for this one.
    function addNew(name) {
        const id = controller.addPerson(name, castDepartment)
        if (id !== "") {
            controller.proposeCast(role.id, id)
        }
    }

    KanteHeading {
        objectName: "roleTitle"
        Layout.fillWidth: true
        level: 4
        text: form.role ? (form.role.group > 0 ? form.role.name + " ×" + form.role.group : form.role.name) : ""
    }

    KanteSectionLabel { text: form.texts.t("casting.number"); rule: true; Layout.fillWidth: true }
    RowLayout {
        Layout.fillWidth: true
        spacing: Kirigami.Units.largeSpacing

        QQC2.Label {
            objectName: "numberInfo"
            text: form.numberInfo()
        }
        KanteTextField {
            id: numberField
            objectName: "numberField"
            Layout.preferredWidth: Kirigami.Units.gridUnit * 10
            placeholderText: form.texts.t("casting.numberHint")
            Accessible.name: form.texts.t("casting.number")
            onTextEdited: invalid = false
            onEditingFinished: form.commitNumber()
        }
    }
    RowLayout {
        spacing: Kirigami.Units.smallSpacing

        KanteButton {
            objectName: "lock"
            visible: !form.locked
            text: form.texts.t("casting.lock")
            onClicked: form.controller.lockNumbers()
        }
        KanteButton {
            objectName: "unlock"
            visible: form.locked
            text: form.texts.t("casting.unlock")
            onClicked: form.controller.unlockNumbers()
        }
    }
    QQC2.Label {
        objectName: "lockedHint"
        visible: form.locked
        Layout.fillWidth: true
        wrapMode: Text.Wrap
        color: KanteStyle.mutedTextColor
        text: form.texts.t("casting.locked")
    }

    KanteSectionLabel { text: form.texts.t("casting.cast"); rule: true; Layout.fillWidth: true }
    QQC2.Label {
        objectName: "nobody"
        visible: form.proposed.length === 0
        Layout.fillWidth: true
        wrapMode: Text.Wrap
        color: KanteStyle.mutedTextColor
        text: form.texts.t("casting.nobody")
    }
    Flow {
        Layout.fillWidth: true
        spacing: Kirigami.Units.largeSpacing

        Repeater {
            model: form.proposed
            delegate: RowLayout {
                id: proposal
                required property string modelData
                readonly property bool fixed: form.casting.fixed === modelData
                readonly property string name: form.personName(modelData)
                objectName: "proposal." + modelData
                spacing: Kirigami.Units.smallSpacing

                // The fixed one is marked in words, not by colour alone. The chip takes the focus:
                // Delete drops the proposal, Enter or Space fixes it (as the ✓ does).
                KanteChip {
                    objectName: "proposalChip"
                    text: proposal.fixed ? proposal.name + " · " + form.texts.t("casting.fixed") : proposal.name
                    checked: proposal.fixed
                    removable: true
                    removeText: form.texts.t("casting.drop", { name: "%1" })
                    onClicked: {
                        if (!proposal.fixed) {
                            form.controller.fixCast(form.role.id, proposal.modelData)
                        }
                    }
                    onRemoveRequested: form.controller.dropProposal(form.role.id, proposal.modelData)
                }
                KanteButton {
                    objectName: "fix"
                    visible: !proposal.fixed
                    size: KanteButton.Size.Small
                    text: form.texts.t("casting.fixAction")
                    Accessible.name: form.texts.t("casting.fix", { name: proposal.name })
                    onClicked: form.controller.fixCast(form.role.id, proposal.modelData)
                }
            }
        }
    }

    // Typing narrows the persons; Enter takes the one under the cursor, or makes the typed one.
    KanteSearchCombo {
        id: search
        objectName: "personSearch"
        Layout.preferredWidth: Kirigami.Units.gridUnit * 20
        keepText: false
        allowNew: true
        model: form.candidates.map(p => p.name)
        placeholderText: form.texts.t("casting.search")
        newText: form.texts.t("casting.new")
        onActivated: index => form.controller.proposeCast(form.role.id, form.candidates[index].id)
        onNewEntered: text => form.addNew(text)
    }

    QQC2.Label {
        objectName: "sceneCount"
        visible: form.casting !== null
        Layout.fillWidth: true
        color: KanteStyle.mutedTextColor
        text: {
            const count = form.casting ? form.casting.scenes : 0
            return form.texts.t(count === 0 ? "casting.scenes0" : (count === 1 ? "casting.scenes1" : "casting.scenes"), { count: count })
        }
    }
}
