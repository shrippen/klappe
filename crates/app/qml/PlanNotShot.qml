import QtQuick
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import Kante

// D-21: the scene on view is shot nowhere yet, so it has no plan. Says so and offers the two
// ways: choose a location, or make one. The choice holds for the scripted location of the
// scene, so for all its scenes; a scene without one takes it for itself. Nothing is changed
// before a click.
KanteEmptyState {
    id: none

    required property var texts
    required property var controller
    /// [{ id, name }]: the project's locations.
    required property var locations
    /// The scripted location, or null: { id, name, variants }.
    required property var place
    /// { place, place_name }, of the project's `not_shot`.
    required property var notShot

    /// Asks to make a new location named `name`; `place` is the scripted location's id, or ""
    /// for the scene on view. The project dialog does it.
    signal newLocationRequested(string name, string place)

    objectName: "notShot"
    // The loader in PlanArea sets the width; without it the notice would be 0 px wide.
    width: parent ? parent.width : implicitWidth
    title: notShot.place_name !== ""
        ? texts.t("plan.notShot", { place: notShot.place_name })
        : texts.t("plan.notShotScene")
    text: texts.t(notShot.place_name !== "" ? "plan.notShotText" : "plan.notShotSceneText")

    // The location `id` for the scripted location, or for the scene when it has none.
    function choose(id) {
        if (place) {
            controller.editPlace(place.id, place.name, JSON.stringify(place.variants), id)
            return
        }
        controller.shootSceneAt(id)
    }

    Row {
        spacing: Kirigami.Units.smallSpacing

        KanteButton {
            id: pick
            objectName: "pickLocation"
            visible: none.locations.length > 0
            text: none.texts.t("plan.pickLocation")
            onClicked: popover.openAt(pick)
        }
        KanteButton {
            objectName: "newLocation"
            text: none.texts.t("plan.newLocation")
            onClicked: none.newLocationRequested(none.controller.nameFromCapitals(none.notShot.place_name),
                none.place ? none.place.id : "")
        }
    }

    KantePopover {
        id: popover
        objectName: "locationPopover"

        ColumnLayout {
            spacing: Kirigami.Units.smallSpacing

            Repeater {
                model: none.locations
                delegate: KanteButton {
                    required property var modelData
                    objectName: "location." + modelData.id
                    Layout.fillWidth: true
                    emphasis: KanteButton.Emphasis.Quiet
                    text: modelData.name
                    onClicked: {
                        popover.close()
                        none.choose(modelData.id)
                    }
                }
            }
        }
    }
}
