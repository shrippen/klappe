import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// Changes a scripted location (D-21): its name, the other spellings a scene heading uses and
// the location it is shot at. There is no form for a new one: they come from the book's
// questions. Spellings show as chips, as the other names of a character do (DB-50); a spelling
// typed and not yet entered counts on Apply. The choice of the location holds for every scene
// at this scripted location.
EditorPane {
    id: form

    // [{ id, name }]: the project's locations.
    property var locations: []

    // The other spellings as the form holds them until Apply.
    property var variants: []

    title: texts.t("place.edit")
    applyText: texts.t("form.apply")
    canApply: item !== null && nameField.text.trim() !== ""
    removeQuestion: item ? texts.t("remove.place", { name: item.name.trim() }) : ""

    // Whether `name` is in the list already, as the session compares place names.
    function hasVariant(name) {
        return variants.some(v => controller.samePlaceName(v, name))
    }

    function addVariant() {
        const name = variantField.text.trim()
        if (name !== "" && !hasVariant(name)) {
            variants = variants.concat([name])
        }
        variantField.text = ""
    }

    // The chips and a spelling typed but not entered.
    function allVariants() {
        const typed = variantField.text.trim()
        return typed === "" || hasVariant(typed) ? variants : variants.concat([typed])
    }

    function load() {
        variants = item ? item.variants.slice() : []
        variantField.text = ""
        nameField.text = item ? item.name : ""
        pick()
    }

    // The box shows the place's location. Entry 0 is "no location yet".
    function pick() {
        locationBox.currentIndex = item ? 1 + locations.findIndex(l => l.id === item.location) : 0
    }

    // The id the box shows; "" for no location yet.
    function chosen() {
        return locationBox.currentIndex > 0 ? locations[locationBox.currentIndex - 1].id : ""
    }

    onItemChanged: load()
    // The locations arrive after the form is made, or change under it.
    onLocationsChanged: load()

    submit: function () {
        return controller.editPlace(item.id, nameField.text, JSON.stringify(allVariants()), chosen())
    }
    remove: function () { return controller.removePlace(item.id) }

    KanteSectionLabel { text: form.texts.t("place.name") }
    KanteTextField {
        id: nameField
        objectName: "nameField"
        Layout.fillWidth: true
    }

    KanteSectionLabel { text: form.texts.t("place.variants") }
    Flow {
        objectName: "variantChips"
        Layout.fillWidth: true
        visible: form.variants.length > 0
        spacing: Kirigami.Units.smallSpacing

        Repeater {
            model: form.variants
            delegate: KanteChip {
                required property string modelData
                objectName: "variant"
                text: modelData
                removable: true
                removeText: form.texts.t("place.variantRemove", { name: "%1" })
                onRemoveRequested: form.variants = form.variants.filter(v => v !== modelData)
            }
        }
    }
    KanteTextField {
        id: variantField
        objectName: "variantField"
        Layout.fillWidth: true
        placeholderText: form.texts.t("place.variantAdd")
        Accessible.name: form.texts.t("place.variants")
        onAccepted: form.addVariant()
    }

    KanteSectionLabel { text: form.texts.t("place.location") }
    KanteComboBox {
        id: locationBox
        objectName: "locationBox"
        Layout.fillWidth: true
        Accessible.name: form.texts.t("place.location")
        model: [form.texts.t("place.noneYet")].concat(form.locations.map(l => l.name))
        // A new model resets the choice.
        onModelChanged: form.pick()
    }

    QQC2.Label {
        objectName: "sceneNumbers"
        Layout.fillWidth: true
        wrapMode: Text.Wrap
        color: KanteStyle.mutedTextColor
        text: form.item && form.item.scenes.length > 0
            ? form.texts.t("place.scenes", { numbers: form.item.scenes.join(", ") })
            : form.texts.t("place.noScenes")
    }
}
