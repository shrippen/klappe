import QtQuick
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import Kante

// Adds a location or changes its name and size (LH-02). The set drawn there stays.
FormDialog {
    id: form

    // The location being edited; null for a new one.
    property var location: null

    // What a new location starts with, in metres.
    readonly property real defaultWidth: 10
    readonly property real defaultDepth: 8

    title: texts.t(location ? "location.edit" : "location.new")
    acceptText: texts.t(location ? "form.apply" : "form.add")
    acceptEnabled: nameField.text.trim() !== ""

    function show(location) {
        form.location = location
        nameField.text = location ? location.name : ""
        widthField.text = texts.num(location ? location.width : defaultWidth, 2)
        depthField.text = texts.num(location ? location.depth : defaultDepth, 2)
        open()
    }

    submit: function () {
        if (!location) {
            return controller.addLocation(nameField.text, widthField.text, depthField.text)
        }
        return controller.editLocation(location.id, nameField.text, widthField.text, depthField.text)
    }

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
