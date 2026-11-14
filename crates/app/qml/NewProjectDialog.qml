import QtQuick
import QtQuick.Layouts
import org.kde.kirigami as Kirigami
import Kante

// The title of a new project, asked after its folder was chosen (LH-01).
FormDialog {
    id: form

    // The folder as the folder dialog gave it (a `file://` URL).
    property string folder: ""

    title: texts.t("project.newTitle")
    acceptText: texts.t("project.create")
    acceptEnabled: titleField.text.trim() !== ""

    // The folder's name is a fair first title.
    function show(folder) {
        form.folder = folder
        const parts = decodeURIComponent(folder).replace(/\/+$/, "").split("/")
        titleField.text = parts[parts.length - 1].replace(/\.klappe$/, "")
        open()
    }

    submit: function () {
        return controller.createProject(folder, titleField.text)
    }

    KanteSectionLabel { text: form.texts.t("project.titleField") }
    KanteTextField {
        id: titleField
        objectName: "titleField"
        Layout.fillWidth: true
    }
}
