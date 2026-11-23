import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// Adds a scene or changes one (LH-01, D-08, D-21). The scene is shot where its scripted location
// is (shown, changed in the list of scripted locations); its own location is optional and holds
// for this scene only. A scene with a plan keeps its id; the number is
// the one the script knows it by, so a new number or heading is carried into the script text
// (the pane warns first).
EditorPane {
    id: form

    // [{ id, name }]: the project's locations.
    property var locations: []
    // [{ name, location, location_name }]: the scripted locations.
    property var places: []
    // The scripted location of the scene, as its row names it: { name, location, location_name }
    // or null.
    readonly property var place: item ? places.find(p => p.name === item.place) || null : null
    // Asks to show the scene on view.
    signal showRequested(string id)

    title: texts.t(item ? "scene.edit" : "scene.new")
    applyText: texts.t(item ? "form.apply" : "form.add")
    canApply: numberField.text.trim() !== ""
    removeQuestion: item
        ? texts.t("remove.scene", { name: (item.number + " " + item.heading).trim() })
        : ""

    function load() {
        numberField.text = item ? item.number : ""
        headingField.text = item ? item.heading : ""
        sunDateField.text = item && item.sunDate ? item.sunDate : ""
        sunTimeField.text = item && item.sunTime ? item.sunTime : ""
        sunOffsetField.text = item && item.sunOffset ? item.sunOffset : ""
        pick()
    }

    // The box shows the scene's own location. Entry 0 is "as its scripted location".
    function pick() {
        locationBox.currentIndex = item && item.ownLocation
            ? 1 + locations.findIndex(l => l.id === item.location)
            : 0
    }

    // The id of the scene's own location; "" for the scripted location's.
    function chosen() {
        return locationBox.currentIndex > 0 ? locations[locationBox.currentIndex - 1].id : ""
    }

    // The entry that follows the scripted location, with the location that gives.
    function sameAs() {
        const name = place ? place.location_name : ""
        return name === ""
            ? texts.t("scene.sameAsNone")
            : texts.t("scene.sameAs", { location: name })
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
        const location = chosen()
        if (!item) {
            return controller.addScene(numberField.text, headingField.text, location)
        }
        return controller.editScene(item.id, numberField.text, headingField.text, location,
                                    JSON.stringify([sunDateField.text, sunTimeField.text, sunOffsetField.text]))
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
    KanteSectionLabel {
        visible: form.item !== null
        text: form.texts.t("scene.place")
    }
    QQC2.Label {
        objectName: "placeName"
        Layout.fillWidth: true
        visible: form.item !== null
        elide: Text.ElideRight
        color: form.item && form.item.place !== "" ? KanteStyle.textColor : KanteStyle.mutedTextColor
        text: form.item && form.item.place !== "" ? form.item.place : "—"
    }
    KanteSectionLabel { text: form.texts.t("scene.location") }
    KanteComboBox {
        id: locationBox
        objectName: "locationBox"
        Layout.fillWidth: true
        Accessible.name: form.texts.t("scene.location")
        model: [form.sameAs()].concat(form.locations.map(l => l.name))
        // A new model (the scripted location's name changed) resets the choice.
        onModelChanged: form.pick()
    }
    // The sun of the scene (LH-52): the plan shows it once the location has a place.
    RowLayout {
        Layout.fillWidth: true
        visible: form.item !== null
        spacing: Kirigami.Units.largeSpacing

        ColumnLayout {
            Layout.fillWidth: true
            KanteSectionLabel { text: form.texts.t("scene.sunDate") }
            KanteTextField {
                id: sunDateField
                objectName: "sunDateField"
                Layout.fillWidth: true
            }
        }
        ColumnLayout {
            Layout.fillWidth: true
            KanteSectionLabel { text: form.texts.t("scene.sunTime") }
            KanteTextField {
                id: sunTimeField
                objectName: "sunTimeField"
                Layout.fillWidth: true
            }
        }
    }
    KanteSectionLabel {
        text: form.texts.t("scene.sunOffset")
        visible: form.item !== null
    }
    KanteTextField {
        id: sunOffsetField
        objectName: "sunOffsetField"
        Layout.fillWidth: true
        visible: form.item !== null
    }
    KanteCallout {
        Layout.fillWidth: true
        visible: form.item !== null
        kind: KanteCallout.Kind.Info
        text: form.texts.t("scene.sunHint")
    }
    extras: KanteButton {
        objectName: "show"
        visible: form.item !== null
        text: form.texts.t("project.show")
        onClicked: form.showRequested(form.item.id)
    }
}
