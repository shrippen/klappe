import QtQuick
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import Kante

// A new look: its name, and the character it is for (a box, unless the character is given).
// With a `scene` the character wears it there at once: the look and its place in the scene are
// two undo steps.
FormDialog {
    id: form

    /// The project's characters, `{ id, name }`.
    property var characters: []
    /// The scene the character puts the look on; "" for none.
    property string scene: ""
    /// The character the look is for; "" lets the box choose.
    property string character: ""

    /// The look `id` was made.
    signal made(string id)

    readonly property string chosen: character !== "" ? character
        : (characters[characterBox.currentIndex] || { id: "" }).id

    title: texts.t("looks.newTitle")
    acceptText: texts.t("form.add")
    acceptEnabled: nameField.text.trim() !== "" && chosen !== ""

    // Asks for a look of `character` ("" any), put on in `scene` ("" none).
    function show(scene, character) {
        form.scene = scene
        form.character = character
        characterBox.currentIndex = Math.max(0, characters.findIndex(c => c.id === character))
        nameField.text = ""
        open()
    }

    onOpened: nameField.forceActiveFocus()

    submit: function () {
        const id = controller.addLook(chosen, nameField.text)
        if (id === "") {
            return false
        }
        if (scene !== "") {
            controller.setSceneLook(scene, chosen, id)
        }
        made(id)
        return true
    }

    KanteSectionLabel {
        visible: form.character === ""
        text: form.texts.t("looks.character")
    }
    KanteComboBox {
        id: characterBox
        objectName: "characterBox"
        visible: form.character === ""
        Layout.fillWidth: true
        Accessible.name: form.texts.t("looks.character")
        model: form.characters.map(c => c.name)
    }

    KanteSectionLabel { text: form.texts.t("field.name") }
    KanteTextField {
        id: nameField
        objectName: "nameField"
        Layout.fillWidth: true
        Accessible.name: form.texts.t("field.name")
    }
}
