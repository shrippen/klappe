import QtQuick
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import Kante

// Adds a scene or changes one (LH-01). A scene with a plan keeps its id; the number is the
// one the script knows it by, so editing the heading alone leaves it as it is.
FormDialog {
    id: form

    // [{ id, name }]: the project's locations.
    property var locations: []
    // The scene being edited; null for a new one.
    property var scene: null

    title: texts.t(scene ? "scene.edit" : "scene.new")
    acceptText: texts.t(scene ? "form.apply" : "form.add")
    acceptEnabled: numberField.text.trim() !== "" && locationBox.currentIndex >= 0

    function show(scene) {
        form.scene = scene
        numberField.text = scene ? scene.number : ""
        headingField.text = scene ? scene.heading : ""
        locationBox.currentIndex = scene
            ? locations.findIndex(l => l.id === scene.location)
            : (locations.length > 0 ? 0 : -1)
        open()
    }

    submit: function () {
        const location = locations[locationBox.currentIndex].id
        if (!scene) {
            return controller.addScene(numberField.text, headingField.text, location)
        }
        if (!controller.editScene(scene.id, numberField.text, headingField.text)) {
            return false
        }
        return scene.location === location || controller.moveScene(scene.id, location)
    }

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
    KanteCallout {
        objectName: "needLocation"
        Layout.fillWidth: true
        visible: form.locations.length === 0
        kind: KanteCallout.Kind.Warn
        text: form.texts.t("scene.needLocation")
    }
}
