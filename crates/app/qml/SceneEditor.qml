import QtQuick
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import Kante

// Adds a scene or changes one (LH-01, D-08). A scene with a plan keeps its id; the number is
// the one the script knows it by, so a new number or heading is carried into the script text
// (the pane warns first).
EditorPane {
    id: form

    // [{ id, name }]: the project's locations.
    property var locations: []
    // Asks to show the scene on view.
    signal showRequested(string id)

    title: texts.t(item ? "scene.edit" : "scene.new")
    applyText: texts.t(item ? "form.apply" : "form.add")
    canApply: numberField.text.trim() !== "" && locationBox.currentIndex >= 0
    removeQuestion: item
        ? texts.t("remove.scene", { name: (item.number + " " + item.heading).trim() })
        : ""

    function load() {
        numberField.text = item ? item.number : ""
        headingField.text = item ? item.heading : ""
        locationBox.currentIndex = item
            ? locations.findIndex(l => l.id === item.location)
            : (locations.length > 0 ? 0 : -1)
    }

    onItemChanged: load()
    // The locations arrive after the form is made, or change under it.
    onLocationsChanged: load()

    preview: function () {
        if (!item) {
            return []
        }
        const raw = controller.sceneChanges(item.id, numberField.text, headingField.text)
        return raw ? JSON.parse(raw) : []
    }
    submit: function () {
        const location = locations[locationBox.currentIndex].id
        if (!item) {
            return controller.addScene(numberField.text, headingField.text, location)
        }
        return controller.editScene(item.id, numberField.text, headingField.text, location)
    }
    remove: function () { return controller.removeScene(item.id) }

    KanteSectionLabel { text: form.texts.t("scene.number") }
    KanteTextField {
        id: numberField
        objectName: "numberField"
        Layout.fillWidth: true
    }
    KanteSectionLabel { text: form.texts.t("scene.heading") }
    KanteTextField {
        id: headingField
        objectName: "headingField"
        Layout.fillWidth: true
    }
    KanteSectionLabel { text: form.texts.t("scene.location") }
    KanteComboBox {
        id: locationBox
        objectName: "locationBox"
        Layout.fillWidth: true
        model: form.locations.map(l => l.name)
    }
    // Another location drops the object ways of the scene's plans (LH-24); undo brings them back.
    KanteCallout {
        objectName: "waysDropped"
        Layout.fillWidth: true
        visible: form.item !== null && locationBox.currentIndex >= 0
            && form.locations[locationBox.currentIndex].id !== form.item.location
        kind: KanteCallout.Kind.Info
        text: form.texts.t("scene.waysDropped")
    }
    KanteCallout {
        objectName: "needLocation"
        Layout.fillWidth: true
        visible: form.locations.length === 0
        kind: KanteCallout.Kind.Warn
        text: form.texts.t("scene.needLocation")
    }

    extras: KanteButton {
        objectName: "show"
        visible: form.item !== null
        text: form.texts.t("project.show")
        onClicked: form.showRequested(form.item.id)
    }
}
