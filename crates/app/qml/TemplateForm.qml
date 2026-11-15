import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// Puts a template on the plan on view, or files the plan as one (LH-07). Placing picks from the
// list (built in first, then the project's own, in groups) and says what the template holds.
FormDialog {
    id: form

    // "apply" or "save".
    property string mode: "apply"
    // [{ id, label, figures, cameras, custom }] as the controller lists them in `choicesJson`.
    readonly property var templates: controller.choicesJson ? (JSON.parse(controller.choicesJson).template || []) : []
    readonly property bool saving: mode === "save"
    readonly property var entries: templates.map(t => ({
        value: t.id,
        text: t.custom ? t.label : texts.t("template." + t.id),
        group: texts.t(t.custom ? "template.custom" : "template.builtin")
    }))
    readonly property var picked: box.currentIndex >= 0 ? templates[box.currentIndex] : null

    title: texts.t(saving ? "plan.templateSaveTitle" : "plan.templateTitle")
    acceptText: texts.t(saving ? "library.save" : "template.insert")
    acceptEnabled: saving ? nameField.text.trim() !== "" : picked !== null

    function show(mode) {
        form.mode = mode
        nameField.text = ""
        box.currentIndex = 0
        open()
    }

    submit: function () {
        return saving ? controller.saveTemplate(nameField.text) : controller.applyTemplate(picked.id)
    }

    KanteSectionLabel { text: form.texts.t("template.pick"); visible: !form.saving }
    KanteSearchCombo {
        id: box
        objectName: "templateBox"
        visible: !form.saving
        Layout.fillWidth: true
        Accessible.name: form.texts.t("template.pick")
        placeholderText: form.texts.t("template.search")
        textRole: "text"
        sectionRole: "group"
        model: form.entries
    }
    QQC2.Label {
        objectName: "summary"
        color: KanteStyle.textColor
        wrapMode: Text.WordWrap
        visible: !form.saving && form.picked !== null
        Layout.fillWidth: true
        text: form.picked ? form.texts.t("template.summary", { figures: form.picked.figures, cameras: form.picked.cameras }) : ""
    }

    KanteSectionLabel { text: form.texts.t("template.name"); visible: form.saving }
    KanteTextField {
        id: nameField
        objectName: "nameField"
        visible: form.saving
        Layout.fillWidth: true
    }

    KanteCallout {
        objectName: "hint"
        Layout.fillWidth: true
        kind: KanteCallout.Kind.Info
        text: form.texts.t(form.saving ? "template.saveHint" : "template.hint")
    }
}
