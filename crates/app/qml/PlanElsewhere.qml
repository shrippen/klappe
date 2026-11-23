import QtQuick
import Kante

// D-21: the plan on view is drawn on another set than the scene is shot at now. The plan
// stays where it is; the notice says so and offers the two ways out. Nothing changes before a
// click.
KanteCallout {
    id: notice

    required property var texts
    required property var controller
    /// The plan's name.
    required property string plan
    /// Where the plan is drawn and where the scene is shot: { drawn_on, drawn_on_name, shot_at,
    /// shot_at_name }.
    required property var elsewhere

    objectName: "planElsewhere"
    kind: KanteCallout.Kind.Warn
    autoHide: false
    text: texts.t("plan.elsewhere", {
        plan: plan,
        drawn: elsewhere.drawn_on_name,
        shot: elsewhere.shot_at_name
    })

    KanteButton {
        objectName: "newPlanThere"
        text: notice.texts.t("plan.elsewhereNew", { shot: notice.elsewhere.shot_at_name })
        onClicked: notice.controller.addPlan("")
    }
    KanteButton {
        objectName: "shootThere"
        text: notice.texts.t("plan.elsewhereBack", { drawn: notice.elsewhere.drawn_on_name })
        onClicked: notice.controller.shootSceneAt(notice.elsewhere.drawn_on)
    }
}
