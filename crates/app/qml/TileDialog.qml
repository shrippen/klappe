import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import Kante

// What the start page's tile menu asks before it changes a project's folder: the new name, or
// the question before the project is archived or put in the trash. A refusal stays in the
// dialog (FormDialog).
FormDialog {
    id: form

    // "rename", "archive" or "trash".
    property string mode: "rename"
    // The tile: { path, name }.
    property var project: null

    readonly property bool renaming: mode === "rename"

    title: texts.t("start." + mode + "Title")
    acceptText: texts.t("start." + mode + "Do")
    acceptEnabled: !renaming || nameField.text.trim() !== ""
    destructive: mode === "trash"

    function show(mode, project) {
        form.mode = mode
        form.project = project
        nameField.text = renaming ? project.name : ""
        open()
    }

    submit: function () {
        switch (mode) {
        case "archive": return controller.archiveRecent(project.path)
        case "trash": return controller.trashRecent(project.path)
        default: return controller.renameRecent(project.path, nameField.text)
        }
    }

    KanteSectionLabel {
        visible: form.renaming
        text: form.texts.t("start.name")
    }
    KanteTextField {
        id: nameField
        objectName: "nameField"
        Layout.fillWidth: true
        visible: form.renaming
    }
    QQC2.Label {
        objectName: "question"
        Layout.fillWidth: true
        visible: !form.renaming
        wrapMode: Text.WordWrap
        color: KanteStyle.textColor
        text: form.project ? form.texts.t("start." + form.mode + "Ask", { name: form.project.name }) : ""
    }
}
