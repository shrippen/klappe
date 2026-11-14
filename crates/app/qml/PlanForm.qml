import QtQuick
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import Kante

// A new plan, a snapshot of the plan on view, or a new name for a plan (LH-04). A new plan
// and a snapshot may leave the name empty: the session then names it (`Plan B`, `Plan A (2)`).
FormDialog {
    id: form

    // "new", "snapshot" or "rename".
    property string mode: "new"
    // The plan being renamed: { id, name }.
    property var plan: null

    readonly property bool renaming: mode === "rename"

    title: texts.t("plan." + mode + "Title")
    acceptText: texts.t(renaming ? "form.apply" : "form.add")
    acceptEnabled: !renaming || nameField.text.trim() !== ""

    function show(mode, plan) {
        form.mode = mode
        form.plan = plan
        nameField.text = renaming ? plan.name : ""
        open()
    }

    submit: function () {
        switch (mode) {
        case "snapshot": return controller.copyPlan(nameField.text)
        case "rename": return controller.renamePlan(plan.id, nameField.text)
        default: return controller.addPlan(nameField.text)
        }
    }

    KanteSectionLabel { text: form.texts.t("plan.name") }
    KanteTextField {
        id: nameField
        objectName: "nameField"
        Layout.fillWidth: true
    }
    KanteCallout {
        objectName: "hint"
        Layout.fillWidth: true
        visible: !form.renaming
        kind: KanteCallout.Kind.Info
        text: form.texts.t("plan.nameAuto") + (form.mode === "snapshot" ? " " + form.texts.t("plan.snapshotHint") : "")
    }
}
