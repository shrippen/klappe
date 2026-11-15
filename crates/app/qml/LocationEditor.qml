import QtQuick
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import Kante

// Adds a location or changes its name and size (LH-02, D-08). The set drawn there stays.
EditorPane {
    id: form

    // What a new location starts with, in metres.
    readonly property real defaultWidth: 10
    readonly property real defaultDepth: 8

    title: texts.t(item ? "location.edit" : "location.new")
    applyText: texts.t(item ? "form.apply" : "form.add")
    canApply: nameField.text.trim() !== ""
    removeQuestion: item ? texts.t("remove.location", { name: item.name.trim() }) : ""

    function load() {
        nameField.text = item ? item.name : ""
        widthField.text = texts.num(item ? item.width : defaultWidth, 2)
        depthField.text = texts.num(item ? item.depth : defaultDepth, 2)
    }

    onItemChanged: load()

    submit: function () {
        if (!item) {
            return controller.addLocation(nameField.text, widthField.text, depthField.text)
        }
        return controller.editLocation(item.id, nameField.text, widthField.text, depthField.text)
    }
    remove: function () { return controller.removeLocation(item.id) }

    KanteSectionLabel { text: form.texts.t("location.name") }
    KanteTextField {
        id: nameField
        objectName: "nameField"
        Layout.fillWidth: true
    }
    RowLayout {
        Layout.fillWidth: true
        spacing: Kirigami.Units.largeSpacing

        ColumnLayout {
            Layout.fillWidth: true
            KanteSectionLabel { text: form.texts.t("location.width") }
            KanteTextField {
                id: widthField
                objectName: "widthField"
                Layout.fillWidth: true
                inputMethodHints: Qt.ImhFormattedNumbersOnly
            }
        }
        ColumnLayout {
            Layout.fillWidth: true
            KanteSectionLabel { text: form.texts.t("location.depth") }
            KanteTextField {
                id: depthField
                objectName: "depthField"
                Layout.fillWidth: true
                inputMethodHints: Qt.ImhFormattedNumbersOnly
            }
        }
    }
}
