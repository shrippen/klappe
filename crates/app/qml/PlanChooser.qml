import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// The variants of the scene's plan (LH-04): a chooser over the plan, and what makes, copies,
// renames and deletes one. A snapshot is a copy; going back to it is choosing it again.
// Switching drops the drafts (the controller) like a change of scene.
//
//  [ Plan A (2)            ▾ ] [Neuer Plan…] [Schnappschuss…] [Umbenennen…] [Löschen…]
RowLayout {
    id: chooser

    required property var texts
    required property var controller

    readonly property var project: controller.projectJson ? JSON.parse(controller.projectJson) : { plans: [], plan: "" }
    readonly property var plans: project.plans
    readonly property int index: plans.findIndex(p => p.id === project.plan)
    readonly property var current: index < 0 ? null : plans[index]

    spacing: Kirigami.Units.smallSpacing

    KanteComboBox {
        id: box
        objectName: "planBox"
        Layout.fillWidth: true
        Layout.minimumWidth: Kirigami.Units.gridUnit * 6
        Accessible.name: chooser.texts.t("plan.choose")
        enabled: chooser.plans.length > 0
        model: chooser.plans.map(p => p.name)
        onActivated: index => {
            chooser.controller.selectPlan(chooser.plans[index].id)
            // A refusal leaves the plan on view; the box shows it again.
            currentIndex = chooser.index
        }

        // The plan on view also changes by undo, redo and a new scene, and a new model (a
        // rename) resets the box to its first row: it shows the plan on view again.
        function follow() { currentIndex = chooser.index }
        onModelChanged: follow()
        Connections {
            target: chooser
            function onIndexChanged() { box.follow() }
        }
    }
    KanteButton {
        objectName: "newPlan"
        text: chooser.texts.t("plan.new")
        enabled: chooser.current !== null
        onClicked: form.show("new", null)
    }
    KanteButton {
        objectName: "snapshotPlan"
        text: chooser.texts.t("plan.snapshot")
        enabled: chooser.current !== null
        onClicked: form.show("snapshot", chooser.current)
    }
    KanteButton {
        objectName: "renamePlan"
        text: chooser.texts.t("plan.rename")
        enabled: chooser.current !== null
        onClicked: form.show("rename", chooser.current)
    }
    KanteButton {
        objectName: "removePlan"
        text: chooser.texts.t("plan.remove")
        enabled: chooser.plans.length > 1
        onClicked: {
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
