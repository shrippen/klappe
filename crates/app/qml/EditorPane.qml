import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// The form half of the project dialog (D-08): fields, then Apply and Delete. No dialog opens
// over it; a question stands in the pane itself.
//  - A change that would also change the script text shows first what changes there
//    (`preview`) and waits for a yes.
//  - Deleting asks first (`removeQuestion`).
//  - A refusal of the controller shows as its message above the fields, and nothing typed
//    is lost.
ColumnLayout {
    id: pane

    required property var texts
    required property var controller

    // The entry being changed; null for a new one.
    property var item: null
    property string title: ""
    property string applyText: ""
    property bool canApply: true
    // What deleting asks; "" for an entry that cannot be deleted (a new one).
    property string removeQuestion: ""
    // Called to apply; true when the change was made.
    property var submit: function () { return true }
    // Called to delete; true when it was done.
    property var remove: function () { return true }
    // The changes `submit` would make to the script text: [{ kind, from, to, count }].
    property var preview: function () { return [] }
    // Extra buttons beside Apply and Delete.
    property alias extras: extraRow.data
    // The fields; the pane's own parts stand around them.
    default property alias fields: fieldsColumn.data

    property var pendingChanges: []
    property bool asking: false
    readonly property bool busy: pendingChanges.length > 0 || asking

    // The applied change or the delete is done: the dialog shows another entry or none.
    signal done()

    function reset() {
        pendingChanges = []
        asking = false
        controller.clearStatus()
    }

    function apply() {
        controller.clearStatus()
        const changes = preview()
        if (changes.length > 0) {
            pendingChanges = changes
            return
        }
        commit()
    }

    function commit() {
        pendingChanges = []
        if (submit()) {
            done()
        }
    }

    function ask() {
        controller.clearStatus()
        asking = true
    }

    // "3 Stellen: ANNA → ANNE", "Szenenkopf: … → …".
    function changeText(change) {
        const args = { count: change.count, from: change.from, to: change.to }
        if (change.kind === "cue") {
            return texts.t(change.count === 1 ? "change.cue1" : "change.cue", args)
        }
        return texts.t(change.kind === "number" ? "change.number" : "change.heading", args)
    }

    onItemChanged: reset()
    spacing: Kirigami.Units.smallSpacing

    KanteHeading {
        Layout.fillWidth: true
        level: 4
        text: pane.title
    }

    KanteCallout {
        objectName: "error"
        Layout.fillWidth: true
        visible: pane.controller.status !== ""
        kind: KanteCallout.Kind.Danger
        text: pane.controller.status
    }

    ColumnLayout {
        id: fieldsColumn
        Layout.fillWidth: true
        // The warning or the question describes the fields as they are.
        enabled: !pane.busy
        spacing: Kirigami.Units.smallSpacing
    }

    // What the change does to the script text, before it is made.
    KanteCallout {
        objectName: "warning"
        Layout.fillWidth: true
        visible: pane.pendingChanges.length > 0
        autoHide: false
        kind: KanteCallout.Kind.Warn
        title: pane.texts.t("change.title")
        text: pane.pendingChanges.map(c => pane.changeText(c)).concat([pane.texts.t("change.undo")]).join("\n")

        KanteButton {
            objectName: "confirmChange"
            emphasis: KanteButton.Emphasis.Primary
            text: pane.texts.t("change.confirm")
            onClicked: pane.commit()
        }
        KanteButton {
            objectName: "cancelChange"
            text: pane.texts.t("form.cancel")
            onClicked: pane.pendingChanges = []
        }
    }

    KanteCallout {
        objectName: "removeAsk"
        Layout.fillWidth: true
        visible: pane.asking
        autoHide: false
        kind: KanteCallout.Kind.Danger
        text: pane.removeQuestion

        KanteButton {
            objectName: "confirmRemove"
            emphasis: KanteButton.Emphasis.Destructive
            text: pane.texts.t("remove.confirm")
            onClicked: {
                pane.asking = false
                if (pane.remove()) {
                    pane.done()
                }
            }
        }
        KanteButton {
            objectName: "cancelRemove"
            text: pane.texts.t("form.cancel")
            onClicked: pane.asking = false
        }
    }

    RowLayout {
        Layout.fillWidth: true
        spacing: Kirigami.Units.smallSpacing

        KanteButton {
            objectName: "apply"
            emphasis: KanteButton.Emphasis.Primary
            text: pane.applyText
            enabled: pane.canApply && !pane.busy
            onClicked: pane.apply()
        }
        KanteButton {
            objectName: "remove"
            visible: pane.removeQuestion !== ""
            text: pane.texts.t("project.remove")
            enabled: !pane.busy
            onClicked: pane.ask()
        }
        Item { Layout.fillWidth: true }
        RowLayout {
            id: extraRow
            spacing: Kirigami.Units.smallSpacing
        }
    }
}
