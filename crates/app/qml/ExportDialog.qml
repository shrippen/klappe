import QtQuick
import QtQuick.Dialogs
import Kante

// LH-80, LH-81, LH-83: asks where an export goes. A plan export (PDF, PNG, SVG, frames) asks
// for the paper (size and orientation) first, then for the file or folder; the shot list asks
// for the file only. Cancelling either writes nothing. The paper starts from the project's and
// is kept there by the export; the folder is kept for the session.
//
//   request("pdf") → paper A4 | A3 | Letter, Hoch | Quer → file dialog
//                  → controller.exportTo("pdf", order, "a4-portrait", file)
FormDialog {
    id: dialog

    /// The order of the exported shot list: "cut" or "shooting".
    property string order: "cut"
    /// The sizes offered, `{ id, label }` each, and the orientations, as ids; from the project
    /// view, so the lists stand once, in the services.
    property var papers: []
    property var orientations: []
    /// The project's sheet as "a4-portrait"; the dialog starts from it.
    property string projectPaper: ""
    /// What is chosen: a size id and an orientation id.
    property string size: "a4"
    property string orientation: "landscape"
    /// Size and orientation as the controller takes them: "a4-portrait".
    readonly property string paper: size + "-" + orientation
    /// The folder of the last export, as a `file://` URL; empty before the first one.
    property string folder: ""
    /// The format being asked for: "pdf", "png", "svg", "csv", "ods", "xlsx", "frames", "view3d" or
    /// "camera" (the last two are PNGs of a view, LH-86).
    property string kind: ""

    readonly property var paperKinds: ["pdf", "png", "svg", "frames"]

    title: texts.t("export.paperTitle")
    acceptText: texts.t("export.next")

    function request(kind) {
        dialog.kind = kind
        if (paperKinds.indexOf(kind) < 0) {
            pick()
            return
        }
        const kept = projectPaper.split("-")
        if (kept.length === 2) {
            size = kept[0]
            orientation = kept[1]
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
        files.defaultSuffix = kind === "view3d" || kind === "camera" ? "png" : kind
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
        model: dialog.papers.map(p => p.label)
        currentIndex: dialog.papers.findIndex(p => p.id === dialog.size)
        onActivated: index => dialog.size = dialog.papers[index].id
    }
    KanteSectionLabel { text: dialog.texts.t("export.orientation") }
    KanteSegmented {
        objectName: "orientation"
        model: dialog.orientations.map(o => dialog.texts.t("export." + o))
        currentIndex: dialog.orientations.indexOf(dialog.orientation)
        onActivated: index => dialog.orientation = dialog.orientations[index]
    }

    // The file dialogs sit in this popup, which has no window while closed; the shot list
    // opens one without the paper form, so they take the window from the popup's parent.
    readonly property var hostWindow: dialog.parent ? dialog.parent.Window.window : null

    // The frames go into a new folder in the chosen one, named as the suggestion.
    FolderDialog {
        id: folders
        objectName: "folders"
        parentWindow: dialog.hostWindow
        onAccepted: dialog.save(selectedFolder.toString() + suggestion().substring(suggestion().lastIndexOf("/")))
    }

    FileDialog {
        id: files
        objectName: "files"
        parentWindow: dialog.hostWindow
        fileMode: FileDialog.SaveFile
        onAccepted: dialog.save(selectedFile.toString())
    }
}
