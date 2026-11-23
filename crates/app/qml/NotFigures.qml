import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// The names of the book that are no character (DB-50), as ALLE or STIMME, under the list of
// characters: chips with a cross, and a field that adds one. Taking away a name the book uses
// says first that the book asks about it again. A refusal (a name a character has) shows here.
ColumnLayout {
    id: section

    required property var texts
    required property var controller
    // The marked names, from the project view.
    property var names: []

    // The name whose removal waits for a yes, and what it leaves unknown.
    property string pending: ""
    property var pendingChanges: []
    property string refused: ""

    function add() {
        if (field.text.trim() === "") {
            return
        }
        if (controller.addNotFigure(field.text)) {
            field.text = ""
            refused = ""
            return
        }
        // The pane beside shows the status too; the reason belongs here.
        refused = controller.status
        controller.clearStatus()
    }

    function ask(name) {
        refused = ""
        const changes = JSON.parse(controller.notFigureChanges(name) || "[]")
        if (changes.length === 0) {
            controller.removeNotFigure(name)
            return
        }
        pending = name
        pendingChanges = changes
    }

    function confirm() {
        controller.removeNotFigure(pending)
        pending = ""
        pendingChanges = []
    }

    spacing: Kirigami.Units.smallSpacing

    KanteSectionLabel { text: section.texts.t("notFigure.title") }
    QQC2.Label {
        Layout.fillWidth: true
        wrapMode: Text.Wrap
        color: KanteStyle.mutedTextColor
        text: section.texts.t("notFigure.hint")
    }
    Flow {
        objectName: "notFigureChips"
        Layout.fillWidth: true
        visible: section.names.length > 0
        spacing: Kirigami.Units.smallSpacing

        Repeater {
            model: section.names
            delegate: KanteChip {
                required property string modelData
                objectName: "notFigure"
                text: modelData
                removable: true
                removeText: section.texts.t("character.aliasRemove", { name: "%1" })
                onRemoveRequested: section.ask(modelData)
            }
        }
    }
    KanteCallout {
        objectName: "notFigureAsk"
        Layout.fillWidth: true
        visible: section.pending !== ""
        autoHide: false
        kind: KanteCallout.Kind.Warn
        title: section.texts.t("change.askTitle")
        text: section.pendingChanges.map(c => section.texts.t(
            c.count === 1 ? "change.unknown1" : "change.unknown",
            { count: c.count, from: c.from })).join("\n")

        KanteButton {
            objectName: "confirmNotFigure"
            emphasis: KanteButton.Emphasis.Primary
            text: section.texts.t("notFigure.confirm")
            onClicked: section.confirm()
        }
        KanteButton {
            objectName: "cancelNotFigure"
            text: section.texts.t("form.cancel")
            onClicked: section.pending = ""
        }
    }
    KanteCallout {
        objectName: "notFigureError"
        Layout.fillWidth: true
        visible: section.refused !== ""
        kind: KanteCallout.Kind.Danger
        text: section.refused
    }
    KanteTextField {
        id: field
        objectName: "notFigureField"
        Layout.fillWidth: true
        placeholderText: section.texts.t("notFigure.add")
        Accessible.name: section.texts.t("notFigure.title")
        onAccepted: section.add()
        onTextEdited: section.refused = ""
    }
}
