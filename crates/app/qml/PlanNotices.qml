import QtQuick
import org.kde.kirigami as Kirigami
import Kante

// What stands over the plan when the plan is not all there is (D-17, D-21):
//  - the scene is shot nowhere yet: an empty state with a choice of location,
//  - it is shot, but has no plan: the empty state with a button for one,
//  - the plan on view is drawn on another set than the scene is shot at: a notice above it.
Item {
    id: notices

    required property var texts
    required property var controller
    /// The overview (`projectJson`), parsed; null without a project.
    required property var project
    /// The scene without a plan: { number } or null.
    required property var planless
    /// The project dialog is to make a location (see PlanNotShot).
    signal newLocationRequested(string name, string place)

    // What the layers button at the top right takes.
    readonly property real layersRoom: Kirigami.Units.gridUnit * 8
    readonly property var notShot: project ? project.not_shot || null : null
    readonly property var elsewhere: project ? project.elsewhere || null : null
    readonly property var planName: {
        const plan = project ? project.plans.find(p => p.id === project.plan) : null
        return plan ? plan.name : ""
    }

    Loader {
        active: notices.notShot !== null
        anchors.centerIn: parent
        width: Math.min(notices.width, Kirigami.Units.gridUnit * 24)
        sourceComponent: PlanNotShot {
            texts: notices.texts
            controller: notices.controller
            locations: notices.project.locations
            place: notices.project.places.find(p => p.id === notices.notShot.place) || null
            notShot: notices.notShot
            onNewLocationRequested: (name, place) => notices.newLocationRequested(name, place)
        }
    }
    Loader {
        active: notices.notShot === null && notices.planless !== null
        anchors.centerIn: parent
        width: Math.min(notices.width, Kirigami.Units.gridUnit * 24)
        sourceComponent: PlanNone {
            texts: notices.texts
            controller: notices.controller
            scene: notices.planless
        }
    }
    Loader {
        active: notices.elsewhere !== null
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.margins: Kirigami.Units.largeSpacing
        // Not over the layers button at the right edge.
        width: Math.min(notices.width - 2 * Kirigami.Units.largeSpacing - notices.layersRoom,
            Kirigami.Units.gridUnit * 34)
        sourceComponent: PlanElsewhere {
            texts: notices.texts
            controller: notices.controller
            plan: notices.planName
            elsewhere: notices.elsewhere
        }
    }
}
