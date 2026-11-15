import QtQuick
import QtQuick.Dialogs
import Kante

// LH-80, LH-81, LH-83: asks where an export goes. A plan export (PDF, PNG, SVG, frames) asks
// for the paper first, then for the file or folder; the shot list asks for the file only.
// Cancelling either writes nothing. Paper and folder are kept for the session.
//
//   request("pdf") → paper A4 | A3 → file dialog → controller.exportTo("pdf", order, paper, file)
FormDialog {
    id: dialog

    /// The order of the exported shot list: "cut" or "shooting".
    property string order: "cut"
    /// What was chosen last: "a4" or "a3".
    property string paper: "a4"
    /// The folder of the last export, as a `file://` URL; empty before the first one.
    property string folder: ""
    /// The format being asked for: "pdf", "png", "svg", "csv", "ods", "xlsx" or "frames".
    property string kind: ""

    readonly property var papers: ["a4", "a3"]
    readonly property var paperKinds: ["pdf", "png", "svg", "frames"]

    title: texts.t("export.paperTitle")
    acceptText: texts.t("export.next")

    function request(kind) {
        dialog.kind = kind
        if (paperKinds.indexOf(kind) < 0) {
            pick()
            return
        }
        open()
    }

    // The file dialog, offered the plan's name in the folder of the last export.
    function pick() {
        if (kind === "frames") {
            folders.title = texts.t("export.pick.frames")
            folders.currentFolder = suggestion().replace(/\/[^/]*$/, "")
            folders.open()
            return
        }
        files.title = texts.t("export.pick." + kind)
        files.nameFilters = [texts.t("export.filter." + kind)]
        files.defaultSuffix = kind
        files.currentFile = suggestion()
        files.open()
    }

    function suggestion() {
        const url = controller.exportFile(kind)
        if (url === "" || folder === "") {
            return url
        }
        return folder + url.substring(url.lastIndexOf("/"))
    }

    // The file was chosen: remembers the folder and writes.
    function save(url) {
        dialog.folder = url.substring(0, url.lastIndexOf("/"))
        return controller.exportTo(kind, order, paper, url)
    }

    submit: function () {
        pick()
        return true
    }

    KanteSectionLabel { text: dialog.texts.t("export.paper") }
    KanteSegmented {
        objectName: "paper"
        model: ["A4", "A3"]
        currentIndex: dialog.papers.indexOf(dialog.paper)
        onActivated: index => dialog.paper = dialog.papers[index]
    }

    // The frames go into a new folder in the chosen one, named as the suggestion.
    FolderDialog {
        id: folders
        objectName: "folders"
        onAccepted: dialog.save(selectedFolder.toString() + suggestion().substring(suggestion().lastIndexOf("/")))
    }

    FileDialog {
        id: files
        objectName: "files"
        fileMode: FileDialog.SaveFile
        onAccepted: dialog.save(selectedFile.toString())
    }
}
