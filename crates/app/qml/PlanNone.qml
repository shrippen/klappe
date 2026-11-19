import QtQuick
import org.kde.kirigami as Kirigami
import Kante

// D-17: the scene on view has no plan. Says so and offers one; nothing is made before the
// click, and the new plan seats the scene's speakers (LH-09).
KanteEmptyState {
    id: none

    required property var texts
    required property var controller
    /// The scene without a plan: { number }.
    required property var scene

    objectName: "noPlan"
    width: Math.min(parent.width, Kirigami.Units.gridUnit * 24)
    title: texts.t("plan.none", { number: scene.number })
    text: texts.t("plan.noneText")

    KanteButton {
        objectName: "createPlan"
        text: none.texts.t("plan.create")
        onClicked: none.controller.addPlan("")
    }
}
