import QtQuick
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import Kante

// Adds a character or changes one (LH-03, D-08): name, the short label drawn in the figure,
// one of Kante's label colours, its size across in metres (LH-25; empty is the standard),
// and whether it is a group of extras with a head count (LH-23). A new name is carried into the cues of the script text (the pane warns first).
EditorPane {
    id: form

    // [{ color }]: the project's characters, so a new one starts with a colour not yet used.
    property var characters: []

    // The letters that fit into a figure.
    readonly property int shortLength: 3
    // A group has two to 999 people; the controller takes 0 for a single person.
    readonly property int minGroup: 2
    readonly property int maxGroup: 999
    readonly property int noGroup: 0
    readonly property var colors: KanteStyle.labels

    title: texts.t(item ? "character.edit" : "character.new")
    applyText: texts.t(item ? "form.apply" : "form.add")
    canApply: nameField.text.trim() !== "" && shortField.text.trim() !== ""
    removeQuestion: item ? texts.t("remove.character", { name: item.name.trim() }) : ""

    function firstFree() {
        const used = characters.map(c => c.color)
        const free = colors.findIndex(c => !used.includes(c))
        return free < 0 ? 0 : free
    }

    function load() {
        nameField.text = item ? item.name : ""
        shortField.text = item ? item.short : ""
        picker.currentIndex = item ? colors.indexOf(item.color) : firstFree()
        sizeField.text = item && item.size ? texts.num(item.size, 2) : ""
        const group = item ? item.group : noGroup
        groupSwitch.checked = group >= minGroup
        groupSize.value = group >= minGroup ? group : minGroup
    }

    onItemChanged: load()

    preview: function () {
        if (!item) {
            return []
        }
        const raw = controller.characterChanges(item.id, nameField.text)
        return raw ? JSON.parse(raw) : []
    }
    submit: function () {
        const color = colors[picker.currentIndex]
        const group = groupSwitch.checked ? groupSize.value : noGroup
        if (!item) {
            return controller.addCharacter(nameField.text, shortField.text, color, group,
                sizeField.text)
        }
        return controller.editCharacter(item.id, nameField.text, shortField.text, color, group,
            sizeField.text)
    }
    remove: function () { return controller.removeCharacter(item.id) }

    KanteSectionLabel { text: form.texts.t("character.name") }
    KanteTextField {
        id: nameField
        objectName: "nameField"
        Layout.fillWidth: true
        // The short label follows the name until it is typed by hand.
        onTextEdited: {
            if (!shortField.activeFocus && form.item === null) {
                shortField.text = text.trim().charAt(0).toUpperCase()
            }
        }
    }
    KanteSectionLabel { text: form.texts.t("character.short") }
    KanteTextField {
        id: shortField
        objectName: "shortField"
        Layout.fillWidth: true
        maximumLength: form.shortLength
    }
    KanteSectionLabel { text: form.texts.t("character.color") }
    KanteSwatchPicker {
        id: picker
        objectName: "colorPicker"
        colors: form.colors.map(c => KanteStyle.labelColor(c))
        names: form.colors.map(c => form.texts.t("color." + c))
        onPicked: index => currentIndex = index
    }
    KanteSectionLabel { text: form.texts.t("character.size") }
    KanteTextField {
        id: sizeField
        objectName: "sizeField"
        Layout.fillWidth: true
        placeholderText: form.texts.t("character.sizeStandard")
        inputMethodHints: Qt.ImhFormattedNumbersOnly
    }
    KanteSwitch {
        id: groupSwitch
        objectName: "groupSwitch"
        text: form.texts.t("character.group")
    }
    KanteSpinBox {
        id: groupSize
        objectName: "groupSize"
        visible: groupSwitch.checked
        from: form.minGroup
        to: form.maxGroup
        value: form.minGroup
        editable: true
        Accessible.name: form.texts.t("character.groupSize")
    }
}
