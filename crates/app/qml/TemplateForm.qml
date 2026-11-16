import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// Puts a template on the plan on view, or files the plan as one (LH-07). Placing picks from the
// list (built in first, then the project's own, in groups) and says what the template holds;
// an own template can be renamed or deleted there (the built-in ones cannot).
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
    readonly property bool own: !saving && picked !== null && picked.custom
    // The delete button has asked and waits for the second click.
    property bool asking: false
    readonly property int askMs: 5000

    title: texts.t(saving ? "plan.templateSaveTitle" : "plan.templateTitle")
    acceptText: texts.t(saving ? "library.save" : "template.insert")
    acceptEnabled: saving ? nameField.text.trim() !== "" : picked !== null

    function show(mode) {
        form.mode = mode
        nameField.text = ""
        box.currentIndex = 0
        fillOwn()
        open()
    }

    // The rename field starts with the name of the picked own template.
    function fillOwn() {
        asking = false
        ownName.text = picked !== null && picked.custom ? picked.label : ""
    }

    // The list is new after a change: back to the template `id`, or the first.
    function reselect(id) {
        const at = templates.findIndex(t => t.id === id)
        box.currentIndex = at >= 0 ? at : 0
        fillOwn()
    }

    function rename() {
        const id = picked.id
        if (controller.renameTemplate(id, ownName.text)) {
            reselect(id)
        }
    }

    function remove() {
        if (!asking) {
            asking = true
            askTimer.restart()
            return
        }
        asking = false
        if (controller.removeTemplate(picked.id)) {
            reselect("")
        }
    }

    onPickedChanged: fillOwn()

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

    // Rename or delete the own template the list has picked; the first click on delete asks.
    KanteSectionLabel { text: form.texts.t("template.name"); visible: form.own }
    RowLayout {
        visible: form.own
        Layout.fillWidth: true
        spacing: Kirigami.Units.smallSpacing

        KanteTextField {
            id: ownName
            objectName: "ownName"
            Layout.fillWidth: true
            Accessible.name: form.texts.t("template.name")
            onAccepted: form.rename()
        }
        KanteButton {
            objectName: "templateRename"
            size: KanteButton.Size.Small
            text: form.texts.t("template.rename")
            enabled: ownName.text.trim() !== ""
            onClicked: form.rename()
            QQC2.ToolTip.text: form.texts.t("template.renameTip")
            QQC2.ToolTip.visible: hovered
            QQC2.ToolTip.delay: Kirigami.Units.toolTipDelay
        }
        KanteButton {
            objectName: "templateDelete"
            size: KanteButton.Size.Small
            destructive: true
            text: form.asking ? form.texts.t("template.deleteSure") : form.texts.t("template.delete")
            onClicked: form.remove()
            QQC2.ToolTip.text: form.texts.t("template.deleteTip")
            QQC2.ToolTip.visible: hovered
            QQC2.ToolTip.delay: Kirigami.Units.toolTipDelay
        }
    }
    Timer {
        id: askTimer
        interval: form.askMs
        onTriggered: form.asking = false
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
