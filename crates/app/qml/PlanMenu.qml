import QtQuick
import QtQuick.Controls as QQC2
import QtQml
import org.kde.kirigami as Kirigami
import Kante

// The variants of the scene's plan (LH-04, D-03): the plan's name in the header's path is a
// menu. It lists the variants (a tick on the one on view), then makes, copies, renames and
// deletes one. A snapshot is a copy; going back to it is choosing it again. Switching drops
// the drafts (the controller) like a change of scene.
//
//  Wartburg › Szene 12 › [ Plan A (2) ▾ ]
//                          ✓ Plan A
//                            Plan A (2)
//                          ──────────────
//                            Schnappschuss…  Neuer Plan…  Umbenennen…
//                          ──────────────
//                            Löschen…
HeaderMenu {
    id: chooser

    required property var texts
    required property var controller

    readonly property var project: controller.projectJson ? JSON.parse(controller.projectJson) : { plans: [], plan: "" }
    readonly property var plans: project.plans
    readonly property int index: plans.findIndex(p => p.id === project.plan)
    readonly property var current: index < 0 ? null : plans[index]
    /// The actions, for the command palette too.
    readonly property var actions: [snapshotAction, newAction, renameAction, removeAction]

    text: current ? current.name : ""
    enabled: plans.length > 0
    Accessible.name: texts.t("plan.choose")

    // The plan names, in the order of the scene; a tick on the one on view.
    Instantiator {
        model: chooser.plans
        delegate: KanteMenuItem {
            required property var modelData
            objectName: "plan." + modelData.id
            text: modelData.name
            checkable: true
            readonly property bool onView: modelData.id === chooser.project.plan
            checked: onView
            // A click toggles the tick. The tick is the plan on view: a refused choice or the
            // plan already on view leaves it, so the binding is put back after each click.
            onTriggered: {
                chooser.controller.selectPlan(modelData.id)
                checked = Qt.binding(() => onView)
            }
        }
        onObjectAdded: (index, object) => chooser.menu.insertItem(index, object)
        onObjectRemoved: (index, object) => chooser.menu.removeItem(object)
    }

    QQC2.MenuSeparator {}
    KanteMenuItem { objectName: "snapshotPlan"; action: snapshotAction; iconRoom: true }
    KanteMenuItem { objectName: "newPlan"; action: newAction; iconRoom: true }
    KanteMenuItem { objectName: "renamePlan"; action: renameAction; iconRoom: true }
    QQC2.MenuSeparator {}
    KanteMenuItem { objectName: "removePlan"; action: removeAction; iconRoom: true; destructive: true }

    CommandAction {
        id: snapshotAction
        cmd: "plan.snapshot"
        group: "plans"
        text: chooser.texts.t("plan.snapshot")
        enabled: chooser.current !== null
        onTriggered: form.show("snapshot", chooser.current)
    }
    CommandAction {
        id: newAction
        cmd: "plan.new"
        group: "plans"
        text: chooser.texts.t("plan.new")
        enabled: chooser.current !== null
        onTriggered: form.show("new", null)
    }
    CommandAction {
        id: renameAction
        cmd: "plan.rename"
        group: "plans"
        text: chooser.texts.t("plan.rename")
        enabled: chooser.current !== null
        onTriggered: form.show("rename", chooser.current)
    }
    CommandAction {
        id: removeAction
        cmd: "plan.remove"
        group: "plans"
        text: chooser.texts.t("plan.remove")
        enabled: chooser.plans.length > 1
        onTriggered: {
            chooser.controller.clearStatus()
            confirm.open()
        }
    }

    PlanForm {
        id: form
        objectName: "planForm"
        texts: chooser.texts
        controller: chooser.controller
    }

    KanteDialog {
        id: confirm
        objectName: "confirm"

        parent: QQC2.Overlay.overlay
        anchors.centerIn: parent
        modal: true
        size: KanteDialog.Size.Small
        destructive: true
        standardButtons: QQC2.Dialog.Ok | QQC2.Dialog.Cancel
        acceptText: chooser.texts.t("remove.confirm")
        rejectText: chooser.texts.t("form.cancel")
        title: chooser.texts.t("plan.removeTitle")

        QQC2.Label {
            objectName: "question"
            width: parent ? parent.width : implicitWidth
            wrapMode: Text.WordWrap
            color: KanteStyle.textColor
            text: confirm.question()
        }

        // With shots, the plan goes with them: the question names how many.
        function question() {
            const plan = chooser.current
            if (!plan) {
                return ""
            }
            if (plan.shots === 0) {
                return chooser.texts.t("remove.plan", { name: plan.name })
            }
            const key = plan.shots === 1 ? "remove.planShot" : "remove.planShots"
            return chooser.texts.t(key, { name: plan.name, count: plan.shots })
        }

        onAccepted: {
            if (chooser.current.shots === 0) {
                chooser.controller.removePlan(chooser.current.id)
                return
            }
            chooser.controller.removePlanWithShots(chooser.current.id)
        }
    }
}
