import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami
import Kante

// D-25: one look. The name is applied with "Apply"; costume and make-up elements go in and out
// at once (a chip's cross, the field below it), each one undo step, like in the scene sheet.
// The scenes that put the look on are chips that show the scene. Deleting takes the look out
// of every scene.
EditorPane {
    id: form

    /// The categories a look is made of, in the sheet's order.
    readonly property var categories: ["costume", "makeup"]
    /// The project's characters and elements, as in `projectJson`.
    property var characters: []
    property var elements: []
    /// The id of the look in the name field: a refresh of the same look keeps what is typed.
    property string loaded: ""

    /// The scene `id` is asked for.
    signal sceneRequested(string id)

    readonly property string figure: item ? (characters.find(c => c.id === item.character) || { name: "" }).name : ""

    title: texts.t("looks.editTitle")
    applyText: texts.t("form.apply")
    canApply: nameField.text.trim() !== ""
    removeQuestion: item ? texts.t("remove.look", { name: item.name }) : ""

    function load() {
        loaded = item ? item.id : ""
        nameField.text = item ? item.name : ""
    }

    // The elements of `category` that this look does not have yet.
    function offered(category) {
        const have = item ? item.elements.map(e => e.id) : []
        return elements.filter(e => e.category === category && !have.includes(e.id))
    }

    function statusOf(id) {
        return (elements.find(e => e.id === id) || { status: "open" }).status
    }

    // The element of that name goes into the look; one that is not there yet is made first.
    function add(category, name) {
        const text = name.trim()
        if (text === "" || !item) {
            return
        }
        const known = elements.find(e => e.category === category && e.name.toLowerCase() === text.toLowerCase())
        const id = known ? known.id : controller.addElement(category, text)
        if (id !== "") {
            controller.addLookElement(item.id, id)
        }
    }

    onItemChanged: {
        if ((item ? item.id : "") !== loaded) {
            load()
        }
    }

    submit: function () { return controller.editLook(item.id, nameField.text) }
    remove: function () { return controller.removeLook(item.id) }

    KanteSectionLabel { text: form.texts.t("field.name") }
    KanteTextField {
        id: nameField
        objectName: "nameField"
        Layout.fillWidth: true
        Accessible.name: form.texts.t("field.name")
    }

    KanteSectionLabel { text: form.texts.t("looks.character") }
    QQC2.Label {
        objectName: "figure"
        Layout.fillWidth: true
        color: KanteStyle.textColor
        text: form.figure
    }

    Repeater {
        model: form.categories
        delegate: ColumnLayout {
            id: row
            required property string modelData
            readonly property string category: modelData
            Layout.fillWidth: true
            spacing: Kirigami.Units.smallSpacing

            KanteSectionLabel { text: form.texts.t("elementCategory." + row.category) }
            Flow {
                objectName: "chips." + row.category
                Layout.fillWidth: true
                spacing: Kirigami.Units.smallSpacing

                Repeater {
                    model: form.item ? form.item.elements.filter(e => e.category === row.category) : []
                    delegate: ElementChip {
                        required property var modelData
                        objectName: "element." + modelData.id
                        text: modelData.name
                        status: form.statusOf(modelData.id)
                        removable: true
                        interactive: false
                        removeText: form.texts.t("looks.elementRemove", { name: "%1" })
                        onRemoveRequested: form.controller.removeLookElement(form.item.id, modelData.id)
                    }
                }
            }
            // Typing narrows the elements of this category the look lacks; Enter takes the one
            // under the cursor, or makes the typed one.
            KanteSearchCombo {
                objectName: "add." + row.category
                Layout.preferredWidth: Kirigami.Units.gridUnit * 18
                keepText: false
                allowNew: true
                model: form.offered(row.category).map(e => e.name)
                placeholderText: form.texts.t("looks.elementAdd", {
                    category: form.texts.t("elementCategory." + row.category) })
                newText: form.texts.t("textMark.new", {
                    category: form.texts.t("elementCategory." + row.category) }) + " “%1”"
                onActivated: index => form.add(row.category, model[index])
                onNewEntered: text => form.add(row.category, text)
            }
        }
    }

    KanteSectionLabel { text: form.texts.t("looks.scenes") }
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
                toolTip: form.texts.t("looks.sceneJump", { number: modelData.number })
                onClicked: form.sceneRequested(modelData.id)
            }
        }
    }
    QQC2.Label {
        objectName: "noScenes"
        visible: form.item !== null && form.item.scenes.length === 0
        Layout.fillWidth: true
        wrapMode: Text.Wrap
        color: KanteStyle.mutedTextColor
        text: form.texts.t("looks.noScenes")
    }
}
