import QtQuick
import QtQuick.Layouts
import Kante
import "ProjectData.js" as ProjectData

// Saves what the plan shows (layers, cameras, figures) as a named view, or deletes a saved
// one (LH-119). A name that exists replaces that view. Choosing a view is a menu or palette
// entry, not this form.
FormDialog {
    id: form

    // "save" or "remove".
    property string mode: "save"
    // [{ id, name }] as the project view lists them.
    readonly property var views: ProjectData.of(controller.projectJson).views || []
    readonly property bool removing: mode === "remove"

    title: texts.t(removing ? "menu.viewRemove" : "view.saveTitle")
    acceptText: texts.t(removing ? "remove.confirm" : "menu.save")
    acceptEnabled: removing ? box.currentIndex >= 0 : nameField.text.trim() !== ""

    function show(mode) {
        form.mode = mode
        nameField.text = ""
        box.currentIndex = views.length > 0 ? 0 : -1
        open()
    }

    submit: function () {
        return removing ? controller.removeView(views[box.currentIndex].id) : controller.saveView(nameField.text)
    }

    KanteSectionLabel { text: form.texts.t("view.saveName"); visible: !form.removing }
    KanteTextField {
        id: nameField
        objectName: "viewName"
        visible: !form.removing
        Layout.fillWidth: true
        Accessible.name: form.texts.t("view.saveName")
    }
    KanteCallout {
        objectName: "viewHint"
        Layout.fillWidth: true
        visible: !form.removing
        kind: KanteCallout.Kind.Info
        text: form.texts.t("view.saveHint")
    }
    KanteComboBox {
        id: box
        objectName: "viewBox"
        visible: form.removing
        Layout.fillWidth: true
        Accessible.name: form.texts.t("menu.viewRemove")
        textRole: "text"
        model: form.views.map(v => ({ text: v.name }))
    }
}
