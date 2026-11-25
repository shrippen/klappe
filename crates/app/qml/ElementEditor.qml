import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// D-22: one element of the breakdown. Name, category, description and status; the scenes that
// need it as chips that show the scene. Without an `item` the form makes a new element: name and
// category only, the rest is changed afterwards. The responsible person (D-26) is chosen from the
// team, the persons of the department that usually answers for the category first. Deleting takes
// the element out of every scene and every set object.
EditorPane {
    id: form

    /// `[name]` in the catalogue's order, and the states.
    property var categories: []
    property var statuses: []
    /// The project's scenes, `{ id, number }`: a chip of the element's scenes shows its scene.
    property var scenes: []
    /// The persons for the element's category, the matching department first (`personsFor`);
    /// follows the project, so a person made or deleted shows at once.
    readonly property var people: {
        void controller.projectJson
        return item ? JSON.parse(controller.personsFor(item.category)) : []
    }

    /// The scene `id` is asked for.
    signal sceneRequested(string id)

    title: texts.t(item ? "elements.edit" : "elements.new")
    applyText: texts.t(item ? "form.apply" : "form.add")
    canApply: nameField.text.trim() !== ""
    removeQuestion: item ? texts.t("remove.element", { name: item.name }) : ""

    // The category the box shows; "" before the catalogue arrived.
    function chosenCategory() {
        return categories[categoryBox.currentIndex] || ""
    }

    function load() {
        nameField.text = item ? item.name : ""
        descriptionField.text = item ? item.description : ""
        categoryBox.currentIndex = item ? categories.indexOf(item.category) : 0
        statusBox.currentIndex = item ? statuses.indexOf(item.status) : 0
        loadResponsible()
    }

    // The box shows "none" first, then the persons.
    function loadResponsible() {
        const at = item && item.responsible ? people.findIndex(p => p.id === item.responsible) : -1
        responsibleBox.currentIndex = at + 1
    }

    // The id of the person the box shows; "" for none.
    function chosenResponsible() {
        const person = people[responsibleBox.currentIndex - 1]
        return person ? person.id : ""
    }

    onItemChanged: load()
    onCategoriesChanged: load()

    submit: function () {
        if (!item) {
            return controller.addElement(chosenCategory(), nameField.text)
        }
        return controller.editElement(item.id, chosenCategory(), nameField.text,
                                      descriptionField.text, statuses[statusBox.currentIndex],
                                      chosenResponsible())
    }
    remove: function () { return controller.removeElement(item.id) }

    KanteSectionLabel { text: form.texts.t("field.name") }
    KanteTextField {
        id: nameField
        objectName: "nameField"
        Layout.fillWidth: true
        Accessible.name: form.texts.t("field.name")
    }

    KanteSectionLabel { text: form.texts.t("elements.category") }
    KanteComboBox {
        id: categoryBox
        objectName: "categoryBox"
        Layout.fillWidth: true
        Accessible.name: form.texts.t("elements.category")
        model: form.categories.map(c => form.texts.t("elementCategory." + c))
    }

    KanteSectionLabel {
        visible: form.item !== null
        text: form.texts.t("elements.description")
    }
    KanteTextField {
        id: descriptionField
        objectName: "descriptionField"
        visible: form.item !== null
        Layout.fillWidth: true
        Accessible.name: form.texts.t("elements.description")
    }

    KanteSectionLabel {
        visible: form.item !== null
        text: form.texts.t("elements.status")
    }
    KanteSegmented {
        id: statusBox
        objectName: "statusBox"
        visible: form.item !== null
        model: form.statuses.map(s => form.texts.t("elementStatus." + s))
    }

    KanteSectionLabel {
        visible: form.item !== null
        text: form.texts.t("elements.responsible")
    }
    KanteComboBox {
        id: responsibleBox
        objectName: "responsibleBox"
        visible: form.item !== null
        Layout.fillWidth: true
        Accessible.name: form.texts.t("elements.responsible")
        model: [form.texts.t("elements.noResponsible")].concat(form.people.map(p => p.name))
        // A new model resets the box: it shows the element's person again.
        onModelChanged: form.loadResponsible()
    }
    QQC2.Label {
        objectName: "noPersons"
        visible: form.item !== null && form.people.length === 0
        Layout.fillWidth: true
        wrapMode: Text.Wrap
        color: KanteStyle.mutedTextColor
        text: form.texts.t("elements.noPersons")
    }

    KanteSectionLabel {
        visible: form.item !== null
        text: form.texts.t("elements.scenes")
    }
    Flow {
        objectName: "sceneChips"
        visible: form.item !== null && form.item.scenes.length > 0
        Layout.fillWidth: true
        spacing: Kirigami.Units.smallSpacing

        Repeater {
            model: form.item ? form.item.scenes : []
            delegate: KanteChip {
                // { id, number }: numbers may repeat (DB-33), the id names the scene.
                required property var modelData
                objectName: "scene." + modelData.id
                text: modelData.number
                toolTip: form.texts.t("elements.sceneJump", { number: modelData.number })
                onClicked: {
                    const scene = form.scenes.find(s => s.id === modelData.id)
                    if (scene) {
                        form.sceneRequested(scene.id)
                    }
                }
            }
        }
    }
    QQC2.Label {
        objectName: "noScenes"
        visible: form.item !== null && form.item.scenes.length === 0
        Layout.fillWidth: true
        wrapMode: Text.Wrap
        color: KanteStyle.mutedTextColor
        text: form.texts.t("elements.noScenes")
    }
}
