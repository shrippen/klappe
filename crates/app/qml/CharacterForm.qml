import QtQuick
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import Kante

// Adds a character or changes one (LH-03): name, the short label drawn in the figure, one of
// Kante's label colours, and whether it is a group of extras with a head count (LH-23).
FormDialog {
    id: form

    // The character being edited; null for a new one.
    property var character: null
    // [{ color }]: the project's characters, so a new one starts with a colour not yet used.
    property var characters: []

    // The letters that fit into a figure.
    readonly property int shortLength: 3
    // A group has two to 999 people; the controller takes 0 for a single person.
    readonly property int minGroup: 2
    readonly property int maxGroup: 999
    readonly property int noGroup: 0
    readonly property var colors: KanteStyle.labels

    title: texts.t(character ? "character.edit" : "character.new")
    acceptText: texts.t(character ? "form.apply" : "form.add")
    acceptEnabled: nameField.text.trim() !== "" && shortField.text.trim() !== ""

    function firstFree() {
        const used = characters.map(c => c.color)
        const free = colors.findIndex(c => !used.includes(c))
        return free < 0 ? 0 : free
    }

    function show(character) {
        form.character = character
        nameField.text = character ? character.name : ""
        shortField.text = character ? character.short : ""
        picker.currentIndex = character ? colors.indexOf(character.color) : firstFree()
        const group = character ? character.group : noGroup
        groupSwitch.checked = group >= minGroup
        groupSize.value = group >= minGroup ? group : minGroup
        open()
    }

    submit: function () {
        const color = colors[picker.currentIndex]
        const group = groupSwitch.checked ? groupSize.value : noGroup
        if (!character) {
            return controller.addCharacter(nameField.text, shortField.text, color, group)
        }
        return controller.editCharacter(character.id, nameField.text, shortField.text, color, group)
    }

    KanteSectionLabel { text: form.texts.t("character.name") }
    KanteTextField {
        id: nameField
        objectName: "nameField"
        Layout.fillWidth: true
        // The short label follows the name until it is typed by hand.
        onTextEdited: {
            if (!shortField.activeFocus && form.character === null) {
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
