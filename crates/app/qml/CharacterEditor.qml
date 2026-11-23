import QtQuick
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import Kante

// Adds a character or changes one (LH-03, D-08): name, the short label drawn in the figure,
// one of Kante's label colours, its size across in metres (LH-25; empty is the standard),
// and whether it is a group of extras with a head count (LH-23). A new name is carried into the cues of the script text (the pane warns first).
// A character's other names in the book (DB-50) show as chips; a name typed and not yet
// entered counts on Apply. A removed name the book uses is said first: the book asks again.
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

    // The other names as the form holds them until Apply.
    property var aliases: []

    title: texts.t(item ? "character.edit" : "character.new")
    applyText: texts.t(item ? "form.apply" : "form.add")
    canApply: nameField.text.trim() !== "" && shortField.text.trim() !== ""
    removeQuestion: item ? texts.t("remove.character", { name: item.name.trim() }) : ""

    function firstFree() {
        const used = characters.map(c => c.color)
        const free = colors.findIndex(c => !used.includes(c))
        return free < 0 ? 0 : free
    }

    // Whether `name` is in the list already, as the session compares character names.
    function hasAlias(name) {
        return aliases.some(a => controller.sameFigureName(a, name))
    }

    function addAlias() {
        const name = aliasField.text.trim()
        if (name !== "" && !hasAlias(name)) {
            aliases = aliases.concat([name])
        }
        aliasField.text = ""
    }

    // The chips and a name typed but not entered.
    function allAliases() {
        const typed = aliasField.text.trim()
        return typed === "" || hasAlias(typed) ? aliases : aliases.concat([typed])
    }

    function load() {
        aliases = item ? item.aliases.slice() : []
        aliasField.text = ""
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
        const raw = controller.characterChanges(item.id, nameField.text,
            JSON.stringify(allAliases()))
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
            sizeField.text, JSON.stringify(allAliases()))
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

    KanteSectionLabel {
        visible: form.item !== null
        text: form.texts.t("character.aliases")
    }
    Flow {
        objectName: "aliasChips"
        Layout.fillWidth: true
        visible: form.item !== null && form.aliases.length > 0
        spacing: Kirigami.Units.smallSpacing

        Repeater {
            model: form.aliases
            delegate: KanteChip {
                required property string modelData
                objectName: "alias"
                text: modelData
                removable: true
                removeText: form.texts.t("character.aliasRemove", { name: "%1" })
                onRemoveRequested: form.aliases = form.aliases.filter(a => a !== modelData)
            }
        }
    }
    KanteTextField {
        id: aliasField
        objectName: "aliasField"
        Layout.fillWidth: true
        visible: form.item !== null
        placeholderText: form.texts.t("character.aliasAdd")
        Accessible.name: form.texts.t("character.aliases")
        onAccepted: form.addAlias()
    }
}
