import QtQuick
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import Kante

// Adds a location or changes its name, size, room height and place on Earth (LH-02, LH-90,
// LH-52, D-08). The set drawn there stays.
EditorPane {
    id: form

    // What a new location starts with, in metres.
    readonly property real defaultWidth: 10
    readonly property real defaultDepth: 8
    readonly property real defaultHeight: 2.6

    title: texts.t(item ? "location.edit" : "location.new")
    applyText: texts.t(item ? "form.apply" : "form.add")
    canApply: nameField.text.trim() !== ""
    removeQuestion: item ? texts.t("remove.location", { name: item.name.trim() }) : ""

    function load() {
        nameField.text = item ? item.name : ""
        widthField.text = texts.num(item ? item.width : defaultWidth, 2)
        depthField.text = texts.num(item ? item.depth : defaultDepth, 2)
        heightField.text = texts.num(item ? item.height : defaultHeight, 2)
        latitudeField.text = item && item.latitude != null ? texts.num(item.latitude, 4) : ""
        longitudeField.text = item && item.longitude != null ? texts.num(item.longitude, 4) : ""
        northField.text = item && item.latitude != null ? texts.num(item.north, 1) : ""
    }

    // The place as the controller takes it: latitude, longitude and north as typed.
    function site() {
        return JSON.stringify([latitudeField.text, longitudeField.text, northField.text])
    }

    onItemChanged: load()

    submit: function () {
        if (!item) {
            return controller.addLocation(nameField.text, widthField.text, depthField.text, heightField.text, site())
        }
        return controller.editLocation(item.id, nameField.text, widthField.text, depthField.text, heightField.text, site())
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
    KanteSectionLabel { text: form.texts.t("location.height") }
    KanteTextField {
        id: heightField
        objectName: "roomHeightField"
        Layout.fillWidth: true
        inputMethodHints: Qt.ImhFormattedNumbersOnly
    }
    RowLayout {
        Layout.fillWidth: true
        spacing: Kirigami.Units.largeSpacing

        ColumnLayout {
            Layout.fillWidth: true
            KanteSectionLabel { text: form.texts.t("location.latitude") }
            KanteTextField {
                id: latitudeField
                objectName: "latitudeField"
                Layout.fillWidth: true
                inputMethodHints: Qt.ImhFormattedNumbersOnly
            }
        }
        ColumnLayout {
            Layout.fillWidth: true
            KanteSectionLabel { text: form.texts.t("location.longitude") }
            KanteTextField {
                id: longitudeField
                objectName: "longitudeField"
                Layout.fillWidth: true
                inputMethodHints: Qt.ImhFormattedNumbersOnly
            }
        }
    }
    KanteSectionLabel { text: form.texts.t("location.north") }
    KanteTextField {
        id: northField
        objectName: "northField"
        Layout.fillWidth: true
        inputMethodHints: Qt.ImhFormattedNumbersOnly
    }
    KanteCallout {
        Layout.fillWidth: true
        kind: KanteCallout.Kind.Info
        text: form.texts.t("location.sunHint")
    }
}
